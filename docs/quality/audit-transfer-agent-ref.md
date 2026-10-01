# Audit — `transfer_agent_ref`

> Revisor: Claude Sonnet 4.6 · Fecha: 2026-05-16 · Rama: `refactor/transfer-agent`  
> Metodología: lectura completa de las 54 fuentes + `Cargo.toml`. Sin modificar código.

---

## 1. Inventario

### 1.1 Módulos y responsabilidades

| Módulo | Responsabilidad |
|--------|----------------|
| `entities/` | Modelos de dominio puros: `TransferProcess`, `TransferMessage`, `TransferProcessIdentifier`, IDs newtype, enums de protocolo, envelope con firma |
| `entities/commands.rs` | DTOs de entrada (deserialización de requests HTTP) |
| `entities/query.rs` | Filtros, paginación cursor-based, `Sort` |
| `entities/events.rs` | Eventos de dominio (definidos, no publicados) |
| `data/repo/` | Traits de repositorio + errores tipados con `thiserror` |
| `data/in_memory/` | Implementación en memoria (tests / desarrollo) |
| `data/sea_orm/` | Implementación producción: ORM models, migrations, repos |
| `data/factory.rs` | Trait `DataFactory` — punto de inyección de dependencias |
| `services/transfer_process/` | Lógica de negocio, paginación, ensamblado de vistas |
| `services/transfer_message/` | Ídem para mensajes |
| `services/filter.rs` | Duplicado de `Page`/`Paginated`/`Sort` (ver hallazgo F-05) |
| `http/extractors.rs` | `ExtractedHeaders` + `AuthClaims` (extractores Axum) |
| `http/transfer_process_router.rs` | Rutas CRUD + batch para transfer processes |
| `http/transfer_message_router.rs` | Rutas CRUD para transfer messages |
| `setup/` | Bootstrap, CLI, HTTP worker, migraciones |

### 1.2 Dependencias internas

```
http → services → data/repo ← data/in_memory
                            ← data/sea_orm
entities ← (todos los anteriores)
setup → http, data/sea_orm, oauth
```

### 1.3 Dependencias externas relevantes

| Crate | Uso |
|-------|-----|
| `axum` | Framework HTTP |
| `sea-orm` / `sea-orm-migration` | ORM + migraciones |
| `oauth` (workspace) | Router OAuth + validación JWT |
| `common` (workspace) | Config, RBAC, middleware auth, batch requests |
| `mockall` | Automock en traits de repositorio |
| `thiserror` | Errores tipados en repos |
| `ymir` | `Outcome<T>`, `Errors`, `VaultService` |
| `compact_str` | Strings compactas sin heap para valores cortos |
| `sha2` / `base64` / `bytes` | Procesamiento de envelope |
| `tonic` / `prost` | gRPC (importado pero stub vacío) |
| `dataplane`, `negotiation_agent`, `catalog_agent`, `connector`, `events` | **Declarados en `Cargo.toml` pero no usados en el código** |

### 1.4 Superficie pública

El crate expone únicamente `pub mod setup` desde `lib.rs`. Todo lo demás es `pub(crate)` o menos. La superficie real de uso externo es:
- `setup::cmd::TransferCommands::init_command_line()`
- `setup::http_worker::TransferHttpWorker::spawn()` / `create_root_http_router()`
- `setup::http_worker::create_root_http_router()` (función libre — duplica al método)

---

## 2. Hallazgos

### API Design

#### AD-01 · **MAJOR** — Función libre `create_root_http_router` duplica el método homónimo
- **Ubicación:** `setup/http_worker.rs:91,121`
- **Problema:** Existe un método asociado `TransferHttpWorker::create_root_http_router` y una función libre `create_root_http_router` con la misma firma. El método delega en la función libre. La API pública exporta ambas sin necesidad.
- **Impacto:** Confusión para consumidores del crate; breaking-change risk si se elimina una.
- **Recomendación:** Eliminar la función libre, hacer el método el único punto de entrada, o hacer la función libre `pub(crate)`.
- **Esfuerzo:** S

#### AD-02 · **MAJOR** — `rehydrate` público en `TransferProcess`
- **Ubicación:** `entities/transfer_process.rs:83`
- **Problema:** `rehydrate` tiene visibilidad `pub(crate)` pero reconstruye el aggregate sin validación. Cualquier módulo del crate puede crear estados imposibles (versión negativa, timestamps inconsistentes).
- **Impacto:** Viola encapsulación del aggregate; los repos ORM e in-memory lo llaman directamente.
- **Recomendación:** Limitar a `pub(super)` o crear un constructor privado de rehydration solo accesible desde `data/`.
- **Esfuerzo:** M · Candidato a regla reutilizable: _"rehydrate solo desde la capa de datos"_.

#### AD-03 · **MINOR** — `TransferMessage` con campos `pub` directos
- **Ubicación:** `entities/transfer_message.rs:39-58`
- **Problema:** Todos los campos son `pub`, pero `TransferProcess` usa accessors privados. Inconsistencia de API.
- **Impacto:** Acoplamiento externo accidental; la vista (`TransferMessageView::assemble`) accede directamente a los campos en lugar de usar accessors.
- **Recomendación:** Hacer los campos `pub(crate)` con accessors consistentes, igual que `TransferProcess`.
- **Esfuerzo:** S

#### AD-04 · **MINOR** — `TransferProcessBuilder` no se usa en ninguna parte del código
- **Ubicación:** `entities/transfer_process.rs:219`
- **Problema:** El builder está definido y tiene `build() -> Outcome<TransferProcess>`, pero ningún llamador lo usa. Los repos crean `TransferProcess` vía `new()` o `rehydrate()` directamente.
- **Impacto:** Dead code; confusión sobre cuál es el camino canónico de construcción.
- **Recomendación:** Eliminarlo o usarlo de forma consistente en los repos.
- **Esfuerzo:** S

#### AD-05 · **MINOR** — `StateMetadata` y `TransferCorrelation` son `pub` sin `#[non_exhaustive]`
- **Ubicación:** `entities/protocol.rs:57,75`
- **Problema:** Structs con todos los campos `pub` expuestos como parte de la API. Añadir un campo es un breaking change.
- **Impacto:** Riesgo de breaking change bajo si el crate se comparte como librería. Actualmente riesgo bajo porque la superficie es interna.
- **Recomendación:** Marcar con `#[non_exhaustive]` o hacer campos `pub(crate)`.
- **Esfuerzo:** S

#### AD-06 · **NIT** — Parámetro mal nombrado en `TransferProcessServiceTrait::create`
- **Ubicación:** `services/transfer_process/mod.rs:39`
- **Problema:** El parámetro se llama `batch_request: &NewTransferProcessCommand` — el nombre `batch_request` es incorrecto, debería ser `cmd`.
- **Esfuerzo:** S

#### AD-07 · **NIT** — `TransferDirection` definida pero nunca usada
- **Ubicación:** `entities/protocol.rs:26`
- **Problema:** El enum `TransferDirection { Push, Pull }` no aparece en ningún campo, comando ni filtro.
- **Recomendación:** Eliminar o usar en `NewTransferProcessCommand`.
- **Esfuerzo:** S

---

### Error Handling

#### EH-01 · **MAJOR** — `unwrap()` silencioso en `apply_base_filters` y `count_transfer_messages`
- **Ubicación:** `data/sea_orm/repos/transfer_process.rs:65,72`, `data/sea_orm/repos/transfer_message.rs:132,137`
- **Problema:** `serde_json::to_value(protocol).unwrap().as_str().unwrap_or("").to_string()` — el primer `unwrap()` puede hacer panic si la serialización falla. Aunque en la práctica es infallible para estos enums, la solución existente en `helpers.rs` (`ser_enum`) ya hace esto correctamente. Estos call sites no la usan.
- **Impacto:** Panic en producción si se añade un variant de enum que serialice de forma distinta.
- **Recomendación:** Reemplazar con `ser_enum(&protocol)` del módulo `helpers`.
- **Esfuerzo:** S · Candidato a regla reutilizable: _"nunca serializar enums con `to_value().unwrap()` fuera de helpers"_.

#### EH-02 · **MAJOR** — Error de cursor silenciado: filtro incorrecto sin notificar
- **Ubicación:** `data/sea_orm/repos/transfer_process.rs:102`, `data/in_memory/repos.rs:76`
- **Problema:** Si el cursor está malformado (base64 inválido o timestamp inválido), el error se ignora silenciosamente y la consulta se ejecuta **sin filtro de cursor** — devolviendo resultados desde el inicio en lugar de retornar un error 400.
- **Impacto:** El cliente recibe datos incorrectos sin saberlo, creyendo que la paginación funciona.
- **Recomendación:** Devolver `Err(Errors::format(BadFormat::Received, "invalid cursor", None))` si el cursor no se puede decodificar.
- **Esfuerzo:** S · Candidato a regla reutilizable: _"un cursor inválido es siempre un error del cliente, no silencio"_.

#### EH-03 · **MINOR** — `db_err` siempre mapea a `ErrorFetchingTransferProcess` independientemente de la operación
- **Ubicación:** `data/sea_orm/repos/transfer_process.rs:44`
- **Problema:** `fn db_err(e)` se usa tanto para fetch, count, como para operaciones de get_by_key. El tipo de error no refleja la operación real.
- **Impacto:** Logs y observabilidad degradados; dificulta el diagnóstico.
- **Recomendación:** Recibir la variante como parámetro o tener funciones `fetch_err`/`update_err` separadas.
- **Esfuerzo:** S

#### EH-04 · **MINOR** — `TransferProcessIdentifier.value: Option<String>` — semántica ambigua
- **Ubicación:** `entities/transfer_process_identifier.rs:24`, `services/transfer_process/service.rs:87`
- **Problema:** `value` es `Option<String>`, y cuando es `None` se reemplaza por `String::default()` en el servicio (`unwrap_or_default()`). Un identificador sin valor es válido estructuralmente pero probablemente sea un dato corrupto.
- **Impacto:** Se insertan entradas con clave pero sin valor en `correlation.identifiers`.
- **Recomendación:** Hacer `value: String` (no opcional) o eliminar las entradas sin valor antes de agregarlas.
- **Esfuerzo:** S

---

### Async / Concurrencia

#### AC-01 · **MAJOR** — `Arc<Mutex<HashMap>>` con lock sostenido a través de toda la operación de escritura (in-memory)
- **Ubicación:** `data/in_memory/repos.rs:211-220`, `data/in_memory/repos.rs:299-313`
- **Problema:** Los repos in-memory llaman `self.processes.lock().unwrap()` y mantienen el `MutexGuard` mientras insertan. Si bien no hay `.await` dentro del bloque, el patrón `lock().unwrap()` en async code puede causar deadlock si otro task intenta el mismo lock en el mismo hilo cooperativo.
- **Impacto:** Bajo en la implementación actual (no hay awaits dentro del lock), pero frágil: cualquier refactor que añada un `await` dentro causará deadlock.
- **Recomendación:** Usar `tokio::sync::Mutex` en lugar de `std::sync::Mutex` para repos async, o reducir el scope del lock explícitamente con un bloque `{ let mut store = ...; ... }`.
- **Esfuerzo:** S · Candidato a regla reutilizable: _"en código async, preferir `tokio::Mutex` o scope explícito del lock"_.

#### AC-02 · **MINOR** — `tokio::spawn` sin `JoinHandle` gestionado en el worker
- **Ubicación:** `setup/http_worker.rs:77`
- **Problema:** `tokio::spawn` devuelve un `JoinHandle<()>` que sí se retorna al llamador (`Ok(handle)`). Correcto. Sin embargo, si el task de servidor falla, el error se loguea (`tracing::error!`) pero no se propaga al proceso principal — el proceso sigue vivo sin servidor HTTP.
- **Impacto:** El proceso aparece como "vivo" en orquestadores pero no sirve tráfico.
- **Recomendación:** Enviar el error por un canal o usar `CancellationToken` para señalizar al proceso principal que debe terminar.
- **Esfuerzo:** M

#### AC-03 · **NIT** — Cursor de paginación no es cancel-safe
- **Ubicación:** `services/transfer_process/service.rs:64-113`
- **Problema:** `get_all` hace dos await secuenciales (`get_all_transfer_processes` y luego `count_transfer_processes`). Si el cliente cancela entre ambos, el count es inconsistente con los datos devueltos. No es un bug grave, pero el total puede diferir de los items.
- **Impacto:** Leve inconsistencia en el header `X-Total-Count`.
- **Recomendación:** Ejecutar `count` y `get_all` en paralelo con `tokio::join!` — elimina la ventana de inconsistencia y mejora la latencia.
- **Esfuerzo:** S

---

### Modelo de tipos

#### MT-01 · **MAJOR** — `ProtocolState` es un `String` transparente sin validación
- **Ubicación:** `entities/protocol.rs:49`
- **Problema:** `ProtocolState(pub CompactString)` acepta cualquier cadena. Los estados de protocolo (DSP) son un conjunto finito conocido. Actualmente es imposible distinguir en tiempo de compilación un estado válido de uno inventado.
- **Impacto:** Los filtros de query pueden usar estados inexistentes sin error; la lógica de transición de estado no puede ser exhaustiva.
- **Recomendación:** Definir un enum `DspTransferState` (con serde rename) o al menos un constructor validador `ProtocolState::try_from_str`.
- **Esfuerzo:** L

#### MT-02 · **MAJOR** — `MessageEnvelope::TryFrom` convierte `String` en errores con `String` en lugar de un tipo tipado
- **Ubicación:** `entities/transfer_message.rs:209`
- **Problema:** `impl TryFrom<MessageEnvelopeInput> for MessageEnvelope { type Error = String; }`. Serde's `try_from` propagará este `String` como un error de deserialización genérico. No hay tipo de error dedicado.
- **Impacto:** El error que llega al cliente es genérico y difícil de formatear consistentemente.
- **Recomendación:** Crear `EnvelopeError` con `thiserror` y usarlo como `type Error`.
- **Esfuerzo:** S

#### MT-03 · **MINOR** — `version: u32` en dominio pero `i32` en BD
- **Ubicación:** `entities/transfer_process.rs:37`, `data/sea_orm/orm/transfer_process.rs:41`
- **Problema:** El dominio usa `u32` (sin signo), la BD usa `i32`. La conversión `as i32` / `as u32` es silenciosa y puede truncar en versión > 2^31.
- **Impacto:** Overflow silencioso a largo plazo.
- **Recomendación:** Usar `i64` en ambos lados o añadir una conversión verificada con `try_into()`.
- **Esfuerzo:** S

#### MT-04 · **MINOR** — `TransferCorrelation.identifiers: HashMap<String, String>` sin clave tipada
- **Ubicación:** `entities/protocol.rs:76`
- **Problema:** Las claves especiales `"consumerPid"` y `"providerPid"` se tratan como strings mágicos en la vista (`services/transfer_process/views.rs:53-60`). El compilador no ayuda si se cambia el nombre.
- **Recomendación:** Constantes `pub const CONSUMER_PID_KEY: &str = "consumerPid"` o un newtype `IdentifierKey`.
- **Esfuerzo:** S

#### MT-05 · **NIT** — `ser_hash_hex` usa format string por byte en lugar de `hex::encode`
- **Ubicación:** `entities/transfer_message.rs:306`
- **Problema:** `h.iter().map(|b| format!("{b:02x}")).collect::<String>()` — alloca un `String` por byte (32 strings).
- **Recomendación:** Usar el crate `hex` o `base16ct` para una codificación sin allocaciones intermedias.
- **Esfuerzo:** S

---

### Testing

#### TS-01 · **BLOCKER** — Cero tests en el crate
- **Ubicación:** Todo el crate
- **Problema:** No existe ningún módulo `#[cfg(test)]`, ningún fichero de test, ninguna integración. El crate tiene `mockall` en dependencias (correcto para repositorios) pero no hay tests que lo usen.
- **Impacto:** Regresiones invisibles; la lógica de paginación, RBAC, ensamblado de vistas y cursor no tienen cobertura.
- **Recomendación:** Mínimo viable: tests unitarios para `apply_edit`, `from_cmd`, cursor encode/decode, RBAC, y `TransferProcessView::assemble` con identifiers. Tests de integración con el repo in-memory para el service layer.
- **Esfuerzo:** L · Candidato a regla reutilizable.

#### TS-02 · **MAJOR** — `mockall::automock` en traits de repositorio sin tests que lo consuman
- **Ubicación:** `data/repo/transfer_process.rs:25`, `data/repo/transfer_message.rs:25`
- **Problema:** `#[mockall::automock]` en producción añade overhead de compilación y superficie de macro sin beneficio si no hay tests.
- **Impacto:** Tiempo de compilación innecesario.
- **Recomendación:** Mover `automock` a `#[cfg_attr(test, mockall::automock)]` o a un feature flag `testing`.
- **Esfuerzo:** S

---

### Observabilidad

#### OB-01 · **MAJOR** — Sin spans de tracing en servicios ni repositorios
- **Ubicación:** `services/transfer_process/service.rs`, `services/transfer_message/service.rs`, todos los repos
- **Problema:** El middleware HTTP crea un span por request (con UUID), pero las llamadas al service y repo no crean sub-spans. No hay correlación end-to-end en traces.
- **Impacto:** Imposible diagnosticar qué operación de BD consume tiempo sin profiler externo.
- **Recomendación:** `#[tracing::instrument(skip(self, ...))]` en métodos de servicio como mínimo.
- **Esfuerzo:** S · Candidato a regla reutilizable: _"todo método de servicio lleva `#[instrument]`"_.

#### OB-02 · **MINOR** — El span de request usa `Uuid::new_v4()` ignorando `X-Request-ID`
- **Ubicación:** `setup/http_worker.rs:101`
- **Problema:** `tracing::info_span!("request", id = %Uuid::new_v4())` — genera un UUID nuevo en lugar de propagar el `X-Request-ID` extraído por `ExtractedHeaders`. La correlación con el cliente se pierde.
- **Recomendación:** Acceder al header `x-request-id` en el closure de `make_span_with` y usarlo como `id`.
- **Esfuerzo:** S

#### OB-03 · **MINOR** — `on_response` en nivel TRACE, nunca visible en producción
- **Ubicación:** `setup/http_worker.rs:106`
- **Problema:** `DefaultOnResponse::new().level(tracing::Level::TRACE)` — en producción el nivel suele ser INFO o DEBUG. Las respuestas HTTP no se loguean en ningún nivel visible.
- **Recomendación:** Subir a `INFO` o `DEBUG`, o registrar explícitamente status code + duración en el span del request.
- **Esfuerzo:** S

---

### Seguridad

#### SE-01 · **MAJOR** — El `X-Tenant-ID` no es validado como identificador limpio
- **Ubicación:** `http/extractors.rs:57-65`
- **Problema:** `TenantId::new(s)` acepta cualquier string del header sin sanitización. Un tenant ID con caracteres especiales puede llegar hasta queries SQL (aunque SeaORM parametriza, el valor se logueará tal cual).
- **Impacto:** Log injection si el tenant ID contiene saltos de línea. No hay inyección SQL directa gracias a SeaORM.
- **Recomendación:** Validar que el tenant ID solo contiene caracteres alfanuméricos, guiones y puntos en el extractor.
- **Esfuerzo:** S

#### SE-02 · **MINOR** — `connector_instance_id: Option<String>` en `NewTransferProcessCommand` no se usa
- **Ubicación:** `entities/commands.rs:42`
- **Problema:** El campo se deserializa de la request pero no se pasa a ningún repositorio ni dominio. Si contiene información sensible, se descarta silenciosamente sin registro.
- **Impacto:** Pérdida de datos inesperada para el cliente.
- **Recomendación:** O usar el campo o eliminarlo del command para evitar confusión.
- **Esfuerzo:** S

#### SE-03 · **MINOR** — `MessageEnvelope.raw_bytes` serializado completo en respuesta HTTP
- **Ubicación:** `services/transfer_message/views.rs:39`, `entities/transfer_message.rs:193`
- **Problema:** El envelope completo (incluyendo `raw_bytes` como base64 y el hash) se serializa en la vista. Según el protocolo, `raw_bytes` puede contener datos de negocio sensibles que no deberían estar en todos los endpoints de listado.
- **Impacto:** Potencial exposición de datos en `GET /transfer-messages`.
- **Recomendación:** Valorar si la vista de listado debe omitir `raw_bytes` y devolver solo `content_hash`.
- **Esfuerzo:** M · Requiere decisión de diseño.

---

### Performance

#### PE-01 · **MAJOR** — N+1 de identifiers en `get_all` (resuelto) pero `count` hace query separada sin aprovechar la paginación
- **Ubicación:** `services/transfer_process/service.rs:70-93`
- **Problema:** `get_all_transfer_processes` y `count_transfer_processes` se ejecutan secuencialmente con los mismos filtros. Son dos queries a BD donde podrían ser una (`SELECT ..., COUNT(*) OVER() AS total`). Además, el `count` **no aplica el cursor** — devuelve el total de todos los registros que coinciden con el filtro, no los restantes, lo cual puede ser confuso pero es probablemente intencional.
- **Recomendación:** Ejecutar con `tokio::join!` para paralelizar (ver AC-03). El total sin cursor es correcto semánticamente para mostrar "total de registros en el conjunto".
- **Esfuerzo:** S

#### PE-02 · **MINOR** — `ids.clone()` innecesario en `create` del servicio
- **Ubicación:** `services/transfer_process/service.rs:183-187`
- **Problema:** `.map(|ids| ids.clone()).unwrap_or_default()` — el `map(|ids| ids.clone())` es equivalente a `.cloned()` y se puede simplificar a `cmd.identifiers.clone().unwrap_or_default()`.
- **Esfuerzo:** S (nit de estilo con impacto de legibilidad)

#### PE-03 · **MINOR** — `HashMap` en `grouped` usa `String` keys cuando `Urn` podría evitar re-parsing
- **Ubicación:** `services/transfer_process/service.rs:82-88`
- **Problema:** Los IDs se convierten a `String` dos veces: al agrupar y al buscar en `grouped.remove(&p.id().to_string())`.
- **Impacto:** Allocaciones extra en respuestas grandes.
- **Recomendación:** Usar `Urn` directamente como clave si implementa `Hash + Eq`, o al menos evitar la doble conversión.
- **Esfuerzo:** S

#### PE-04 · **NIT** — `ser_enum` en `apply_base_filters` clona y alloca por cada filtro activo
- **Ubicación:** `data/sea_orm/repos/transfer_process.rs:65,72`
- **Problema:** Se crea un `String` temporal por cada filtro de enum activo. Menor pero repetitivo en cada query.
- **Recomendación:** Usar `ser_enum` de helpers (que ya hace esto) y pasar el `&str` directamente.
- **Esfuerzo:** S

---

### Documentación

#### DO-01 · **MAJOR** — Sin rustdoc en ningún tipo o trait público/crate-público
- **Ubicación:** Todo el crate
- **Problema:** Los traits de repositorio, service traits, tipos de dominio, commands y queries no tienen documentación. El uso de `pub(crate)` en casi todo reduce el impacto externo, pero la ausencia total dificulta el onboarding.
- **Recomendación:** Documentar al menos los traits de servicio, los tipos de query y los commands (son la interfaz que el HTTP layer expone).
- **Esfuerzo:** M

#### DO-02 · **NIT** — `grpc_worker.rs` es un stub con solo un comentario
- **Ubicación:** `setup/grpc_worker.rs`
- **Problema:** El archivo existe para satisfacer una declaración de módulo pero no hace nada. Confunde a nuevos contribuidores.
- **Recomendación:** Eliminar el archivo y la declaración del módulo, o añadir un `todo!()` explícito con ticket.
- **Esfuerzo:** S

---

### Build Hygiene

#### BH-01 · **MAJOR** — 5 dependencias declaradas no usadas en el código
- **Ubicación:** `Cargo.toml:17-20`
- **Problema:** `dataplane`, `negotiation_agent`, `catalog_agent`, `connector`, `events` aparecen en `[dependencies]` pero no hay ningún `use` de ellos en el código fuente. Esto infla el tiempo de compilación y el binario.
- **Impacto:** Tiempo de compilación muy elevado; si son grandes crates con macros proc, el impacto es significativo.
- **Recomendación:** Eliminar las dependencias no usadas. Usar `cargo +nightly udeps` para verificarlo.
- **Esfuerzo:** S · Candidato a regla de CI: _"`cargo udeps` en cada PR"_.

#### BH-02 · **MAJOR** — `#![allow(unused)]` global en `lib.rs`
- **Ubicación:** `src/lib.rs:18`
- **Problema:** Suprime todos los warnings de código muerto a nivel de crate. Oculta dead code legítimo (como `TransferProcessBuilder`, `TransferDirection`, `TransferProcessEvent`, `UnitOfWorkTrait`, `grpc_worker`).
- **Impacto:** El compilador no avisa de símbolos muertos; se acumula deuda silenciosa.
- **Recomendación:** Eliminar el allow global y solucionar cada warning individualmente o anotar con `#[allow(dead_code)]` solo donde sea deliberado.
- **Esfuerzo:** M

#### BH-03 · **MINOR** — Sin MSRV declarado
- **Ubicación:** `Cargo.toml`
- **Problema:** No hay `rust-version` en el manifest. El crate usa edition 2024 (estable desde 1.85) pero no lo declara, haciendo incierto el mínimo de Rust requerido.
- **Recomendación:** Añadir `rust-version = "1.85"`.
- **Esfuerzo:** S

#### BH-04 · **MINOR** — `log` y `tracing` importados simultáneamente
- **Ubicación:** `Cargo.toml:27,33`
- **Problema:** `log` y `tracing` coexisten. `tracing` tiene compatibilidad con `log` vía feature `log`, pero tener ambos puede generar duplicidad de macros.
- **Recomendación:** Eliminar `log` y usar solo `tracing` con la feature `log` si se necesita compatibilidad con crates que usan `log`.
- **Esfuerzo:** S

#### BH-05 · **NIT** — `services/filter.rs` duplica tipos de `entities/query.rs`
- **Ubicación:** `services/filter.rs:18-33`
- **Problema:** `Page`, `Paginated<T>`, `Sort` están definidos en `entities/query.rs` Y re-definidos en `services/filter.rs` (versión sin serde). El módulo `filter.rs` no se usa en ningún import.
- **Recomendación:** Eliminar `services/filter.rs`.
- **Esfuerzo:** S

---

## 3. Tabla resumen de hallazgos

| ID | Severidad | Dimensión | Esfuerzo | Regla reutilizable |
|----|-----------|-----------|----------|--------------------|
| TS-01 | **BLOCKER** | Testing | L | ✓ |
| AD-01 | MAJOR | API Design | S | |
| AD-02 | MAJOR | API Design | M | ✓ |
| EH-01 | MAJOR | Error Handling | S | ✓ |
| EH-02 | MAJOR | Error Handling | S | ✓ |
| AC-01 | MAJOR | Concurrencia | S | ✓ |
| AC-02 | MAJOR | Concurrencia | M | |
| MT-01 | MAJOR | Modelo de tipos | L | |
| MT-02 | MAJOR | Modelo de tipos | S | |
| OB-01 | MAJOR | Observabilidad | S | ✓ |
| SE-01 | MAJOR | Seguridad | S | |
| SE-03 | MINOR | Seguridad | M | |
| BH-01 | MAJOR | Build Hygiene | S | ✓ |
| BH-02 | MAJOR | Build Hygiene | M | |
| PE-01 | MAJOR | Performance | S | |
| TS-02 | MAJOR | Testing | S | |
| AD-03 | MINOR | API Design | S | |
| AD-04 | MINOR | API Design | S | |
| AD-05 | MINOR | API Design | S | |
| EH-03 | MINOR | Error Handling | S | |
| EH-04 | MINOR | Error Handling | S | |
| AC-03 | MINOR | Concurrencia | S | |
| MT-03 | MINOR | Modelo de tipos | S | |
| MT-04 | MINOR | Modelo de tipos | S | |
| OB-02 | MINOR | Observabilidad | S | |
| OB-03 | MINOR | Observabilidad | S | |
| SE-02 | MINOR | Seguridad | S | |
| PE-02 | MINOR | Performance | S | |
| PE-03 | MINOR | Performance | S | |
| DO-01 | MAJOR | Documentación | M | |
| BH-03 | MINOR | Build Hygiene | S | |
| BH-04 | MINOR | Build Hygiene | S | |
| AD-06 | NIT | API Design | S | |
| AD-07 | NIT | API Design | S | |
| MT-05 | NIT | Modelo de tipos | S | |
| PE-04 | NIT | Performance | S | |
| DO-02 | NIT | Documentación | S | |
| BH-05 | NIT | Build Hygiene | S | |

---

## 4. Resumen ejecutivo — Top 10 prioridades

| # | Hallazgo | Por qué es urgente |
|---|----------|--------------------|
| 1 | **TS-01** — Cero tests | Sin tests, cualquier cambio es un riesgo ciego. Es el desbloqueador para iterar con seguridad. |
| 2 | **EH-02** — Cursor inválido silenciado | Bug activo: un cursor malformado devuelve la primera página sin avisar, rompiendo la paginación del cliente. |
| 3 | **BH-01** — 5 dependencias no usadas | Infla el tiempo de compilación con crates grandes (dataplane, connectors). Fácil de eliminar. |
| 4 | **BH-02** — `#![allow(unused)]` global | Oculta dead code real. Eliminar revela los hallazgos AD-04, AD-07, BH-05 automáticamente. |
| 5 | **EH-01** — `unwrap()` en serialización de enums en filtros | Panic potencial en producción al añadir nuevos variants de enum. Corrección trivial con `ser_enum`. |
| 6 | **OB-01** — Sin spans en servicios/repos | Sin trazabilidad, es imposible diagnosticar latencias en producción. |
| 7 | **SE-01** — Tenant ID sin validar | Log injection potencial; validación trivial en el extractor. |
| 8 | **MT-01** — `ProtocolState` como string libre | Representa el mayor riesgo de modelo: estados ilegales son imposibles de detectar sin tests. |
| 9 | **AC-01** — `std::Mutex` en repos async | Frágil ante cualquier refactor que añada un `await` dentro del lock. |
| 10 | **AD-02** — `rehydrate` accesible desde cualquier módulo | Viola la encapsulación del aggregate y permite construir estados inconsistentes. |
