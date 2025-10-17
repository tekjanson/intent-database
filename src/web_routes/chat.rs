use super::WebContext;
use crate::ai_sim::{SimulatedConnector, SimulatedModelAdapter};
use crate::web_helpers::{broadcast_graph, PendingReview};
use chrono::Utc;
use serde_json::{json, Value};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Instant;
use tiny_http::{Header, Response};

pub fn handle_chat(mut request: tiny_http::Request, context: Arc<WebContext>) {
    let mut body = String::new();
    request.as_reader().read_to_string(&mut body).ok();
    let v: Value = serde_json::from_str(&body).unwrap_or(json!({}));
    let ctx = v.get("context").and_then(|s| s.as_str()).unwrap_or("");
    let adapter = v
        .get("adapter")
        .and_then(|s| s.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| context.default_adapter.clone());

    let q = crate::conversation::Conversation::new(
        format!("ui-{}", Utc::now().timestamp_millis()),
        crate::conversation::Intent::new(ctx.to_string()),
        crate::conversation::Sentiment::Neutral,
    );

    let user_response = context.rt.block_on(async {
        match context.ai.generate_with(&adapter, ctx).await {
            Ok(s) => s,
            Err(_) => {
                // fall back to template adapter if available
                if adapter != "template" {
                    match context.ai.generate_with("template", ctx).await {
                        Ok(s2) => s2,
                        Err(_) => "error".to_string(),
                    }
                } else {
                    "error".to_string()
                }
            }
        }
    });

    let connector: Option<Arc<dyn crate::connector::ChatConnector>> = match adapter.as_str() {
        "sim" => Some(Arc::new(SimulatedConnector::new(
            Arc::new(SimulatedModelAdapter::new(99, "sim-gen")),
            0,
        ))),
        _ => None,
    };

    let ai_clone = context.ai.clone();
    let adapter_str = adapter.to_string();
    let intent_db_clone = context.intent_db.clone();
    let connector_clone = connector.clone();
    let q_clone = q.clone();
    let req_id = context.log_counter.fetch_add(1, Ordering::SeqCst);
    let start = Instant::now();
    let result = context.rt.block_on(async move {
        intent_db_clone
            .find_or_generate(
                &q_clone,
                None,
                move |q| {
                    let q_owned = q.clone();
                    let ai = ai_clone.clone();
                    let adapter = adapter_str.clone();
                    async move {
                        let ctx = q_owned.intent.purpose.clone();
                        let text = ai
                            .generate_with(&adapter, &ctx)
                            .await
                            .unwrap_or_else(|_| "error".to_string());
                        let mut conv = q_owned.clone();
                        conv.add_entry(crate::conversation::ConversationEntry::new(
                            "assistant".to_string(),
                            text.clone(),
                        ));
                        Ok(conv)
                    }
                },
                connector_clone,
            )
            .await
    });
    let elapsed = start.elapsed().as_millis();

    let resp_json = match &result {
        Ok(Some(conv)) => {
            let bm = context.rt.block_on(context.intent_db.get_best_match(&q));
            match bm {
                Ok(Some((id, score))) => json!({
                    "matched": {"id": id, "score": score.score},
                    "user_response": user_response,
                    "response": user_response,
                    "db_response": conv.entries
                        .last()
                        .map(|e| e.content.clone())
                        .unwrap_or_default(),
                    "persisted": true
                }),
                _ => json!({
                    "matched": null,
                    "user_response": user_response,
                    "response": user_response,
                    "db_response": conv.entries
                        .last()
                        .map(|e| e.content.clone())
                        .unwrap_or_default(),
                    "persisted": true
                }),
            }
        }
        Ok(None) => json!({
            "error": "no-result",
            "user_response": user_response,
            "response": user_response
        }),
        Err(e) => json!({
            "error": e,
            "user_response": user_response,
            "response": user_response
        }),
    };

    // structured logging
    {
        if let Ok(meta) = fs::metadata(&context.log_path) {
            if meta.len() > 5_000_000 {
                let _ = fs::rename(&context.log_path, context.log_path.with_extension("old"));
                let newf = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&context.log_path)
                    .expect("open log");
                let mut nf = context.log_file.lock().unwrap();
                *nf = newf;
            }
        }
        let mut lf = context.log_file.lock().unwrap();
        let matched_info = match &result {
            Ok(Some(_)) => {
                if let Ok(Some((id, score))) =
                    context.rt.block_on(context.intent_db.get_best_match(&q))
                {
                    json!({
                        "id": id,
                        "score": score.score,
                    })
                } else {
                    json!(null)
                }
            }
            _ => json!(null),
        };

        let log_entry = json!({
            "req_id": req_id,
            "ts": Utc::now().to_rfc3339(),
            "url": request.url(),
            "method": format!("{:?}", request.method()),
            "adapter": adapter,
            "request_body": v,
            "response": resp_json,
            "matched": matched_info,
            "status_ms": elapsed,
        });
        let s = serde_json::to_string(&log_entry).unwrap_or_else(|_| "{}".to_string());
        let _ = lf.write_all(s.as_bytes());
        let _ = lf.write_all(b"\n");
        let _ = lf.flush();
    }

    if let Ok(Some(_)) = &result {
        if let Ok(g) = context.rt.block_on(context.intent_db.list_graph()) {
            let s = g.to_string();
            broadcast_graph(&context.broadcasters, s);
        }
    }

    if let Ok(Some(conv)) = &result {
        let pr = PendingReview {
            id: conv.id.clone(),
            conv: conv.clone(),
            user_response: user_response.clone(),
            ai_response: conv.entries.last().map(|e| e.content.clone()).unwrap_or_default(),
        };
        let mut q = context.review_queue.lock().unwrap();
        q.push(pr);
    }

    let _ =
        request.respond(Response::from_string(resp_json.to_string()).with_header(
            Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
        ));
}
