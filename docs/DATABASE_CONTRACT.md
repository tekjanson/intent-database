# Intent Database - Storage Contract

This document describes the storage keys, blob formats, and async semantics
used by the `intent-database` library. It is intentionally minimal and
explicit so other systems can interoperate with the sled-backed storage.

Storage keys
------------
- Funnels: `funnel:builder:{funnel_id}` — stores a bincode-serialized `Funnel` struct.
- Funnel nodes: `funnel:node:{node_id}` — stores a bincode-serialized `FunnelNode`.
- Conversations and other blobs: (future) prefixed with `conv:` or `blob:`.

Blob formats
------------
- All runtime objects persisted by the library are serialized using `bincode`.
- The on-disk representation is opaque; rely on the typed APIs (`get_blob` / `put_blob`)
  for correctness. Consumers that need to read blobs directly should use the
  library's deserialization routines.

Async semantics
---------------
- All public APIs are async and non-blocking. Blocking sled calls are executed
  via `spawn_blocking` inside the storage adapter. Callers should `.await` the
  returned `Future` and handle errors explicitly.

Error modes
-----------
- Storage errors are returned as `Err(String)` from storage and higher-level
  APIs. The system avoids silent failures: errors must be handled by callers.

Compatibility and migration
---------------------------
- Sled is the single canonical backend. There are no in-memory fallbacks for
  persistent operations in the production code paths.
- When changing persisted struct fields, prefer additive and optional changes
  and provide a migration path in library releases.

Examples
--------
Persist a funnel (pseudocode):

```rust
let funnel = Funnel::new("f1");
let bytes = bincode::serialize(&funnel).unwrap();
storage.put_blob(&format!("funnel:builder:{}", funnel.id), &bytes).await?;
```

Auditability
------------
- Because blobs are deterministic (bincode) and storage uses a single backend,
  local audits are straightforward: read the blob and deserialize to verify
  invariants.

Contact
-------
For schema changes or migration guidance, open an issue in the repository and
tag the `storage` and `migration` teams.
# Intent Database Contract

This document describes the storage and runtime contract for the intent-database crate.
It covers storage key formats, async semantics, error modes, persistence guarantees, and example usage.

## Storage key conventions
- Conversations (raw persisted copy): `conv:<conversation_id>` -> bincode(serialized Conversation)
- Funnels created by builders: `funnel:builder:<funnel_id>` -> bincode(serialized Funnel)
- Funnel nodes: `funnel:node:<node_id>` -> bincode(serialized FunnelNode)
- Backups / exports: `export:db:<timestamp>` -> JSON blob (human readable)

Keys are plain UTF-8 strings and are expected to be stable across versions. If you change
a key prefix in a future release, include a migration strategy in release notes.

## Async semantics
- All public storage operations on `Storage` are async and return `Result<_, String>`.
- Implementations must avoid blocking the async runtime. The provided `SledStorage` uses
  `tokio::task::spawn_blocking` internally to run blocking DB operations.
- Public APIs that call storage (for example `AsyncIntentDatabase::persist` and
  `ClusterFunnelBuilder::build_and_persist`) must propagate storage errors as `Err(String)`.

## Consistency & durability
- `SledStorage::put_blob` flushes the DB (calls `flush`) after inserts to provide stronger
  durability guarantees; callers should expect the write to be durable on success.
- There is no cross-key transaction support. Multi-key invariants (for example, funnel
  document pointing to several node documents) are written in sequence. Tests assume the
  builder writes nodes first and the funnel doc last.

## Error modes and expectations
- Storage errors (I/O, serialization) are returned to callers as `Err(String)` and must be
  handled by higher-level code; builders do not attempt complex retries by default.
- Embedding failures (for example, ModelAdapter errors) are likewise bubbled up as `Err(String)`.
- Tests may simulate failing storage implementations to assert error propagation behavior.

## Versioning and migrations
- Blob shapes (serialized structs) are raw `bincode` by default. For long-term stability you
  may switch to JSON or versioned protobuf/fbs. When changing struct fields, add migration
  helpers and consider storing a `schema_version` in the blob or as a separate key.

## Examples
Persisting a conversation (async):

```rust
let storage: Arc<dyn Storage> = Arc::new(SledStorage::new(path_buf));
let conv = Conversation::new("id", Intent::new("purpose"), Sentiment::Neutral);
let bytes = bincode::serialize(&conv)?;
storage.put_blob(&format!("conv:{}", conv.id), &bytes).await?;
```

Building and persisting a funnel (async):

```rust
let builder = ClusterFunnelBuilder::new(4, 100);
let embedder = Arc::new(HashEmbedder::new(32));
let funnel = builder.build_and_persist("my-funnel", &convs, embedder, storage.clone()).await?;
```

## Migration notes
- When you modify serialized structs, provide a small CLI migration utility that reads
  old blobs, transforms them to the new struct, and writes them back under the same key.

## Testing recommendations
- Use `tempfile::TempDir` and per-test `SledStorage` (backed by a temp path) for isolation.
- Use deterministic embedders (the provided `HashEmbedder`) for reproducible clustering tests.
- Create failing storage mocks to assert error propagation.
