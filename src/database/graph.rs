use crate::database::IntentDatabase;
use serde_json::json;
use serde_json::Value;
use std::sync::Arc;

pub fn build_graph_blocking(inner: Arc<std::sync::Mutex<IntentDatabase>>) -> Result<Value, String> {
    let guard = inner.lock().map_err(|e| format!("lock error: {:?}", e))?;
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    for conv in guard.conversations.values() {
        let snippet = conv.entries.last().map(|e| e.content.clone());
        nodes.push(json!({"id": conv.id.clone(), "snippet": snippet}));
    }

    for conv in guard.conversations.values() {
        if let Some((id, score)) = guard.get_best_match(conv) {
            edges.push(json!({"source": conv.id.clone(), "target": id, "score": score.score}));
        }
    }

    Ok(json!({"nodes": nodes, "edges": edges}))
}
