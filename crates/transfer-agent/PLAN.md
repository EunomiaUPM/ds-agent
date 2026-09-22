# PLAN — Refactor `transfer-agent-ref` (inbound → outbound, los 5 mensajes)

> **Regla:** mira solo el día de hoy. Tapa el resto con la mano si hace falta.
> Objetivo: los 5 mensajes DSP en ambos sentidos, verdes contra in-memory, colgando
> de un molde limpio en `protocols/`.
> Sesión = 4h. Si un día se alarga, sacrifica celdas secundarias (`todo!()` marcado),
> NUNCA la limpieza del molde.

> **Contexto real (no es greenfield).** El crate ya tiene dominio, ports, persistencia,
> servicios CRUD y transportes. El molde DSP vive en `protocols/` (hoy vacío). No crees
> un árbol `domain/application/infrastructure` paralelo: reusa lo que hay.
>
> | Concepto del plan original | Dónde vive aquí |
> |---|---|
> | domain/model | `entities/` (ya hecho) |
> | domain/errors | `ymir::errors` (`Outcome`, `Errors`) — NO crear `DomainError` |
> | domain/events | `entities/events.rs` (ya hecho) |
> | domain/ports | `data/repo/*Trait` (ya hecho) + ports nuevos aquí |
> | domain/state (matriz) | `entities/state.rs` ← **falta** |
> | infra/persistence | `data/{sea_orm,in_memory}` (ya hecho) |
> | application/context+pipeline+manager+strategies | `protocols/` ← **el trabajo real** |
> | interfaces/http (inbound DSP) | `http/transfer_message_router.rs` (ya cableado) |
> | interfaces/rpc (outbound) | `grpc/transfer_messages` (ya cableado) |
> | bootstrap | `setup/` (ya hecho) |

---

## Estado actual — lo HECHO (compila, 7 tests dsp verdes)

> **Desviación de estructura (intencional):** el molde vive en `protocols/dsp/{entities,facades,services}`
> (scoped por protocolo), no en `protocols/{context,pipeline,manager,strategies}`. El canonicalizer
> se fue a `common/rdf/dsp` para reuso entre crates. Nada de esto está cableado a los transportes
> todavía → los warnings `dead_code` son esperados (el pipeline aún no tiene quien lo llame).

- [x] **Cadena de contexto (Día 2 typestate)** — `TransferDspContextRaw → …Parsed → …Rdf → …Typed → TransferDSPContextDomain` en `entities/dsp_context.rs`. Cada fase = constructor `from_<prev>`.
- [x] **Canonicalizer (Día 2 pipeline)** — expansión JSON-LD + URDNA2015/RDFC-1.0 **reales**, offline (contexto DSP + ODRL profile embebidos), en `common::rdf::dsp::DspCanonicalizer`. Cableado en `TransferContextRdf::from_parsed`. Da `canonical_n_quads` + `canonical_hash` (base de idempotencia/firma).
- [x] **typed_deserializer (Día 2)** — `DspTransferRdfExtractor`: saca pids, `dataAddress`, tipo de mensaje. Puerta pública `TransferContextTyped::from_rdf`.
- [x] **domain_loader (Día 2, más que stub)** — `services/dsp_domain_loader`: slot de proceso vía repo (`Existing`/`New`), `role` (existing→proceso, new→resolver), `agreement` (resolver), `connector` (DataService facade por `agreementId`), `transfer_direction` (dataAddress→Push/Pull, guard `is_restart`). Inyecta `FacadeTrait` + `DspDomainResolver`.
- [x] **Capa de facades (infra Día 3)** — `DataServiceFacadeTrait` + `DataPlaneFacadeTrait` (agregadas por `FacadeTrait`), strategies dataplane (Consumer/Provider × Pull/Push), `strategy_for(role, direction)`, conversiones `DataAddress ↔ DataplaneAddress`. Todo compila.

**Pendiente dentro del Día 2:** stages `json_schema` + `shacl`; `context/helpers.rs` (accesores transversales).

**Deuda marcada (`ponytail:`):** `is_restart`/`is_idempotent_replay` provisionales (`false`); rama restart de `transfer_direction` diferida al proceso persistido; `DspDomainResolver` sin impl concreta (solo el seam).

---

## Día 1 — Cerrar el andamiaje (lo que falta del dominio + ports nuevos)

> Día 1 del plan original está ~80% hecho en `entities/` + `data/`. Solo faltan la
> máquina de estados y los ports que el molde necesitará. No re-hagas modelo ni repos.

- [ ] `entities/state.rs`: `TRANSITION_MATRIX` (por `ProtocolState` + `TransferRole` + `ProtocolMessageType`) — hoy `protocol_state` es un `CompactString` opaco; la transición no vive en ninguna parte. **← empieza aquí**
- [ ] `entities/state.rs`: semáforo `start ↔ suspension` (reanudar solo desde suspended)
- [ ] `entities/transfer_process.rs`: `try_transition(msg_type, role) -> Outcome<()>` que consulta la matriz y muta `protocol_state` + `version` (ya hay `apply_edit`; añade la transición validada, no strings sueltos)
- [ ] `data/repo/`: ports nuevos **solo firmas** — `outbox` (encolar mensaje saliente en TX), `event_bus` (publicar `TransferEvent`), `external`/`peer` (`send_to_peer`). Repos de proceso/mensaje/identifier ya existen.
- [ ] UoW con TX semántica: envolver los repos actuales tras un `TransferUow` (persist proceso + outbox atómicos). `data/sea_orm/` → TX real; `data/in_memory/repos.rs` → snapshot→replay/descarte.
- [ ] Tests unitarios de la matriz de transición → verde antes de seguir

---

## Día 2 — Context typestate + pipeline (nuevo, en `protocols/`)

- [x] `protocols/context/`: cadena `Raw → Parsed → Rdf → Typed → Domain` — hecha en `dsp/entities/dsp_context.rs`
- [ ] `protocols/context/helpers.rs`: accesores transversales (`request_id()`, `tenant()`, `correlation_id()`) — el tenant/scope ya existe vía `AccessScope`, reúsalo
- [x] `canonicalizer` + `jsonld_parser` — hechos (`common::rdf::dsp::DspCanonicalizer`, real). Faltan `json_schema` y `shacl`.
- [x] `typed_deserializer` → `DspTransferRdfExtractor` (produce pids/dataAddress/tipo)
- [x] `domain_loader` → `DspDomainLoader` (carga proceso vía repo + resuelve el resto de hechos)
- [x] La cadena inbound **compila** con typestate

> ⚠️ Si `c.typed.rdf.parsed.raw.x` te mata: colapsa fases (fusiona `CtxParsed`/`CtxRdf`). NUNCA `HashMap<String,Any>`.
> ⚠️ Antes de escribir un stage, `grep`/codegraph en `negotiation-agent` y `common`: el jsonld/shacl casi seguro ya existe.

---

## Día 3 — Los DOS templates con Request (el día que decide la semana)

- [ ] `protocols/manager/`: `DspManager` (template `run`) + `DspManagerBuilder` + `DspResponse`
- [ ] `protocols/strategies/traits.rs`: `DspLifecycleStrategy`
- [ ] `RequestStrategy` (inbound): corta en persist + outbox dentro de la TX (`TransferUow` del Día 1)
- [ ] `OutboundRequestStrategy`: añade graph_builder → sign → compact → `send_to_peer` **fuera de TX**
- [ ] Validators de Request: `PidConsistency` (pre), `Transition` (usa la matriz del Día 1), `AgreementActive` (sem, client fakeado)
- [ ] `Hook` trait + `HookRegistry` con noop
- [ ] `manager.run(ctx_inbound_request)` → verde
- [ ] `manager.run(ctx_outbound_request)` → verde ← **si esto está limpio, el resto es calco**

> ⚠️ `send_to_peer` va fuera de TX y su fallo NO rollea el estado persistido. Eso separa "capas bien hechas" de spaghetti.

---

## Día 4 — Replicar los otros 4 × 2 sentidos (mecánico)

- [ ] Start — inbound / outbound
- [ ] Completion — inbound / outbound
- [ ] Termination — inbound / outbound
- [ ] Suspension — inbound / outbound
- [ ] Semáforo `start ↔ suspension` enganchado como validator semántico (reusa `entities/state.rs`)

> Si un mensaje se resiste, casi siempre es una fuga del molde del Día 3. Arréglalo en el molde, no en la strategy.

---

## Día 5 — Cablear en los transportes existentes + barrido de tests

> Los transportes YA existen y enrutan a los servicios CRUD. Aquí los enganchas al
> `DspManager`, no creas un árbol `interfaces/` nuevo.

- [ ] `http/transfer_message_router.rs`: handlers inbound `/transfer/*` → `ctx_init → pipeline → manager.run` (reusa `http/extractors.rs` + middleware auth actual)
- [ ] `grpc/transfer_messages/`: handlers outbound análogos → `manager.run(ctx_outbound_*)`
- [ ] `TransferTestHarness` + builders (`ctx_inbound_request()`, `agreement().active()`) — apóyate en `data/in_memory` y en los `tests.rs` que ya hay en `services/`
- [ ] Un test de integración por celda → **10 tests verdes = semana cumplida**

---

## Fuera de esta semana (sin remordimiento, encaja después sobre el core)

Outbox poller como worker separado (ya hay `setup/*_worker.rs` como patrón) · EventBus real ·
CQRS read side + endpoints query (los `views.rs` ya apuntan ahí) · OTel/Prometheus ·
idempotency HTTP layer · semantic store subscriber · DSP TCK · proptest.
