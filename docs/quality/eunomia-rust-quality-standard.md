# Eunomia Rust Quality Standard

> Derivado exclusivamente de hallazgos en `audit-transfer-agent-ref.md` y correcciones en `applied-transfer-agent-ref.md`.
> Rama de referencia: `refactor/transfer-agent`.
> Nada en este documento es "best practice general": cada regla cita el hallazgo que la originó.

---

## 1. Principios

Derivados del patrón de los 38 hallazgos del primer audit.

**P1 — El compilador es el primer revisor.**
`#![allow(unused)]` a nivel de crate desactiva el análisis estático más básico. Cualquier símbolo muerto se suprime intencionadamente con `#[allow(dead_code)]` localizado, no con un silenciador global. *Origen: BH-02.*

**P2 — Un error ignorado es siempre un bug activo.**
Si una operación puede fallar (deserialización, decodificación de cursor, validación de header), el error se propaga con `?`. Silenciarlo con `unwrap_or_default()` o `if let Ok()` devuelve datos incorrectos al cliente sin que nadie lo sepa. *Origen: EH-01, EH-02.*

**P3 — La superficie pública es un contrato.**
Todo lo que no necesita ser visible fuera del crate es `pub(crate)` o menos. Exponer más de lo necesario es deuda de API: eliminar un símbolo público es un breaking change. *Origen: AD-01, AD-03.*

**P4 — Las dependencias no usadas son deuda de compilación.**
Cada crate en `[dependencies]` que no tiene un `use` en el código cuesta tiempo de compilación en cada CI. Se auditan antes de mergear. *Origen: BH-01.*

**P5 — La observabilidad es infraestructura, no opcional.**
Sin spans en los métodos de servicio, un entorno de producción es una caja negra. `#[tracing::instrument]` es tan obligatorio como `pub` o `async`. *Origen: OB-01.*

**P6 — Los tipos llevan su validación consigo.**
Un `String` que representa un `TenantId` validado y un `String` arbitrario son indistinguibles para el compilador. Los newtypes y los constructores validadores hacen que el estado inválido sea irrepresentable. *Origen: SE-01, MT-02.*

**P7 — Async y locks estándar no se mezclan.**
`std::sync::MutexGuard` no puede cruzar un punto de await. El scope del guard se acota explícitamente antes de cualquier `.await`. *Origen: AC-01.*

---

## 2. Reglas obligatorias

### API Design

#### R-API-01 — Un único punto de entrada público por funcionalidad
**Regla:** Si un método de struct y una función libre ofrecen la misma firma, elimina la función libre o restrígela a `pub(crate)`.  
**Racional:** Dos entradas con la misma semántica generan ambigüedad y duplican el surface de breaking changes. *(BH-02, AD-01)*  
**Bien:**
```rust
// La función libre es pub(crate): solo el método es la API externa.
pub(crate) async fn create_root_http_router(...) -> Outcome<Router> { ... }

impl TransferHttpWorker {
    pub async fn create_root_http_router(...) -> Outcome<Router> {
        create_root_http_router(...).await?  // delega en la interna
    }
}
```
**Mal:**
```rust
pub async fn create_root_http_router(...) -> Outcome<Router> { ... }  // duplicado público
```
**Detección:** `grep -rn "^pub async fn\|^pub fn" src/setup/` — dos funciones con el mismo nombre es señal.

---

#### R-API-02 — Campos de struct de dominio son `pub(crate)`, no `pub`
**Regla:** Los campos de structs de dominio (`TransferProcess`, `TransferMessage`, etc.) usan `pub(crate)`. Las vistas HTTP y los types expuestos externamente pueden tener `pub`.  
**Racional:** `pub` en una struct `pub(crate)` no añade accesibilidad real pero confunde la intención. Consistencia con el estándar del crate. *(AD-03)*  
**Bien:**
```rust
pub(crate) struct TransferMessage {
    pub(crate) id: MessageId,
    pub(crate) envelope: MessageEnvelope,
}
```
**Mal:**
```rust
pub(crate) struct TransferMessage {
    pub id: MessageId,  // aparenta ser más público de lo que es
}
```
**Detección:** `grep -n "^\s*pub [a-z]" src/entities/` — campos con `pub` sin modificador en structs de dominio.

---

#### R-API-03 — Structs con campos `pub` externas llevan `#[non_exhaustive]`
**Regla:** Cualquier struct con campos `pub` expuesta como parte de la API de un crate lleva `#[non_exhaustive]`.  
**Racional:** Añadir un campo en el futuro es un breaking change sin `#[non_exhaustive]`. *(AD-05)*  
**Bien:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StateMetadata {
    pub attribute: Option<String>,
    pub reason: Option<Vec<String>>,
}
```
**Detección:** `grep -B3 "^pub struct" src/entities/protocol.rs | grep -v non_exhaustive`.

---

#### R-API-04 — Los parámetros de trait usan nombres semánticos
**Regla:** El nombre del parámetro en la firma del trait refleja su semántica, no su posición. `cmd`, `id`, `filters`, nunca `batch_request` para un `&NewTransferProcessCommand`. *(AD-06)*  
**Detección:** Revisión humana al definir o modificar traits.

---

### Error Handling

#### R-EH-01 — Nunca `unwrap()` en serialización de enums de dominio
**Regla:** Para serializar enums a `String` en filtros SQL o queries, usar `ser_enum(&value)` del módulo `data/sea_orm/orm/helpers.rs`. Nunca `serde_json::to_value(x).unwrap().as_str().unwrap()`.  
**Racional:** `to_value().unwrap()` hace panic si el enum tiene un variant que no serializa a string. `ser_enum` usa `expect` con mensaje claro y centraliza la lógica. *(EH-01)*  
**Bien:**
```rust
q = q.filter(orm::Column::Protocol.eq(ser_enum(&filters.protocol)));
```
**Mal:**
```rust
q = q.filter(orm::Column::Protocol.eq(
    serde_json::to_value(&filters.protocol).unwrap().as_str().unwrap().to_string()
));
```
**Detección:** `grep -rn "to_value.*unwrap" src/data/`.

---

#### R-EH-02 — Un cursor inválido es siempre un error 400, nunca silencio
**Regla:** Si un cursor de paginación no se puede decodificar (base64 inválido, timestamp mal formado), devolver `Err` con una variante `InvalidCursor`. Nunca ejecutar la query sin filtro de cursor.  
**Racional:** Silenciar el error devuelve la primera página como si el cursor fuera vacío. El cliente no sabe que su cursor está corrupto. *(EH-02)*  
**Bien:**
```rust
#[allow(clippy::result_large_err)]
fn decode_cursor(&self, cursor: &str) -> Outcome<DateTime<FixedOffset>> {
    let bytes = URL_SAFE_NO_PAD.decode(cursor)
        .map_err(|_| TransferProcessRepoErrors::InvalidCursor.into_errors())?;
    let s = String::from_utf8(bytes)
        .map_err(|_| TransferProcessRepoErrors::InvalidCursor.into_errors())?;
    DateTime::parse_from_rfc3339(&s)
        .map_err(|_| TransferProcessRepoErrors::InvalidCursor.into_errors())
}
```
**Mal:**
```rust
if let Ok(cursor_dt) = self.decode_cursor(cursor) {
    q = q.filter(orm::Column::CreatedAt.gt(cursor_dt));
}
// Si falla, la query continúa sin filtro — bug silencioso.
```
**Detección:** `grep -n "if let Ok.*cursor\|decode_cursor" src/data/` — cualquier uso de `if let Ok` en decodificación de cursor.

---

#### R-EH-03 — Los errores de repo son tipados y granulares por operación
**Regla:** Cada repo tiene un enum `XxxRepoErrors` con variantes por operación: `ErrorFetchingXxx`, `ErrorCreatingXxx`, `ErrorUpdatingXxx`, `ErrorDeletingXxx`, `XxxNotFound`, `InvalidCursor`. Implementa `RepoIntoErrors` (de `ymir`).  
**Racional:** Un solo `db_err` para fetch, count, update y delete impide diagnosticar qué operación falló en los logs. *(EH-03)*  
**Bien:**
```rust
#[derive(Debug, Error)]
pub enum TransferProcessRepoErrors {
    #[error("Transfer Process not found")]
    TransferProcessNotFound,
    #[error("Invalid pagination cursor")]
    InvalidCursor,
    #[error("Error fetching transfer process. {0}")]
    ErrorFetchingTransferProcess(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error creating transfer process. {0}")]
    ErrorCreatingTransferProcess(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error updating transfer process. {0}")]
    ErrorUpdatingTransferProcess(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error deleting transfer process. {0}")]
    ErrorDeletingTransferProcess(Box<dyn std::error::Error + Send + Sync>),
}
impl RepoIntoErrors for TransferProcessRepoErrors {}
```
**Detección:** `grep -n "fn.*_err\|fn db_err" src/data/sea_orm/repos/` — un único helper de error para todas las operaciones.

---

#### R-EH-04 — `TryFrom` con error tipado, nunca `String`
**Regla:** `impl TryFrom<Input> for DomainType` usa `type Error = XxxError` con un tipo concreto derivado de `thiserror::Error`. Nunca `type Error = String`.  
**Racional:** `String` como tipo de error impide pattern matching en el llamador y elimina la posibilidad de gestión estructurada. *(MT-02)*  
**Bien:**
```rust
#[derive(Debug, Error)]
pub(crate) enum EnvelopeError {
    #[error("invalid base64 in field '{field}': {source}")]
    InvalidBase64 { field: &'static str, source: base64::DecodeError },
}

impl TryFrom<MessageEnvelopeInput> for MessageEnvelope {
    type Error = EnvelopeError;
    fn try_from(input: MessageEnvelopeInput) -> Result<Self, EnvelopeError> { ... }
}
```
**Mal:**
```rust
impl TryFrom<MessageEnvelopeInput> for MessageEnvelope {
    type Error = String;
    fn try_from(input: MessageEnvelopeInput) -> Result<Self, String> { ... }
}
```
**Detección:** `grep -n "type Error = String" src/`.

---

### Async / Concurrencia

#### R-AC-01 — `std::sync::MutexGuard` nunca cruza un `.await`
**Regla:** En código `async`, el scope de cualquier `MutexGuard` se acota con un bloque explícito antes del primer `await` posterior.  
**Racional:** Un `MutexGuard` que cruza un `.await` puede causar deadlock cuando el runtime reutiliza el hilo para otro task que necesita el mismo lock. *(AC-01)*  
**Bien:**
```rust
let updated = {
    let mut store = self.processes.lock().unwrap();
    let p = store.get_mut(&id.to_string())
        .ok_or_else(|| TransferProcessRepoErrors::TransferProcessNotFound.into_errors())?;
    p.apply_edit(edit_model.clone());
    p.clone()
}; // guard dropped here, antes de cualquier .await
self.some_other_repo.notify(&updated).await?;
```
**Mal:**
```rust
let mut store = self.processes.lock().unwrap();
let p = store.get_mut(&id).ok_or_else(|| ...)?;
p.apply_edit(cmd.clone());
self.some_other_repo.notify(p).await?;  // guard still held across await
```
**Detección:** `grep -A10 "\.lock()" src/data/in_memory/` — buscar `.await` en el mismo scope después de `lock()`.

---

#### R-AC-02 — Queries independientes se ejecutan en paralelo con `tokio::try_join!`
**Regla:** Cuando un método de servicio hace dos o más queries a BD que no dependen entre sí (ej. count + list), usar `tokio::try_join!` en lugar de dos `await` secuenciales.  
**Racional:** Dos awaits secuenciales con los mismos filtros añaden latencia de red sin beneficio. `try_join!` las ejecuta en paralelo y elimina la ventana de inconsistencia. *(PE-01, AC-03)*  
**Bien:**
```rust
let (items, total) = tokio::try_join!(
    self.repo.get_all(filters, page, sort),
    self.repo.count(filters),
)?;
```
**Mal:**
```rust
let items = self.repo.get_all(filters, page, sort).await?;
let total = self.repo.count(filters).await?;
```
**Detección:** `grep -A5 "get_all\|count_" src/services/` — dos awaits secuenciales con los mismos filtros.

---

### Tipos

#### R-MT-01 — Conversiones numéricas entre tipos de diferente signo usan `try_from`
**Regla:** Las conversiones entre tipos de distinto rango (ej. `u32` dominio vs `i32` BD) usan `try_from()` con un fallback explícito. Nunca `as u32` / `as i32` sin comentario.  
**Racional:** `as` trunca silenciosamente. Un `version: u32 = 3_000_000_000` guardado como `i32` devuelve `-1_294_967_296`. *(MT-03)*  
**Bien:**
```rust
let version = u32::try_from(self.version).unwrap_or(0);       // BD → dominio
version: Set(i32::try_from(process.version()).unwrap_or(i32::MAX)),  // dominio → BD
```
**Mal:**
```rust
let version = self.version as u32;
version: Set(process.version() as i32),
```
**Detección:** `grep -n " as u32\| as i32\| as usize" src/data/sea_orm/`.

---

#### R-MT-02 — Las claves mágicas de protocolo son constantes nombradas
**Regla:** Las strings literales que representan claves de protocolo DSP (ej. `"consumerPid"`, `"providerPid"`) se declaran como `pub(crate) const` en el módulo que las define y se referencian por nombre.  
**Racional:** Un typo en `"consumerPid"` es silencioso en tiempo de compilación y solo falla en runtime. Una constante nombrada rompe en compilación. *(MT-04)*  
**Bien:**
```rust
// entities/protocol.rs
pub(crate) const CONSUMER_PID_KEY: &str = "consumerPid";
pub(crate) const PROVIDER_PID_KEY: &str = "providerPid";

// views.rs
if let Some(v) = correlation.identifiers.get(CONSUMER_PID_KEY) { ... }
```
**Mal:**
```rust
if let Some(v) = correlation.identifiers.get("consumerPid") { ... }
```
**Detección:** `grep -rn '"consumerPid"\|"providerPid"' src/` — cualquier literal de estas claves fuera de la definición de la constante.

---

### Testing

#### R-TS-01 — `mockall::automock` solo en builds de test
**Regla:** Los traits de repositorio llevan `#[cfg_attr(test, mockall::automock)]`, no `#[mockall::automock]`.  
**Racional:** `#[mockall::automock]` genera código de mock en producción, incrementando el tiempo de compilación sin beneficio. *(TS-02)*  
**Bien:**
```rust
#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait TransferProcessRepoTrait: Send + Sync { ... }
```
**Mal:**
```rust
#[mockall::automock]
#[async_trait::async_trait]
pub trait TransferProcessRepoTrait: Send + Sync { ... }
```
**Detección:** `grep -rn "#\[mockall::automock\]" src/` — sin `cfg_attr`.

---

#### R-TS-02 — El backend in-memory existe y se preserva para tests
**Regla:** Todo crate con repos SeaORM tiene un módulo `data/in_memory/` con implementaciones funcionales. El código se anota con `#[allow(dead_code)]` si no hay tests aún, pero no se elimina.  
**Racional:** Sin backend in-memory, los tests de integración del service layer requieren una BD real. El in-memory permite tests unitarios rápidos y deterministas. *(TS-01)*  
**Detección:** `ls src/data/in_memory/` — debe existir.

---

### Observabilidad

#### R-OB-01 — Todo método de servicio lleva `#[tracing::instrument]` con `level = "info"`
**Regla:** Cada método de `impl XxxServiceTrait` lleva la macro `#[tracing::instrument(level = "info", ...)]`. El nivel es explícito. Los métodos con ID del recurso incluyen `fields(id = %id)`.  
**Racional:** El nivel por defecto de `#[instrument]` es `TRACE`. El filtro típico de producción es `INFO` o `DEBUG`. Sin `level = "info"` explícito, los spans son invisibles. *(OB-01, y el bug de configuración posterior)*  
**Bien:**
```rust
// Método sin ID significativo:
#[tracing::instrument(level = "info", skip_all, err)]
async fn get_all(&self, filters: &Filter, page: &Page, sort: &Sort) -> Outcome<Paginated<View>>

// Método con ID de recurso:
#[tracing::instrument(level = "info", skip(self), fields(id = %id), err)]
async fn get_one(&self, id: &Urn) -> Outcome<View>

// Método con ID + payload voluminoso:
#[tracing::instrument(level = "info", skip(self, cmd), fields(id = %id), err)]
async fn edit(&self, id: &Urn, cmd: &EditCommand) -> Outcome<View>
```
**Mal:**
```rust
#[tracing::instrument(skip_all, err)]  // nivel TRACE — invisible en producción
async fn get_all(...)
```
**Detección:** `grep -n "#\[tracing::instrument\]" src/services/` — sin parámetros, nivel implícito TRACE.

---

#### R-OB-02 — El subscriber fmt emite eventos de span y el filtro incluye INFO
**Regla:** El subscriber en `main.rs` incluye `with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)` y el `EnvFilter` tiene como mínimo nivel `INFO` o `DEBUG`. No `TRACE` como único nivel.  
**Racional:** `tracing_subscriber::fmt()` sin `with_span_events` no imprime apertura/cierre de spans — solo los eventos dentro. Con `FmtSpan::CLOSE` se ve la duración de cada span. *(OB-01, fix posterior)*  
**Bien:**
```rust
tracing_subscriber::fmt()
    .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
    .with_env_filter(EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .parse("debug,sqlx::query=off")?)
    .init();
```

---

#### R-OB-03 — El span de request HTTP propaga `X-Request-ID`
**Regla:** El `TraceLayer::make_span_with` usa el header `x-request-id` si está presente, y solo genera un UUID nuevo si está ausente. El nivel de `on_response` es al menos `INFO`.  
**Racional:** Un UUID generado en el servidor es inútil para correlacionar con los logs del cliente que envió el request-id. *(OB-02, OB-03)*  
**Bien:**
```rust
.make_span_with(|req: &Request<_>| {
    let id = req.headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_owned())
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    tracing::info_span!("request", id = %id)
})
.on_response(DefaultOnResponse::new().level(tracing::Level::INFO))
```

---

### Seguridad

#### R-SE-01 — Todo header que llega a logs o queries se valida en el extractor
**Regla:** Cualquier header usado como identificador (`X-Tenant-ID`, `X-Request-ID`) se valida en el extractor Axum antes de construir el newtype. La validación rechaza caracteres fuera del conjunto permitido.  
**Racional:** SeaORM parametriza las queries, pero los valores llegan a los logs tal cual. Un `X-Tenant-ID: foo\nINFO injected-log-line` es log injection. *(SE-01)*  
**Bien:**
```rust
fn is_safe_id(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '.')
}

// En el extractor:
if !is_safe_id(tenant_raw) {
    return Err(Errors::format(BadFormat::Received,
        "X-Tenant-ID contains invalid characters", None));
}
```
**Mal:**
```rust
let tenant_id = TenantId::new(parts.headers.get("x-tenant-id")?.to_str()?);
```
**Detección:** Revisión humana de cualquier `TenantId::new(` / `RequestId::new(` sin validación previa.

---

### Build

#### R-BH-01 — Sin dependencias no usadas en `[dependencies]`
**Regla:** Cada crate en `[dependencies]` tiene al menos un `use` en el código. Se verifica con `cargo +nightly udeps` o revisión manual antes de mergear.  
**Racional:** Crates no usados incrementan el tiempo de compilación. Si son crates con macros proc, el impacto es multiplicador. *(BH-01)*  
**Detección:** `cargo +nightly udeps -p transfer_agent_ref`.

---

#### R-BH-02 — Sin `#![allow(unused)]` a nivel de crate
**Regla:** `#![allow(unused)]` está prohibido en `lib.rs` y `main.rs`. El código muerto intencional lleva `#[allow(dead_code)]` en el símbolo concreto, con un comentario si la razón no es obvia.  
**Racional:** Un silenciador global oculta deuda acumulada. El compilador deja de avisar de símbolos muertos nuevos. *(BH-02)*  
**Detección:** `grep -n "allow(unused)" src/lib.rs src/main.rs`.

---

#### R-BH-03 — `rust-version` declarado en `Cargo.toml`
**Regla:** Todo crate nuevo declara `rust-version` en `[package]`. El valor es la edición mínima estabilizada requerida. Para edition 2024: `rust-version = "1.85"`.  
**Racional:** Sin MSRV declarado, las dependencias pueden exigir versiones más nuevas sin advertencia visible. *(BH-03)*  
**Detección:** `grep -l "rust-version" crates/*/Cargo.toml` — crates sin la clave.

---

## 3. Patrones canónicos de Eunomia

Estos son los patrones establecidos en `transfer-agent-ref` como referencia para módulos nuevos o refactors.

### 3.1 Pipeline ORM → Dominio → Vista

El flujo de datos tiene tres transformaciones distintas con responsabilidades separadas:

```
BD → orm::Model::into_domain() → DomainType → XxxView::assemble() → HTTP JSON
HTTP JSON → Command → orm::ActiveModel::from_cmd() → BD (create)
HTTP JSON → Command → DomainType::apply_edit() → orm::ActiveModel::from_domain() → BD (update)
```

**Capa ORM** (`data/sea_orm/orm/xxx.rs`): solo mapeo fiel de columnas. Sin lógica de negocio.
```rust
#[allow(clippy::result_large_err)]
impl Model {
    pub(crate) fn into_domain(self) -> Outcome<DomainType> {
        let id = parse_urn(&self.id, "xxx.id")?;
        let role = deser_enum::<Role>(&self.role)?;
        let metadata = deser_json::<Metadata>(self.metadata, "xxx.metadata")?;
        Ok(DomainType::rehydrate(id, role, metadata, ...))
    }
}

#[allow(clippy::result_large_err)]
impl ActiveModel {
    pub(crate) fn from_cmd(cmd: &NewXxxCommand) -> Outcome<Self> {
        let now = chrono::Utc::now();
        let domain = DomainType::rehydrate(
            cmd.id.clone().unwrap_or_else(XxxId::generate),
            cmd.role,
            now, now, 0, ...
        );
        Ok(Self::from_domain(&domain))
    }

    pub(crate) fn from_domain(domain: &DomainType) -> Self {
        Self {
            id: Set(domain.id().to_string()),
            role: Set(ser_enum(&domain.role())),
            version: Set(i32::try_from(domain.version()).unwrap_or(i32::MAX)),
            ...
        }
    }
}
```

**Helpers ORM** (`data/sea_orm/orm/helpers.rs`): funciones de bajo nivel reutilizables.
- `ser_enum(&value)` → `String` para filtros SQL
- `deser_enum::<T>(&str)` → `Outcome<T>` desde columna de BD
- `ser_json(&value)` → `Json` para columnas JSONB
- `deser_json::<T>(json, field)` → `Outcome<T>` desde columna JSONB
- `parse_urn(&str, field)` → `Outcome<Urn>` desde columna de texto

**Capa de dominio** (`entities/xxx.rs`): sin dependencias de infraestructura.
```rust
pub(crate) struct DomainType {
    // campos pub(crate), no pub
    pub(crate) id: XxxId,
    pub(crate) version: u32,
}

impl DomainType {
    // Constructor para reconstrucción desde persistencia (no para lógica nueva)
    pub(crate) fn rehydrate(id: XxxId, ..., version: u32, ...) -> Self { ... }

    // Mutación en memoria para updates (sin acceso a BD)
    pub(crate) fn apply_edit(&mut self, cmd: EditXxxCommand) { ... }

    // Accessors, nunca campos públicos en lógica de negocio
    pub(crate) fn id(&self) -> &XxxId { &self.id }
    pub(crate) fn version(&self) -> u32 { self.version }
}
```

**Vista** (`services/xxx/views.rs`): serialización para HTTP. Sin lógica.
```rust
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct XxxView { ... }

impl XxxView {
    pub(crate) fn assemble(domain: DomainType, extra: HashMap<String, String>) -> Self {
        Self { id: domain.id().clone(), ... }
    }
}
```

---

### 3.2 Trait de repositorio

```rust
#[allow(dead_code)]                          // intencional: usado en tests y service layer
#[cfg_attr(test, mockall::automock)]         // mock solo en test builds
#[async_trait::async_trait]
pub trait XxxRepoTrait: Send + Sync {
    async fn get_all(&self, filters: &XxxFilter, page: &Page, sort: &Sort) -> Outcome<Vec<Xxx>>;
    async fn count(&self, filters: &XxxFilter) -> Outcome<u64>;
    async fn get_batch(&self, ids: &[Urn]) -> Outcome<Vec<Xxx>>;   // slice, nunca Vec
    async fn get_by_id(&self, id: &Urn) -> Outcome<Option<Xxx>>;
    async fn create(&self, cmd: &NewXxxCommand) -> Outcome<Xxx>;
    async fn put(&self, id: &Urn, cmd: &EditXxxCommand) -> Outcome<Xxx>;
    async fn delete(&self, id: &Urn) -> Outcome<()>;
}

#[derive(Debug, Error)]
pub enum XxxRepoErrors {
    #[error("Xxx not found")]
    XxxNotFound,
    #[error("Invalid pagination cursor")]
    InvalidCursor,
    #[error("Error fetching xxx. {0}")]
    ErrorFetchingXxx(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error creating xxx. {0}")]
    ErrorCreatingXxx(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error updating xxx. {0}")]
    ErrorUpdatingXxx(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error deleting xxx. {0}")]
    ErrorDeletingXxx(Box<dyn std::error::Error + Send + Sync>),
}
impl RepoIntoErrors for XxxRepoErrors {}
```

---

### 3.3 Implementación SeaORM del repositorio

```rust
pub(crate) struct SeaOrmXxxRepo {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmXxxRepo {
    fn fetch_err(e: sea_orm::DbErr) -> Errors {
        XxxRepoErrors::ErrorFetchingXxx(Box::new(e)).into_errors()
    }

    #[allow(clippy::result_large_err)]
    fn decode_cursor(&self, cursor: &str) -> Outcome<DateTime<FixedOffset>> {
        let bytes = URL_SAFE_NO_PAD.decode(cursor)
            .map_err(|_| XxxRepoErrors::InvalidCursor.into_errors())?;
        let s = String::from_utf8(bytes)
            .map_err(|_| XxxRepoErrors::InvalidCursor.into_errors())?;
        DateTime::parse_from_rfc3339(&s)
            .map_err(|_| XxxRepoErrors::InvalidCursor.into_errors())
    }

    fn apply_base_filters(
        mut q: Select<orm::Entity>,
        filters: &XxxFilter,
    ) -> Select<orm::Entity> {
        if let Some(tid) = &filters.tenant_id {
            q = q.filter(orm::Column::TenantId.eq(tid.as_str()));
        }
        if let Some(state) = &filters.state {
            q = q.filter(orm::Column::State.eq(ser_enum(state)));
        }
        q
    }
}

#[async_trait::async_trait]
impl XxxRepoTrait for SeaOrmXxxRepo {
    async fn get_all(&self, filters: &XxxFilter, page: &Page, sort: &Sort) -> Outcome<Vec<Xxx>> {
        let mut q = Self::apply_base_filters(orm::Entity::find(), filters);
        if let Some(cursor) = &page.cursor {
            let dt = self.decode_cursor(cursor)?;    // ? propaga error, nunca silencio
            q = q.filter(orm::Column::CreatedAt.gt(dt));
        }
        q.limit(page.limit as u64)
            .all(self.db.as_ref()).await
            .map_err(Self::fetch_err)?
            .into_iter().map(orm::Model::into_domain).collect()
    }
}
```

---

### 3.4 Service layer

```rust
pub(crate) struct XxxService {
    repo: Arc<dyn XxxRepoTrait>,
}

impl XxxService {
    pub fn new(repo: Arc<dyn XxxRepoTrait>) -> Self { Self { repo } }
}

#[async_trait::async_trait]
impl XxxServiceTrait for XxxService {
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn get_all(&self, filters: &XxxFilter, page: &Page, sort: &Sort)
        -> Outcome<Paginated<XxxView>>
    {
        let (items, total) = tokio::try_join!(
            self.repo.get_all(filters, page, sort),
            self.repo.count(filters),
        )?;
        let next_cursor = (items.len() == page.limit as usize)
            .then(|| items.last().map(encode_cursor))
            .flatten();
        Ok(Paginated {
            items: items.into_iter().map(XxxView::assemble).collect(),
            next_cursor,
            total: Some(total),
        })
    }

    #[tracing::instrument(level = "info", skip(self), fields(id = %id), err)]
    async fn get_one(&self, id: &Urn) -> Outcome<XxxView> {
        let item = self.repo.get_by_id(id).await?
            .ok_or_else(|| XxxRepoErrors::XxxNotFound.into_errors())?;
        Ok(XxxView::assemble(item))
    }
}
```

---

### 3.5 Paginación cursor-based

El cursor es un timestamp RFC3339 codificado en base64url-no-pad:

```rust
fn encode_cursor(item: &DomainType) -> String {
    URL_SAFE_NO_PAD.encode(item.created_at().to_rfc3339())
}

// En el repo, decode_cursor() devuelve Outcome — ver R-EH-02.
// En la query SQL:
// ASC: WHERE created_at > cursor_dt ORDER BY created_at ASC
// DESC: WHERE created_at < cursor_dt ORDER BY created_at DESC
```

El `X-Total-Count` en la respuesta es el total de registros que coinciden con los filtros **sin** el cursor — semántica correcta para "total del conjunto, no total de páginas restantes".

---

### 3.6 HTTP layer: extractores y RBAC

Patrón establecido en `http/extractors.rs` y aplicado en todos los routers:

```rust
async fn handle_get_all(
    State(state): State<Self>,
    auth: AuthClaims,              // JWT validado por middleware — falla si ausente
    headers: ExtractedHeaders,     // X-Tenant-ID validado, X-Request-ID propagado
    Query(q): Query<XxxQuery>,
) -> AppResult<(HeaderMap, Json<Paginated<XxxView>>)> {
    Rbac::require_read(&auth, headers.tenant_id.as_str())?;
    // Si no es Admin, forzar el tenant_id del header en el filtro:
    let tenant_filter = (auth.role != Role::Admin).then(|| headers.tenant_id.clone());
    ...
    Ok((headers.response_headers_paged(result.total), Json(result)))
}
```

Los query params de paginación **no se flatten** desde `Page`:

```rust
// MAL: serde_urlencoded falla al deserializar u32 con #[serde(flatten)]
#[derive(Deserialize)]
struct XxxQuery {
    #[serde(flatten)]
    filter: XxxFilter,
    #[serde(flatten)]
    page: Page,  // falla en runtime
}

// BIEN: limit y cursor inlineados en el query struct
#[derive(Deserialize)]
struct XxxQuery {
    #[serde(flatten)]
    filter: XxxFilter,
    #[serde(default = "default_limit")]
    limit: u32,
    cursor: Option<String>,
    #[serde(default)]
    sort: Sort,
}
```

---

### 3.7 Factory de datos

La inyección de dependencias usa un trait `DataFactory` implementado por `SeaOrmDataFactory` (producción) e `InMemoryDataFactory` (tests):

```rust
pub(crate) trait DataFactory {
    fn xxx_repo(&self) -> Arc<dyn XxxRepoTrait>;
    fn yyy_repo(&self) -> Arc<dyn YyyRepoTrait>;
}
```

El service layer solo conoce los traits de repositorio, nunca la implementación concreta.

---

## 4. Anti-patrones observados

Estos patrones aparecieron en `transfer-agent-ref` antes del refactor. Se documentan para que no reaparezcan en código nuevo.

| Anti-patrón | Problema | Commit que lo corrigió |
|-------------|----------|------------------------|
| `#![allow(unused)]` en `lib.rs` | Silencia todos los warnings de dead code a nivel de crate | `272c7b14` |
| `serde_json::to_value(x).unwrap().as_str().unwrap()` para serializar enums en filtros SQL | Panic si el enum añade un variant que no serializa a string | `668d4839` |
| `if let Ok(cursor_dt) = decode_cursor(cursor) { q = q.filter(...) }` | Cursor inválido → query sin filtro → devuelve primera página sin aviso | `64bd16e7` |
| `std::sync::MutexGuard` mantenido implícitamente hasta el final del scope async | Riesgo de deadlock si se añade `.await` dentro del scope | `96598416` |
| `type Error = String` en `impl TryFrom` de tipos de dominio | Imposible hacer pattern matching; errors genéricos en el cliente | `ad6d722d` |
| `#[mockall::automock]` sin `cfg_attr(test, ...)` | Genera código de mock en builds de producción | `59726b31` |
| `#[tracing::instrument(skip_all, err)]` sin `level = "info"` | Span al nivel TRACE, invisible con filtro DEBUG/INFO | `15a9712c` |
| `Uuid::new_v4()` en `make_span_with` ignorando `X-Request-ID` | Correlación cliente-servidor imposible | `18f73fd4` |
| `HashMap<String, V>` con `id.transfer_process_id.to_string()` como clave cuando `Urn: Hash + Eq` | Doble conversión → doble allocación por cada item | `47987416` |
| `TransferProcessBuilder` definido pero nunca usado | Dead code que confunde sobre cuál es el path canónico de construcción | `272c7b14` |
| `dataplane`, `negotiation_agent`, etc. en `[dependencies]` sin `use` | Inflan el tiempo de compilación | `72591c8f` |
| `DefaultOnResponse::new().level(tracing::Level::TRACE)` | Respuestas HTTP nunca se loguean en producción | `18f73fd4` |

---

## 5. Checklist de PR

Copiar en la plantilla de PR de GitHub (`.github/pull_request_template.md`):

```markdown
## Checklist de calidad

### Antes de pedir review

- [ ] `cargo clippy -p <crate> --no-deps -- -D warnings` pasa sin errores
- [ ] `cargo test -p <crate>` pasa (si hay tests)
- [ ] `cargo doc --no-deps -p <crate>` pasa sin warnings
- [ ] No hay `#![allow(unused)]` nuevo en lib.rs / main.rs
- [ ] No hay `#[mockall::automock]` sin `cfg_attr(test, ...)` 
- [ ] No hay `unwrap()` nuevo en serialización de enums para SQL
- [ ] No hay `.as u32` / `.as i32` nuevas sin comentario
- [ ] Los métodos de servicio nuevos tienen `#[tracing::instrument(level = "info", ...)]`
- [ ] Los cursores de paginación fallan con error, no con silencio

### Si añades un crate nuevo

- [ ] `rust-version` declarado en `Cargo.toml`
- [ ] Sin dependencias en `[dependencies]` sin un `use` correspondiente
- [ ] `DataFactory` trait + implementación in-memory + implementación SeaORM
- [ ] Backend in-memory funcional (aunque no haya tests aún)
- [ ] Traits de repo con variantes de error granulares por operación

### Si añades un endpoint nuevo

- [ ] Validación de `X-Tenant-ID` delega en `ExtractedHeaders` (no reimplementar)
- [ ] RBAC check (`Rbac::require_read` / `require_write`) como primera línea del handler
- [ ] `limit` y `cursor` inlineados en el query struct, no en `Page` con `#[serde(flatten)]`
- [ ] `on_response` configurado con `tracing::Level::INFO` mínimo
```

---

## 6. Configuración compartida propuesta

### `.cargo/config.toml`

```toml
[alias]
# Ejecutar clippy en un crate sin propagar warnings de dependencias
ck = "clippy --no-deps -- -D warnings"
# Uso: cargo ck -p transfer_agent_ref
```

**Justificación:** `--no-deps` es crítico. Sin él, warnings de crates de workspace que no controlamos (`oauth`, `common`) rompen `cargo clippy -- -D warnings` aunque el crate objetivo esté limpio.

---

### Propuesta de `clippy.toml` (en la raíz del workspace)

```toml
# Permite Result<T, ymir::errors::Errors> que es large por diseño de ymir.
# Sin este allow, todo método que devuelve Outcome<T> es un warning.
# Aplicar con: #[allow(clippy::result_large_err)] en las funciones afectadas.
# No se pone a nivel global aquí porque queremos que sea explícito en cada sitio.

# El único ajuste global válido a día de hoy:
cognitive-complexity-threshold = 30   # default 25, sube levemente para repos con muchos filtros
```

**Justificación:** `result_large_err` no tiene solución en código porque `ymir::Errors` no implementa `Box<dyn Error>`. Se suprime por sitio con `#[allow]`, no globalmente, para que sea visible.

---

### Convención de `EnvFilter` en `main.rs`

```rust
// Patrón canónico derivado de transfer-agent-ref:
let filter = EnvFilter::builder()
    .with_default_directive(LevelFilter::INFO.into())
    .parse("debug,sqlx::query=off")  // debug general, queries de sqlx silenciadas
    .map_err(|e| Errors::crazy(e.to_string(), Some(Box::new(e))))?;

tracing_subscriber::fmt()
    .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)  // ver duración de spans
    .event_format(tracing_subscriber::fmt::format().with_line_number(true))
    .with_env_filter(filter)
    .init();
```

Nunca `tracing_subscriber::fmt::init()` directamente — no configura el filtro.

---

## 7. Proceso para aplicar a un crate nuevo

### Orden de ejecución

```
1. cargo clippy -p <crate> --no-deps -- -D warnings
   → Registrar todos los errores y warnings. Sin corregir nada todavía.

2. Lectura completa del código fuente.
   → Generar audit-<crate>.md con la misma estructura que audit-transfer-agent-ref.md.

3. Aplicar hallazgos en este orden de prioridad:
   a. BH-xx primero: eliminar allow(unused) global, limpiar deps no usadas.
      → Esto expone dead code real que otros hallazgos necesitan ver.
   b. EH-xx: propagación de errores, cursores, unwrap en serialización.
      → Son bugs activos, no estilo.
   c. SE-xx: validación de inputs en extractores HTTP.
   d. AC-xx: Mutex scope, tokio::try_join!.
   e. MT-xx: tipos incorrectos, TryFrom, constantes.
   f. OB-xx: tracing::instrument, subscriber, propagación de headers.
   g. TS-xx: cfg_attr en mocks, estructura del backend in-memory.
   h. AD-xx, PE-xx: API pública, allocaciones evitables.
   i. DO-xx, BH-03: rustdoc, MSRV.

4. Un commit por hallazgo o grupo cohesivo (máximo 3 relacionados).
   Formato: quality(<área>): <hallazgo> — refs audit#<id>

5. Después de cada commit:
   cargo clippy -p <crate> --no-deps -- -D warnings
   cargo test -p <crate>

6. Crear open-questions.md con los hallazgos que requieren decisión de diseño.

7. Crear applied-<crate>.md con el registro de lo aplicado.
```

### Criterios de "done"

- `cargo clippy -p <crate> --no-deps -- -D warnings` pasa limpio.
- No hay `#![allow(unused)]` en `lib.rs` / `main.rs`.
- No hay `#[mockall::automock]` sin `cfg_attr`.
- Los métodos de servicio tienen `#[tracing::instrument(level = "info", ...)]`.
- Los cursores de paginación fallan con error tipado, no con silencio.
- `rust-version` declarado en `Cargo.toml`.
- `applied-<crate>.md` generado.
- `open-questions.md` actualizado con los deferred.

### Guía para vibe coding (generación de código con IA)

Cuando se usa un LLM para generar un módulo nuevo o refactorizar uno existente, verificar antes de commitear:

1. **Nivel de `#[instrument]`:** el modelo tiende a omitir `level = "info"`. Buscarlo con `grep -n "instrument" src/services/` — si no tiene `level`, los spans serán invisibles.

2. **`as` casts en tipos numéricos:** buscar `grep -n " as u32\| as i32\| as usize" src/data/` — sustituir por `try_from`.

3. **Errores silenciados:** buscar `grep -n "unwrap_or_default\|if let Ok" src/data/` en paths de datos de request — cada uno es un candidato a bug.

4. **`pub` en structs `pub(crate)`:** el modelo tiende a generar campos `pub` por defecto. Revisar los campos de entidades de dominio.

5. **`#[mockall::automock]` sin guarda:** el modelo no conoce esta convención. Siempre añadir `cfg_attr(test, ...)`.

6. **Dependencias generadas en `Cargo.toml`:** verificar que cada crate añadido tiene al menos un `use` en el código.

7. **`serde(flatten)` en tipos de paginación:** el modelo tiende a usar `Page` con flatten. Confirmar que `limit` y `cursor` están inlineados en el query struct.
