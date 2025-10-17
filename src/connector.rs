use crate::conversation::Conversation;
use async_trait::async_trait;

#[async_trait]
pub trait ChatConnector: Send + Sync {
    /// Send a conversation to the chat system; returns true if user gave thumbs-up
    async fn send_and_get_feedback(&self, conv: &Conversation) -> Result<bool, String>;
}

/// A mock connector that simulates feedback (configurable)
pub struct MockConnector {
    pub positive: bool,
}

#[async_trait]
impl ChatConnector for MockConnector {
    async fn send_and_get_feedback(&self, _conv: &Conversation) -> Result<bool, String> {
        Ok(self.positive)
    }
}
