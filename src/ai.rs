use crate::funnel::ModelAdapter;
use async_trait::async_trait;

/// A tiny deterministic TemplateModelAdapter used for tests and examples.
pub struct TemplateModelAdapter {}

impl TemplateModelAdapter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for TemplateModelAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ModelAdapter for TemplateModelAdapter {
    async fn generate_question(&self, context: &str) -> Result<String, String> {
        // Use context to create a deterministic question template.
        Ok(format!("Is the issue about: {}?", context))
    }
}
