use crate::connector::ChatConnector;
use crate::conversation::Conversation;
use crate::database::AsyncIntentDatabase;
use async_trait::async_trait;
use std::sync::Arc;

/// Result from a short-circuit attempt. If `matched` is Some, it contains the
/// ID of the stored conversation and the similarity score. If None, no match
/// was found and the inner connector was called; `feedback` contains the
/// boolean response from the underlying connector.
pub struct ShortCircuitResult {
    pub matched: Option<(
        String,
        crate::matcher::SimilarityScore,
        Option<crate::conversation::Conversation>,
    )>,
    pub feedback: bool,
}

/// A connector that can short-circuit to the database when a similar
/// conversation exists to avoid invoking the model/connector. Use
/// `send_and_get_feedback_and_maybe_short_circuit` to obtain the richer result
/// (matched id + feedback); `send_and_get_feedback` still implements the
/// trait and will return the boolean (true if matched or inner returned true).
pub struct ShortCircuitingConnector<C> {
    pub db: Arc<AsyncIntentDatabase>,
    pub inner: Arc<C>,
    /// Minimum similarity score required to consider a DB match. If None, use
    /// the database's default threshold.
    pub min_score: Option<f64>,
}

impl<C> ShortCircuitingConnector<C> {
    pub fn new(db: Arc<AsyncIntentDatabase>, inner: Arc<C>) -> Self {
        Self { db, inner, min_score: None }
    }

    /// Attempt to short-circuit: if a matching conversation exists in DB,
    /// return it (matched) and feedback=true without calling inner. Otherwise
    /// call the inner connector and return its feedback with matched=None.
    pub async fn send_and_get_feedback_and_maybe_short_circuit(
        &self,
        conv: &Conversation,
    ) -> Result<ShortCircuitResult, String>
    where
        C: ChatConnector + 'static,
    {
        if let Some((id, score)) = self.db.get_best_match(conv).await? {
            // check min_score if configured
            if let Some(min) = self.min_score {
                if score.score < min {
                    // not sufficient; fallthrough to inner connector
                } else {
                    let conv_opt = self.db.get_conversation_by_id(&id).await?;
                    return Ok(ShortCircuitResult {
                        matched: Some((id, score, conv_opt)),
                        feedback: true,
                    });
                }
            } else {
                let conv_opt = self.db.get_conversation_by_id(&id).await?;
                return Ok(ShortCircuitResult {
                    matched: Some((id, score, conv_opt)),
                    feedback: true,
                });
            }
        }

        let fb = self.inner.send_and_get_feedback(conv).await?;
        Ok(ShortCircuitResult { matched: None, feedback: fb })
    }
}

#[async_trait]
impl<C: ChatConnector + 'static> ChatConnector for ShortCircuitingConnector<C> {
    async fn send_and_get_feedback(&self, conv: &Conversation) -> Result<bool, String> {
        Ok(self.send_and_get_feedback_and_maybe_short_circuit(conv).await?.feedback)
    }
}
