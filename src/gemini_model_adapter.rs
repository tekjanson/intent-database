#[cfg(feature = "gemini-model-adapter")]
mod inner {
    use crate::funnel::ModelAdapter;
    use async_trait::async_trait;
    use reqwest::Client;
    use serde::Serialize;
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

    #[derive(Serialize)]
    struct GeminiRequest<'a> {
        context: &'a str,
    }

    #[async_trait]
    impl ModelAdapter for GeminiModelAdapter {
        async fn generate_question(&self, context: &str) -> Result<String, String> {
            let req_body = GeminiRequest { context };
            let mut rq = self.client.post(&self.endpoint).json(&req_body);
            if let Some(key) = &self.api_key {
                rq = rq.header("Authorization", format!("Bearer {}", key));
            }
            let resp = rq.send().await.map_err(|e| e.to_string())?;
            let text = resp.text().await.map_err(|e| e.to_string())?;
            Ok(text)
        }
    }

    pub fn boxed(endpoint: impl Into<String>, api_key: Option<String>) -> Arc<dyn ModelAdapter> {
        Arc::new(GeminiModelAdapter::new(endpoint.into(), api_key))
    }
}

#[cfg(feature = "gemini-model-adapter")]
pub use inner::boxed as GeminiModelAdapter;
