//! Intent Database - Main storage and retrieval functionality

use crate::conversation::Conversation;
use crate::matcher::{ConversationMatcher, SimilarityScore};
use std::collections::HashMap;

/// The main intent database for caching conversations
pub struct IntentDatabase {
    /// Storage for conversations by ID
    conversations: HashMap<String, Conversation>,
    /// Matcher for finding similar conversations
    matcher: ConversationMatcher,
}

impl IntentDatabase {
    /// Create a new intent database
    pub fn new() -> Self {
        Self {
            conversations: HashMap::new(),
            matcher: ConversationMatcher::default(),
        }
    }

    /// Create a new intent database with custom match threshold
    pub fn with_threshold(threshold: f64) -> Self {
        Self {
            conversations: HashMap::new(),
            matcher: ConversationMatcher::new(threshold),
        }
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

        matches.sort_by(|a, b| b.1.score.partial_cmp(&a.1.score).unwrap());
        matches
    }

    /// Get the best matching conversation for a query
    pub fn get_best_match(&self, query: &Conversation) -> Option<(String, SimilarityScore)> {
        let matches = self.find_similar(query);
        matches.first().cloned()
    }

    /// Clean up expired conversations (fact invalidation pattern)
    pub fn cleanup_expired(&mut self) -> usize {
        let expired_ids: Vec<String> = self
            .conversations
            .values()
            .filter(|c| c.is_expired())
            .map(|c| c.id.clone())
            .collect();

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
}

impl Default for IntentDatabase {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversation::{Intent, Sentiment};
    use chrono::{Duration, Utc};

    #[test]
    fn test_database_store_and_get() {
        let mut db = IntentDatabase::new();
        let intent = Intent::new("Test".to_string());
        let conv = Conversation::new("test-1".to_string(), intent, Sentiment::Neutral);

        assert!(db.store(conv).is_ok());
        assert_eq!(db.len(), 1);

        let retrieved = db.get("test-1");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "test-1");
    }

    #[test]
    fn test_database_duplicate_store() {
        let mut db = IntentDatabase::new();
        let intent = Intent::new("Test".to_string());
        let conv1 = Conversation::new("test-1".to_string(), intent.clone(), Sentiment::Neutral);
        let conv2 = Conversation::new("test-1".to_string(), intent, Sentiment::Neutral);

        assert!(db.store(conv1).is_ok());
        assert!(db.store(conv2).is_err());
    }

    #[test]
    fn test_database_remove() {
        let mut db = IntentDatabase::new();
        let intent = Intent::new("Test".to_string());
        let conv = Conversation::new("test-1".to_string(), intent, Sentiment::Neutral);

        db.store(conv).unwrap();
        assert_eq!(db.len(), 1);

        let removed = db.remove("test-1");
        assert!(removed.is_some());
        assert_eq!(db.len(), 0);
    }

    #[test]
    fn test_database_update() {
        let mut db = IntentDatabase::new();
        let intent = Intent::new("Test".to_string());
        let mut conv = Conversation::new("test-1".to_string(), intent.clone(), Sentiment::Neutral);

        db.store(conv.clone()).unwrap();

        conv.add_metadata("key".to_string(), "value".to_string());
        assert!(db.update(conv).is_ok());

        let retrieved = db.get("test-1").unwrap();
        assert_eq!(retrieved.metadata.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_database_update_nonexistent() {
        let mut db = IntentDatabase::new();
        let intent = Intent::new("Test".to_string());
        let conv = Conversation::new("test-1".to_string(), intent, Sentiment::Neutral);

        assert!(db.update(conv).is_err());
    }

    #[test]
    fn test_database_find_similar() {
        let mut db = IntentDatabase::with_threshold(0.5);

        let intent1 = Intent::new("Get weather forecast".to_string())
            .with_topics(vec!["weather".to_string(), "temperature".to_string()]);
        let conv1 = Conversation::new("conv1".to_string(), intent1, Sentiment::Neutral);
        db.store(conv1).unwrap();

        let intent2 = Intent::new("Get weather information".to_string())
            .with_topics(vec!["weather".to_string(), "forecast".to_string()]);
        let conv2 = Conversation::new("conv2".to_string(), intent2, Sentiment::Neutral);
        db.store(conv2).unwrap();

        let intent3 = Intent::new("Order pizza".to_string())
            .with_topics(vec!["food".to_string()]);
        let conv3 = Conversation::new("conv3".to_string(), intent3, Sentiment::Positive);
        db.store(conv3).unwrap();

        let query_intent = Intent::new("Get weather".to_string())
            .with_topics(vec!["weather".to_string()]);
        let query = Conversation::new("query".to_string(), query_intent, Sentiment::Neutral);

        let matches = db.find_similar(&query);
        assert!(matches.len() >= 1);
        assert!(matches.iter().any(|(id, _)| id == "conv1" || id == "conv2"));
    }

    #[test]
    fn test_database_get_best_match() {
        let mut db = IntentDatabase::with_threshold(0.6);

        let intent1 = Intent::new("Get weather".to_string())
            .with_topics(vec!["weather".to_string()]);
        let conv1 = Conversation::new("conv1".to_string(), intent1, Sentiment::Neutral);
        db.store(conv1).unwrap();

        let intent2 = Intent::new("Order food".to_string())
            .with_topics(vec!["food".to_string()]);
        let conv2 = Conversation::new("conv2".to_string(), intent2, Sentiment::Positive);
        db.store(conv2).unwrap();

        let query_intent = Intent::new("Get weather forecast".to_string())
            .with_topics(vec!["weather".to_string()]);
        let query = Conversation::new("query".to_string(), query_intent, Sentiment::Neutral);

        let best_match = db.get_best_match(&query);
        assert!(best_match.is_some());
        let (matched_id, _score) = best_match.unwrap();
        assert_eq!(matched_id, "conv1");
    }

    #[test]
    fn test_database_cleanup_expired() {
        let mut db = IntentDatabase::new();

        let intent1 = Intent::new("Test 1".to_string());
        let expired_conv = Conversation::new("expired".to_string(), intent1, Sentiment::Neutral)
            .with_expiry(Utc::now() - Duration::hours(1));
        db.store(expired_conv).unwrap();

        let intent2 = Intent::new("Test 2".to_string());
        let active_conv = Conversation::new("active".to_string(), intent2, Sentiment::Neutral)
            .with_expiry(Utc::now() + Duration::hours(1));
        db.store(active_conv).unwrap();

        let intent3 = Intent::new("Test 3".to_string());
        let no_expiry_conv = Conversation::new("no-expiry".to_string(), intent3, Sentiment::Neutral);
        db.store(no_expiry_conv).unwrap();

        assert_eq!(db.len(), 3);

        let cleaned = db.cleanup_expired();
        assert_eq!(cleaned, 1);
        assert_eq!(db.len(), 2);
        assert!(db.get("expired").is_none());
        assert!(db.get("active").is_some());
        assert!(db.get("no-expiry").is_some());
    }

    #[test]
    fn test_database_clear() {
        let mut db = IntentDatabase::new();
        let intent = Intent::new("Test".to_string());
        let conv = Conversation::new("test-1".to_string(), intent, Sentiment::Neutral);

        db.store(conv).unwrap();
        assert_eq!(db.len(), 1);

        db.clear();
        assert_eq!(db.len(), 0);
        assert!(db.is_empty());
    }

    #[test]
    fn test_database_list_ids() {
        let mut db = IntentDatabase::new();

        let intent1 = Intent::new("Test 1".to_string());
        let conv1 = Conversation::new("id1".to_string(), intent1, Sentiment::Neutral);
        db.store(conv1).unwrap();

        let intent2 = Intent::new("Test 2".to_string());
        let conv2 = Conversation::new("id2".to_string(), intent2, Sentiment::Neutral);
        db.store(conv2).unwrap();

        let ids = db.list_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"id1".to_string()));
        assert!(ids.contains(&"id2".to_string()));
    }
}
