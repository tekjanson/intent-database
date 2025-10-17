# Intent Database

A Rust-based conversation caching and matching system designed to cache AI conversations and intelligently match them with future conversations based on intent, thoughts, and feelings rather than exact data matches.

## Overview

This library provides a fast, efficient way to:
- Cache AI conversations with semantic meaning (intent, sentiment, topics)
- Match new conversations with similar cached ones to potentially shortcut AI processing
- Implement fact invalidation patterns with expiry times
- Focus on thoughts, intent, and feelings rather than exact data

## Features

- **Intent-Based Matching**: Conversations are matched based on their purpose, topics, and context rather than exact text
- **Sentiment Tracking**: Capture and match conversations by emotional tone (Positive, Negative, Neutral, Mixed)
- **Fact Invalidation**: Set expiry times on conversations to automatically invalidate stale information
- **Similarity Scoring**: Get confidence scores (0-1) on how well conversations match
- **Flexible Storage**: In-memory database with easy CRUD operations
- **Comprehensive Testing**: Full test coverage for all core functionality

## Quick Start

### Running the Demo

```bash
cargo run
```

### Using as a Library

```rust
use intent_database::{Conversation, ConversationEntry, Intent, IntentDatabase, Sentiment};
use chrono::{Duration, Utc};

// Create a new database
let mut db = IntentDatabase::with_threshold(0.6);

// Create a conversation
let intent = Intent::new("Get weather forecast".to_string())
    .with_topics(vec!["weather".to_string(), "temperature".to_string()])
    .with_context_tags(vec!["casual".to_string()]);

let mut conv = Conversation::new(
    "conv-001".to_string(),
    intent,
    Sentiment::Neutral,
);

// Add conversation entries
conv.add_entry(ConversationEntry::new(
    "user".to_string(),
    "What's the weather like?".to_string(),
));

// Store in database
db.store(conv).unwrap();

// Query for similar conversations
let query_intent = Intent::new("Check weather".to_string())
    .with_topics(vec!["weather".to_string()]);
let query = Conversation::new("query".to_string(), query_intent, Sentiment::Neutral);

// Find matches
let matches = db.find_similar(&query);
for (conv_id, score) in matches {
    println!("Match: {} (score: {:.2})", conv_id, score.score);
}
```

## Architecture

### Core Components

1. **Conversation**: Represents a complete conversation with intent, sentiment, and message history
2. **Intent**: Captures the purpose, topics, and contextual tags of a conversation
3. **IntentDatabase**: Main storage system with matching and retrieval capabilities
4. **ConversationMatcher**: Implements similarity scoring algorithms

### Data Structures

- **Intent**: Purpose-driven conversation metadata
  - `purpose`: Primary goal or purpose
  - `topics`: Key topics discussed
  - `context_tags`: Contextual categorization

- **Conversation**: Complete conversation record
  - `id`: Unique identifier
  - `intent`: The conversation's intent
  - `sentiment`: Emotional tone
  - `entries`: Message history
  - `expires_at`: Optional expiry for fact invalidation
  - `metadata`: Flexible key-value storage

### Matching Algorithm

The matcher uses a weighted similarity calculation:
- **Purpose similarity** (50% weight): Text-based word overlap
- **Topic similarity** (30% weight): Common topics between conversations
- **Context similarity** (20% weight): Matching context tags
- **Sentiment bonus** (10%): Additional boost for matching sentiment

## Building and Testing

```bash
# Build the project
cargo build

# Run all tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Build for release
cargo build --release
```

## Use Cases

1. **AI Response Caching**: Cache frequently asked questions to reduce API calls
2. **Conversation Templates**: Store and reuse conversation patterns
3. **Intent Recognition**: Match user queries to known conversation patterns
4. **Semantic Search**: Find conversations by meaning, not just keywords
5. **Thought Databases**: Store conceptual knowledge with expiry patterns

## Why Rust?

This project is built in Rust because:
- **Performance**: Fast in-memory operations for real-time matching
- **Safety**: Strong type system prevents common bugs
- **Cool Factor**: Rust is indeed cool! 🦀

## Future Enhancements

Potential areas for expansion:
- Persistence layer (disk storage, database integration)
- Advanced NLP-based similarity (embeddings, transformers)
- Distributed caching support
- Query optimization and indexing
- REST API or gRPC interface
- Streaming conversation updates

## License

This project is open source and available for experimentation and learning.

## Contributing

This is an experimental project exploring novel ideas in conversation caching and intent matching. Contributions, ideas, and feedback are welcome!
