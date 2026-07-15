# Plan de evolución del DSL de conectores

**Estado:** planificación, sin implementación.
**Alcance:** (A) rediseño del DSL de `connector_template`, (B) inyección de módulos WASM hookeables, (C) reutilización de semántica Apache Camel, (E) semántica de canales para los 10 transfer cases del dataplane (`eunomia_dataplane_cases.pdf`), (F) ABI WASM v1 con ejemplo completo.
**Código de referencia:** `crates/connector/src/entities/connector_template/mod.rs`, `crates/connector/src/entities/parameters/*`, `crates/dataplane/src/entities/dataplane_manager/dataplane_driver_factory.rs`.

---

## 0. Diagnóstico del estado actual

Antes de proponer nada, lo que hay hoy y por qué se queda corto.

### 0.1 Estructura actual

```
ConnectorTemplateDto
├── metadata            (name, version, author, description)
├── authentication      enum cerrado: NoAuth | BasicAuth | BearerToken | ApiKey | OAuth2
├── interaction         enum cerrado: Pull { data_access } | Push { subscribe, unsubscribe? }
│                       └── ProtocolSpec: enum cerrado: Http | Kafka
└── parameters          Vec<ParameterDefinition> (name, title, type, required, default)
```

Placeholders soportados, cada uno con su propia regex (`parameters/mod.rs:136-171`):

| Sintaxis | Semántica |
|---|---|
| `{{__NAME__}}` | parámetro de instancia |
| `{{__SYS_*__}}` | parámetro de sistema (URN, token, timestamp, own_url…) |
| `{{__RUNTIME_JSON_{jq.path}__}}` | jq sobre respuestas previas |
| `{{__RUNTIME_PARAMETER_{/key}__}}` | parámetro runtime |
| `{{__RUNTIME_SECRET_{/key}__}}` | secreto del keystore |

### 0.2 Problemas concretos

1. **Tres enums cerrados en cascada** (`AuthenticationConfig`, `InteractionConfig`, `ProtocolSpec`). Añadir una tecnología (MQTT, gRPC, S3, OData…) exige tocar: el enum, `ConnectorTemplateWalker` (visita manual campo a campo), `DataplaneDriverFactory` (match `(protocolo, rol, modo)` hard-codeado), el OpenAPI y los tipos orval del GUI. Kafka está declarado en el DSL pero el runtime devuelve `FeatureNotImplemented` — síntoma de que el DSL promete más de lo que el runtime cumple.

2. **`HttpSpec` es irreal para casos de uso serios.** Una API real necesita: paginación, reintentos/timeouts, negociación de contenido, mapeo de respuesta, mTLS, query params estructurados, y a menudo *varias* llamadas encadenadas (login → fetch → logout). Hoy solo hay `url_template + method + headers + body_template`. Además `method: TemplateVecString` (lista de métodos) es un tipo incorrecto: una operación tiene *un* método.

3. **El ciclo de vida es fijo.** `Pull` = una operación; `Push` = subscribe + unsubscribe opcional. No hay sitio para healthcheck, renovación de suscripción, handshake previo, ni operaciones de validación.

4. **Los parámetros no tienen restricciones.** `ParameterDefinition` no puede expresar "enum de valores", "regex", "rango", "es secreto → keystore", ni agrupación para el formulario del GUI. `default_value` es siempre `String` aunque el tipo sea `Int`.

5. **Cinco gramáticas de placeholder distintas**, cada una con su regex y su resolver. La resolución además está acoplada al tipado (`TemplateInt`, `TemplateBoolean`… enums `untagged` que existen solo porque la sustitución ocurre *después* de deserializar a structs tipados).

6. **El walker es mantenimiento puro.** `ConnectorTemplateWalker` enumera a mano cada campo de cada variante. Cada campo nuevo = una edición más en el walker o un placeholder silenciosamente sin resolver.

---

## A. Rediseño del DSL

### A.1 Principios

- **El DSL describe *operaciones*, no protocolos.** El template declara un conjunto de operaciones con nombre; cada modo de interacción referencia las operaciones que necesita.
- **Extensibilidad por registro, no por enum.** Las tecnologías (bindings) se identifican por clave (`http`, `kafka`, `mqtt`, `wasm:<plugin>`) y se resuelven contra un registro. El core no conoce la lista completa.
- **Una sola gramática de placeholders.**
- **Validación por capacidades declaradas**, no por match hard-codeado en el driver factory.
- **`dslVersion` explícito** — el spec ya se persiste como JSON (`spec` en `NewConnectorTemplateModel`), así que la migración es un upgrade de serde, no de esquema SQL.

### A.2 Nueva forma del template: operaciones + flujo

La unidad central del DSL v2 es la **operación**: una acción nombrada, ejecutada por un binding, que puede exportar valores al contexto runtime. La **interacción** deja de ser un enum y pasa a ser un *flujo* de operaciones con dependencias — un DAG estilo Airflow — más la declaración de qué operación produce los datos.

#### A.2.1 Anatomía completa de una operación

```jsonc
"operations": {
  "fetch-orders": {
    "binding": "http",                 // clave del registro de bindings (A.7)
    "spec": { /* spec propio del binding — HttpSpec v2, S3Spec, etc. */ },

    // ── Campos comunes a TODA operación, independientes del binding ──

    "extract": {                       // qué exporta al contexto runtime.*
      "order_ids": "jq('.items[].id')",
      "next_sync": "jq('.sync_token')"
    },
    // Tras ejecutarse: {{ runtime.fetch-orders.order_ids }} disponible
    // para operaciones posteriores. Namespace = nombre de la operación,
    // así dos operaciones pueden exportar la misma clave sin colisión.

    "retry": {                         // override del default del sistema
      "max": 3,
      "backoff": "EXPONENTIAL",
      "retryIf": "jq('.status >= 500')"   // opcional; default: error del binding
    },
    "timeout": "PT30S",

    "condition": "{{ runtime.login.session_active }}"  // opcional: si resuelve
    // a falsy, la operación se marca SKIPPED (no ejecutada, no error)
  }
}
```

Decisiones:

- **`extract` sube del spec del binding al nivel de operación.** En el borrador anterior estaba dentro de `response` en HttpSpec; generalizado aquí, todo binding lo soporta uniformemente (sobre el "resultado canónico JSON" que cada binding define: body de la respuesta HTTP, mensaje Kafka, metadatos del objeto S3…). Es el equivalente exacto del XCom de Airflow, pero declarativo.
- **Namespacing por operación** (`runtime.<op>.<clave>`): evita el problema actual de `RUNTIME_JSON` donde el contexto es un saco plano y una operación puede pisar a otra.
- `retry`/`timeout` a nivel de operación (no solo dentro del spec HTTP) porque son semántica de *ejecución del flujo*, no del protocolo: reintentar una operación S3 y una HTTP es la misma decisión.
- `condition` cubre el 90% de lo que Airflow resuelve con `BranchOperator` sin introducir branching real en el DAG: la operación se salta, y sus dependientes deciden vía trigger rules (A.2.2) si corren igualmente.

#### A.2.2 `interaction`: el flujo como DAG

```jsonc
"interaction": {
  "mode": "PULL",

  // Cuándo se dispara el flujo (solo PULL):
  "trigger": {
    "onRequest": true,               // default: cada request del consumer ejecuta el flujo
    "schedule": "PT15M",             // opcional: polling; el resultado se cachea y el
                                     // consumer recibe la última materialización
    "staleness": "PT1H"              // opcional con schedule: máx. antigüedad aceptable
  },

  // El DAG: nodos = operaciones declaradas, aristas = dependsOn.
  "flow": {
    "login":   {},
    "fetch-orders": { "dependsOn": ["login"] },
    "fetch-customers": { "dependsOn": ["login"] },
    "merge":   { "dependsOn": ["fetch-orders", "fetch-customers"] },
    "logout":  { "dependsOn": ["merge"], "rule": "ALL_DONE" }
  },

  // Qué operación es la fuente de los datos entregados al consumer:
  "output": "merge",

  // Flujos secundarios del ciclo de vida, mismos DAGs (normalmente de un nodo):
  "healthcheck": { "flow": { "ping": {} }, "output": "ping" }
}
```

Y para PUSH, mismos bloques con nombres de fase:

```jsonc
"interaction": {
  "mode": "PUSH",
  "subscribe":   { "flow": { "auth": {}, "sub": { "dependsOn": ["auth"] } }, "output": "sub" },
  "unsubscribe": { "flow": { "unsub": {} } },
  "renew":       { "flow": { "renew": {} }, "every": "PT1H" }   // opcional
}
```

**Forma corta para el caso común.** La mayoría de flujos son lineales; obligar a escribir el DAG completo para "login y luego fetch" es hostil. Azúcar sintáctico con semántica Airflow (`>>`):

```jsonc
"flow": "login >> fetch-orders >> logout"
// ≡ fetch-orders dependsOn login; logout dependsOn fetch-orders
// Y el caso trivial de una sola operación:
"flow": "fetch"
```

El parser expande la forma corta a la larga al cargar; el modelo interno es único (el DAG explícito). Solo se admite `>>` encadenado (lineal) en la forma corta — fan-out/fan-in requieren la forma larga, deliberadamente: si el flujo es complejo, que se vea.

#### A.2.3 Semántica de ejecución (lo que se roba de Airflow y lo que no)

**Se adopta:**

| Concepto Airflow | Aquí |
|---|---|
| DAG + `dependsOn` | validado acíclico al crear el template; ejecución por orden topológico |
| Estados de task instance | `PENDING → RUNNING → SUCCESS \| FAILED \| SKIPPED` por operación, trazados en los logs del dataplane transfer (la tabla `dataplane_transfers.logs` ya existe) |
| Trigger rules | solo tres: `ALL_SUCCESS` (default), `ALL_DONE` (cleanup: logout/unsubscribe corren pase lo que pase), `ONE_FAILED` (compensación/alerta) |
| Retries por task | el bloque `retry` de A.2.1 |
| XCom | `extract` → `runtime.<op>.*` |
| `schedule` | solo para PULL con polling (trigger del flujo completo, no cron general) |

**Se descarta (decidido):**

- **Scheduler/executor como infraestructura**: no hay procesos worker ni cola; el flujo se ejecuta in-process en el dataplane, secuencialmente en orden topológico. Las ramas independientes del DAG *podrían* paralelizarse (tokio lo da casi gratis) — fase posterior, cuando un template real lo necesite; la semántica del DAG ya lo permite sin cambio de sintaxis.
- **Branching operators**: `condition` + trigger rules cubren los casos reales. Un `BranchOperator` de verdad es un ESB.
- **Sensors**: esperar-hasta-que es un `retry` con `retryIf` sobre una operación de consulta. Si algún día no basta, será un binding `wasm:`.
- **Backfill, catchup, pools, SLAs**: no aplican a un conector.

**Manejo de fallo del flujo:** una operación `FAILED` (agotados sus retries) marca el flujo como fallido salvo que ningún dependiente la necesite; las operaciones con `ALL_DONE` pendientes se ejecutan igualmente (cleanup garantizado) y después el transfer transiciona a error con el log completo del DAG. Es el contrato mínimo que hace que un `logout` o `unsubscribe` nunca se quede colgado.

**Límites duros de validación al crear el template:** DAG acíclico, toda operación referenciada existe, `output` alcanzable, toda referencia `runtime.<op>.<clave>` corresponde a un `extract` declarado de una operación que precede topológicamente a quien la usa. Este último punto convierte errores de runtime en errores de creación — es la validación más valiosa de todo el DSL v2.

- La compatibilidad con v1 sigue siendo mecánica: `Pull { data_access }` → una operación `fetch` + `"flow": "fetch"` + `output: fetch`; `Push { subscribe, unsubscribe }` → dos flujos de un nodo.

### A.3 Gramática unificada de placeholders

Sustituir las cinco regex por una sola gramática `{{ fuente.ruta | filtro }}`:

| v2 | v1 equivalente |
|---|---|
| `{{ params.API_KEY }}` | `{{__API_KEY__}}` |
| `{{ sys.own_url }}` / `{{ sys.own_url_docker }}` | `{{__SYS_OWN_URL__}}` |
| `{{ runtime.subscribe.data.ID }}` | `{{__RUNTIME_JSON_{subscribe.data.ID}__}}` |
| `{{ runtime_params./key }}` | `{{__RUNTIME_PARAMETER_{/key}__}}` |
| `{{ secrets./path/key }}` | `{{__RUNTIME_SECRET_{/key}__}}` |

- Un solo parser (una regex de captura `fuente` + `ruta` + filtros opcionales), un solo trait `PlaceholderSource` con implementaciones `params`, `sys`, `runtime`, `secrets` — y en el futuro fuentes WASM (sección B).
- Filtros opcionales estilo pipe para lo que hoy hace jq ad-hoc: `{{ runtime.body | jq('.items[0].id') }}`, `{{ params.USER | urlencode }}`, `| base64`. Empezar con `jq`, `urlencode`, `base64`; son los tres que el código actual ya necesita implícitamente.
- **Fase de resolución sobre `serde_json::Value`, no sobre structs.** El pipeline pasa a ser: `spec JSON → resolver placeholders sobre el Value → deserializar al struct tipado del binding`. Esto:
  - elimina `TemplateInt/TemplateBoolean/TemplateVecString/TemplateMapString` (existen solo porque hoy se resuelve post-tipado),
  - elimina `ConnectorTemplateWalker` entero — recorrer un `Value` es genérico, un campo nuevo nunca más se queda sin visitar,
  - permite que un placeholder produzca tipos no-string de forma natural (`"port": "{{ params.PORT }}"` → `8080` si el parámetro es `INT` y el placeholder ocupa el valor completo).
- Retrocompatibilidad: mientras conviva v1, el parser acepta ambas sintaxis (la v1 se normaliza a v2 al cargar); los templates v1 persistidos no se tocan.

### A.4 HttpSpec v2 — realista para casos de uso

```jsonc
{
  "url": "https://api.example.com/orders",
  "method": "GET",                          // un método, string con placeholders
  "query": { "since": "{{ runtime.last_sync }}" },
  "headers": { "Accept": "application/json" },
  "body": { "contentType": "application/json", "template": "{ ... }" },

  "response": {
    "expectStatus": [200, 201]
    // la extracción de valores ya no vive aquí: es el campo `extract`
    // a nivel de operación (A.2.1), uniforme para todos los bindings
  },

  "pagination": {                           // opcional
    "strategy": "CURSOR",                   // CURSOR | OFFSET | LINK_HEADER | NONE
    "cursorPath": "jq('.next_cursor')",
    "cursorParam": "cursor",
    "stopWhen": "jq('.items | length == 0')"
  },

  "resilience": {                           // opcional, con defaults del sistema
    "timeout": "PT30S",
    "retry": { "max": 3, "backoff": "EXPONENTIAL", "on": [502, 503, 429] },
    "rateLimit": { "requestsPerSecond": 10 }
  },

  "tls": {                                  // opcional → habilita mTLS
    "caCert": "{{ secrets./certs/ca }}",
    "clientCert": "{{ secrets./certs/client }}",
    "clientKey": "{{ secrets./certs/key }}",
    "insecureSkipVerify": false
  }
}
```

Notas de diseño:

- El `extract` a nivel de operación (A.2.1) reemplaza y formaliza el mecanismo `RUNTIME_JSON`: hoy la extracción está implícita ("lo que devolvió subscribe queda accesible por jq"); en v2 cada operación declara qué exporta y con qué nombre. Documentación gratis y validación en creación (referencias `runtime.*` sin exportador → error al crear el template).
- `pagination` y `resilience` son opcionales con defaults: el template mínimo sigue siendo pequeño. No implementar las cuatro estrategias de paginación de golpe: `NONE` + `CURSOR` cubren la mayoría; el resto cuando haya un template que lo pida.
- El `resilience.retry` del spec HTTP y el `retry` de la operación (A.2.1) se unifican en el segundo — el spec del binding solo conserva lo específico del protocolo (`rateLimit`, `tls`); durante el diseño fino decidir si `timeout` vive en uno u otro, pero en un solo sitio.

### A.5 ParameterDefinition v2

```jsonc
{
  "name": "REGION",
  "title": "Región AWS",
  "description": "…",
  "type": "STRING",
  "required": true,
  "default": "eu-west-1",                   // tipado como su type, no siempre string
  "constraints": {                          // subconjunto de JSON Schema
    "enum": ["eu-west-1", "us-east-1"],
    "pattern": null, "minimum": null, "maximum": null,
    "minLength": null, "maxLength": null
  },
  "sensitive": false,                       // true → el valor va al keystore, nunca a la BD en claro
  "group": "Conexión"                       // agrupación para el formulario del GUI
}
```

- **`constraints` es deliberadamente un subconjunto de JSON Schema** (mismos nombres de campo). Dos razones: el GUI puede validar con cualquier librería JSON Schema sin código nuevo, y es exactamente lo que hacen los Kamelets de Camel (ver C.3) — convergencia gratuita.
- `sensitive: true` sustituye la convención implícita actual de "usa `SecretString` y reza": al instanciar, el valor se escribe en el keystore y el spec persiste solo la referencia `{{ secrets./instances/<id>/<name> }}`.
- `sys` params dejan de ser un tipo de parámetro declarable y pasan a ser solo una fuente de placeholder (`{{ sys.* }}`): hoy `SysParameterType` mezcla dos conceptos (declaración vs resolución).

### A.6 Autenticación

Mantener `authentication` como sección propia (no como operación): el dataplane la necesita identificada para elegir authenticator y para el flujo de proxy. Cambios:

- Mismo patrón de registro que los bindings: `{ "type": "OAUTH2", ... }` donde `type` se resuelve contra un registro de authenticators. Variantes nuevas de corto plazo con demanda real: `MTLS` (ya casi gratis con `tls` de A.4), `PRIVATE_KEY_JWT` (OAuth2 con client assertion), `CUSTOM` → delega en un plugin WASM (sección B).
- `OAuth2` v2: añadir `audience`, `extraParams` (map), y `tokenCaching` explícito.

### A.7 Registro de bindings y validación por capacidades

El reemplazo del match hard-codeado de `DataplaneDriverFactory`:

```rust
// Conceptual — no implementar aún
trait BindingDescriptor {
    fn key(&self) -> &str;                        // "http", "kafka", "wasm:<plugin>"
    fn capabilities(&self) -> Capabilities;        // { pull, push_subscribe, streaming, proxying }
    fn config_schema(&self) -> serde_json::Value;  // JSON Schema del spec — el GUI renderiza el form
    fn validate(&self, spec: &Value) -> Outcome<()>;
    fn make_executor(&self, spec: &Value) -> Outcome<Arc<dyn OperationExecutor>>;
}

// El contrato uniforme que ejecuta el DAG de A.2.2:
trait OperationExecutor {
    // spec ya resuelto (placeholders sustituidos) + contexto runtime acumulado.
    // Devuelve el "resultado canónico JSON" sobre el que se aplica `extract`.
    async fn execute(&self, ctx: &OperationContext) -> Outcome<serde_json::Value>;
}
```

- Al **crear el template**: cada operación se valida contra el `config_schema` de su binding, y el `interaction.mode` contra las `capabilities` (un binding sin `push_subscribe` no puede ser la operación `subscribe`). Esto convierte el error runtime `FeatureNotImplemented` de Kafka-push en un error de validación en tiempo de creación, que es donde debe estar.
- `config_schema` viaja también por la API: el GUI deja de necesitar tipos orval por protocolo y renderiza formularios desde el schema. Se acaba el ciclo "nuevo protocolo → regenerar orval → tocar React".
- Los bindings nativos se registran al arrancar; los WASM se registran al instalar el plugin (sección B.5). La clave `wasm:<plugin>` hace el puente.

#### A.7.1 Catálogo inicial de bindings

Criterio de selección: lo que un dataspace real mueve hoy (APIs HTTP, ficheros en object storage, streams), no lo que quedaría bonito en la lista. Tres tiers:

**Tier 1 — con el DSL v2 desde el día uno:**

| Binding | Capabilities | Por qué primero |
|---|---|---|
| `http` | pull, push_subscribe | El caballo de batalla; ya existe, se migra a HttpSpec v2. Cubre REST, OData, webhooks, la mayoría de APIs de datos. |
| `s3` | pull | Object storage S3-compatible (AWS, MinIO). Los dataspaces intercambian ficheros constantemente y hoy hay que disfrazarlo de HTTP con URLs prefirmadas hechas a mano. Operaciones: `get-object`, `list-objects`, `presign` (genera URL prefirmada como resultado — encaja con el modelo proxy del dataplane). |
| `wasm:*` | según manifiesto | El puente de extensibilidad (sección B). No es un protocolo: es la garantía de que ningún protocolo futuro bloquea el core. |

**Tier 2 — siguiente iteración, con demanda ya visible:**

| Binding | Capabilities | Nota |
|---|---|---|
| `kafka` | pull (consumer), push_subscribe (producer/consumer) | Ya está *prometido* en el DSL v1 y el runtime lo rechaza — o se completa aquí o se retira del DSL con la validación por capacidades. No dejarlo en el limbo actual. Spec v2: `brokers`, `topic`, `groupId`, `security` (SASL/TLS), `offsetReset`, `maxBatch`. |
| `mqtt` | push_subscribe | IoT dataspaces (sensores, smart city — relevante en el contexto UPM). Spec pequeño: `broker`, `topic`, `qos`, `tls`. |

**Tier 3 — solo bajo demanda de un template real (candidatos naturales a plugin `wasm:` antes que a binding nativo):**

`sftp/ftp` (legado industrial), `sql` (consulta directa a BD — cuidado: superficie de seguridad grande, mejor como plugin), `grpc` (requiere gestión de descriptores/proto, complejidad alta para demanda incierta), `amqp`, `file` (solo tests/demos).

Regla general: **un binding nuevo entra como plugin `wasm:` primero**; se promociona a nativo cuando su uso lo justifique (rendimiento, streaming real que el contrato JSON del plugin no cubra bien, o adopción amplia). Así el catálogo nativo crece por evidencia, no por especulación — y la existencia del tier `wasm:` es lo que hace defendible mantener el tier 1 en tres entradas.

**El resultado canónico JSON por binding** (sobre lo que opera `extract`):

| Binding | Resultado canónico |
|---|---|
| `http` | `{ "status": 200, "headers": {...}, "body": <JSON o string> }` |
| `s3` | `{ "objects": [{ "key", "size", "etag", "lastModified" }], "presignedUrl": ... }` según operación |
| `kafka` | `{ "messages": [{ "key", "value", "offset", "timestamp", "headers" }] }` |
| `wasm:*` | lo que devuelva el plugin (JSON libre, descrito en su manifiesto) |

Definir este resultado canónico en el `config_schema` de cada binding: es lo que permite validar los `extract` en creación y documentar qué puede referenciar `runtime.*`.

### A.8 Fases de implementación de A

| Fase | Contenido | Riesgo |
|---|---|---|
| A1 | `dslVersion` + gramática unificada de placeholders + resolución sobre `Value` (elimina walker y `Template*`) | Medio — toca el corazón de la resolución; mitigar con suite de equivalencia v1↔v2 sobre los templates existentes |
| A2 | `operations` + `flow` DAG (ejecutor topológico secuencial, trigger rules, `extract`→`runtime.<op>.*`) + forma corta `>>` + adaptador v1→v2 | Medio — el ejecutor de DAG es la pieza nueva más grande; acotado por descartar paralelismo, branching y scheduler |
| A3 | HttpSpec v2 (`resilience`, `tls`; `pagination` solo `NONE`/`CURSOR`) | Medio |
| A4 | Registro de bindings + capacidades + validación en creación; binding `s3` como segunda implementación (valida que el registro no está sesgado a HTTP) | Medio |
| A5 | ParameterDefinition v2 + formularios GUI desde `config_schema` | Bajo, mucho volumen de GUI |
| A6 | Tier 2 de bindings (`kafka` completo o retirado, `mqtt`) + `trigger.schedule` para PULL con polling | Bajo — todo aditivo sobre A2/A4 |

A1 y A2 son el prerequisito de todo lo demás (incluida la parte B). A3–A5 son paralelizables.

---

## B. Módulos WASM hookeables

### B.1 Objetivo y modelo mental

Permitir que terceros extiendan el conector **sin recompilar el agente**: nuevos bindings de protocolo, authenticators custom, transformaciones, y hooks de observación/mutación en el ciclo de vida del dataplane. WASM y no dylibs/subprocesos porque:

- **Sandbox por defecto**: un módulo WASM no tiene sockets, ni filesystem, ni reloj salvo que el host se los dé. Para un componente que va a ejecutar lógica de terceros dentro de un agente que maneja credenciales, esto no es un nice-to-have, es el requisito.
- Portable (un `.wasm` corre en cualquier despliegue), versionable por hash, y escribible en Rust/Go/JS/Python.

**Principio rector: I/O mediado por el host.** El plugin nunca abre conexiones: pide al host `http_request(...)` y el host aplica política (allowlist de hosts, timeouts, cuotas) centralmente. Los secretos solo entran al plugin si su manifiesto los declara y el operador los concedió.

### B.2 Elección de runtime

| Opción | Pros | Contras |
|---|---|---|
| **wasmtime + Component Model (WIT)** | Estándar (WASI p2), tipado fuerte de interfaces, interrupción por epoch, límites de memoria/fuel, respaldo Bytecode Alliance | Component Model aún en maduración; más plumbing inicial |
| **Extism** (capa sobre wasmtime) | Trivial de integrar (JSON in/out), SDKs de plugin en 10+ lenguajes, host functions fáciles | Interfaz débilmente tipada (bytes/JSON), una abstracción más |
| wasmer | Similar a wasmtime | Menos tracción en el ecosistema Rust server-side |

**Recomendación: empezar con Extism.** Los contratos de esta sección son todos "JSON entra, JSON sale", que es exactamente el modelo de Extism, y el time-to-first-plugin es días en vez de semanas. Diseñar los contratos como *interfaces lógicas* (B.4) para que una migración posterior a WIT/Component Model sea mecánica si el tipado fuerte acaba compensando. El coste de serializar JSON en frontera es irrelevante frente a la latencia de red que estos plugins median.

### B.3 Puntos de extensión

Cuatro tipos de plugin, alineados con las costuras que ya existen en el código:

1. **`binding`** — implementa un protocolo nuevo. Se registra en el registro de bindings de A.7 como `wasm:<nombre>`. Exporta las operaciones del ciclo de vida que soporte (declaradas como capabilities en su manifiesto): `fetch`, `subscribe`, `unsubscribe`, `healthcheck`. Es el equivalente WASM de `HttpSpec` + `HttpPubSubscriber`.
2. **`authenticator`** — implementa `AuthenticationConfig::Custom`. Espejo de `DriverAuthenticatorTrait`: recibe contexto, devuelve credenciales materializadas (headers/token/query params). Casos: AWS SigV4, HMAC propietario, firmas de body.
3. **`transformer`** — transforma payloads en frontera (request body antes de enviar, response antes de extraer). También registrable como filtro de placeholder (`{{ x | plugin('mi-transform') }}`). Casos: XML→JSON, formatos propietarios, cifrado de campo.
4. **`hook`** — observador/interceptor del ciclo de vida del dataplane (B.6).

### B.4 Contrato de plugin (interfaz lógica)

Toda función de plugin tiene la misma forma: `f(input_json) -> output_json`. Contratos por tipo:

```jsonc
// binding.fetch / binding.subscribe / ...
// IN:
{
  "operation": "subscribe",
  "spec": { /* el spec de la operación, placeholders YA resueltos */ },
  "context": {
    "transferProcessId": "urn:...", "role": "PROVIDER", "mode": "PUSH",
    "callbackUrl": "https://...",
    "runtime": { /* valores extraídos por operaciones previas */ }
  },
  "config": { /* config del plugin declarada en el template */ }
}
// OUT:
{ "extract": { "subscription_id": "abc" }, "status": "OK" }  // o { "error": {...} }
```

```jsonc
// authenticator.authenticate
// IN:  { "authConfig": {...}, "context": {...} }
// OUT: { "headers": {...}, "queryParams": {...}, "expiresAt": "..." }

// hook.on_event  — ver B.6
// IN:  { "phase": "before_subscribe", "context": { /* snapshot inmutable */ } }
// OUT: { "action": "CONTINUE" }
//   |  { "action": "PATCH", "patch": { /* RFC 7396 merge patch sobre partes mutables */ } }
//   |  { "action": "VETO", "reason": "..." }
```

Funciones host importables (la superficie completa, deliberadamente pequeña):

| Host fn | Semántica | Control |
|---|---|---|
| `http_request(req) -> resp` | única vía de red | allowlist de hosts del manifiesto, timeout, tamaño máx. de respuesta |
| `secret_get(path) -> value` | lectura de keystore | solo paths declarados en manifiesto y concedidos al instalar |
| `kv_get/kv_set(key, value)` | estado persistente por (plugin, instancia de conector) | cuota de tamaño |
| `log(level, msg)` | logging al tracing del host | rate limit |
| `now() -> timestamp` | reloj | — |

Nada más. Ni filesystem, ni env vars, ni sockets crudos. Cada función host adicional futura es una decisión de seguridad explícita.

**Decisión importante — hooks reciben snapshot + devuelven patch, no `&mut`:** el `DataplaneContext` interno nunca cruza la frontera WASM. El host serializa un snapshot de la parte expuesta, y aplica el merge patch devuelto solo sobre campos whitelisted (p.ej. headers salientes sí, estado del transfer no). Esto mantiene el determinismo del state machine y hace imposible que un plugin corrompa estado interno.

### B.5 Manifiesto, empaquetado y registro

Un plugin se distribuye como `.wasm` + manifiesto:

```jsonc
{
  "name": "sap-odata",
  "version": "1.2.0",
  "kind": "binding",                        // binding | authenticator | transformer | hook
  "capabilities": { "pull": true, "push_subscribe": false },
  "configSchema": { /* JSON Schema de su bloque config — el GUI renderiza el form */ },
  "permissions": {
    "network": ["*.sap.example.com:443"],
    "secrets": ["/plugins/sap-odata/*"],
    "kv": true
  },
  "hooks": ["before_request", "after_response"],   // solo kind=hook
  "checksum": "sha256:...",
  "signature": "..."                        // opcional, fase 2
}
```

- **Almacenamiento**: nueva tabla `plugins` (manifiesto + metadatos) y el binario en blob/objeto, direccionado por hash (integridad + dedupe + cache de compilación). Endpoints CRUD nuevos en el crate `connector` (misma estructura que `connector_template`: DTO + service trait + data layer).
- **Instalación = concesión de permisos**: el operador ve `permissions` y las aprueba. El host functions solo honran lo concedido, no lo pedido.
- **Referencia desde el template**: `"binding": "wasm:sap-odata@1.2"`, con el bloque `"config"` validado contra `configSchema`. Versionado semver con pin por template (un template no cambia de comportamiento porque alguien suba un plugin nuevo).

### B.6 Hooks del ciclo de vida

Fases de hook, derivadas de las transiciones que el dataplane ya tiene (`DataplaneCommandStateMachine` / los handlers `set_init`, `set_subscribing`, `set_started`…):

```
on_transfer_init
before_authenticate / after_authenticate
before_subscribe / after_subscribe
before_request / after_response          (por cada request de datos — el más útil)
before_unsubscribe / after_unsubscribe
on_transfer_complete / on_error
```

- **Registro**: el manifiesto declara a qué fases se suscribe; el host mantiene un `HookRegistry` (fase → lista ordenada de plugins). El orden es explícito (campo `priority` en la instalación), no implícito.
- **Semántica de ejecución**: secuencial por prioridad; `PATCH` se acumula; `VETO` corta la cadena y aborta la transición con error trazado. Timeout duro por hook (epoch interruption de wasmtime) — un hook colgado nunca cuelga un transfer: se trata como error del hook, y la política por hook decide si el error es fatal (`failClosed: true`) o solo se loguea.
- **Ámbito**: hooks globales (todas las transfers) o por template (declarados en el template: `"hooks": ["audit-logger@1"]`). Empezar solo con "por template" — el ámbito global es política de despliegue y puede esperar.
- Casos de uso que esto habilita sin tocar el core: auditoría/notificación externa, firmas de request, enriquecimiento de headers, watermarking de datos, políticas de rechazo custom.

### B.7 Ciclo de vida del runtime WASM en el host

- **Pool de instancias por hash de módulo**: compilar una vez (cache en disco keyed por hash), instanciar por ejecución o pool pequeño. Los plugins son stateless entre llamadas (estado solo vía `kv_*`), lo que hace el pooling trivial y el escalado horizontal gratis.
- **Límites por ejecución**: memoria máxima (p.ej. 64 MB), timeout (p.ej. 10 s binding / 1 s hook), fuel opcional. Todos configurables por despliegue.
- **Observabilidad**: cada invocación emite span de tracing con plugin@versión, fase, duración, y resultado. Los `log()` del plugin se anidan bajo ese span.

### B.8 SDK y experiencia de desarrollo

- Crate `connector-plugin-sdk` (Rust) con los tipos del contrato (serde) y helpers; con Extism, los SDKs de Go/JS/Python salen casi gratis reutilizando los JSON Schemas del contrato.
- **Harness de test**: binario `plugin-test` que ejecuta un plugin contra fixtures de contexto grabados (los `test_fixtures.rs` del dataplane ya tienen la forma adecuada) y valida el contrato. Es lo que hace viable que terceros desarrollen sin levantar el agente entero.
- Un plugin de ejemplo mantenido en el repo (p.ej. un authenticator HMAC) que sirve de plantilla y de test de integración del sistema completo.

### B.9 Fases de implementación de B

| Fase | Contenido | Depende de |
|---|---|---|
| B1 | Runtime host (Extism embebido) + host functions + límites; entidad `plugins` + CRUD | — |
| B2 | Tipo `authenticator` end-to-end (el más pequeño: una función, contrato mínimo) + SDK Rust + harness | B1 |
| B3 | Tipo `hook` + `HookRegistry` + emisión de eventos desde el state machine | B1 |
| B4 | Tipo `binding` integrado en el registro de A.7 | B1 + A4 |
| B5 | Tipo `transformer` + filtros de placeholder plugin | B1 + A1 |
| B6 | Firma de módulos, ámbito global de hooks, pool avanzado | resto |

Empezar por `authenticator` (B2) y no por `binding`: valida toda la cadena (manifiesto, permisos, host fns, límites, SDK, harness) con el contrato más pequeño posible, y es la carencia más citada (SigV4/HMAC no se pueden expresar hoy).

---

## C. Semántica Apache Camel

### C.1 Qué es reutilizable de Camel (y qué no)

Camel aporta tres cosas separables: (1) un **vocabulario** (Enterprise Integration Patterns: route, endpoint, exchange, processor, dead-letter…), (2) una **sintaxis de endpoints como URIs** (`kafka:orders?brokers=...`), y (3) un **runtime JVM** con ~300 componentes.

El runtime (3) no es viable embebido (JVM dentro de un agente Rust, no). Lo valioso es (1) y (2), y sobre todo una pieza concreta: **los Kamelets**.

### C.2 Endpoints como URIs

Adoptar la semántica URI de Camel para las operaciones como *forma corta* del DSL:

```jsonc
"operations": {
  "fetch": "https://api.example.com/orders?since={{ runtime.last_sync }}",
  "subscribe": "wasm:sap-odata:subscribe?dataset={{ params.DATASET }}"
}
// forma corta ≡ forma larga: el esquema de la URI selecciona el binding,
// los query params se mapean al spec. La forma larga (A.2) sigue disponible
// para specs con resilience/pagination/tls.
```

- El **esquema URI = clave del registro de bindings** de A.7. Es exactamente el modelo de componentes de Camel, y encaja sin fricción con los plugins WASM (`wasm:` como esquema).
- Ventaja real: los templates triviales (la mayoría) caben en una línea legible, y la semántica es familiar para cualquiera que venga de Camel/Spring.
- No adoptar el formato *interno* de opciones de componentes de Camel (cientos de opciones por componente): solo la convención sintáctica.

### C.3 Kamelets: el mismo concepto, robar la estructura

Un Kamelet de Camel K es *exactamente* lo que este proyecto llama connector template: un blueprint de ruta parametrizado, con:

- `spec.definition`: parámetros declarados **con JSON Schema** (title, description, type, default, `x-descriptors` para hints de UI) → valida la decisión de A.5 (constraints como subconjunto de JSON Schema). Conviene copiar los nombres de campo de Kamelet donde coincidan conceptualmente, para abaratar C.4.
- `spec.types`: tipos de entrada/salida declarados (media types) → equivalente a formalizar qué produce una operación (`response.extract` + content type), refuerza A.4.
- `spec.template`: el flujo (from → steps → to).

Acción concreta: al diseñar los JSON finales de A.2/A.5, tener el spec de Kamelet al lado y alinear nombres y estructura donde no cueste nada. No es dependencia, es convergencia deliberada.

### C.4 Pipeline de pasos (EIP subset) — opcional, fase tardía

Si en el futuro una operación necesita más que "una llamada": generalizar `operation.spec` a una lista de pasos con vocabulario EIP mínimo:

```jsonc
"fetch": {
  "from": "https://api.example.com/orders",
  "steps": [
    { "transform": { "jq": ".items[]" } },
    { "filter": { "jq": ".status == \"ACTIVE\"" } },
    { "wiretap": "wasm:audit-logger" },
    { "errorHandler": { "retry": 3, "deadLetter": "wasm:dlq-notifier" } }
  ]
}
```

- **Subset estricto**: pipelines lineales — `transform`, `filter`, `enrich` (segunda llamada), `wiretap`, `errorHandler`. Sin content-based router, sin splitter/aggregator, sin choice: eso es construir un ESB, que no es el objetivo de un conector de dataspace. Si algún día hace falta, el escape hatch es un plugin WASM `binding`, no más DSL.
- El modelo `Exchange` de Camel (message + headers + properties) ya tiene equivalente aquí: el snapshot de contexto de B.4. Formalizarlo con esos tres campos cuando se haga esto.

### C.5 Interoperabilidad directa con Camel (evaluar, no comprometer)

Dos opciones de menor a mayor coste, ninguna para la primera iteración:

1. **Importador de Kamelets**: un comando que traduce un Kamelet YAML (el subconjunto representable: source/sink HTTP y Kafka con parámetros) a un connector template v2. Valor: catálogo de cientos de Kamelets existentes como semilla de templates. Coste moderado, puramente aditivo. Candidato razonable a fase post-A.
2. **Camel K como sidecar**: para organizaciones que ya operan Camel, un binding `camel:` que delega la operación a un runtime Camel K externo vía HTTP. Coste de despliegue alto (JVM/K8s operator). Solo si aparece demanda real; el diseño de A.7 lo permite sin cambios (es un binding más).

---

## D. Orden global y dependencias

```
A1 (placeholders + resolución sobre Value)  ──┐
A2 (operations + interaction por referencia) ─┼─→ A3 (HttpSpec v2) → A5 (params v2 + GUI)
                                              └─→ A4 (registro bindings) ─→ B4 (binding WASM)
B1 (runtime WASM + plugins CRUD) → B2 (authenticator) → B3 (hooks) → B5 (transformer)
C2 (URIs) tras A4 · C3 informa A2/A5 desde el día 1 · C4/C5 sin fecha
```

- **Primer hito con valor visible**: A1+A2 (DSL v2 mínimo con migración transparente) + B1+B2 (primer plugin authenticator funcionando). Todo lo demás cuelga de esas cuatro piezas.
- **Riesgos principales**: (1) A1 toca la resolución de todos los templates existentes — la mitigación es una suite de equivalencia que resuelva cada template v1 real por ambas rutas y compare byte a byte antes de conmutar; (2) la superficie de seguridad de B — mantener las host functions en las cinco listadas y revisar cada adición como decisión de seguridad; (3) scope creep en C — el vocabulario sí, el ESB no.
- **Qué NO hacer** (decidido, no pendiente): embeber JVM/Camel; dar `&mut` del contexto a plugins; sockets crudos en WASM; implementar las 4 estrategias de paginación antes de que un template las pida; content-based routing en el DSL; construir un scheduler/executor tipo Airflow (el DAG se ejecuta in-process, secuencial, por orden topológico); branching operators (con `condition` + trigger rules basta); paralelizar ramas del DAG antes de que un template real lo necesite; más de 3 trigger rules; bindings nativos nuevos sin pasar antes por `wasm:`.

---

## E. Semántica de canales — cobertura de los 10 transfer cases

Fuente: `eunomia_dataplane_cases.pdf` (UPM, abril 2026). Los 10 casos revelan tres carencias del modelo A.2 tal como estaba:

1. **La fase activa de la mayoría de los casos no es una operación, es un canal.** Un proxy pgwire, un túnel TCP, un bucle consume-produce de Kafka o un bridge MQTT viven mientras vive el transfer. El DAG de operaciones (A.2.2) modela bien setup/teardown y pull request-response, pero no un proceso de larga duración con streaming y backpressure.
2. **La autenticación es asimétrica en todos los casos**: cómo se autentica el consumer ante el dataplane (ingress) y cómo se autentica el dataplane ante el backend real (egress) son cosas distintas (Bearer/OAuth2 en ingress, SigV4 en egress, en el caso 2). El `authentication` único del DSL v1/v2 no lo puede expresar.
3. **La autorización (PDP) es parte de la semántica del conector**, no un detalle del runtime: cada caso define de dónde sale el `subject`, cómo se mapea la `action`, qué es el `resource` y con qué granularidad se evalúa. Eso debe ser declarable en el template.

### E.1 `authentication` se divide en ingress/egress

```jsonc
"authentication": {
  "ingress": { "type": "BEARER_JWT", "jwks": "{{ params.JWKS_URL }}" },
  // tipos ingress: NO_AUTH | BEARER_JWT | API_KEY | MTLS | SCRAM_SERVER | IP_ACL | SSH_SERVER
  "egress":  { "type": "SIGV4", "accessKey": "{{ secrets./aws/ak }}",
               "secretKey": "{{ secrets./aws/sk }}", "region": "eu-west-1" }
  // tipos egress: los v1 (NoAuth/Basic/Bearer/ApiKey/OAuth2) + SIGV4 | MTLS | SCRAM |
  //               SASL_PLAIN | SASL_SCRAM | SASL_OAUTHBEARER | SSH_KEY | TRANSPARENT
}
```

- `TRANSPARENT` (casos 5 y 7): el handshake de autenticación del protocolo pasa por el túnel sin tocar; el dataplane solo *observa* (extrae el username SCRAM de Mongo para el subject del PDP). Es un tipo propio porque su semántica — "no autentiques, extrae identidad" — no es ninguna de las otras.
- Casos compuestos (caso 3, S3→S3 cross-cloud): `egress` admite forma por-destino: `{ "source": {...SIGV4...}, "destination": {...AZURE_SAS...} }`.
- Migración: el `authentication` v1 plano equivale a `egress` con `ingress: NO_AUTH`.

### E.2 El bloque `channel`: fase activa de larga duración

`interaction` se generaliza a tres fases; la activa es un flow (DAG, modelo A.2) **o** un canal:

```jsonc
"interaction": {
  "mode": "PULL",                          // semántica DSP hacia el consumer, sin cambios
  "setup":    { "flow": "provision-topic >> set-acls" },   // DAG de operaciones, opcional
  "active": {
    "type": "CHANNEL",                     // o "FLOW" (el modelo A.2.2, sigue válido)
    "channel": { /* ver abajo */ }
  },
  "teardown": { "flow": "revoke-acls" }    // rule ALL_DONE implícita: corre siempre
}
```

Anatomía del canal — cuatro bloques, calcados de los tres ejes ortogonales del PDF (authenticator ya está en E.1; proxy configurator → `family`+`spec`; subscriber → `monitor`) más la autorización:

```jsonc
"channel": {
  "binding": "postgres",
  "family": "PROTOCOL_SERVER",
  // HTTP_PROXY       — casos 1, 2, 8: listener HTTP/2, egress según binding
  // PROTOCOL_SERVER  — casos 4, 5: el dataplane SE HACE PASAR por el backend (pgwire, DPI Mongo)
  // ACTIVE_TRANSFER  — casos 3, 6, 9, 10: sin listener; tarea que mueve datos origen→destino
  // TCP_TUNNEL       — caso 7: copy_bidirectional, cero conocimiento del protocolo

  "spec": { /* spec del binding para el canal: listen port, endpoint egress, topic map… */ },

  "authorization": {                       // construcción declarativa del AuthzRequest
    "granularity": "PER_QUERY",
    // PER_CONNECTION | PER_TRANSFER | PER_TOPIC | PER_OBJECT | PER_FILE |
    // PER_REQUEST | PER_METHOD | PER_QUERY | PER_COMMAND | PER_MESSAGE
    "subject":  "{{ ingress.identity }}",  // resuelto por el tipo de ingress: sub del JWT,
                                           // CN del mTLS, username SCRAM/SASL, source IP
    "action":   { "from": "sql.operation",           // clave del contexto canónico del binding
                  "map": { "SELECT": "READ", "INSERT": "WRITE", "UPDATE": "WRITE",
                            "DELETE": "DELETE", "DROP": "DENY" } },
    "resource": "table:{{ sql.tables }}",
    "context":  { "transferId": "{{ sys.transfer_id }}",
                  "agreementId": "{{ sys.agreement_id }}", "sourceIp": "{{ ingress.ip }}" }
  },

  "streaming": {
    "mode": "PASSTHROUGH",                 // PASSTHROUGH | INSPECT_PREFIX | BUFFER
    "inspectPrefixBytes": 65536,           // solo INSPECT_PREFIX (patrón tee del PDF §5.1.3)
    "batch": { "rows": 5000 },             // protocol servers: cursor batching
    "chunk": "8MB",                        // active transfers: tamaño de parte/chunk
    "multipartThreshold": "5GB"            // s3: umbral de multipart
  },

  "monitor": {                             // qué mide el Subscriber/TransferMonitor
    "progress": "RECORDS",                 // BYTES | RECORDS | OBJECTS | MESSAGES | QUERIES
    "completion": "EXPLICIT"               // EXPLICIT (DSP) | CONSUMER_LAG (kafka) |
  }                                        // BYTE_COUNT | OBJECT_COUNT
}
```

Puntos de diseño:

- **`action.from` referencia el "contexto canónico" del canal**, el análogo streaming del resultado canónico JSON de A.7.1: cada binding de canal define qué claves expone por unidad autorizable (`http.method/path/headers`, `sql.operation/tables/columns`, `mongo.command/collection/db`, `kafka.topic/operation`, `mqtt.topic`, `s3.key/operation`, `file.path`, `tcp.target`). Se documenta en el `config_schema` del binding y se valida en creación, igual que `extract`.
- **La granularidad la limita el binding** (capability): `tcp` solo puede `PER_CONNECTION`; `postgres` llega a `PER_QUERY`. Granularidad no soportada → error de validación al crear el template, no sorpresa en runtime.
- **`streaming.mode` es la decisión buffer-vs-stream del PDF hecha explícita y auditable**: `PASSTHROUGH` autoriza por metadatos y transmite ciego (máximo rendimiento); `INSPECT_PREFIX` implementa el tee de N bytes para PDP con inspección de body; `BUFFER` solo para payloads pequeños con límite duro. El default del sistema es `PASSTHROUGH`.
- El backpressure NO es DSL: es obligación del runtime de cada familia (channels acotados, `copy_bidirectional`, pausa de cursor), como describe el PDF §5. El DSL solo declara tamaños.
- Los `setup`/`teardown` reutilizan el DAG tal cual: la provisión de topic + ACLs del caso 6 o el registro de webhook son operaciones one-shot normales — la separación operación/canal es exactamente la separación one-shot/long-lived.

### E.3 Los 10 casos mapeados

| # | Caso | Binding | Family | Ingress → Egress | Granularidad | Streaming |
|---|---|---|---|---|---|---|
| 1 | HTTP REST proxy | `http` | HTTP_PROXY | BEARER_JWT/API_KEY → Bearer/OAuth2/mTLS | PER_REQUEST | PASSTHROUGH (chunked/SSE) |
| 2 | HTTP → S3 | `s3` | HTTP_PROXY | BEARER_JWT → SIGV4 | PER_OBJECT | PASSTHROUGH, multipart >5GB |
| 3 | S3 → S3 | `s3` | ACTIVE_TRANSFER | — → SIGV4 + SIGV4/AZURE_SAS | PER_TRANSFER | chunk 10–50MB |
| 4 | PostgreSQL | `postgres` | PROTOCOL_SERVER | SCRAM_SERVER → SCRAM | PER_QUERY | batch 5000 filas |
| 5 | MongoDB | `mongodb` | PROTOCOL_SERVER | TRANSPARENT → TRANSPARENT | PER_COMMAND | cursor nativo |
| 6 | Kafka → Kafka | `kafka` | ACTIVE_TRANSFER | — → SASL_SCRAM/OAUTHBEARER/mTLS | PER_TOPIC | batches transaccionales |
| 7 | TCP passthrough | `tcp` | TCP_TUNNEL | MTLS/IP_ACL → TRANSPARENT | PER_CONNECTION | copy_bidirectional |
| 8 | gRPC proxy | `grpc` | HTTP_PROXY | BEARER_JWT/MTLS → Bearer/mTLS | PER_METHOD | HTTP/2 frames |
| 9 | MQTT bridge | `mqtt` | ACTIVE_TRANSFER | — → PASSWORD/MTLS | PER_MESSAGE | pub/sub QoS≥1 |
| 10 | FTP/SFTP | `sftp` | ACTIVE_TRANSFER | — → SSH_KEY/PASSWORD | PER_FILE | chunks 1MB |

Nota sobre los tiers de A.7.1: este documento de casos **es** la demanda real que la regla de promoción pedía. Re-tier a la luz de él: tier 1 pasa a `http`, `s3`, `tcp` (el túnel L4 es el fallback universal — cubre Oracle, TDS, OPC-UA sin escribir nada más — y es la familia más barata de implementar) + `wasm:*`; tier 2: `postgres`, `kafka`, `mqtt`; tier 3 (candidatos a plugin `wasm:` primero): `mongodb` (DPI de wire protocol, complejidad alta), `grpc`, `sftp`.

Dos casos completos como referencia de la semántica:

**Caso 4 — PostgreSQL → PostgreSQL:**

```jsonc
{
  "dslVersion": 2,
  "metadata": { "name": "pg-governed-proxy", "version": "1.0.0" },
  "parameters": [
    { "name": "BACKEND_DSN", "title": "DSN del Postgres real", "type": "STRING",
      "required": true, "sensitive": true },
    { "name": "ALLOWED_TABLES", "title": "Tablas expuestas", "type": "VEC<STRING>", "required": true }
  ],
  "authentication": {
    "ingress": { "type": "SCRAM_SERVER" },                       // el dataplane ES un pg server
    "egress":  { "type": "SCRAM", "dsn": "{{ secrets./pg/dsn }}" }
  },
  "interaction": {
    "mode": "PULL",
    "active": {
      "type": "CHANNEL",
      "channel": {
        "binding": "postgres", "family": "PROTOCOL_SERVER",
        "spec": { "listen": ":15432", "dialect": "postgresql" },
        "authorization": {
          "granularity": "PER_QUERY",
          "subject": "{{ ingress.identity }}",
          "action": { "from": "sql.operation",
                      "map": { "SELECT": "READ", "INSERT": "WRITE", "UPDATE": "WRITE",
                                "DELETE": "DELETE", "DROP": "DENY", "TRUNCATE": "DENY" } },
          "resource": "table:{{ sql.tables }}",
          "context": { "columns": "{{ sql.columns }}", "transferId": "{{ sys.transfer_id }}" }
        },
        "streaming": { "mode": "PASSTHROUGH", "batch": { "rows": 5000 } },
        "monitor": { "progress": "QUERIES", "completion": "EXPLICIT" }
      }
    }
  }
}
```

**Caso 6 — Kafka → Kafka (con setup/teardown de DAG):**

```jsonc
{
  "dslVersion": 2,
  "metadata": { "name": "kafka-topic-bridge", "version": "1.0.0" },
  "authentication": {
    "ingress": null,
    "egress": { "source":      { "type": "SASL_SCRAM", "username": "{{ params.SRC_USER }}",
                                  "password": "{{ secrets./kafka/src }}" },
                 "destination": { "type": "SASL_OAUTHBEARER", "tokenUrl": "{{ params.IDP_URL }}",
                                  "clientId": "{{ params.CLIENT_ID }}",
                                  "clientSecret": "{{ secrets./kafka/oauth }}" } }
  },
  "operations": {
    "provision-topic": { "binding": "kafka", "spec": { "op": "create-topic",
        "topic": "{{ params.DEST_TOPIC }}", "partitions": 12, "replication": 3 },
        "extract": { "topic_ready": "jq('.created')" } },
    "set-acls":    { "binding": "kafka", "spec": { "op": "set-acls" } },
    "revoke-acls": { "binding": "kafka", "spec": { "op": "revoke-acls" } }
  },
  "interaction": {
    "mode": "PUSH",
    "setup": { "flow": "provision-topic >> set-acls" },
    "active": {
      "type": "CHANNEL",
      "channel": {
        "binding": "kafka", "family": "ACTIVE_TRANSFER",
        "spec": { "source": { "brokers": "{{ params.SRC_BROKERS }}", "topic": "{{ params.SRC_TOPIC }}" },
                   "destination": { "brokers": "{{ params.DEST_BROKERS }}", "topic": "{{ params.DEST_TOPIC }}" },
                   "transactional": true },
        "authorization": { "granularity": "PER_TOPIC", "subject": "{{ sys.connector_id }}",
                            "action": { "const": "SUBSCRIBE" },
                            "resource": "topic:{{ params.SRC_TOPIC }}" },
        "streaming": { "mode": "PASSTHROUGH" },
        "monitor": { "progress": "MESSAGES", "completion": "CONSUMER_LAG" }
      }
    },
    "teardown": { "flow": "revoke-acls" }
  }
}
```

### E.4 Impacto en el resto del plan

- **A.2/A.7 quedan como están**: el flow DAG es la fase activa `FLOW` (el pull request-response clásico) y setup/teardown de los canales. `channel` es un bloque hermano, no un reemplazo.
- **Fase nueva A7 en la tabla de A.8**: `interaction` trifásico + `channel` para las familias HTTP_PROXY y TCP_TUNNEL primero (reutilizan el proxy Axum y `copy_bidirectional` que ya existen); PROTOCOL_SERVER y ACTIVE_TRANSFER después.
- **El PDP**: el bloque `authorization` presupone el AuthzRequest de 4 campos del PDF (§1.2). El DSL solo *construye* la request; la evaluación (Cedar embebido vs OPA externo) es configuración de despliegue, no de template.
- **WASM**: los canales abren dos puntos de extensión nuevos que encajan en el contrato existente: transformers por-unidad dentro de un canal (por mensaje/fila/objeto — ver el ejemplo de F.3) y hooks en las fronteras del canal (`on_channel_open`, `on_channel_close`, más las fases por-unidad `before_forward`/`after_forward`). Un binding de canal *completo* en WASM (familia ACTIVE_TRANSFER) es viable con el mismo ABI; las familias con listener (PROTOCOL_SERVER) no — el listener es siempre del host.

---

## F. ABI WASM v1 — especificación y ejemplo completo

La sección B fijó el modelo (Extism sobre wasmtime, JSON en frontera, I/O mediado). Esto lo baja a un ABI concreto y verificable. Con Extism, el plumbing de punteros lo dan los PDKs y solo queda el contrato JSON; el ABI crudo se especifica igualmente para que el contrato no dependa de Extism.

### F.1 Especificación ABI v1

**Convenciones globales**

- Módulo `wasm32-unknown-unknown` (o wasip2 si se migra a Component Model).
- Todo payload cruza la frontera como **UTF-8 JSON**. Un puntero+longitud empaquetados en `u64`: `(ptr << 32) | len`.
- Una instancia por invocación lógica (o pool); el guest es **stateless entre invocaciones** — estado solo vía `kv_get`/`kv_set`.

**Exports obligatorios del guest**

```wat
(func (export "abi_version") (result i32))         ;; devuelve 1; el host rechaza otros valores
(func (export "alloc") (param i32) (result i32))   ;; reserva n bytes en memoria del guest
```

**Exports por tipo de plugin** — todos con la firma `(param i32 i32) (result i64)`: reciben `(input_ptr, input_len)` y devuelven `(output_ptr << 32) | output_len`:

| `kind` del manifiesto | Export(s) obligatorios |
|---|---|
| `authenticator` | `authenticate` |
| `binding` | `execute` (one-shot); `channel_tick` opcional (ACTIVE_TRANSFER, ver F.4) |
| `transformer` | `transform` |
| `hook` | `on_event` |

**Sobre de salida (todas las funciones):**

```jsonc
{ "ok": { /* payload según contrato de la función */ } }
// o bien
{ "err": { "code": "AUTH_FAILED", "message": "…", "retryable": false } }
```

Un **trap** WASM (pánico, OOM, fuel agotado) se trata como `err` no-retryable con `code: "TRAP"`. Timeout del host (epoch) → `code: "TIMEOUT"`.

**Imports del host** — módulo `eunomia:host/v1`:

```wat
;; Toda función que devuelve datos: el host llama a alloc() del guest, escribe el JSON
;; y devuelve (ptr << 32) | len. La memoria pertenece al guest desde ese momento.

(import "eunomia:host/v1" "http_request" (func (param i32 i32) (result i64)))
;;   in:  { "method": "GET", "url": "…", "headers": {…}, "body": "…"|null, "timeoutMs": 30000 }
;;   out: { "status": 200, "headers": {…}, "body": "…" }
;;   El host aplica la allowlist de hosts del manifiesto ANTES de conectar.

(import "eunomia:host/v1" "secret_get" (func (param i32 i32) (result i64)))
;;   in: path UTF-8 (sin JSON). out: { "value": "…" }. Solo paths concedidos.

(import "eunomia:host/v1" "kv_get" (func (param i32 i32) (result i64)))
(import "eunomia:host/v1" "kv_set" (func (param i32 i32 i32 i32) (result i32)))
;;   Ámbito: (plugin, instancia de conector). kv_set devuelve 0=ok, 1=cuota excedida.

(import "eunomia:host/v1" "log" (func (param i32 i32 i32)))
;;   (level 0..3, msg_ptr, msg_len) → span de tracing del host, rate-limited.

(import "eunomia:host/v1" "now_ms" (func (result i64)))
```

Estas seis funciones son la superficie completa. Añadir una séptima es una decisión de seguridad con revisión explícita (regla de B.4).

**Versionado**: `abi_version` + el campo `abiVersion: 1` en el manifiesto. Cambios incompatibles → `eunomia:host/v2` conviviendo con v1; el host carga según manifiesto.

### F.2 Contratos JSON por export

```jsonc
// authenticate — in:
{ "authConfig": { /* bloque authentication.egress del template, resuelto */ },
  "context": { "transferProcessId": "…", "role": "PROVIDER", "attempt": 1 } }
// authenticate — ok:
{ "headers": { "Authorization": "AWS4-HMAC-SHA256 …" },
  "queryParams": {}, "expiresAtMs": 1780000000000 }

// execute (binding one-shot, operación del DAG) — in:
{ "operation": "fetch", "spec": { /* resuelto */ },
  "context": { "runtime": { /* runtime.<op>.* acumulado */ }, "transferProcessId": "…" },
  "config": { /* bloque config del plugin en el template */ } }
// execute — ok:  el "resultado canónico JSON" del plugin (libre, descrito en su manifiesto);
//                el host aplica `extract` sobre él.

// transform — in:
{ "phase": "before_forward",             // o after_forward
  "unit": { /* unidad canónica del canal: mensaje MQTT, fila, objeto… */ },
  "config": { /* config del plugin */ } }
// transform — ok:
{ "action": "FORWARD", "unit": { /* unidad posiblemente modificada */ } }
// o  { "action": "DROP", "reason": "…" }   — la unidad no se reenvía (filtrado)

// on_event (hook) — in:
{ "phase": "after_subscribe", "context": { /* snapshot inmutable, B.4 */ } }
// on_event — ok:
{ "action": "CONTINUE" }
// | { "action": "PATCH", "patch": { /* RFC 7396 sobre campos whitelisted */ } }
// | { "action": "VETO", "reason": "…" }
```

### F.3 Ejemplo completo: transformer `gps-masker` (caso 9, MQTT per-message)

El caso de uso del PDF §2.9: reenviar lecturas de sensores pero enmascarar coordenadas GPS de los payloads. Es un `transformer` colgado del canal MQTT con granularidad por-mensaje.

**Manifiesto (`gps-masker.manifest.json`):**

```jsonc
{
  "name": "gps-masker",
  "version": "1.0.0",
  "abiVersion": 1,
  "kind": "transformer",
  "phases": ["before_forward"],
  "configSchema": {
    "type": "object",
    "properties": {
      "fields":    { "type": "array", "items": { "type": "string" },
                     "default": ["lat", "lon", "location"] },
      "precision": { "type": "integer", "minimum": 0, "maximum": 4, "default": 1 }
    }
  },
  "permissions": { "network": [], "secrets": [], "kv": false },   // puro: sin permisos
  "checksum": "sha256:…"
}
```

**Plugin (Rust, compila a `wasm32-unknown-unknown`):**

```rust
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

// ── Plumbing ABI v1 (esto es lo que el PDK de Extism regala; aquí explícito) ──

#[no_mangle]
pub extern "C" fn abi_version() -> i32 { 1 }

#[no_mangle]
pub extern "C" fn alloc(n: i32) -> i32 {
    let buf = Vec::<u8>::with_capacity(n as usize);
    let ptr = buf.as_ptr() as i32;
    std::mem::forget(buf);
    ptr
}

fn read_input(ptr: i32, len: i32) -> Vec<u8> {
    unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize).to_vec() }
}

fn write_output(v: &Value) -> u64 {
    let bytes = serde_json::to_vec(v).unwrap();
    let ptr = alloc(bytes.len() as i32);
    unsafe { std::ptr::copy(bytes.as_ptr(), ptr as *mut u8, bytes.len()) };
    ((ptr as u64) << 32) | (bytes.len() as u64)
}

// ── Contrato transform ──

#[derive(Deserialize)]
struct Config {
    #[serde(default = "default_fields")] fields: Vec<String>,
    #[serde(default = "default_precision")] precision: u32,
}
fn default_fields() -> Vec<String> { vec!["lat".into(), "lon".into(), "location".into()] }
fn default_precision() -> u32 { 1 }

#[derive(Deserialize)]
struct Input { unit: Value, config: Config }

#[no_mangle]
pub extern "C" fn transform(ptr: i32, len: i32) -> u64 {
    let input: Input = match serde_json::from_slice(&read_input(ptr, len)) {
        Ok(i) => i,
        Err(e) => return write_output(&json!({
            "err": { "code": "BAD_INPUT", "message": e.to_string(), "retryable": false }
        })),
    };
    let mut unit = input.unit;
    // La unidad canónica MQTT: { "topic": "…", "qos": 1, "payload": <JSON|string> }
    if let Some(payload) = unit.get_mut("payload") {
        mask(payload, &input.config);
    }
    write_output(&json!({ "ok": { "action": "FORWARD", "unit": unit } }))
}

fn mask(v: &mut Value, cfg: &Config) {
    match v {
        Value::Object(map) => {
            for (k, val) in map.iter_mut() {
                if cfg.fields.iter().any(|f| f.eq_ignore_ascii_case(k)) {
                    round_coords(val, cfg.precision);
                } else {
                    mask(val, cfg);
                }
            }
        }
        Value::Array(items) => items.iter_mut().for_each(|i| mask(i, cfg)),
        _ => {}
    }
}

fn round_coords(v: &mut Value, precision: u32) {
    let factor = 10f64.powi(precision as i32);
    match v {
        Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                *v = json!((f * factor).round() / factor);   // 40.4168 → 40.4
            }
        }
        Value::Object(_) | Value::Array(_) => {
            // {"lat": …, "lon": …} anidado bajo "location", o pares [lat, lon]
            if let Value::Object(map) = v {
                for val in map.values_mut() { round_coords(val, precision); }
            } else if let Value::Array(items) = v {
                for val in items.iter_mut() { round_coords(val, precision); }
            }
        }
        _ => {}
    }
}
```

**Referencia desde el template (canal MQTT del caso 9):**

```jsonc
"channel": {
  "binding": "mqtt", "family": "ACTIVE_TRANSFER",
  "spec": { "source": { "broker": "{{ params.BROKER }}", "topics": ["sensors/#"], "qos": 1 },
             "destination": { "broker": "{{ params.DEST_BROKER }}" } },
  "transformers": [
    { "plugin": "gps-masker@1.0", "phase": "before_forward",
      "config": { "fields": ["lat", "lon", "gps"], "precision": 1 } }
  ],
  "authorization": { "granularity": "PER_MESSAGE", "subject": "{{ sys.connector_id }}",
                      "action": { "const": "READ" }, "resource": "topic:{{ mqtt.topic }}" }
}
```

**Secuencia host-side por mensaje** (bucle del bridge MQTT):

```
mensaje llega del broker origen
  → construir unidad canónica { topic, qos, payload }
  → PDP: AuthzRequest(subject, READ, topic:sensors/building-A/temp, ctx)   [PER_MESSAGE]
  → PERMIT → invocar gps-masker.transform({ phase, unit, config })
       timeout 1s (epoch) · memoria máx 64MB · sin host imports (permissions vacíos)
  → { ok: { action: FORWARD, unit } } → publicar unit.payload en broker destino con QoS 1
  → { ok: { action: DROP } }          → no publicar; contar en monitor
  → { err } o trap                    → política failClosed del transformer:
       true (default para transformers de governance) → DROP + log
```

### F.4 Nota: binding de canal completo en WASM

Para la familia ACTIVE_TRANSFER, un plugin puede ser el canal entero (p.ej. un protocolo propietario de polling): el host llama a `channel_tick(state) -> { units: [...], nextState, delayMs }` en bucle — el plugin produce lotes de unidades canónicas y el host las pasa por PDP + transformers + destino, manteniendo el streaming y el backpressure en el host. El plugin nunca mantiene la conexión (no puede: sin sockets); la mantiene el host vía `http_request` o, para protocolos no-HTTP, no es viable como plugin puro — esos son exactamente los casos que se promocionan a binding nativo. Esta asimetría es deliberada y es el límite honesto del modelo: WASM extiende todo lo request-shaped; lo connection-shaped (pgwire, túneles) es territorio del host.

