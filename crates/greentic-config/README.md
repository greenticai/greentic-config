# greentic-config

Enterprise configuration loader for Greentic hosts. This crate resolves configuration from defaults, user-level config, project config, environment variables, and CLI overrides with clear precedence and provenance tracking.

## Features
- Loads canonical schema from `greentic-config-types` (no secrets).
- Precedence: CLI > env > project > user > defaults.
- Provenance tracking for every resolved field.
- Validation hooks for dev-only fields, path sanity, and unsafe combinations.
- Explain output (string/JSON) for operators.
- Deployer defaults resolved here so deploy tools never hard-code routing domains.

## Services and events mapping
- Services endpoints (e.g., `services.events.url`) live in `greentic-config-types`; resolver applies precedence and exposes provenance.
- Events knobs (`events.reconnect`, `events.backoff`) get defaults (reconnect enabled, max_retries=50, backoff initial=250ms, max=30s, multiplier=2.0, jitter=true) and validation (offline + remote endpoint is rejected; backoff sanity enforced).
- No secrets in config; route auth material via your secrets backend.

## Status
- Schema and loader implemented; CLI binary behind the `cli` feature (`greentic-config show|explain|validate`).
- Defaults are non-secret and filesystem/network safe.

## Deployer defaults
- `deployer.base_domain` defaults to `deploy.greentic.ai` and is used when generating deployment URLs / routing domains.
- Provenance is tracked like other fields; override via config layers instead of baking domains into deployer code.
