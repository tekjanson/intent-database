# Funnel Design (Overview)

This document sketches the funnel design for the Intent Database. It's a compact, focused
plan to replace the current `SimpleFunnelBuilder` prototype with a more capable,
learnable funnel that can route queries to intent clusters efficiently.

Goals
- Reduce LLM calls by routing queries through a short question flow.
- Persist funnel metadata and representative examples so funnel activation is quick.
- Keep logic declarative; training and heavy tasks run offline or in background workers.

Data shapes (serializable)
- Funnel: id, description, root_node_id, metadata
- FunnelNode: id, parent_id, question_spec, centroid (optional), example_ids
- QuestionSpec: id, prompt_template, answer_type (choice/free), choices

Runtime
- Activation: compute embedding for query, walk tree using centroid distance and question answers.
- Question generation: ModelAdapter provides generation; templates stored in FunnelNode.question_spec.
- Answers are recorded and used to select child node.

Training / Builder
- Builder consumes labeled conversations and produces a tree via clustering (kmeans) or
  hierarchical clustering.
- Each node stores centroid vector computed from example embeddings, and a small set of
  representative examples (ids) persisted to storage via `Storage::put_blob`.

Edge cases
- Sparse topics: fallback to global similarity search when no confident node match.
- Drift: background worker periodically retrains or rebalances nodes.
- Corrupt blobs: storage get_blob errors must be handled; builder should skip corrupt examples.

Safety
- All blocking disk ops use `spawn_blocking`.
- No arbitrary code execution from model-generated templates. Templates are data only.

Next steps
1. Implement Node and Funnel persistence schema in `src/funnel.rs`.
2. Replace `SimpleFunnelBuilder` with `ClusterFunnelBuilder` using an offline clustering step
   (for now, simple k-means over embedding vectors). Use `tempfile`-backed sled DBs in tests.
3. Add background worker primitives to schedule retraining.
4. Add tests: builder correctness, activation routing, storage persistence and corrupt blob
   handling.

