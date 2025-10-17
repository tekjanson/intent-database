Intent Database - Funnel Design

Overview
--------
This document captures the design for the "funnel" system: a learned, AI-assisted routing layer that narrows candidate data via a small sequence of clarifying questions (20-questions style) and routes queries to small candidate sets for final similarity matching.

Goals
-----
- Reduce AI calls by quickly narrowing the candidate set.
- Persist funnel metadata and routing state.
- Support high concurrency and async model adapters.
- Provide safe AI-generated question templates and extractor rules.

Architecture
------------
- Storage layer: pluggable `Storage` trait. Default implementations:
  - `SledStorage` (disk-backed, durable)
  - `InMemoryStorage` (fast tests, ephemeral)
- Vector index: store embeddings for conversations (HNSW or rebuildable in-memory index in prototype).
- Funnel: hierarchical nodes with centroids/classifiers, question specs, and candidate references.
- Model adapters: async trait for calling external LLM or local model for question generation and rephrasing.
- Extractor runtime: small DSL for deterministic extraction (keyword match, regex, numeric compare). Optionally WASM sandbox for more complex logic.

Data shapes
-----------
- Funnel: serialized via `serde` + `bincode`.
- QuestionSpec: `id, prompt, answer_type, extractor`.

Persistence
-----------
- Funnel metadata persisted as blobs via `Storage::put_blob`.
- Vectors persisted either in a vector index format or as per-item blobs; vector index can be rebuilt on startup from blobs.

Concurrency
-----------
- All public APIs are async and non-blocking.
- Sled calls are run inside `spawn_blocking` tasks.
- Background jobs (indexing, compaction) run on bounded worker queues.

Testing
-------
- Unit tests for storage, funnel session flows, and async DB wrapper.
- Integration tests using `tempfile` and `SledStorage`.

Roadmap
-------
1. Phase 0: async APIs, storage trait, funnel prototype (done)
2. Phase 1: persist funnel, integrate vector index, add tests (in progress)
3. Phase 2: HNSW integration, WASM extractor, performance hardening
4. Phase 3: production deployment patterns and monitoring
*** End Patch