use crate::connector::ChatConnector;
use crate::conversation::Conversation;
use crate::funnel::ModelAdapter;
use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// A simple deterministic simulated model adapter that produces a "thought outcome"
/// string based on the provided context and an internal seed. Useful for load
/// tests and deterministic simulations.
pub struct SimulatedModelAdapter {
    seed: u64,
    prefix: String,
}

impl SimulatedModelAdapter {
    pub fn new(seed: u64, prefix: impl Into<String>) -> Self {
        Self { seed, prefix: prefix.into() }
    }
}

#[async_trait]
impl ModelAdapter for SimulatedModelAdapter {
    async fn generate_question(&self, context: &str) -> Result<String, String> {
        // deterministic, lightweight string that can be used as a "generated"
        // question or thought outcome in tests.
        Ok(format!("{}-s{}:{}", self.prefix, self.seed, context))
    }
}

/// A connector that uses a ModelAdapter to produce a synthetic response and
/// returns thumbs-up/downs deterministically based on a simple counter.
pub struct SimulatedConnector {
    pub model: Arc<dyn ModelAdapter>,
    /// Every `positive_every`-th request returns positive=true. Use 0 to make
    /// every response positive.
    pub positive_every: usize,
    counter: AtomicUsize,
}

impl SimulatedConnector {
    pub fn new(model: Arc<dyn ModelAdapter>, positive_every: usize) -> Self {
        Self { model, positive_every, counter: AtomicUsize::new(0) }
    }
}

#[async_trait]
impl ChatConnector for SimulatedConnector {
    async fn send_and_get_feedback(&self, conv: &Conversation) -> Result<bool, String> {
        // use the model to "generate" a question (side-effect only for test realism)
        let _ = self.model.generate_question(&conv.intent.purpose).await?;

        // determine positivity
        if self.positive_every == 0 {
            return Ok(true);
        }
        let i = self.counter.fetch_add(1, Ordering::SeqCst);
        Ok(i % self.positive_every == 0)
    }
}
