use intent_database::funnel::{Funnel, StubEmbedder, StubModelAdapter};
use std::sync::Arc;

#[tokio::test]
async fn test_funnel_empty_flow() {
    let funnel = Arc::new(Funnel::new("empty"));
    let embedder = Arc::new(StubEmbedder {});
    let model = Arc::new(StubModelAdapter {});
    let mut session = funnel.clone().activate(embedder, model).await;

    // consume question
    let _q = session.next_question();
    session.submit_answer("x").await.unwrap();

    // repeated answers should be allowed but not crash
    let res = session.submit_answer("another").await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_funnel_long_prompt_handling() {
    let funnel = Arc::new(Funnel::new("long"));
    let embedder = Arc::new(StubEmbedder {});
    let model = Arc::new(StubModelAdapter {});
    let mut session = funnel.clone().activate(embedder, model).await;

    // Ensure long answer strings are handled
    let long_answer = "a".repeat(10000);
    session.submit_answer(&long_answer).await.unwrap();
}
