//! Intent Database - A conversation caching and matching system
//!
//! This library provides functionality to cache AI conversations and match them
//! with future conversations based on intent, thoughts, and feelings rather than
//! exact data matches.

pub mod ai;
pub mod ai_manager;
pub mod ai_sim;
pub mod connector;
pub mod conversation;
pub mod database;
pub mod funnel;
pub mod funnel_builder;
#[cfg(feature = "gemini-model-adapter")]
pub mod gemini_model_adapter;
pub mod matcher;
pub mod math;
pub mod shortcircuit;
pub mod storage;
pub mod web_helpers;
pub mod web_routes;

#[cfg(feature = "gemini-model-adapter")]
pub use crate::gemini_model_adapter::GeminiModelAdapter;
pub use ai::TemplateModelAdapter;
pub use ai_manager::AiManager;
pub use ai_sim::{SimulatedConnector, SimulatedModelAdapter};
pub use connector::{ChatConnector, MockConnector};
pub use conversation::{Conversation, ConversationEntry, Intent, Sentiment};
pub use database::AsyncIntentDatabase;
pub use database::IntentDatabase;
pub use funnel::{Embedder, Funnel, FunnelSession, ModelAdapter};
pub use funnel_builder::SimpleFunnelBuilder;
pub use matcher::SimilarityScore;
pub use math::{
    centroid, cosine_distance, cosine_similarity, dot, l2_norm, normalize_inplace,
    unit_vector_or_zero,
};
pub use shortcircuit::ShortCircuitingConnector;
pub use storage::SledStorage;
