//! Conversation data structures representing intents, thoughts, and feelings

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the intent or purpose behind a conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Intent {
    /// The primary goal or purpose
    pub purpose: String,
    /// Key topics discussed
    pub topics: Vec<String>,
    /// Contextual tags for categorization
    pub context_tags: Vec<String>,
}

impl Intent {
    pub fn new(purpose: String) -> Self {
        Self {
            purpose,
            topics: Vec::new(),
            context_tags: Vec::new(),
        }
    }

    pub fn with_topics(mut self, topics: Vec<String>) -> Self {
        self.topics = topics;
        self
    }

    pub fn with_context_tags(mut self, tags: Vec<String>) -> Self {
        self.context_tags = tags;
        self
    }
}

/// Represents emotional sentiment in conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Sentiment {
    Positive,
    Negative,
    Neutral,
    Mixed,
}

/// A single entry in a conversation (user message or AI response)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationEntry {
    /// Role of the speaker (user, assistant, system)
    pub role: String,
    /// The actual message content
    pub content: String,
    /// Timestamp of the entry
    pub timestamp: DateTime<Utc>,
}

impl ConversationEntry {
    pub fn new(role: String, content: String) -> Self {
        Self {
            role,
            content,
            timestamp: Utc::now(),
        }
    }
}

/// Represents a complete conversation with intent and sentiment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    /// Unique identifier for the conversation
    pub id: String,
    /// The intent behind this conversation
    pub intent: Intent,
    /// Overall sentiment of the conversation
    pub sentiment: Sentiment,
    /// The conversation entries (messages)
    pub entries: Vec<ConversationEntry>,
    /// When this conversation was created
    pub created_at: DateTime<Utc>,
    /// When this conversation expires (for fact invalidation)
    pub expires_at: Option<DateTime<Utc>>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl Conversation {
    pub fn new(id: String, intent: Intent, sentiment: Sentiment) -> Self {
        Self {
            id,
            intent,
            sentiment,
            entries: Vec::new(),
            created_at: Utc::now(),
            expires_at: None,
            metadata: HashMap::new(),
        }
    }

    pub fn add_entry(&mut self, entry: ConversationEntry) {
        self.entries.push(entry);
    }

    pub fn with_expiry(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expiry) = self.expires_at {
            Utc::now() > expiry
        } else {
            false
        }
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_intent_creation() {
        let intent = Intent::new("Get weather information".to_string())
            .with_topics(vec!["weather".to_string(), "forecast".to_string()])
            .with_context_tags(vec!["casual".to_string()]);

        assert_eq!(intent.purpose, "Get weather information");
        assert_eq!(intent.topics.len(), 2);
        assert_eq!(intent.context_tags.len(), 1);
    }

    #[test]
    fn test_conversation_entry() {
        let entry = ConversationEntry::new("user".to_string(), "Hello!".to_string());
        assert_eq!(entry.role, "user");
        assert_eq!(entry.content, "Hello!");
    }

    #[test]
    fn test_conversation_expiry() {
        let intent = Intent::new("Test".to_string());
        let mut conv = Conversation::new("test-1".to_string(), intent, Sentiment::Neutral);

        assert!(!conv.is_expired());

        let expiry = Utc::now() - Duration::hours(1);
        conv = conv.with_expiry(expiry);
        assert!(conv.is_expired());

        let future_expiry = Utc::now() + Duration::hours(1);
        conv = conv.with_expiry(future_expiry);
        assert!(!conv.is_expired());
    }

    #[test]
    fn test_conversation_entries() {
        let intent = Intent::new("Chat".to_string());
        let mut conv = Conversation::new("test-2".to_string(), intent, Sentiment::Positive);

        conv.add_entry(ConversationEntry::new(
            "user".to_string(),
            "Hello".to_string(),
        ));
        conv.add_entry(ConversationEntry::new(
            "assistant".to_string(),
            "Hi there!".to_string(),
        ));

        assert_eq!(conv.entries.len(), 2);
        assert_eq!(conv.entries[0].role, "user");
        assert_eq!(conv.entries[1].role, "assistant");
    }
}
