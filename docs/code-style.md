# Guía de estilo de código — ds-protocol

Esta guía describe el "estilo de la casa" del workspace. No inventa nada: **destila los
patrones que ya existen** en los crates de referencia y los convierte en reglas aplicables
crate a crate. Cuando dudes cómo hacer algo, abre el archivo de referencia citado y cópialo.

Crates de referencia (el estándar de oro):

| Patrón | Crate / archivo de referencia |
|---|---|
| Arquitectura por capas (hexagonal moderado) | `keystore` |
| Entities = dominio | `keystore/src/entities/` |
| Services = casos de uso | `keystore/src/services/secrets/service.rs` |
| Data access (puertos + adaptadores) | `transfer-agent-ref/src/data/`, `keystore/src/data/` |
| Borde de transporte + autorización (`AccessScope`) | `transfer-agent-ref/src/services/access.rs`, `transfer-agent-ref/src/http/`, `transfer-agent-ref/src/grpc/` |
| Errores | `dataplane/src/errors/mod.rs` |
| Setup / cadena de instancias | `dataplane/src/setup/mod.rs`, `keystore/src/setup/mod.rs` |

---

## 0. Principios transversales

Estas reglas aplican a **todo** el código, sin excepción.

### 0.1 Nada de funciones sueltas

El código se organiza en **tipos con métodos**, no en funciones libres dispersas por un módulo.
La unidad de organización es el `struct`/`trait` + su `impl`.

- Una operación pertenece al tipo que posee sus datos o sus dependencias.
- Si descubres que escribes una función libre, pregúntate **a qué tipo pertenece** y hazla método.

### 0.2 Si una función libre es inevitable, recibe TODO por parámetros

A veces hace falta un helper libre (lógica pura reutilizable, sin estado, compartida entre dos
`impl`). Cuando ocurra:

- **Todas sus dependencias entran por parámetro.** Cero estado global, cero `static` mutable,
  cero lectura de entorno dentro de la función, cero `Singleton::get()`.
- Debe ser **pura y testeable en aislamiento**: mismas entradas → misma salida.
- Vive junto al `impl` que la usa, o en un módulo `utils`/helpers explícito.

Referencia: `create_process_record` en transfer-agent (helper compartido por protocol y RPC) —
recibe el contexto completo por parámetro en lugar de leer estado.

```rust
// ✅ helper libre aceptable: todo entra por parámetro, es puro
fn build_target_url(egress: &Egress, path: Option<&str>, query: Option<&str>) -> Result<String, ProxyError> { ... }

// ❌ prohibido: depende de estado implícito
fn build_target_url() -> String { GLOBAL_CONFIG.lock().unwrap().base_url.clone() }
```

### 0.3 Inyección de dependencias siempre explícita

Las dependencias se reciben en el **constructor** (`new`) o vía **builder** (`with_*`) y se
guardan como `Arc<dyn Trait>`. El resto del código depende de **traits, nunca de tipos
concretos**. El único lugar que conoce los tipos concretos es `setup/`.

### 0.4 Estilo de archivo

- Cabecera de licencia GPL en cada archivo `.rs` (copia la de cualquier archivo existente).
- **Doc-comment (`///`) en cada método público** y en cada tipo público. Explica el *porqué*,
  no el *qué*.
- Agrupa métodos con separadores de sección: `// --- Validación y lookup ---------------`.
- Pipelines lineales: un método público que orquesta + helpers privados cortos y nombrados.
  Referencia: `TestingHTTPProxy::proxy` en `dataplane/src/testing_proxy/http/http.rs`.

---

## 1. Arquitectura por capas (hexagonal moderado)

`keystore` es el ejemplo canónico. Hexagonal **pragmático**, no dogmático: separamos dominio,
casos de uso y adaptadores, pero sin ceremonia innecesaria (sin DTOs duplicados por capa si no
aportan, sin mappers gratuitos).

```
crates/<agent>/src/
├── entities/        # DOMINIO: tipos, invariantes, comandos. Sin I/O, sin frameworks.
├── services/        # CASOS DE USO: orquestan dominio + puertos. Un trait (puerto) + *Impl.
├── data/            # ADAPTADORES DE SALIDA (persistencia)
│   ├── repo/        #   - PUERTOS: traits *RepoTrait + sus *RepoErrors
│   ├── sea_orm/     #   - ADAPTADOR SQL  (repos/, orm/, migrations/)
│   ├── in_memory/   #   - ADAPTADOR en memoria (tests / dev)
│   ├── vault/       #   - ADAPTADOR opcional (decorador sobre otro repo)
│   └── factory.rs   #   - DataFactory: entrega Arc<dyn *RepoTrait>
├── http/ | grpc/    # ADAPTADORES DE ENTRADA: routers/handlers → llaman a services
├── setup/           # COMPOSITION ROOT: cablea tipos concretos (único sitio que los conoce)
└── errors/ (o error.rs)  # Enum de error del crate + From<_> for Errors
```

**Regla de dependencias** (las flechas solo apuntan hacia adentro):

```
http/grpc  ──►  services  ──►  entities
                   │
                   ▼
              data/repo (puertos)  ◄──  data/sea_orm, in_memory, vault (adaptadores)
```

- `entities` no depende de nadie (ni de `data`, ni de `services`, ni de axum/sea-orm).
- `services` depende de `entities` y de los **puertos** `data/repo`, nunca de los adaptadores.
- `setup` es el único módulo que importa tipos concretos de `data/sea_orm`, etc.

---

## 2. Entities = dominio

`entities/` es el dominio puro. Referencia: `keystore/src/entities/`.

- Un archivo por concepto: `key.rs`, `version.rs`, `metadata.rs`, `secret_value.rs`, `entry.rs`.
- Tipos de comando agrupados en `commands.rs` (`NewSecretCommand`, `EditSecretCommand`).
- **Sin I/O, sin async, sin frameworks** (nada de axum, sea-orm, reqwest aquí).
- Newtypes para identidades y valores con invariantes (`Key`, `KeyPrefix`, `Version`) en lugar
  de `String`/`u64` sueltos.
- Genéricos cuando aportan: `Entry<T>` + alias `type SecretEntry = Entry<SecretValue>`.

```rust
// keystore/src/entities/entry.rs
#[derive(Clone, Debug, Serialize)]
pub struct Entry<T> {
    pub metadata: Metadata,
    pub value: T,
}
pub type SecretEntry = Entry<SecretValue>;
```

> En `dataplane` el dominio vive bajo `entities/` con submódulos por agregado
> (`dataplane_transfers/`, `dataplane_manager/`, ...). Mismo principio: `entities/` = dominio.

---

## 3. Services = casos de uso

Un caso de uso es un **trait (el puerto)** + una **`*Impl`** que lo implementa.
Referencia: `keystore/src/services/secrets/`.

Estructura por servicio:

```
services/<nombre>/
├── mod.rs       # re-exporta el trait público (el puerto del caso de uso)
├── service.rs   # *Impl: struct con sus deps (Arc<dyn ...>) + new(deps)
└── views.rs     # tipos de presentación/respuesta propios del servicio (si hacen falta)
```

Reglas:

- El `*Impl` recibe sus dependencias por `new(...)` y las guarda como `Arc<dyn Trait>`.
- Métodos `async`, devuelven `Outcome<T>` (el `Result` unificado de `ymir`).
- Instrumentación con `#[tracing::instrument(level = "info", skip_all, err)]` en cada método.
- La lógica de negocio vive aquí; el repo solo persiste. (Ej.: `upsert` decide create vs put.)

```rust
// keystore/src/services/secrets/service.rs
pub struct SecretStoreImpl {
    repo: Arc<dyn SecretRepoTrait>,
}

impl SecretStoreImpl {
    pub fn new(repo: Arc<dyn SecretRepoTrait>) -> Self { Self { repo } }
}

#[async_trait::async_trait]
impl SecretStore for SecretStoreImpl {
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn create(&self, cmd: &NewSecretCommand) -> Outcome<SecretEntry> {
        self.repo.create_secret(cmd).await
    }
    // ...
}
```

---

## 4. Data access — como `transfer-agent-ref` / `keystore`

Patrón puertos + adaptadores con factoría. Referencia: `transfer-agent-ref/src/data/`.

### 4.1 Puertos: `data/repo/`

- Un trait por agregado: `transfer_process.rs`, `transfer_message.rs`, `secrets.rs`...
- Cada trait: `Send + Sync`, métodos `async` que devuelven `Outcome<T>`.
- Decorar con `#[cfg_attr(test, mockall::automock)]` para poder mockear en tests.
- **El error del repo vive junto al trait**: un enum `*RepoErrors` con `thiserror` que implementa
  `RepoIntoErrors` (puente hacia `Errors` de ymir).

```rust
// keystore/src/data/repo/secrets.rs
#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait SecretRepoTrait: Send + Sync {
    async fn get_secret_by_key(&self, key: &Key) -> Outcome<Option<SecretEntry>>;
    async fn create_secret(&self, new_model: &NewSecretCommand) -> Outcome<SecretEntry>;
    // ...
}

#[derive(Debug, Error)]
pub enum SecretRepoErrors {
    #[error("Secret not found")]
    SecretNotFound,
    #[error("Version conflict: expected {expected:?}, actual {actual:?}")]
    VersionConflict { expected: Version, actual: Version },
    // ...
}
impl RepoIntoErrors for SecretRepoErrors {}
```

### 4.2 Adaptadores: `data/sea_orm/`, `data/in_memory/`, `data/vault/`

- `sea_orm/repos/` implementa los traits; `sea_orm/orm/` son los modelos ORM; `sea_orm/migrations/`.
- `in_memory/` implementa los mismos traits para tests y desarrollo.
- Un adaptador puede **decorar** a otro (ej.: `VaultSecretRepo` envuelve `SeaOrmSecretRepo`).
- Los tipos ORM **no salen** de `data/`: se mapean a entities del dominio en el borde.

### 4.3 Factoría: `data/factory.rs`

Un trait `DataFactory` que entrega los repos como `Arc<dyn *RepoTrait>`. El resto del crate
depende del factory/los traits, nunca del backend concreto.

```rust
// transfer-agent-ref/src/data/factory.rs
pub(crate) trait DataFactory: Send + Sync {
    fn transfer_process_repo(&self) -> Arc<dyn TransferProcessRepoTrait>;
    fn transfer_message_repo(&self) -> Arc<dyn TransferMessageRepoTrait>;
    fn transfer_identifier_repo(&self) -> Arc<dyn TransferIdentifierRepoTrait>;
}
```

---

## 5. Errores — como `dataplane`

Dos niveles, ambos confluyen en `Errors` de `ymir`.

### 5.1 Error de crate: `errors/mod.rs`

Referencia: `dataplane/src/errors/mod.rs`.

- **Un** enum `<Crate>Error` con `#[derive(Debug, Error)]` (thiserror).
- Variantes **agrupadas por dominio** con separadores de comentario:
  ```rust
  // Transfer lifecycle ──────────────────────────────────────────────
  // Driver / connector setup ────────────────────────────────────────
  // Proxy HTTP ──────────────────────────────────────────────────────
  ```
- Variantes con **campos nombrados**, nunca tuplas anónimas: `{ transfer_process_id: String }`.
- Mensajes con **prefijo del crate**: `#[error("Dataplane Error: ...")]`.
- Un `From<<Crate>Error> for Errors` que mapea cada variante al constructor semántico correcto
  de ymir (`missing_resource`, `not_impl`, `petition`, `format`, `crazy`, ...), usando una
  constante de prefijo corto (`const DP: &str = "[Dataplane]";`).

### 5.2 Errores locales de un handler

Cuando un error solo vive dentro de un módulo (p. ej. condiciones de salida temprana de un
handler HTTP), declara un enum **local** con su propio `IntoResponse`, mapeando cada variante a
su `StatusCode`. Referencia: `ProxyError` en `dataplane/src/testing_proxy/http/http.rs`.

### 5.3 Errores de repo

Viven en `data/repo/<x>.rs` como `*RepoErrors` + `impl RepoIntoErrors` (ver §4.1). Que el repo
falle es un detalle del adaptador; `RepoIntoErrors` lo traduce al `Errors` del dominio.

---

## 6. Setup / cadena de instancias

El `setup/` es el **composition root**: el único lugar que conoce tipos concretos y los cablea.
Referencias: `dataplane/src/setup/mod.rs` y `keystore/src/setup/mod.rs`.

Reglas de la "buena cadena de instancias":

1. **Un struct `<Crate>Setup`** como raíz (sin estado, o el mínimo). Implementa `Default` vía `new()`.
2. **Helpers privados pequeños** que construyen una pieza cada uno: `redis_client`, `build_repo`,
   `transfers_entity`, `build_keystore`. Nombre = lo que devuelven.
3. **Agrupa la infraestructura compartida en un struct `<Crate>Infra`** y constrúyela una sola vez
   (`build_infra`). Evita repetir el bootstrap de cache/repo en cada builder público.
4. **Métodos públicos `build_*` / `get_*`** como únicos puntos de entrada (uno por router/manager).
5. **Construcción fluida** con builder cuando hay piezas opcionales:
   `X::new(req).with_driver_factory(f).with_secret_store(s)`.
6. La selección de adaptador (real vs fake, vault vs plano) se decide **aquí**, no en services.

```rust
// dataplane/src/setup/mod.rs  — infraestructura compartida construida una vez
struct DataplaneInfra {
    cache: Arc<DataplaneTransferCacheForRedis>,
    repo: Arc<dyn DataplaneRepoTrait>,
}

impl DataplaneSetup {
    async fn build_infra(&self, config: &TransferConfig, vault: Arc<VaultService>) -> DataplaneInfra { ... }
    fn transfers_entity(&self, infra: &DataplaneInfra) -> Arc<DataplaneTransfersEntityService> { ... }

    pub async fn get_data_plane_manager(&self, /* deps por parámetro */) -> DataplaneManager {
        let infra = self.build_infra(config.as_ref(), vault.clone()).await;
        let entity = self.transfers_entity(&infra);
        let (keystore_lookup, secret_store) = self.build_keystore(config.as_ref(), vault).await;

        DataplaneManager::new(entity, connector_entity, config.clone())
            .with_driver_factory(Arc::new(DataplaneDriverFactory::new().with_keystore(keystore_lookup)))
            .with_secret_store(secret_store)
    }
}
```

---

## 7. Borde de transporte fino + autorización delegada (`AccessScope`)

Un crate puede exponer el **mismo caso de uso por varios transportes** (HTTP y gRPC en
`transfer-agent-ref`). La regla de oro:

> **Si una pieza de lógica aparece en el handler HTTP *y* en el handler gRPC, no es lógica de
> transporte: es del caso de uso y va al `service`.** El handler solo traduce el cable.

Esto evita lo que pasaba antes en `transfer-agent-ref`: RBAC, scoping por tenant, comprobación de
propiedad, filtrado de `batch`, forzado de tenant en `create`, clamp de `limit` y validación de
fechas estaban **duplicados y divergentes** entre HTTP y gRPC (p. ej. el clamp de `limit` tenía
máximo en HTTP pero no en gRPC; gRPC no validaba el rango de fechas).

### 7.1 Qué hace cada lado

**Adaptador de entrada (`http/`, `grpc/`) — fino, solo transporte:**

- Extraer credenciales/headers/metadata del protocolo (`Authorization`, `X-Tenant-ID`, …).
- Parsear path/query/body a tipos de dominio (URN, comandos, filtros).
- Construir el **`AccessScope`** del llamante y pasárselo al service.
- Mapear el resultado/`Errors` al formato del cable (JSON+status, o `Status` gRPC).
- **Nada de reglas de negocio ni de autorización por recurso aquí.**

**Service — dueño del caso de uso y de TODA la autorización:**

- Inyección del tenant en los filtros de listado (desde el `AccessScope`).
- Comprobación de propiedad en `get_one`/`edit`/`delete` → **404, no 403** (no se filtra la
  existencia de recursos de otros tenants).
- Filtrado del `batch` y forzado de tenant en `create`.
- Normalización transversal compartida: clamp de `limit`, validación de rango de fechas, topes.

### 7.2 `AccessScope`: construir una vez en el borde, decidir en el service

`AccessScope` (ver `transfer-agent-ref/src/services/access.rs`) es un **value object** que
encapsula RBAC + tenant del llamante. Se construye una sola vez en el handler y se pasa por
referencia al service, que es quien aplica las reglas.

```rust
// services/access.rs — el RBAC vive aquí dentro, no en cada transporte
#[derive(Debug, Clone)]
pub(crate) struct AccessScope { acting_tenant: TenantId, unrestricted: bool }

impl AccessScope {
    pub fn for_read(claims: &Claims, tenant: &TenantId) -> Outcome<Self> {
        Rbac::require_read(claims, tenant.as_str())?;     // RBAC encapsulado
        Ok(Self::from_role(claims.role, tenant))
    }
    pub fn for_write(claims: &Claims, tenant: &TenantId) -> Outcome<Self> { /* require_write */ }

    pub fn tenant_filter(&self) -> Option<TenantId> { (!self.unrestricted).then(|| self.acting_tenant.clone()) }
    pub fn acting_tenant(&self) -> &TenantId { &self.acting_tenant }
    pub fn permits(&self, owner: &TenantId) -> bool { self.unrestricted || &self.acting_tenant == owner }
}
```

```rust
// ✅ handler HTTP: solo transporte
let scope = AccessScope::for_read(&auth, &headers.tenant_id)?;
let urn = extract_path_urn(&id)?;
let view = state.service.get_one(&scope, &urn).await?;   // el service decide propiedad

// ❌ antes: el handler reimplementaba la regla (y gRPC la copiaba distinto)
Rbac::require_read(&auth, headers.tenant_id.as_str())?;
let view = state.service.get_one(&urn).await?;
if auth.role != Role::Admin && view.tenant_id != headers.tenant_id { return Err(not_found(&id)); }
```

```rust
// service: la regla vive una sola vez y la comparten HTTP y gRPC
async fn get_one(&self, scope: &AccessScope, id: &Urn) -> Outcome<View> {
    self.repo.get_by_id(id).await?
        .filter(|r| scope.permits(r.tenant_id()))   // foreign tenant → None → 404
        .ok_or_else(|| not_found(id))
        .map(View::assemble)
}
```

### 7.3 Mapeo de errores por transporte

El service devuelve siempre `Outcome<T>` (`Errors` de ymir, que ya lleva su `StatusCode`):

- **HTTP**: gratis, vía `IntoResponse for Errors` de ymir. El handler no mapea nada.
- **gRPC**: un **único** helper compartido `to_status(Errors) -> tonic::Status`
  (`transfer-agent-ref/src/grpc/mod.rs`) que respeta el `status_code` del dominio
  (404→`not_found`, 403→`permission_denied`, 400→`invalid_argument`, …). **Nunca** colapses todo a
  `Status::internal`: enmascara 404/403/400 como 500.

```rust
// grpc/mod.rs — compartido por todos los services gRPC del crate
pub(crate) fn to_status(err: Errors) -> Status {
    match err.info().status_code {
        StatusCode::NOT_FOUND => Status::not_found(err.reason().to_string()),
        StatusCode::FORBIDDEN => Status::permission_denied(err.reason().to_string()),
        StatusCode::UNAUTHORIZED => Status::unauthenticated(err.reason().to_string()),
        StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY => Status::invalid_argument(err.reason().to_string()),
        _ => Status::internal(err.reason().to_string()),
    }
}
```

> **Ojo con `RepoIntoErrors`**: su `into_errors()` por defecto envuelve en `Errors::db` (**500**).
> Para un "no encontrado" que debe ser **404**, el service traduce a `Errors::missing_resource`,
> no propagues el `*RepoErrors::NotFound` tal cual.

### 7.4 Reglas

- El `service` recibe `&AccessScope` como primer parámetro en toda operación con autorización.
- `404` (no `403`) para recursos de otro tenant; ambos transportes lo heredan del service.
- Mappers/extractors **no** inyectan el tenant: lo hace el service desde el `AccessScope`.
- Validación de entrada transversal (clamp, rangos) en el service, para que la compartan ambos
  transportes — no en `into_domain`/mappers.

---

## 8. Checklist de revisión (PR de homogeneización)

Antes de dar por bueno un crate:

- [ ] Sin funciones libres salvo helpers puros que reciben **todo** por parámetro (§0.1, §0.2).
- [ ] `entities/` no importa axum / sea-orm / reqwest ni nada de `data/` o `services/` (§2).
- [ ] Cada caso de uso es `trait` (puerto) + `*Impl` con deps por `new()` (§3).
- [ ] `data/repo/` define traits `*RepoTrait` + `*RepoErrors`; adaptadores en `sea_orm`/`in_memory` (§4).
- [ ] Existe un `DataFactory` que entrega `Arc<dyn *RepoTrait>` (§4.3).
- [ ] Handlers HTTP/gRPC finos: construyen `AccessScope` y delegan; sin reglas de negocio ni de
      autorización duplicadas entre transportes. Errores gRPC vía `to_status` (§7).
- [ ] Un `<Crate>Error` agrupado por secciones + `From<_> for Errors`; errores locales con `IntoResponse` (§5).
- [ ] `setup/` es composition root con `<Crate>Setup`, `<Crate>Infra` y builders `build_*`/`with_*` (§6).
- [ ] Licencia GPL + doc-comments en todo lo público.
- [ ] `cargo build && cargo clippy && cargo test` en verde, **sin cambios de comportamiento**.
