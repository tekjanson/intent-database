//! Intent Database - A conversation caching and matching system
//!
//! This library provides functionality to cache AI conversations and match them
//! with future conversations based on intent, thoughts, and feelings rather than
//! exact data matches.

pub mod conversation;
pub mod database;
pub mod matcher;

pub use conversation::{Conversation, ConversationEntry, Intent, Sentiment};
pub use database::IntentDatabase;
pub use matcher::SimilarityScore;
