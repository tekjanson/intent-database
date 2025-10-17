//! Funnel prototype: async-friendly funnel/session API and minimal prototypes

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Pluggable embedder trait (async)
#[async_trait]
pub trait Embedder: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, String>;
}

/// Pluggable model adapter (async) for generating questions, rephrasing, etc.
#[async_trait]
pub trait ModelAdapter: Send + Sync {
    async fn generate_question(&self, context: &str) -> Result<String, String>;
}

/// Minimal question spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionSpec {
    pub id: String,
    pub prompt: String,
    pub answer_type: String,
}

/// Funnel data structure (serializable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Funnel {
    pub id: String,
    pub description: Option<String>,
    /// Node ids belonging to this funnel
    pub node_ids: Vec<String>,
}

/// Runtime session for a funnel activation
pub struct FunnelSession {
    pub funnel: Arc<Funnel>,
    pub state: usize,
}

impl Funnel {
    pub fn new(id: &str) -> Self {
        Self { id: id.to_string(), description: None, node_ids: Vec::new() }
    }

    /// Start an async session
    pub async fn activate(
        self: Arc<Self>,
        _embedder: Arc<dyn Embedder>,
        _model: Arc<dyn ModelAdapter>,
    ) -> FunnelSession {
        // In a full implementation we'd compute initial embedding and pick a node
        FunnelSession { funnel: self, state: 0 }
    }
}

impl FunnelSession {
    pub fn next_question(&mut self) -> Option<QuestionSpec> {
        // Prototype: return a canned question then finish
        if self.state == 0 {
            self.state += 1;
            Some(QuestionSpec {
                id: "q1".to_string(),
                prompt: "Is this about billing or usage?".to_string(),
                answer_type: "choice".to_string(),
            })
        } else {
            None
        }
    }

    pub async fn submit_answer(&mut self, _answer: &str) -> Result<(), String> {
        // Prototype: accept any answer and progress
        self.state += 1;
        Ok(())
    }
}

/// A tiny stub embedder for tests/dev
pub struct StubEmbedder {}

#[async_trait]
impl Embedder for StubEmbedder {
    async fn embed(&self, _text: &str) -> Result<Vec<f32>, String> {
        Ok(vec![0.0; 8])
    }
}

/// A deterministic, hash-based embedder used for tests and deterministic clustering.
pub struct HashEmbedder {
    pub dim: usize,
}

impl HashEmbedder {
    pub fn new(dim: usize) -> Self {
        Self { dim }
    }
}

#[async_trait]
impl Embedder for HashEmbedder {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, String> {
        // simple hash of the text into dim floats in [-1.0,1.0]
        let mut out = vec![0.0f32; self.dim];
        let mut h: u64 = 1469598103934665603u64; // fnv offset basis
        for b in text.as_bytes() {
            h ^= *b as u64;
            h = h.wrapping_mul(1099511628211u64);
        }
        // expand hash into dim numbers
        let mut seed = h;
        for i in 0..self.dim {
            seed = seed.wrapping_mul(6364136223846793005u64).wrapping_add(1442695040888963407u64);
            let f = ((seed >> (i % 32)) & 0xffff) as f32 / 65535.0;
            if let Some(x) = out.get_mut(i) {
                *x = f * 2.0 - 1.0;
            }
        }
        Ok(out)
    }
}

/// A tiny stub model adapter
pub struct StubModelAdapter {}

#[async_trait]
impl ModelAdapter for StubModelAdapter {
    async fn generate_question(&self, context: &str) -> Result<String, String> {
        Ok(format!("Question based on: {}", context))
    }
}

/// A funnel node represents a decision point in the funnel tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunnelNode {
    pub id: String,
    pub parent_id: Option<String>,
    pub centroid: Option<Vec<f32>>,
    pub example_ids: Vec<String>,
    pub question: Option<QuestionSpec>,
}

pub mod kmeans;

impl FunnelNode {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            parent_id: None,
            centroid: None,
            example_ids: Vec::new(),
            question: None,
        }
    }
}
