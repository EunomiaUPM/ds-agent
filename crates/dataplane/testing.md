# Plan de Testing — `crates/dataplane`

> Objetivo: dejar una batería de tests **inamovible** (regression-safe) que cubra
> (1) los tests **unitarios** del crate `dataplane`, (2) los tests de **integración**
> del facade `crates/transfer-agent/src/protocols/dsp/facades/dataplane_facade`, y
> (3) los tests de integración de `dataplane` con `crates/connector` y `crates/keystore`.
>
> Este documento es el contrato de cobertura: cada test listado debe existir y pasar.
> Los marcados con ✅ **ya existen** hoy; los marcados con ⬜ son **a implementar**.

---

## 0. Estado actual de la cobertura

Tests ya presentes en el crate (`#[cfg(test)]` inline):

| Módulo | Fichero | Nº tests | Qué cubre |
|---|---|---|---|
| Manager | `entities/dataplane_manager/dataplane_manager.rs` | 34 | `execute_command` end-to-end con mocks (todos los comandos × rol/modo) |
| Strategy factory | `entities/dataplane_manager/dataplane_handlers_strategy.rs` | 8 | Ruteo `(role, mode) → handler` |
| Handler ConsumerPull | `.../dataplane_handlers_consumer_pull.rs` | 18 | Máquina de estados del handler |
| Handler ConsumerPush | `.../dataplane_handlers_consumer_push.rs` | 20 | idem |
| Handler ProviderPull | `.../dataplane_handlers_provider_pull.rs` | 16 | idem |
| Handler ProviderPush | `.../dataplane_handlers_provider_push.rs` | 20 | idem |
| Auth ApiKey | `entities/dataplane_drivers/authentication/api_key.rs` | 4 | Construcción de credenciales |
| Auth Basic | `.../authentication/basic_config.rs` | 3 | idem |
| Auth Bearer | `.../authentication/bearer_token.rs` | 3 | idem |
| Auth OAuth | `.../authentication/oauth.rs` | 4 | idem |
| Proxy HTTP | `entities/dataplane_drivers/proxy/http.rs` | 7 | `build_auth_artifacts` |
| Fixtures | `test_fixtures.rs` | — | Helpers compartidos de contexto/auth |

**Convención del crate** (a mantener en todo lo nuevo):
- `mockall::automock` sobre los traits (`DataplaneTransfersEntitiesTrait`,
  `ConnectorInstanceTrait`, `DataplaneDriverFactoryTrait`, `KeystoreLookup`, `SecretStore`...).
- Fixtures de config vía `common::test_utils::config_fixtures::transfer_config_fixture()`.
- Tests `async` con `#[tokio::test]`.
- Naming `test_<sujeto>_<escenario>_<resultado_esperado>`.

---

## 1. Tests unitarios — huecos a cubrir

Orden por valor (lógica pura y rica primero, infraestructura después).

### 1.1 `RuntimeSecretVault` / `DataplaneRuntime` — `entities/dataplane_manager/dataplane_runtime.rs` ⬜ **(máxima prioridad)**

Lógica pura de vaulting/resolución de secretos; sólo depende de `SecretStore`
(usar `MockSecretStore` o el `InMemorySecretRepo` de keystore detrás de `SecretStoreImpl`).

- ⬜ `test_export_noauth_passthrough` — `NoAuth` se devuelve intacto, sin escrituras al store.
- ⬜ `test_export_bearer_passthrough_no_vault_write` — `BearerToken` se devuelve igual; **no** se escribe en el store (verificar 0 upserts).
- ⬜ `test_export_apikey_passthrough` — `ApiKey` pasa intacto, sin upserts.
- ⬜ `test_export_basic_passthrough` — `BasicAuth` pasa intacto, sin upserts.
- ⬜ `test_export_oauth_without_refresh_vaults_access_token` — un upsert en `/runtime/<id>/access-token`; el `access_token` queda como placeholder `{{__RUNTIME_SECRET_{/runtime/<id>/access-token}__}}`; `token_type`/`expires_at` intactos; `refresh_token = None`.
- ⬜ `test_export_oauth_with_refresh_vaults_both` — dos upserts (access+refresh), ambos placeholders.
- ⬜ `test_export_propagates_store_write_error` — si `upsert` falla, `export` devuelve `Err`.
- ⬜ `test_resolve_substitutes_exact_match_preserving_type` — placeholder que ocupa todo el string → se reemplaza por el `Value` con su tipo JSON original (número/objeto, no stringificado).
- ⬜ `test_resolve_interpolated_stringifies` — placeholder dentro de un string más largo → se interpola como texto.
- ⬜ `test_resolve_no_placeholders_is_noop` — runtime sin placeholders se devuelve sin tocar el store (0 reads).
- ⬜ `test_resolve_missing_secret_leaves_placeholder` — si el store no tiene la clave, el placeholder permanece (no panic).
- ⬜ `test_export_then_resolve_roundtrip` — `export` → `resolve` reconstruye el `access_token` original (test de ida y vuelta sobre OAuth2).
- ⬜ `test_resolve_with_lookup_uses_keystore` — `resolve_with_lookup` resuelve vía `KeystoreLookup::get_secret` (mock que devuelve un valor conocido).
- ⬜ `test_cleanup_deletes_all_prefixed_entries` — `cleanup` lista por prefijo `/runtime/<id>` y borra todas las entradas.
- ⬜ `test_cleanup_tolerates_delete_errors` — si un `delete` falla, `cleanup` sigue y devuelve `Ok` (sólo warn).
- ⬜ `test_path_prefix_strips_urn` — `urn:dataplane-transfer:abc-123` → `/runtime/abc-123` (exponer helper o testear vía `export`).

### 1.2 `DataplaneDriverFactory` — `entities/dataplane_manager/dataplane_driver_factory.rs` ⬜

Ruteo de autenticadores/configuradores/subscribers + caminos de error. Construir
`DataplaneContext` con `test_fixtures` y connectores con distintas `AuthenticationConfig`/`InteractionConfig`.

- ⬜ `test_resolve_authenticator_noauth` → `NoAuthAuthenticator`.
- ⬜ `test_resolve_authenticator_basic/bearer/apikey/oauth` → autenticador correcto por variante.
- ⬜ `test_resolve_authenticator_consumer_without_connector_is_noop` → `NoOpAuthenticator`.
- ⬜ `test_resolve_authenticator_provider_without_connector_errors` → `DataplaneError::ConnectorNotAvailable`.
- ⬜ `test_resolve_proxy_configurator_provider_pull_http` → `HttpProviderPullConfigurator`.
- ⬜ `test_resolve_proxy_configurator_provider_push_http` → `HttpProviderPushConfigurator`.
- ⬜ `test_resolve_proxy_configurator_consumer_pull/push` → configuradores consumer correctos.
- ⬜ `test_resolve_proxy_configurator_provider_non_http_errors` → `NoDriverForCombination` (p.ej. Kafka en provider).
- ⬜ `test_resolve_subscriber_pull_is_none` → `Ok(None)` en Pull (provider y consumer).
- ⬜ `test_resolve_subscriber_provider_push_http` → `Some(HttpPubSubscriber)`.
- ⬜ `test_resolve_subscriber_provider_push_kafka_not_implemented` → `FeatureNotImplemented{ feature: "Kafka push subscriber" }`.
- ⬜ `test_resolve_subscriber_consumer_push_noop` → `Some(NoOpPubSubscriber)`.
- ⬜ `test_get_or_create_driver_composes_all_three` → `get_or_create_driver` devuelve `DataplaneDriver` con los tres componentes coherentes.

### 1.3 `DataplaneContext` — `entities/dataplane_manager/dataplane_context.rs` ⬜

- ⬜ `test_from_init_provider_sets_role_mode_connector` — provider/pull guarda rol, modo, connector_instance y forward address; llama `create_dataplane_transfer` una vez con `state=Init`.
- ⬜ `test_from_init_consumer_has_no_connector` — consumer → `connector_instance = None`.
- ⬜ `test_from_init_push_vs_pull_interaction_mode` — direction mapea a `InteractionMode` correcto.
- ⬜ `test_from_continuation_not_found_errors` — `get_dataplane_transfer_by_process_id → None` ⇒ `TransferNotFound`.
- ⬜ `test_from_continuation_loads_connector_when_present` — con `connector_instance_id` se llama `get_instance_by_id` y se crea driver vía factory (mock).
- ⬜ `test_from_continuation_consumer_no_driver` — sin connector ⇒ `driver = None`.
- ⬜ `test_from_continuation_hydrates_proxy_from_ingress_egress` — si `ingress_config`/`egress_config` deserializan, se reconstruye `proxy` y `forward_dataplane_address` desde egress.
- ⬜ `test_from_continuation_hydrates_runtime_from_flow_control` — `flow_control` JSON → `runtime`.
- ⬜ `test_set_forward_dataplane_address_from_ingress_httplistener` — ingress `HttpListener` produce `DataplaneAddress` con URL conformada por config y token.
- ⬜ `test_set_forward_dataplane_address_from_ingress_noop_is_none` — ingress `NoOp` ⇒ address `None`.

### 1.4 Configuradores de proxy — `entities/dataplane_drivers/configuration/*` ⬜

Por cada configurador (`http_provider_pull`, `http_provider_push`, `http_consumer_pull`,
`http_consumer_push`, `no_op`): construir contexto y verificar el `DataplaneProxy` resultante.

- ⬜ `test_provider_pull_configures_ingress_listener_and_egress` — genera `HttpListener` (path/token) en ingress y egress hacia el connector.
- ⬜ `test_provider_push_configures_proxy` — ingress/egress acordes a push.
- ⬜ `test_consumer_pull_configures_proxy` — egress hacia la forward address recibida.
- ⬜ `test_consumer_push_configures_proxy`.
- ⬜ `test_no_op_configurator_returns_noop_proxy` — ingress/egress `NoOp`.
- ⬜ `test_configurator_missing_connector_or_address_errors` — caminos de error (`MissingTransferContext`).

### 1.5 PubSub HTTP — `entities/dataplane_drivers/pubsub/http.rs` ⬜

Requiere servidor HTTP simulado (**`wiremock`** recomendado, ver §4).

- ⬜ `test_subscribe_posts_and_stores_subscription` — mockear endpoint subscribe; tras `subscribe`, `runtime.subscription` contiene la respuesta.
- ⬜ `test_subscribe_resolves_ingress_and_runtime_placeholders` — el body/URL se resuelven con `RuntimeParametersResolver` (placeholders `{{...}}` sustituidos).
- ⬜ `test_subscribe_applies_bearer_auth_header` — con runtime `BearerToken`, la petición sale con `Authorization: Bearer`.
- ⬜ `test_subscribe_without_connector_errors` — `PubSubConnectorNotAvailable{ operation: "subscribe" }`.
- ⬜ `test_subscribe_non_http_protocol_errors` — `UnsupportedProtocol`.
- ⬜ `test_subscribe_http_error_propagates` — endpoint 500 ⇒ `PubSubRequestFailed`.
- ⬜ `test_unsubscribe_delete/post/put/get_methods` — cada método HTTP soportado dispara la llamada correcta y guarda `runtime.unsubscription`.
- ⬜ `test_unsubscribe_unsupported_method_errors` — método raro ⇒ `UnsupportedProtocol`.
- ⬜ `test_unsubscribe_wrong_interaction_type_errors` — connector Pull ⇒ `WrongInteractionType`.

### 1.6 Proxy de datos HTTP — `entities/dataplane_drivers/proxy/http.rs` (ampliar) ⬜

`build_auth_artifacts` ya tiene 7 tests ✅. Completar lo que falta:

- ⬜ `test_build_auth_artifacts_apikey_query` — `ApiKeyLocation::Query` añade query param, no header.
- ⬜ `test_build_auth_artifacts_invalid_header_value_errors` — valor inválido ⇒ `InvalidHeaderValue`.
- ⬜ `test_proxy_forward_request_*` — si hay función de forwarding del proxy de datos: éxito 2xx, propagación de status, error de red ⇒ `ProxyRequestFailed` (con `wiremock`).

### 1.7 `KeystoreClientImpl` — `entities/dataplane_drivers/keystore_lookup.rs` ⬜

- ⬜ `test_get_parameter_hit/miss` — devuelve `Some/None` según el `ParameterStore` (mock o in-memory).
- ⬜ `test_get_secret_hit/miss` — idem con `SecretStore`.
- ⬜ `test_get_secret_invalid_key_returns_none` — `Key::new` inválida ⇒ `None` sin panic.

### 1.8 `DataplaneTransfersEntityService` — `entities/dataplane_transfers/dataplane_transfers_entity.rs` ⬜

Mockear `DataplaneRepoTrait` (con sus sub-repos mock) + `EntityCacheTrait`.

- ⬜ `test_get_by_id_cache_hit_skips_db` — cache devuelve valor ⇒ no se toca el repo.
- ⬜ `test_get_by_id_cache_miss_loads_db_and_sets_cache` — miss ⇒ lee DB, enriquece, `set_single`.
- ⬜ `test_get_by_id_not_found_returns_none`.
- ⬜ `test_create_logs_creation_and_caches` — crea, inserta log `trigger="Creation"`, cachea.
- ⬜ `test_create_log_failure_does_not_fail_creation` — si `create_log` falla, `create` sigue `Ok` (sólo error!()).
- ⬜ `test_put_logs_only_on_state_change` — cambio de estado ⇒ log `trigger="Update"`; mismo estado ⇒ sin log.
- ⬜ `test_put_replaces_fields_when_present` — `fields=Some(..)` ⇒ borra y recrea fields.
- ⬜ `test_put_updates_cache`.
- ⬜ `test_delete_removes_from_cache`.
- ⬜ `test_enrich_process_merges_fields_and_logs` — `enrich_process` arma el DTO con fields (map) y logs.

### 1.9 Entidades secundarias ⬜

- ⬜ `transfer_events/transfer_event_entity.rs` — alta/listado/asociación de eventos (mock repo).
- ⬜ `dataplane_transfer_logs/dataplane_transfer_logs_entity.rs` — creación/consulta de logs por process id.

### 1.10 Triviales / serialización ⬜

- ⬜ `dataplane_commands.rs::test_display_for_each_variant` — `Display` de `DataplaneCommand` para todas las variantes.
- ⬜ `authentication/no_auth.rs` + `no_op.rs` — `authenticate` devuelve el contexto sin cambios.
- ⬜ `pubsub/no_op.rs` — `subscribe`/`unsubscribe` no-op devuelven el contexto.
- ⬜ DTOs (`dataplane_transfers/mod.rs`) — round-trip serde de `DataplaneTransferDto`/`NewDataplaneTransferDto`/`EditDataplaneTransferDto` (camelCase, `deny_unknown_fields`).
- ⬜ `From<NewDataplaneTransferDto> for NewDataplaneTransfer`.

### 1.11 Capa HTTP (routers) — `http/*` ⬜

Tests de router con `axum` (`tower::ServiceExt::oneshot`), inyectando entidad mock.

- ⬜ `http/dataplane_info` — GET lista/único, 404 si no existe.
- ⬜ `http/dataplane_transfer_logs` — GET logs por process id.
- ⬜ `http/transfer_events` — sub-router de feed por proceso y lookup global.
- ⬜ Verificar status codes y forma del JSON (camelCase).

> **Nota cobertura no-unitaria:** `cache/cache_redis/*` (necesita Redis) y `data/repo_sql/*`
> (necesita BD) se cubren en la sección de integración (§3), no aquí.

---

## 2. Integración del facade — `transfer-agent/.../dataplane_facade`

Ubicación sugerida: `crates/transfer-agent/tests/dataplane_facade_it.rs`
(o módulo `#[cfg(test)]` si se prefiere acceso a tipos `pub(crate)`; dado que el
trait y las strategies son `pub(crate)`/`pub(super)`, lo más práctico es un módulo
de test **dentro** del crate, p.ej. `src/protocols/dsp/facades/dataplane_facade/tests.rs`).

**Sujeto bajo test:** `DspDataPlaneFacade` (implementa `DataPlaneFacadeTrait`) ruteando
a las 4 strategies (`consumer_pull/push`, `provider_pull/push`) y delegando en un
`DataplaneManager` **real**. Sólo se mockean las dependencias de borde del manager
(`DataplaneTransfersEntitiesTrait`, `ConnectorInstanceTrait`).

**Harness:** construir un `DataplaneManager::new(mock_entity, mock_connector, transfer_config_fixture())`
y un `DspDataPlaneFacade::new(Arc::new(manager), proxy_base_url)`. Fabricar
`DspTransferContext` con `process` (TransferProcessDto) y `input_data_address` según el caso.

### 2.1 Ruteo de strategy — `strategy.rs` ⬜
- ⬜ `test_strategy_for_consumer_pull/consumer_push/provider_pull/provider_push` — `strategy_for` mapea `(role, direction)` correctamente.
- ⬜ `test_strategy_for_request_pre_with_address_is_consumer_push`.
- ⬜ `test_strategy_for_request_pre_without_address_is_consumer_pull`.

### 2.2 Ciclo de vida por hook (los 10 hooks del trait) ⬜

Para cada combinación rol/dirección, recorrer el ciclo y verificar:
(a) el `DataplaneCommand` que llega al manager (vía el mock_entity, comprobando
los `state` esperados en `put_dataplane_transfer_by_id`), y (b) el `DataAddressDto`
devuelto cuando aplica.

- ⬜ **Consumer/Pull**
  - `test_consumer_pull_request_pre_returns_no_address_or_proxy_address`
  - `test_consumer_pull_request_post_inits_transfer` (crea transfer, estados Init→Configuring→Auth→Ready)
  - `test_consumer_pull_start_pre/post_sets_started`
  - `test_consumer_pull_complete/terminate_sets_terminated`
- ⬜ **Consumer/Push**
  - `test_consumer_push_request_pre_with_input_address_returns_proxy_address`
  - `test_consumer_push_start_subscribing_flow`
  - `test_consumer_push_terminate_cleans_up`
- ⬜ **Provider/Pull**
  - `test_provider_pull_request_post_inits_with_connector`
  - `test_provider_pull_start_returns_forward_address` (egress proxy address)
  - `test_provider_pull_suspend/complete/terminate`
- ⬜ **Provider/Push**
  - `test_provider_push_start_subscribes`
  - `test_provider_push_suspension_unsubscribes`
  - `test_provider_push_termination_unsubscribes_and_terminates`
- ⬜ **Errores de contrato**
  - `test_hook_without_process_errors` — los hooks que exigen `ctx.process` (`*_post`, `start_*`, etc.) devuelven `Err("process required ...")` si `process = None`.

### 2.3 Conversiones DTO — `mod.rs` ⬜
- ⬜ `test_dataaddress_to_dataplane_address_maps_auth_props` — `authType`/`authorization` desde `endpoint_properties`.
- ⬜ `test_dataplane_address_to_dataaddress_roundtrip` — ida y vuelta preserva campos; props vacías ⇒ `None`.
- ⬜ `test_dataaddress_missing_endpoint_defaults_empty`.

---

## 3. Integración con `connector` y `keystore`

Ubicación sugerida: `crates/dataplane/tests/` (tests de integración "de caja negra"
sobre la API pública del crate) **o** módulo interno si se requiere acceso a tipos
`pub(crate)` como `DataplaneDriverFactory`/`DataplaneContext`. Recomendado: un
**módulo interno** `entities/.../*_it_tests.rs` para poder ejercitar la factory y el
runtime vault directamente.

### 3.1 `dataplane` + `keystore` (sin BD — usar repos in-memory) ⬜ **(recomendado empezar por aquí)**

`keystore` expone `InMemoryParameterRepo`/`InMemorySecretRepo` (en `data/in_memory`).
Montar `SecretStoreImpl`/`ParameterStoreImpl` reales sobre ellos — **integración real
del keystore sin infraestructura externa**.

- ⬜ `test_runtime_vault_export_persists_oauth_token_in_keystore` — `RuntimeSecretVault::export` sobre un `SecretStore` real escribe el access-token en `/runtime/<id>/access-token` y devuelve placeholder; leer el store confirma el valor.
- ⬜ `test_runtime_vault_resolve_reads_back_from_keystore` — tras export, `resolve` reconstruye el token leyendo del store real.
- ⬜ `test_runtime_vault_cleanup_removes_all_runtime_secrets` — tras `cleanup`, el store no lista entradas bajo el prefijo del transfer.
- ⬜ `test_keystore_client_impl_get_secret_against_real_store` — `KeystoreClientImpl::get_secret` recupera un secreto previamente `upsert`-eado.
- ⬜ `test_keystore_client_impl_get_parameter_against_real_store`.
- ⬜ `test_resolve_with_lookup_against_real_keystore` — `resolve_with_lookup` resuelve un placeholder cuyo valor vive en el `SecretStore` real (vía `KeystoreClientImpl` como `KeystoreLookup`).
- ⬜ `test_driver_factory_with_keystore_builds_http_subscriber_with_lookup` — `DataplaneDriverFactory::new().with_keystore(real_lookup)` produce un `HttpPubSubscriber` que usa el keystore real para resolver secretos del connector.
- ⬜ `test_export_static_credentials_not_duplicated_in_keystore` — `BearerToken/ApiKey/BasicAuth` **no** crean entradas extra en el store (confirmación del diseño: sólo OAuth2 se vaultea).

### 3.2 `dataplane` + `connector` ⬜

Dos niveles, elegir según coste/fidelidad (ver §4):

**Nivel A — Connector real en memoria (preferido, rápido):**
Construir `ConnectorInstanceEntitiesService` con `MockConnectorRepoTrait` +
`MockDistributionFacadeTrait` (patrón ya usado en
`connector/src/entities/connector_instance/service/tests.rs`) y pasarlo como
`Arc<dyn ConnectorInstanceTrait>` al `DataplaneManager`/`DataplaneContext`.

- ⬜ `test_context_from_continuation_loads_real_connector_instance` — el manager pide `get_instance_by_id` al servicio real y el `DataplaneContext` queda con el connector hidratado.
- ⬜ `test_driver_factory_resolves_authenticator_from_real_connector_config` — por cada `AuthenticationConfig` instanciada en el connector real, la factory elige el autenticador correcto (NoAuth/Basic/Bearer/ApiKey/OAuth2).
- ⬜ `test_driver_factory_resolves_proxy_configurator_from_real_pull_connector` — connector Pull/HTTP real ⇒ `HttpProviderPullConfigurator`.
- ⬜ `test_driver_factory_resolves_subscriber_from_real_push_connector` — connector Push/HTTP real ⇒ `HttpPubSubscriber`.
- ⬜ `test_provider_pull_full_init_with_real_connector` — `SetInit` provider/pull con connector real: estados Init→Configuring→Auth→Ready, egress address calculada desde el connector.
- ⬜ `test_runtime_parameters_resolved_against_real_connector_template` — placeholders del connector (`{{__...__}}`) resueltos por `RuntimeParametersResolver` en subscribe/unsubscribe.

**Nivel B — Connector con BD real (fidelidad máxima, opcional):**
`ConnectorSetup::get_connector_instance_entity` sobre una BD de test
(SQLite-memory con `get_connector_migrations`, o Postgres vía testcontainers).

- ⬜ `test_connector_instance_crud_then_dataplane_init` — instanciar un connector vía el servicio real con BD, luego arrancar un dataplane provider que lo consume por id.

### 3.3 `dataplane` con su propia capa SQL (repos + cache) ⬜

Para blindar `data/repo_sql/*` y `DataplaneTransfersEntityService` end-to-end.

- ⬜ Harness `setup_test_db()` — `DatabaseConnection` SQLite-memory + `get_dataplane_migrations()` aplicadas; construir `DataplaneRepoForSql::create_repo(conn)`.
- ⬜ `test_transfers_repo_create_get_put_delete_roundtrip`.
- ⬜ `test_fields_repo_create_replace_delete_by_process_id`.
- ⬜ `test_logs_repo_append_and_query_by_process_id`.
- ⬜ `test_transfer_events_repo_roundtrip`.
- ⬜ `test_entity_service_over_real_repo_and_noop_cache` — `DataplaneTransfersEntityService` con repo SQL real + cache no-op: create→get→put(state change ⇒ log)→delete, verificando logs persistidos.
- ⬜ (opcional, requiere Redis) `test_cache_redis_set_get_delete` — `DataplaneTransferCacheForRedis` contra Redis efímero (testcontainers); marcar `#[ignore]` si no hay Redis en CI.

---

## 4. Infraestructura de tests y dependencias

`[dev-dependencies]` a añadir en `crates/dataplane/Cargo.toml` (y análogos en transfer-agent):

```toml
[dev-dependencies]
mockall = { workspace = true }          # ya presente
tokio = { workspace = true, features = ["macros", "rt-multi-thread"] }
wiremock = "0.6"                         # servidor HTTP simulado para pubsub/proxy
serde_json = { workspace = true }
# Para §3.3 con SQLite en memoria:
sea-orm = { workspace = true, features = ["sqlx-sqlite", "runtime-tokio-rustls"] }
# Para integración con keystore in-memory: keystore ya es dependencia; exponer
# InMemoryParameterRepo/InMemorySecretRepo o SecretStoreImpl en su API pública/de test.
# Para Nivel B / Redis: testcontainers = "0.20" (opcional, tests #[ignore] sin infra)
```

**Decisiones de harness recomendadas:**
1. **HTTP simulado:** `wiremock` (async, idiomático con tokio) para `pubsub/http`, `proxy/http` y los flujos del facade que tocan red.
2. **BD:** SQLite en memoria (`sea_orm::Database::connect("sqlite::memory:")` + migraciones) como opción por defecto — rápido y sin Docker. Reservar testcontainers/Postgres para una suite `#[ignore]` de fidelidad.
3. **Keystore:** usar los repos `InMemory*` reales del crate `keystore` → integración auténtica sin mocks ni BD. **Requisito previo:** confirmar que `InMemoryParameterRepo`/`InMemorySecretRepo` (o un builder `SecretStoreImpl::new`) sean accesibles desde fuera del crate; si no, exponerlos tras `#[cfg(feature = "test-util")]` o como `pub`.
4. **Fixtures compartidas:** extraer a un `tests/common/mod.rs` (o ampliar `test_fixtures.rs`): builders de `ConnectorInstanceDto` por tipo de auth/interacción, de `DataplaneTransferDto` por estado, de `DspTransferContext`, y `setup_test_db()`.

---

## 5. Resumen de ejecución y criterios de "inamovible"

- **Unit** (§1): `cargo test -p dataplane` — sin red ni BD (excepto los que usan `wiremock`, que levanta servidor local efímero).
- **Integración keystore** (§3.1): `cargo test -p dataplane` — in-memory, sin infra.
- **Integración connector Nivel A** (§3.2 A) y **facade** (§2): mocks/in-memory, sin infra.
- **Integración SQL/Redis/Postgres** (§3.3, Nivel B): suite separada o `#[ignore]`, ejecutable en CI con servicios.

Criterios para considerar la batería "blindada":
1. Cada estado de la máquina (`Init, Configuring, Auth, Ready, Started, Subscribing, Unsubscribing, Stopped, Terminated`) tiene al menos un test que verifica la transición que lo produce.
2. Cada variante de `AuthenticationConfig` y cada `(TransferRole, InteractionMode)` está cubierta en la factory.
3. Cada variante de `DataplaneError` relevante tiene un test que la provoca.
4. El round-trip de secretos OAuth2 (export→DB→resolve→uso→cleanup) está cubierto contra un keystore real.
5. Los 10 hooks del `DataPlaneFacadeTrait` × 4 strategies tienen cobertura de ruteo y de error por `process` ausente.

---

### Apéndice — Mapa de prioridades

| Prioridad | Bloque | Razón |
|---|---|---|
| P0 | §1.1 RuntimeSecretVault | Lógica pura crítica de seguridad, hoy sin tests |
| P0 | §1.2 DriverFactory | Núcleo del ruteo, hoy sin tests |
| P0 | §3.1 keystore in-memory | Integración real barata, alto valor |
| P1 | §1.3 Context, §1.4 Configuradores | Construcción del estado del transfer |
| P1 | §2 Facade lifecycle | Contrato externo del dataplane |
| P1 | §1.8 EntityService | Caché/logging |
| P2 | §1.5/§1.6 PubSub/Proxy HTTP | Requieren wiremock |
| P2 | §3.2 connector | Integración cruzada |
| P3 | §3.3 SQL/Redis, §1.11 HTTP | Requieren infra; valor de regresión |
