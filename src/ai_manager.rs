use crate::funnel::ModelAdapter;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// A simple AI manager that holds named ModelAdapter instances. This allows
/// runtime selection of different adapters (e.g., Template, HTTP Gemini, Sim).
pub struct AiManager {
    adapters: RwLock<HashMap<String, Arc<dyn ModelAdapter>>>,
}

impl AiManager {
    pub fn new() -> Self {
        Self { adapters: RwLock::new(HashMap::new()) }
    }

    /// Register a model adapter with a name.
    pub fn register(&self, name: impl Into<String>, adapter: Arc<dyn ModelAdapter>) {
        let mut guard = self.adapters.write().unwrap();
        guard.insert(name.into(), adapter);
    }

    /// Unregister an adapter by name.
    pub fn unregister(&self, name: &str) {
        let mut guard = self.adapters.write().unwrap();
        guard.remove(name);
    }

    /// Get an adapter by name.
    pub fn get(&self, name: &str) -> Option<Arc<dyn ModelAdapter>> {
        let guard = self.adapters.read().unwrap();
        guard.get(name).cloned()
    }

    /// Convenience async proxy to generate a question using a named adapter.
    /// Returns an error if adapter is missing or generation fails.
    pub async fn generate_with(&self, name: &str, context: &str) -> Result<String, String> {
        let adapter = self.get(name).ok_or_else(|| format!("adapter '{}' not found", name))?;
        adapter.generate_question(context).await
    }
}

impl Default for AiManager {
    fn default() -> Self {
        Self::new()
    }
}
