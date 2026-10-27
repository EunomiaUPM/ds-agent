# DCAT-AP 3.0.1 SHACL shapes

Official shapes published by SEMIC, copied without changes.

- Repository: https://github.com/SEMICeu/DCAT-AP
- Path: `releases/3.0.1/html/shacl/` on the `gh-pages` branch
- Commit: `8e5abb60250e1a23d237aa7a12093f16080a52e7` (2025-07-10)
- Licence: CC-BY-4.0, © SEMIC / Interoperable Europe

These are the files the Interoperable Europe Testbed validator loads for its DCAT-AP 3.0.1
profiles (https://github.com/ISAITB/validator-resources-dcat-ap, `resources/config.properties`):

| File | Testbed profile | Use here |
|---|---|---|
| `shapes.ttl` | Base | `base`: existence, literal ranges, cardinalities |
| `range.ttl` | Range | `range`: classes of the objects; needs typed objects (background knowledge) |
| `shapes_recommended.ttl` | Recommendations | `recommended`: warnings |
| `deprecateduris.ttl` | Full | `deprecated`: warnings on DCAT-AP 2 properties |
| `mdr-vocabularies.shape.ttl` | Controlled Vocabularies | `codelists`: needs the EU code lists loaded |
| `imports.ttl`, `mdr_imports.ttl` | (background knowledge) | not loaded: the agent never fetches imports |

Without background knowledge this matches the Testbed's "Zero" profiles.
