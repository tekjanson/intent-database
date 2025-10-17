use chrono::{Duration, Utc};
use intent_database::{Conversation, ConversationEntry, Intent, IntentDatabase, Sentiment};

fn main() {
    println!("=== Basic Usage Example ===\n");

    // Load or create a database (demonstrates persistence)
    let db_path = "intent_db.json";
    let mut db = IntentDatabase::load_or_new(db_path, 0.65).expect("Failed to load or create DB");

    // Example 1: Simple conversation storage
    println!("1. Storing a simple conversation");
    let intent = Intent::new("Greet the user".to_string())
        .with_topics(vec!["greeting".to_string()])
        .with_context_tags(vec!["friendly".to_string()]);

    let mut conv = Conversation::new("greeting-001".to_string(), intent, Sentiment::Positive);
    conv.add_entry(ConversationEntry::new("user".to_string(), "Hello!".to_string()));
    conv.add_entry(ConversationEntry::new(
        "assistant".to_string(),
        "Hi there! How can I help you today?".to_string(),
    ));

    db.store(conv).expect("Failed to store conversation");
    println!("   ✓ Stored greeting conversation\n");

    // Example 2: Store with expiry
    println!("2. Storing a conversation with expiry");
    let temp_intent =
        Intent::new("Temporary query".to_string()).with_topics(vec!["temporary".to_string()]);

    let temp_conv = Conversation::new("temp-001".to_string(), temp_intent, Sentiment::Neutral)
        .with_expiry(Utc::now() + Duration::hours(1));

    db.store(temp_conv).expect("Failed to store temp conversation");
    println!("   ✓ Stored temporary conversation (expires in 1 hour)\n");

    // Example 3: Query for similar conversations
    println!("3. Finding similar conversations");
    let query_intent = Intent::new("Say hello".to_string())
        .with_topics(vec!["greeting".to_string()])
        .with_context_tags(vec!["friendly".to_string()]);

    let query = Conversation::new("query-001".to_string(), query_intent, Sentiment::Positive);

    let matches = db.find_similar(&query);
    println!("   Found {} matches", matches.len());
    for (id, score) in matches {
        println!("   - {} (score: {:.2}%)", id, score.score * 100.0);
    }
    println!();

    // Example 4: Get best match
    println!("4. Getting best match");
    if let Some((best_id, score)) = db.get_best_match(&query) {
        println!("   Best match: {} with {:.2}% similarity", best_id, score.score * 100.0);
        if let Some(conv) = db.get(&best_id) {
            println!("   Intent: {}", conv.intent.purpose);
            println!("   Entries: {}", conv.entries.len());
        }
    }
    println!();

    // Example 5: Update conversation
    println!("5. Updating a conversation");
    if let Some(mut conv) = db.get("greeting-001").cloned() {
        conv.add_metadata("updated".to_string(), "true".to_string());
        db.update(conv).expect("Failed to update conversation");
        println!("   ✓ Updated greeting conversation with metadata\n");
    }

    // Example 6: Cleanup expired
    println!("6. Cleaning up expired conversations");
    let expired_count = db.cleanup_expired();
    println!("   Cleaned up {} expired conversation(s)", expired_count);
    println!("   Database now contains {} conversations\n", db.len());

    // Example 7: List all conversations
    println!("7. Listing all conversation IDs");
    let ids = db.list_ids();
    for id in ids {
        println!("   - {}", id);
    }

    // Save DB to disk to demonstrate persistence
    if let Err(e) = db.save_to_file(db_path) {
        eprintln!("Warning: failed to save db to {}: {}", db_path, e);
    } else {
        println!("Saved database to {}\n", db_path);
    }

    println!("\n=== Example Complete ===");
}
