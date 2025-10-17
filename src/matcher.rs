//! Conversation matching and similarity scoring

use crate::conversation::{Conversation, Intent};

/// Represents a similarity score between two conversations
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct SimilarityScore {
    pub score: f64,
}

impl SimilarityScore {
    pub fn new(score: f64) -> Self {
        Self {
            score: score.clamp(0.0, 1.0),
        }
    }

    pub fn is_match(&self, threshold: f64) -> bool {
        self.score >= threshold
    }
}

/// Match conversations based on intent, topics, and context
pub struct ConversationMatcher {
    /// Minimum similarity threshold for considering a match
    pub match_threshold: f64,
}

impl ConversationMatcher {
    pub fn new(match_threshold: f64) -> Self {
        Self { match_threshold }
    }

    pub fn default() -> Self {
        Self {
            match_threshold: 0.6,
        }
    }

    /// Calculate similarity between two intents
    pub fn calculate_intent_similarity(&self, intent1: &Intent, intent2: &Intent) -> f64 {
        let mut similarity = 0.0;
        let mut weight_sum = 0.0;

        // Purpose similarity (weight: 0.5)
        let purpose_weight = 0.5;
        let purpose_similarity = self.calculate_text_similarity(&intent1.purpose, &intent2.purpose);
        similarity += purpose_similarity * purpose_weight;
        weight_sum += purpose_weight;

        // Topic similarity (weight: 0.3)
        let topic_weight = 0.3;
        let topic_similarity = self.calculate_vec_similarity(&intent1.topics, &intent2.topics);
        similarity += topic_similarity * topic_weight;
        weight_sum += topic_weight;

        // Context tag similarity (weight: 0.2)
        let context_weight = 0.2;
        let context_similarity =
            self.calculate_vec_similarity(&intent1.context_tags, &intent2.context_tags);
        similarity += context_similarity * context_weight;
        weight_sum += context_weight;

        if weight_sum > 0.0 {
            similarity / weight_sum
        } else {
            0.0
        }
    }

    /// Calculate similarity between two conversations
    pub fn calculate_similarity(&self, conv1: &Conversation, conv2: &Conversation) -> SimilarityScore {
        // Don't match expired conversations
        if conv1.is_expired() || conv2.is_expired() {
            return SimilarityScore::new(0.0);
        }

        let intent_similarity = self.calculate_intent_similarity(&conv1.intent, &conv2.intent);

        // Sentiment match bonus
        let sentiment_bonus = if conv1.sentiment == conv2.sentiment {
            0.1
        } else {
            0.0
        };

        let total_score = (intent_similarity + sentiment_bonus).min(1.0);
        SimilarityScore::new(total_score)
    }

    /// Find matching conversations from a list
    pub fn find_matches<'a>(
        &self,
        query: &Conversation,
        candidates: &'a [Conversation],
    ) -> Vec<(&'a Conversation, SimilarityScore)> {
        let mut matches: Vec<(&Conversation, SimilarityScore)> = candidates
            .iter()
            .filter(|c| c.id != query.id) // Don't match with self
            .map(|c| {
                let score = self.calculate_similarity(query, c);
                (c, score)
            })
            .filter(|(_, score)| score.is_match(self.match_threshold))
            .collect();

        // Sort by score descending
        matches.sort_by(|a, b| b.1.score.partial_cmp(&a.1.score).unwrap());
        matches
    }

    /// Simple text similarity using word overlap
    fn calculate_text_similarity(&self, text1: &str, text2: &str) -> f64 {
        let text1_lower = text1.to_lowercase();
        let text2_lower = text2.to_lowercase();
        let words1: Vec<&str> = text1_lower.split_whitespace().collect();
        let words2: Vec<&str> = text2_lower.split_whitespace().collect();

        if words1.is_empty() || words2.is_empty() {
            return 0.0;
        }

        let mut common_words = 0;
        for word1 in &words1 {
            if words2.contains(word1) {
                common_words += 1;
            }
        }

        let max_words = words1.len().max(words2.len()) as f64;
        common_words as f64 / max_words
    }

    /// Calculate similarity between two string vectors
    fn calculate_vec_similarity(&self, vec1: &[String], vec2: &[String]) -> f64 {
        if vec1.is_empty() && vec2.is_empty() {
            return 1.0;
        }
        if vec1.is_empty() || vec2.is_empty() {
            return 0.0;
        }

        let mut common_count = 0;
        for item1 in vec1 {
            if vec2.contains(item1) {
                common_count += 1;
            }
        }

        let max_len = vec1.len().max(vec2.len()) as f64;
        common_count as f64 / max_len
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversation::Sentiment;

    #[test]
    fn test_similarity_score() {
        let score = SimilarityScore::new(0.8);
        assert_eq!(score.score, 0.8);
        assert!(score.is_match(0.7));
        assert!(!score.is_match(0.9));
    }

    #[test]
    fn test_similarity_score_clamping() {
        let score1 = SimilarityScore::new(1.5);
        assert_eq!(score1.score, 1.0);

        let score2 = SimilarityScore::new(-0.5);
        assert_eq!(score2.score, 0.0);
    }

    #[test]
    fn test_text_similarity() {
        let matcher = ConversationMatcher::default();

        let sim1 = matcher.calculate_text_similarity("hello world", "hello world");
        assert_eq!(sim1, 1.0);

        let sim2 = matcher.calculate_text_similarity("hello world", "goodbye moon");
        assert_eq!(sim2, 0.0);

        let sim3 = matcher.calculate_text_similarity("hello world", "hello there");
        assert!(sim3 > 0.0 && sim3 < 1.0);
    }

    #[test]
    fn test_intent_similarity() {
        let matcher = ConversationMatcher::default();

        let intent1 = Intent::new("Get weather forecast".to_string())
            .with_topics(vec!["weather".to_string(), "temperature".to_string()])
            .with_context_tags(vec!["casual".to_string()]);

        let intent2 = Intent::new("Get weather forecast".to_string())
            .with_topics(vec!["weather".to_string(), "temperature".to_string()])
            .with_context_tags(vec!["casual".to_string()]);

        let similarity = matcher.calculate_intent_similarity(&intent1, &intent2);
        assert_eq!(similarity, 1.0);
    }

    #[test]
    fn test_conversation_similarity() {
        let matcher = ConversationMatcher::new(0.5);

        let intent1 = Intent::new("Get weather".to_string())
            .with_topics(vec!["weather".to_string()]);

        let intent2 = Intent::new("Get weather".to_string())
            .with_topics(vec!["weather".to_string()]);

        let conv1 = Conversation::new("c1".to_string(), intent1, Sentiment::Neutral);
        let conv2 = Conversation::new("c2".to_string(), intent2, Sentiment::Neutral);

        let score = matcher.calculate_similarity(&conv1, &conv2);
        assert!(score.score > 0.8);
    }

    #[test]
    fn test_find_matches() {
        let matcher = ConversationMatcher::new(0.6);

        let query_intent = Intent::new("Get weather information".to_string())
            .with_topics(vec!["weather".to_string(), "forecast".to_string()]);
        let query = Conversation::new("query".to_string(), query_intent, Sentiment::Neutral);

        let match_intent = Intent::new("Get weather information".to_string())
            .with_topics(vec!["weather".to_string(), "forecast".to_string()]);
        let matching_conv = Conversation::new("match".to_string(), match_intent, Sentiment::Neutral);

        let no_match_intent = Intent::new("Order pizza".to_string())
            .with_topics(vec!["food".to_string()]);
        let no_match_conv = Conversation::new("nomatch".to_string(), no_match_intent, Sentiment::Positive);

        let candidates = vec![matching_conv, no_match_conv];
        let matches = matcher.find_matches(&query, &candidates);

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].0.id, "match");
    }
}
