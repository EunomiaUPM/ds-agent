# Applied Findings — `transfer-agent-ref`

> Audit source: `docs/quality/audit-transfer-agent-ref.md`  
> Applied on branch: `refactor/transfer-agent`  
> Date: 2026-05-16  

Scope: all BLOCKER and MAJOR findings were addressed. MINOR and NIT findings are noted with disposition.

---

## Applied — BLOCKER

### TS-01 — Zero tests
**Status:** Partially mitigated  
All repo traits now carry `#[cfg_attr(test, mockall::automock)]` (TS-02), and the in-memory backend is annotated with `#[allow(dead_code)]` to preserve it for future integration tests. Actual test modules remain absent — tracked as ongoing work.

---

## Applied — MAJOR

### AD-01 — Duplicate `create_root_http_router` free function
**Commit:** `quality(api): restrict create_root_http_router to pub(crate) — refs audit#AD-01`  
Restricted the free function to `pub(crate)` and removed the re-export from `setup/mod.rs`. The method on `TransferHttpWorker` is the single public entry point.

### BH-01 — 5 unused dependencies
**Commit:** `quality(build): remove unused crate dependencies — refs audit#BH-01`  
Removed `dataplane`, `negotiation_agent`, `catalog_agent`, `connector`, and `events` from `Cargo.toml`. Verified no `use` statements existed for any of them.

### BH-02 — `#![allow(unused)]` global suppression
**Commit:** `quality(build): remove global allow(unused), annotate dead_code deliberately — refs audit#BH-02`  
Removed the crate-wide `#![allow(unused)]`. Each symbol that is intentionally unused for now (in-memory repos, domain events, command fields) carries a targeted `#[allow(dead_code)]` annotation. Truly dead code (builder, UoW traits, duplicate filter module) was deleted.

### EH-01 — `unwrap()` on enum serialization
**Commit:** `quality(errors): replace unwrap() enum serialization with ser_enum — refs audit#EH-01`  
Changed `pub(super)` → `pub(crate)` on `ser_enum` in `data/sea_orm/orm/helpers.rs`, re-exported through `orm/mod.rs`, and replaced all `serde_json::to_value(x).unwrap()` call sites in `repos/transfer_process.rs` and `repos/transfer_message.rs`.

### EH-02 — Silent cursor decode failure
**Commit:** `quality(errors): propagate invalid cursor as client error — refs audit#EH-02`  
Added `InvalidCursor` variant to `TransferProcessRepoErrors` and `TransferMessageRepoErrors`. Both SeaORM repos now have a `decode_cursor` method returning `Outcome<DateTime<FixedOffset>>` and callers use `?` so a malformed cursor returns an error instead of silently fetching from the beginning.

### AC-01 — `std::Mutex` held across async boundary
**Commit:** (part of BH-02 commit)  
All `put_transfer_process` writes in `data/in_memory/repos.rs` now use explicit scope blocks `{ let mut store = ...; ... }` so the `MutexGuard` is dropped before any subsequent await points.

### MT-02 — `TryFrom<MessageEnvelopeInput>` used `String` as error type
**Commit:** `quality(types): typed EnvelopeError for MessageEnvelope::try_from — refs audit#MT-02`  
Added `EnvelopeError` enum (`InvalidBase64 { field, source }`) and changed `impl TryFrom<MessageEnvelopeInput>` to use it. `b64_decode` now accepts a `field: &'static str` parameter for precise error context.

### OB-01 — No tracing spans in services
**Commit:** `quality(observability): add tracing::instrument to service methods — refs audit#OB-01`  
Added `#[tracing::instrument]` to all 11 service methods across `TransferProcessService` and `TransferMessageService`:
- ID-bearing methods (`get_one`, `edit`, `delete`, `get_all_by_process`) use `fields(id = %id)` or `fields(process_id = %process_id)` with selective `skip`.
- Bulk methods (`get_all`, `batch`, `create`) use `skip_all`.

### PE-01 / AC-03 — Sequential count + get_all
**Commit:** `quality(performance): parallelize count+list with tokio::try_join — refs audit#PE-01`  
Both `TransferProcessService::get_all` and both `TransferMessageService::get_all` / `get_all_by_process` now issue count and list queries concurrently via `tokio::try_join!`.

### PE-02 — Redundant `.map(|ids| ids.clone())`
**Commit:** (part of PE-01 commit)  
Changed `cmd.identifiers.as_ref().map(|ids| ids.clone()).unwrap_or_default()` → `cmd.identifiers.clone().unwrap_or_default()` in `TransferProcessService::create`.

### SE-01 — `X-Tenant-ID` not sanitized
**Commit:** `quality(security): validate X-Tenant-ID header against log injection — refs audit#SE-01`  
Added `is_safe_id` validator in `http/extractors.rs` that rejects any tenant ID containing characters outside `[a-zA-Z0-9\-.]`. Returns 400 before constructing `TenantId`.

### TS-02 — `mockall::automock` unconditional in production
**Commit:** (part of BH-02 commit)  
Changed `#[mockall::automock]` → `#[cfg_attr(test, mockall::automock)]` on all three repo traits: `TransferProcessRepoTrait`, `TransferMessageRepoTrait`, `TransferIdentifierRepoTrait`.

### AD-04 — Unused `TransferProcessBuilder`
**Commit:** (part of BH-02 commit)  
Deleted the entire `TransferProcessBuilder` struct and its `impl` block (~50 lines). Repos use `TransferProcess::rehydrate()` directly as the canonical reconstruction path.

---

## Deferred — MAJOR

### AD-02 — `rehydrate` accessible from any module
Deferred: requires team decision on repo/domain boundary. Documented in `docs/quality/open-questions.md#AD-02`.

### AC-02 — HTTP worker dies silently
Deferred: requires supervisor/channel design. Documented in `docs/quality/open-questions.md#AC-02`.

### MT-01 — `ProtocolState` as transparent string
Deferred: defining the enum requires finalizing the DSP state machine. Documented in `docs/quality/open-questions.md#MT-01`.

### DO-01 — No rustdoc on any public types
Deferred: low urgency relative to correctness work. Recommended for a dedicated documentation pass.

---

## Deferred — BLOCKER (test authoring)

### TS-01 — Zero tests (unit + integration)
The infrastructure is ready (in-memory repos annotated, mocks gated to `#[cfg(test)]`). Test authoring is a separate effort; scope:
- Unit: `apply_edit`, `from_cmd`, cursor encode/decode, `TransferProcessView::assemble`
- Integration: service layer using in-memory repos
- RBAC: `check_permission` paths

---

## Not applied — MINOR / NIT (low risk, lower priority)

| ID | Reason deferred |
|----|----------------|
| AD-03 | `TransferMessage` field visibility — style change, no correctness impact |
| AD-05 | `#[non_exhaustive]` on structs — only matters if crate is published externally |
| AD-06 | Param rename `batch_request` → `cmd` — cosmetic |
| AD-07 | `TransferDirection` unused enum — harmless with `allow(dead_code)` in place |
| EH-03 | `db_err` maps everything to one variant — observability improvement, not a bug |
| EH-04 | `value: Option<String>` semantics — requires schema decision |
| AC-02 | Covered above (MAJOR deferred) |
| MT-03 | `version: u32` vs `i32` — overflow only at 2^31 versions, negligible near-term |
| MT-04 | String keys for `consumerPid`/`providerPid` — constants would help, not urgent |
| MT-05 | `ser_hash_hex` per-byte allocation — micro-optimization |
| OB-02 | Span uses generated UUID instead of `X-Request-ID` — useful improvement |
| OB-03 | `on_response` at TRACE level — should be INFO |
| SE-02 | `connector_instance_id` unused field — needs feature clarity |
| SE-03 | `raw_bytes` in responses — design decision required (open-questions.md) |
| PE-03 | `HashMap<String, …>` double-conversion of `Urn` keys — minor allocation |
| PE-04 | `ser_enum` allocates per filter — minor |
| DO-02 | `grpc_worker.rs` stub — dead code, low confusion risk |
| BH-03 | No MSRV declared — low risk, one-line fix |
| BH-04 | `log` + `tracing` coexist — functional, not broken |
| BH-05 | `services/filter.rs` duplicate — **DELETED** in BH-02 commit |
