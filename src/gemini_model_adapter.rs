#[cfg(feature = "gemini-model-adapter")]
mod inner {
    use crate::funnel::ModelAdapter;
    use async_trait::async_trait;
    use reqwest::Client;

    use std::sync::Arc;

    #[derive(Clone)]
    pub struct GeminiModelAdapter {
        client: Client,
        endpoint: String,
        api_key: Option<String>,
    }

    impl GeminiModelAdapter {
        pub fn new(endpoint: impl Into<String>, api_key: Option<String>) -> Self {
            Self { client: Client::new(), endpoint: endpoint.into(), api_key }
        }
    }

    use serde_json::Value;

    #[async_trait]
    impl ModelAdapter for GeminiModelAdapter {
        async fn generate_question(&self, context: &str) -> Result<String, String> {
            // Build request in the shape expected by the Google Generative
            // Language `generateContent` endpoint.
            let req_body = serde_json::json!({
                "contents": [
                    {
                        "parts": [
                            { "text": context }
                        ]
                    }
                ]
            });

            let mut rq = self.client.post(&self.endpoint).json(&req_body);
            if let Some(key) = &self.api_key {
                // Google example uses X-goog-api-key header for api-key style auth
                rq = rq.header("X-goog-api-key", key);
            }

            let resp = rq.send().await.map_err(|e| e.to_string())?;
            let status = resp.status();
            let body_text = resp.text().await.map_err(|e| e.to_string())?;
            if !status.is_success() {
                return Err(format!("http {}: {}", status.as_u16(), body_text));
            }
            // Try to parse JSON, but if that fails fall back to raw text.
            let v: Value = match serde_json::from_str(&body_text) {
                Ok(j) => j,
                Err(_) => return Ok(body_text),
            };

            // Try several common response shapes used by generation APIs.
            // Prefer structured locations but fall back to the first string
            // leaf found so we don't fail on minor shape differences.
            fn first_string_leaf(v: &Value) -> Option<String> {
                match v {
                    Value::String(s) => Some(s.clone()),
                    Value::Array(arr) => {
                        for e in arr {
                            if let Some(s) = first_string_leaf(e) {
                                return Some(s);
                            }
                        }
                        None
                    }
                    Value::Object(map) => {
                        for (_, e) in map.iter() {
                            if let Some(s) = first_string_leaf(e) {
                                return Some(s);
                            }
                        }
                        None
                    }
                    _ => None,
                }
            }

            // Common: { candidates: [ { content: [ { text: "..." } ] } ] }
            if let Some(t) = v
                .get("candidates")
                .and_then(|c| c.get(0))
                .and_then(|c0| c0.get("content"))
                .and_then(|cont| cont.get(0))
                .and_then(|c1| c1.get("text"))
                .and_then(|tv| tv.as_str())
            {
                return Ok(t.to_string());
            }

            // Another possible shape: { candidates: [ { output: [ { content: [ { text: "..." } ] } ] } ] }
            if let Some(t) = v
                .get("candidates")
                .and_then(|c| c.get(0))
                .and_then(|c0| c0.get("output"))
                .and_then(|out| out.get(0))
                .and_then(|o0| o0.get("content"))
                .and_then(|cont| cont.get(0))
                .and_then(|c1| c1.get("text"))
                .and_then(|tv| tv.as_str())
            {
                return Ok(t.to_string());
            }

            // Generic: outputs[0].content[0].text
            if let Some(t) = v
                .get("outputs")
                .and_then(|o| o.get(0))
                .and_then(|o0| o0.get("content"))
                .and_then(|cont| cont.get(0))
                .and_then(|c1| c1.get("text"))
                .and_then(|tv| tv.as_str())
            {
                return Ok(t.to_string());
            }

            // Fallback: first string leaf in JSON
            if let Some(s) = first_string_leaf(&v) {
                return Ok(s);
            }

            Err("no-text-output-found".to_string())
        }
    }

    pub fn boxed(endpoint: impl Into<String>, api_key: Option<String>) -> Arc<dyn ModelAdapter> {
        Arc::new(GeminiModelAdapter::new(endpoint.into(), api_key))
    }
}

#[cfg(feature = "gemini-model-adapter")]
pub use inner::boxed as GeminiModelAdapter;
