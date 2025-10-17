# Using ModelAdapter and Embedder

This document shows how to plug a ModelAdapter (question generator / LLM wrapper) and
an Embedder into the funnel builder and how to run a funnel session.

1. Implement or reuse an `Embedder` (async) that converts text to Vec<f32>. For tests, use
   `HashEmbedder` or `StubEmbedder`.

2. Implement a `ModelAdapter` (async) that wraps a real LLM or a deterministic template.
   The project includes `TemplateModelAdapter` for deterministic examples.

3. Build a funnel using embeddings and the `ClusterFunnelBuilder`:

```rust
let convs: Vec<Conversation> = ...;
let embedder = Arc::new(HashEmbedder::new(32));
let builder = ClusterFunnelBuilder::new(4, 100);
let funnel = builder.build_and_persist("my-funnel", &convs, embedder.clone(), storage.clone()).await?;
```

4. Activate a funnel session with an embedder and model adapter:

```rust
let model = Arc::new(TemplateModelAdapter::new());
let session = Arc::new(funnel).activate(embedder.clone(), model.clone()).await;
while let Some(q) = session.next_question() {
    // present q.prompt to the user or a test harness
    session.submit_answer("some answer").await?;
}
```

Notes
- Keep ModelAdapter and Embedder trait boundaries thin; tests should provide simple implementations
  for determinism.
- For production, use an async HTTP-based ModelAdapter behind a feature flag and guard tests with
  recorded responses or mock servers.
