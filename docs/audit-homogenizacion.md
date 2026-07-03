# Auditoría de homogeneización — ds-protocol

Estado de cada crate frente a [`docs/code-style.md`](./code-style.md). Fecha: 2026-06-27.

**Leyenda:** ✅ cumple · ⚠️ parcial / diverge · ❌ no cumple · ➖ no aplica al rol del crate.

**Método:** señales estructurales (layout de módulos, presencia de traits/enums/From, imports
de framework en el dominio, cobertura de licencia). Es un mapa de calor para priorizar, no una
revisión línea a línea; cada celda ⚠️/❌ enlaza a la acción concreta en el backlog del final.

**Rol del crate** (determina qué criterios aplican):
- **lib componible** — la compone otro binario vía `<Crate>Setup`: `keystore`, `oauth`,
  `connector`, `dataplane`, `events`, `common`.
- **agente-binario** — proceso propio con CLI/boot/workers: `transfer-agent`,
  `transfer-agent-ref`, `catalog-agent`, `negotiation-agent`, `auth`, `bff`, `monolith`.

---

## Tabla maestra

| Crate | Rol | Dominio §2 | Data access §4 | Errores §5 | Setup §6 | Sin fns sueltas §0 | Licencia/docs §0.4 |
|---|---|:--:|:--:|:--:|:--:|:--:|:--:|
| **keystore** | lib | ✅ | ✅ | ⚠️ | ✅ | ✅ | ❌ |
| **transfer-agent-ref** | bin | ✅ | ✅✅ | ⚠️ | ✅ʷ | ⚠️ | ⚠️ |
| **oauth** | lib | ✅ | ✅ | ⚠️ | ✅ | ✅ | ✅ |
| **dataplane** | lib | ✅ | ⚠️ | ✅✅ | ✅✅ | ✅ | ⚠️ |
| **negotiation-agent** | bin | ✅ | ⚠️ | ✅ | ✅ʷ | ✅ | ⚠️ |
| **catalog-agent** | bin | ✅ | ⚠️ | ⚠️ | ✅ʷ | ✅ | ✅ |
| **transfer-agent** | bin | ✅ | ⚠️ | ⚠️ | ✅ʷ | ⚠️ | ⚠️ |
| **connector** | lib | ✅ | ⚠️ | ❌ | ✅ | ✅ | ⚠️ |
| **events** | lib | ⚠️ | ⚠️ | ⚠️ | ❌ | ✅ | ✅ |
| **common** | lib util | ➖ | ➖ | ✅ | ➖ | ⚠️ | ✅ |
| **auth** | bin | ⚠️ | ❌ | ❌ | ⚠️ | ✅ | ✅ |
| **bff** | bin (gw) | ➖ | ➖ | ❌ | ✅ʷ | ✅ | ✅ |
| **monolith** | bin (agg) | ➖ | ➖ | ➖ | ✅ʷ | ✅ | ✅ |

> `✅✅` = crate de referencia para ese eje. `✅ʷ` = patrón **boot/worker** correcto (ver §A);
> equivalente válido a `<Crate>Setup` para agentes-binario.

---

## A. Hallazgo transversal 1 — dos familias en la capa de datos

El estándar (§4) es el de `transfer-agent-ref` / `keystore` / `oauth`:

```
data/repo/        traits *RepoTrait + *RepoErrors      ← puertos
data/sea_orm/     adaptador SQL (repos/, orm/, migrations/)
data/in_memory/   adaptador en memoria (tests/dev)
data/factory.rs   trait DataFactory -> Arc<dyn *RepoTrait>   ← multi-backend
```

Pero **la mayoría de los agentes usan una variante reducida**:

| Familia | Crates | Diferencias vs estándar |
|---|---|---|
| **A — estándar** | transfer-agent-ref, keystore, oauth | `repo/` + `sea_orm/` + `in_memory/` + `factory.rs` (trait) |
| **B — solo SQL** | dataplane, negotiation, catalog, transfer-agent, connector | `repo_traits/` + `repo_sql\|repos_sql/` + `factory_sql.rs` **concreto**. Sin `in_memory`, sin trait `DataFactory`. |
| **híbrida** | events | `data/repo/` sin factoría multi-backend |

Dos divergencias dentro de la familia B, ambas a unificar:
1. **Naming inconsistente**: `repo_traits` vs `repo`; `repo_sql` (connector, dataplane) vs
   `repos_sql` (catalog, negotiation, transfer-agent). Elegir **uno**.
2. **Factoría concreta** (`factory_sql.rs`) en vez de **trait `DataFactory`** → el resto del
   crate acaba acoplado al backend SQL y no hay adaptador `in_memory` para tests.

> Nota destacable: `dataplane` es la **referencia de errores y setup**, pero su capa de datos
> es familia B — ni el propio crate de referencia usa el data-access ideal.

---

## B. Hallazgo transversal 2 — dos familias en setup (no es un problema)

| Familia | Crates | Forma |
|---|---|---|
| **`<Crate>Setup`** (composition root de lib) | dataplane, keystore, oauth, connector | struct `XSetup` + helpers privados + `build_*` |
| **boot/worker** (arranque de binario) | transfer-agent-ref, catalog, negotiation, transfer-agent, monolith, bff | `XBoot: BootstrapServiceTrait` + `XHttpWorker`/`XGrpcWorker` + `XCliArgs`/`XCommands` + `XMigration` |
| **propia** | auth | `AuthApplication` (`app.rs`) — no sigue ninguna de las dos |

Son **dos capas, no dos opciones rivales**: un `XHttpWorker` cablea la app y, donde compone una
lib, llama a `DataplaneSetup::build_*` etc. El patrón boot/worker es consistente y limpio
(`TransferBoot` delega en `spawn` de cada worker con `CancellationToken`). **Acciones:**
- Alinear `auth` al patrón boot/worker (hoy es el único outlier).
- Verificar que la cadena de instancias **dentro de cada `http_worker.rs`** sigue §6 (agrupar
  infra compartida, helpers nombrados). No auditado a fondo aquí — pendiente de lectura.

---

## C. Notas por crate

- **keystore** ✅ referencia de dominio/data/setup. Pero: (1) error de crate como `error.rs`
  + `RepoIntoErrors`, sin un `KeystoreError` agrupado por secciones estilo `dataplane`; (2)
  **licencia: solo 21/44 archivos** con cabecera GPL — la peor cobertura del repo.
- **transfer-agent-ref** ✅✅ referencia de data-access (familia A completa con `in_memory` +
  `DataFactory`). Errores solo a nivel repo (sin enum de crate). ~26 fns libres en producción
  (revisar `services/*/views.rs` y mappers). Licencia 58/60.
- **oauth** ✅ familia A completa + `OAuthSetup`. Falta enum de crate + `From<_> for Errors`
  (hoy solo `RepoIntoErrors` a nivel repo). Por lo demás, muy alineado.
- **dataplane** ✅✅ referencia de **errores** (`DataplaneError` agrupado + `From<_> for Errors`)
  y **setup** (`DataplaneSetup` + `DataplaneInfra`). Pendiente: migrar su data-access de
  familia B → A. Licencia 77/83.
- **negotiation-agent** ✅ errores completos (`errors/mod.rs` + `From`). Data familia B; setup
  boot/worker. Licencia 121/123.
- **catalog-agent** ⚠️ tiene `errors/mod.rs` pero **sin `From<_> for Errors`**; además duplica
  errores en `protocols/dsp/errors`. Data familia B. Dominio casi puro (1/22 impuro).
- **transfer-agent** ⚠️ 3 enums thiserror dispersos, sin `errors/mod.rs` centralizado ni `From`.
  Data familia B (`repos_sql`). ~11 fns libres. Licencia 100/103.
- **connector** ⚠️ **solo 1 enum de error** en todo el crate, sin `errors/mod.rs` ni `From` →
  el peor en errores entre las libs. Tiene `ConnectorSetup` ✅. Data familia B (`repo_sql`).
- **events** ⚠️ modelo propio: dominio en `core/notification` + `core/subscription` (no
  `entities/`); `data/repo` sin factoría multi-backend; **sin `setup/`** (se compone desde
  fuera). `errors/mod.rs` sin `From`. Decidir si se alinea o se documenta como excepción.
- **common** ✅ define parte del patrón base de errores (`errors/mod.rs` + `From`). Es lib
  utilitaria: ~17 fns libres es esperable, pero conviene revisar que cumplan §0.2 (deps por
  parámetro).
- **auth** ❌ **el mayor outlier**. Usa `types/` (con `types/entities`, `types/business`) +
  `core/traits` en vez de `entities/`; **no tiene `data/repo` + adaptadores + factory** (datos
  en `data/` plano + `services/repo`); **0 enums thiserror, sin `errors/mod.rs`**; setup propio
  `AuthApplication`. Dominio puro pero estructura totalmente divergente. Es el crate que más
  trabajo de homogeneización requiere.
- **bff** ➖ gateway: sin dominio ni capa de datos (proxy/subscriptions). Solo aplica errores
  (hoy ❌, sin enum) y setup (boot/worker ✅).
- **monolith** ➖ binario agregador: solo `setup/` (boot/worker) que compone las libs. OK.

---

## D. Backlog priorizado (para un solo developer)

Orden por **riesgo creciente** y dependencia. Una rama por fila.

### Quick wins (mecánicos, riesgo nulo)
1. **Licencias**: añadir cabecera GPL a los archivos sin ella. Prioridad `keystore` (21/44),
   luego `dataplane` (77/83), `transfer-agent`, `negotiation`, `transfer-agent-ref`, `connector`.
   *Una sola pasada, scriptable.*

### Eje Errores (bajo riesgo, alto retorno de homogeneidad)
2. **connector**: crear `errors/mod.rs` con `ConnectorError` agrupado + `From<_> for Errors`.
3. **transfer-agent**: centralizar los 3 enums dispersos en `errors/mod.rs` + `From`.
4. **catalog-agent**: añadir `From<_> for Errors`; unificar con `protocols/dsp/errors`.
5. **oauth / keystore / transfer-agent-ref**: añadir enum de crate + `From` por encima del
   `RepoIntoErrors` a nivel repo (alinear con patrón `dataplane`).
6. **events / bff**: enum de crate mínimo + `From`.

### Eje Naming de datos (riesgo medio, mecánico pero amplio)
7. Unificar nomenclatura familia B: elegir `repo_traits` + `repos_sql` (o `repo`/`sea_orm`) y
   renombrar `connector` y `dataplane` (hoy `repo_sql`) para que coincidan.

### Eje Data access (riesgo medio-alto)
8. Migrar familia B → estándar: introducir trait `DataFactory` (sustituir `factory_sql.rs`
   concreto) y, donde aporte, adaptador `in_memory`. Empezar por **un piloto**
   (`negotiation-agent`, ya conocido) antes de propagar.

### Eje Setup (riesgo alto, toca wiring)
9. Alinear `auth` al patrón boot/worker estándar.
10. Auditar la cadena de instancias dentro de cada `http_worker.rs` (§6: agrupar infra, helpers).

### Caso especial
11. **auth**: reestructuración mayor (`types/` → `entities/`, introducir `data/repo` + adapters
    + factory, `errors/mod.rs`). Tratar como proyecto aparte, al final, con su propia serie de
    PRs pequeños.

### Decisión de diseño pendiente (te toca a ti)
- **events**: ¿se fuerza a `entities/` + `setup/` + `DataFactory`, o se documenta como excepción
  legítima por su naturaleza (pub/sub)?
- **Naming de datos**: ¿familia A (`repo`/`sea_orm`) como destino único, o se acepta familia B
  como estándar para agentes y A solo para libs? Esto define el alcance del eje 7–8.
</content>
