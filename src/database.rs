//! Intent Database - Main storage and retrieval functionality

use crate::conversation::Conversation;
use crate::matcher::{ConversationMatcher, SimilarityScore};
use serde::{Deserialize, Serialize};
pub mod graph;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

/// The main intent database for caching conversations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentDatabase {
    /// Storage for conversations by ID
    conversations: HashMap<String, Conversation>,
    /// Matcher for finding similar conversations (serializes its threshold)
    matcher: ConversationMatcher,
}

impl IntentDatabase {
    /// Create a new intent database
    pub fn new() -> Self {
        Self { conversations: HashMap::new(), matcher: ConversationMatcher::default() }
    }

    /// Create a new intent database with custom match threshold
    pub fn with_threshold(threshold: f64) -> Self {
        Self { conversations: HashMap::new(), matcher: ConversationMatcher::new(threshold) }
    }

    /// Store a conversation in the database
    pub fn store(&mut self, conversation: Conversation) -> Result<(), String> {
        if self.conversations.contains_key(&conversation.id) {
            return Err(format!("Conversation with id {} already exists", conversation.id));
        }
        self.conversations.insert(conversation.id.clone(), conversation);
        Ok(())
    }

    /// Retrieve a conversation by ID
    pub fn get(&self, id: &str) -> Option<&Conversation> {
        self.conversations.get(id)
    }

    /// Remove a conversation by ID
    pub fn remove(&mut self, id: &str) -> Option<Conversation> {
        self.conversations.remove(id)
    }

    /// Update an existing conversation
    pub fn update(&mut self, conversation: Conversation) -> Result<(), String> {
        if !self.conversations.contains_key(&conversation.id) {
            return Err(format!("Conversation with id {} does not exist", conversation.id));
        }
        self.conversations.insert(conversation.id.clone(), conversation);
        Ok(())
    }

    /// Find similar conversations to the given query
    pub fn find_similar(&self, query: &Conversation) -> Vec<(String, SimilarityScore)> {
        let mut matches: Vec<(String, SimilarityScore)> = self
            .conversations
            .values()
            .filter(|c| c.id != query.id)
            .map(|c| {
                let score = self.matcher.calculate_similarity(query, c);
                (c.id.clone(), score)
            })
            .filter(|(_, score)| score.is_match(self.matcher.match_threshold))
            .collect();

        matches
            .sort_by(|a, b| b.1.score.partial_cmp(&a.1.score).unwrap_or(std::cmp::Ordering::Equal));
        matches
    }

    /// Get the best matching conversation for a query
    pub fn get_best_match(&self, query: &Conversation) -> Option<(String, SimilarityScore)> {
        let matches = self.find_similar(query);
        matches.first().cloned()
    }

    /// Clean up expired conversations (fact invalidation pattern)
    pub fn cleanup_expired(&mut self) -> usize {
        let expired_ids: Vec<String> =
            self.conversations.values().filter(|c| c.is_expired()).map(|c| c.id.clone()).collect();

        let count = expired_ids.len();
        for id in expired_ids {
            self.conversations.remove(&id);
        }
        count
    }

    /// Get the total number of conversations
    pub fn len(&self) -> usize {
        self.conversations.len()
    }

    /// Check if the database is empty
    pub fn is_empty(&self) -> bool {
        self.conversations.is_empty()
    }

    /// Clear all conversations
    pub fn clear(&mut self) {
        self.conversations.clear();
    }

    /// Get all conversation IDs
    pub fn list_ids(&self) -> Vec<String> {
        self.conversations.keys().cloned().collect()
    }

    /// Persist the database to a JSON file (pretty-printed)
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let path_ref = path.as_ref();
        let file = File::create(path_ref).map_err(|e| e.to_string())?;
        serde_json::to_writer_pretty(file, &self).map_err(|e| e.to_string())
    }

    /// Load the database from a JSON file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let path_ref = path.as_ref();
        let file = File::open(path_ref).map_err(|e| e.to_string())?;
        let db: IntentDatabase = serde_json::from_reader(file).map_err(|e| e.to_string())?;
        Ok(db)
    }

    /// Try to load a database from file, or create a new one with given threshold
    pub fn load_or_new<P: AsRef<Path>>(path: P, threshold: f64) -> Result<Self, String> {
        match Self::load_from_file(path) {
            Ok(db) => Ok(db),
            Err(_) => Ok(Self::with_threshold(threshold)),
        }
    }
}

impl Default for IntentDatabase {
    fn default() -> Self {
        Self::new()
    }
}
mod async_db;
pub use async_db::AsyncIntentDatabase;

// Unit tests moved to `tests/database_tests.rs` to keep this file under the
// project's line-length and file-length linting constraints.
