use chrono::{Duration, Utc};
use intent_database::{Conversation, ConversationEntry, Intent, IntentDatabase, Sentiment};

fn main() {
    println!("Intent Database - AI Conversation Caching Demo\n");

    // Create a new database
    let mut db = IntentDatabase::with_threshold(0.6);

    // Create and store some conversations
    println!("Storing conversations...");

    // Conversation 1: Weather query
    let weather_intent = Intent::new("Get weather forecast".to_string())
        .with_topics(vec!["weather".to_string(), "temperature".to_string()])
        .with_context_tags(vec!["casual".to_string()]);

    let mut weather_conv =
        Conversation::new("conv-001".to_string(), weather_intent, Sentiment::Neutral);
    weather_conv.add_entry(ConversationEntry::new(
        "user".to_string(),
        "What's the weather like today?".to_string(),
    ));
    weather_conv.add_entry(ConversationEntry::new(
        "assistant".to_string(),
        "It's sunny with a high of 75°F.".to_string(),
    ));

    db.store(weather_conv).unwrap();
    println!("✓ Stored weather conversation (conv-001)");

    // Conversation 2: Another weather query
    let weather_intent2 = Intent::new("Get weather information".to_string())
        .with_topics(vec!["weather".to_string(), "forecast".to_string()])
        .with_context_tags(vec!["casual".to_string()]);

    let mut weather_conv2 =
        Conversation::new("conv-002".to_string(), weather_intent2, Sentiment::Neutral);
    weather_conv2.add_entry(ConversationEntry::new(
        "user".to_string(),
        "Tell me about the weather".to_string(),
    ));
    weather_conv2.add_entry(ConversationEntry::new(
        "assistant".to_string(),
        "The forecast shows clear skies.".to_string(),
    ));

    db.store(weather_conv2).unwrap();
    println!("✓ Stored weather conversation (conv-002)");

    // Conversation 3: Food order with expiry
    let food_intent = Intent::new("Order pizza".to_string())
        .with_topics(vec!["food".to_string(), "pizza".to_string()])
        .with_context_tags(vec!["transactional".to_string()]);

    let mut food_conv = Conversation::new("conv-003".to_string(), food_intent, Sentiment::Positive)
        .with_expiry(Utc::now() + Duration::hours(24));

    food_conv.add_entry(ConversationEntry::new(
        "user".to_string(),
        "I want to order a pizza".to_string(),
    ));
    food_conv.add_entry(ConversationEntry::new(
        "assistant".to_string(),
        "Sure! What toppings would you like?".to_string(),
    ));

    db.store(food_conv).unwrap();
    println!("✓ Stored food order conversation (conv-003) with 24h expiry");

    println!("\nDatabase contains {} conversations\n", db.len());

    // Now try to match a new query
    println!("Searching for similar conversations...");
    let query_intent = Intent::new("Check weather".to_string())
        .with_topics(vec!["weather".to_string()])
        .with_context_tags(vec!["casual".to_string()]);

    let query = Conversation::new("query-001".to_string(), query_intent, Sentiment::Neutral);

    let matches = db.find_similar(&query);

    println!("\nFound {} matching conversations:", matches.len());
    for (conv_id, score) in &matches {
        if let Some(conv) = db.get(conv_id) {
            println!(
                "  - {} (similarity: {:.2}%) - Intent: {}",
                conv.id,
                score.score * 100.0,
                conv.intent.purpose
            );
        }
    }

    // Get the best match
    if let Some((best_id, score)) = db.get_best_match(&query) {
        if let Some(best) = db.get(&best_id) {
            println!("\n🎯 Best match: {} with {:.2}% similarity", best.id, score.score * 100.0);
            println!("   Intent: {}", best.intent.purpose);
            println!("   Topics: {:?}", best.intent.topics);
            println!("   Entries: {} messages", best.entries.len());
        }
    }

    // Demo fact invalidation
    println!("\n\nDemonstrating fact invalidation...");
    let expired_intent = Intent::new("Old fact".to_string());
    let expired_conv =
        Conversation::new("conv-expired".to_string(), expired_intent, Sentiment::Neutral)
            .with_expiry(Utc::now() - Duration::hours(1));

    db.store(expired_conv).unwrap();
    println!("Added expired conversation. Database now has {} conversations", db.len());

    let cleaned = db.cleanup_expired();
    println!(
        "Cleaned up {} expired conversation(s). Database now has {} conversations",
        cleaned,
        db.len()
    );

    println!("\n✨ Demo complete!");
}
