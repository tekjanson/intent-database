Intent Database - Single Canonical Way

Philosophy
----------
This repository enforces a single way to operate:

- Only one storage backend: sled (embedded KV). No in-memory fallbacks or alternative persistence.
- Minimal, deterministic logic in runtime. The system is explicit and auditable.
- All public APIs are async and non-blocking (Tokio). Blocking IO uses `spawn_blocking`.
- AI may generate question *templates* and extractor DSL expressions only; no arbitrary generated code execution.

Developer Guidelines
--------------------
- Use `sled` for all persistence. Store blobs using `Storage::put_blob`.
- Keep logic declarative. Prefer data-driven rules over code paths.
- Tests must use `tempfile` backed sled databases. No in-memory tests.
- Format with `cargo fmt` and run `cargo clippy -- -D warnings` before PR.

Testing philosophy
------------------
- We enforce "no fallbacks": tests must use sled-backed temporary databases (via `tempfile`).
- Tests should cover concurrency, large payloads, invalid/corrupt blobs, and recovery scenarios.
- Add tests for funnel edge cases, async concurrency, and model adapter failures.

Repository patterns learned
-------------------------
- Single canonical storage: `SledStorage` is the only supported persistence implementation. Do not add in-memory or alternate backends.
- All IO must be async-safe: wrap blocking calls in `tokio::task::spawn_blocking`.
- AI-generated artifacts must be declarative templates or DSL expressions, never arbitrary code execution.

Repo tools
----------
- `Makefile` targets: `fmt`, `lint`, `test`, `ci`.
- GitHub Actions: runs fmt check, clippy, and tests on PRs.

Design
------
See `DESIGN.md` for funnel architecture and roadmap.
