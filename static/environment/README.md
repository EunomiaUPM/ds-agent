# Environment & Vault Fixture Notice

All tracked files under `static/environment/` (`*.mini.yaml`, `dev/*.yaml`, `envs/*.example`)
and `static/vault/**/*.example` are **synthetic, local-only development fixtures** (e.g.,
`mini_provider`, `mini_consumer`, self-signed test certificates).

- Real secrets (`static/vault/**/secrets/*.json`, `*.pem`, `static/environment/envs/*.env`) are
  excluded via `.gitignore` and never committed.
- In production (`connection.is_prod: true`, `connection.is_vault_real: true`), database, wallet,
  federation, and connector secrets are loaded from HashiCorp Vault (`VaultSecretRepo`), TLS
  certificate verification is enforced (`proxy_accept_invalid_certs: false`), and private/loopback
  egress is blocked (`proxy_allow_private_egress: false`).
