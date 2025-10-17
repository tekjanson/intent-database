use intent_database::funnel::{Funnel, StubEmbedder, StubModelAdapter};
use std::sync::Arc;

#[tokio::test]
async fn test_funnel_session_flow() {
    let funnel = Arc::new(Funnel::new("f1"));
    let embedder = Arc::new(StubEmbedder {});
    let model = Arc::new(StubModelAdapter {});

    let mut session = funnel.clone().activate(embedder, model).await;
    let q = session.next_question();
    assert!(q.is_some());
    assert_eq!(q.unwrap().id, "q1");

    session.submit_answer("billing").await.unwrap();
    assert!(session.next_question().is_none());
}
