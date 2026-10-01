# Open Questions — transfer-agent-ref

Deferred design decisions surfaced during the quality audit. Each entry has a finding ID from `audit-transfer-agent-ref.md`, the question, and what would be needed to resolve it.

---

## AD-02 — `rehydrate()` called directly by repos

**Finding:** Repositories call `TransferProcess::rehydrate()` directly to reconstruct domain objects from persistence. This bypasses any domain invariants that `rehydrate` is meant to enforce.

**Question:** Should `rehydrate` be restricted so only a dedicated factory or a trait in the domain layer can call it? Or is direct repo usage acceptable given that the ORM layer is considered trusted infrastructure?

**Resolution needed:** Team agreement on the boundary between infrastructure and domain. If `rehydrate` must stay open, document explicitly that callers are responsible for providing valid data.

---

## AC-02 — HTTP worker spawned without supervision

**Finding:** In `setup/http_worker.rs`, the HTTP server future is spawned with `tokio::spawn`. If the HTTP subsystem dies (e.g., bind error at startup, or runtime panic), the process continues running silently — it will accept no traffic but report healthy.

**Question:** Should the HTTP task be joined on the main task so a fatal HTTP error propagates and exits the process? Or is there a supervisor/health-check mechanism that would detect this?

**Resolution needed:** Decide on the desired failure mode. The simplest fix is `http_handle.await` in the main task instead of detaching it; anything more sophisticated requires a supervision tree.

---

## MT-01 — `ProtocolState` has no state machine validation

**Finding:** `ProtocolState` is a thin wrapper around `CompactString`. Any string value passes through without validation against the DSP Transfer Process state machine (REQUESTED → STARTED → SUSPENDED/COMPLETED/TERMINATED).

**Question:** Should `ProtocolState` enforce valid transitions at construction time, or is validation the responsibility of the orchestration layer above?

**Resolution needed:** If enforcement is desired, define a `TransferState` enum, derive `TryFrom<&str>`, and use it everywhere `ProtocolState` is created from external input.

---

## SE-03 — `raw_bytes` included in HTTP responses

**Finding:** `MessageEnvelope` serializes `raw_bytes` (a `Vec<u8>`) in API responses. Depending on the message type, this field may contain sensitive payload data (keys, tokens, binary credentials).

**Question:** Should `raw_bytes` be omitted from response serialization by default? If it is needed by certain consumers, should it be gated behind a query parameter or a separate endpoint?

**Resolution needed:** Confirm what data `raw_bytes` carries in practice. If sensitive, add `#[serde(skip_serializing)]` or expose it only via a dedicated `/raw` endpoint with appropriate authorization checks.
