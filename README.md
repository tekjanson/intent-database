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
    # Intent Database

    An experimental Rust library and in-memory store for caching, matching, and replaying "intentful" conversations.

    This project is intentionally small and focused: instead of storing raw text blobs and re-querying an LLM for every request, the intent-database stores conversations annotated by purpose (intent), topics, context tags, sentiment and message history — then matches new queries to past conversations by intent and structure. The goal is to allow programmatic replay, templating and light-weight simulation of AI behaviour for common flows.

    ## What this repo provides

    - A lightweight in-memory IntentDatabase suitable for caching conversation patterns
    - A matching layer that compares Intent objects (purpose, topics, tags) and conversation metadata to compute a similarity score
    - Expiry/invalidation support so cached facts can be time-limited
    - Example usage showing how to store and query conversation templates

    ## Design contract (short)

    - Inputs: Conversation records (id, Intent, sentiment, entries, optional expiry, metadata)
    - Outputs: Match results: (conversation id, similarity score 0.0–1.0) and access to stored Conversation data
    - Error modes: store/find operations return Result types; expired entries are treated as non-matches
    - Success: a match score above a configurable threshold indicates a likely reusable conversation/template

    Edge cases handled: missing topics, empty entries, expired records, and configurable similarity thresholds.

    ## Why this approach

    Many practical AI interactions are repetitive and templatable. By focusing on the developer-visible intent and replaying prior agent behaviour with data substitution, you can reduce external LLM calls, provide deterministic behaviour for common flows, and prototype "fake" AI agents that are fast, free, and auditable.

    This is not a replacement for full NLP/embedding systems. Instead it's a pragmatic cache + matching layer that sits in front of or alongside LLMs.

    ## Quick start

    Prerequisites: Rust toolchain (stable) and Cargo.

    To build the project and run the demo:

    ```bash
    cargo build
    cargo run --example basic_usage
    ```

    Persistence

    This crate now supports simple file-backed persistence using JSON. The
    database can be saved to and loaded from disk with `save_to_file` and
    `load_from_file` (or `load_or_new` to load if present or create a new DB).

    Example:

    ```rust
    let db_path = "intent_db.json";
    let mut db = IntentDatabase::load_or_new(db_path, 0.6)?;
    // ... store/update conversations ...
    db.save_to_file(db_path)?;
    ```

    To use the library from your code (example):

    ```rust
    use intent_database::{Conversation, ConversationEntry, Intent, IntentDatabase, Sentiment};

    // Create DB with a threshold for what you consider a good match (0.0 - 1.0)
    let mut db = IntentDatabase::with_threshold(0.6);

    // Build an intent describing the user's purpose and important topics
    let intent = Intent::new("Get weather forecast".to_string())
        .with_topics(vec!["weather".to_string(), "temperature".to_string()])
        .with_context_tags(vec!["casual".to_string()]);

    let mut conv = Conversation::new("conv-001".to_string(), intent, Sentiment::Neutral);
    conv.add_entry(ConversationEntry::new("user".to_string(), "What's the weather like?".to_string()));

    // Store and later query
    db.store(conv)?;

    let q_intent = Intent::new("Check weather".to_string()).with_topics(vec!["weather".to_string()]);
    let query = Conversation::new("q".to_string(), q_intent, Sentiment::Neutral);

    let matches = db.find_similar(&query);
    for m in matches {
        println!("Matched: {} (score: {:.2})", m.id, m.score);
    }
    ```

    See `examples/basic_usage.rs` for a runnable demonstration of storing, matching and replaying simple conversations.

    ## Implementation notes

    - Matching is intentionally heuristic and lightweight: purpose overlap, topic overlap and context tags are weighted to form a single similarity score. Sentiment can optionally boost matches.
    - The database is in-memory for simplicity. Adding persistence or a disk-backed store is a clear next step.
    - The API favors deterministic, testable behaviour over opaque ML models.

    ## Running tests and lint

    ```bash
    cargo test
    cargo test -- --nocapture
    ```

    ## Next steps and roadmap

    1. Add optional persistence (sled / sqlite / rocksdb)
    2. Optional embedding-based similarity as a pluggable matcher
    3. Tools for templating and safe data substitution when replaying conversations
    4. Small HTTP/gRPC shim for easy integration

    ## Contributing

    Contributions are welcome. Please open issues for design discussions and PRs for small, well-scoped improvements.

    ## License

    This project is provided for experimentation and learning. See the repository license (if any) for details.
