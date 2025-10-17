# Funnel Builder Contract

This document defines the public contract for the ClusterFunnelBuilder and related
funnel persistence behavior. Tests should exhaust permutations of these contracts.

Data shapes
- Funnel: { id: String, description: Option<String>, node_ids: Vec<String> }
- FunnelNode: { id: String, parent_id: Option<String>, centroid: Option<Vec<f32>>, example_ids: Vec<String>, question: Option<QuestionSpec> }

ClusterFunnelBuilder::build_and_persist contract
- Inputs:
  - id: &str (name for funnel)
  - convs: &[Conversation] (may be empty)
  - embedder: Arc<dyn Embedder> (async)
  - storage: Arc<dyn Storage> (async)
- Behavior:
  - If convs is empty: persist an empty Funnel (node_ids empty) under `funnel:builder:<id>`.
  - Compute embeddings via embedder.embed(intent.purpose) for each conversation.
  - Normalize embeddings (unit vectors). If an embedding is zero vector, leave as-is.
  - Run spherical k-means (cosine) with deterministic initialization.
  - Create FunnelNode objects with centroid (normalized), example_ids, and persist each under `funnel:node:<node_id>`.
  - Persist Funnel under `funnel:builder:<id>`.
- Outputs:
  - On success: returns Funnel with node_ids matching persisted nodes.
  - On storage write failure: returns Err and no partial success guaranteed.

Error modes
- Storage failures bubble up as Err(String).
- Embedding failures bubble up as Err(String).

Success criteria for tests
- For each permutation of:
  - Embedders: deterministic hash embedder, identical embedder, zero embedder
  - k choices: 1, 2, 3 (k > n allowed)
  - Storage: sled-backed (working) and a failing storage
  - Input datasets: empty, small distinct, identical intent texts
- Validate the expected outcomes described above.
