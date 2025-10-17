use crate::funnel::ModelAdapter;
use async_trait::async_trait;

#[cfg(feature = "http-model-adapter")]
mod http_impl {
    use super::*;
    use reqwest::Client;
    use std::time::Duration;

    pub struct HttpModelAdapter {
        client: Client,
        endpoint: String,
        timeout: Duration,
        retries: usize,
    }

    impl HttpModelAdapter {
        pub fn new(endpoint: String, timeout: Duration, retries: usize) -> Self {
            let client =
                Client::builder().timeout(timeout).build().expect("failed to build reqwest client");
            Self { client, endpoint, timeout, retries }
        }
    }

    #[async_trait]
    impl ModelAdapter for HttpModelAdapter {
        async fn generate_question(&self, context: &str) -> Result<String, String> {
            let mut last_err: Option<String> = None;
            for _ in 0..self.retries {
                let req = self
                    .client
                    .post(&self.endpoint)
                    .json(&serde_json::json!({"context": context}))
                    .build();
                match req {
                    Ok(r) => {
                        let resp = self.client.execute(r).await;
                        match resp {
                            Ok(r2) => {
                                if r2.status().is_success() {
                                    match r2.text().await {
                                        Ok(txt) => return Ok(txt),
                                        Err(e) => last_err = Some(e.to_string()),
                                    }
                                } else {
                                    last_err = Some(format!("bad status: {}", r2.status()));
                                }
                            }
                            Err(e) => last_err = Some(e.to_string()),
                        }
                    }
                    Err(e) => last_err = Some(e.to_string()),
                }
                // simple backoff
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            Err(last_err.unwrap_or_else(|| "unknown http error".to_string()))
        }
    }

    pub fn adapter(endpoint: &str, timeout_ms: u64, retries: usize) -> HttpModelAdapter {
        HttpModelAdapter::new(endpoint.to_string(), Duration::from_millis(timeout_ms), retries)
    }
}

#[cfg(feature = "http-model-adapter")]
pub use http_impl::*;

// When feature disabled, provide no-op stub to avoid compilation errors in other modules
#[cfg(not(feature = "http-model-adapter"))]
mod http_nop {
    // nothing exported
}
