# greentic-config

Enterprise configuration loader for Greentic hosts. This crate resolves configuration from defaults, user-level config, project config, environment variables, and CLI overrides with clear precedence and provenance tracking.

## Features
- Loads canonical schema from `greentic-config-types` (no secrets).
- Precedence: CLI > env > project > user > defaults.
- Provenance tracking:
  - `ResolvedConfig` includes per-field source (`default|user|project|env|cli`).
  - `ResolvedConfigDetailed` includes per-leaf source plus origin (file path / env var name / `cli`).
- Validation hooks for dev-only fields, path sanity, and unsafe combinations.
- Explain output (string/JSON) for operators.
- Deployer defaults resolved here so deploy tools never hard-code routing domains.

## Operator semantics
- `--config <path>` / `ConfigResolver::with_config_path(...)` is strict and deterministic: if the file does not exist, resolution fails.
- If `--config` is set, it replaces project discovery (`<project_root>/.greentic/config.toml`) but keeps the same precedence position (project layer).

## Services and events mapping
- Services endpoints (e.g., `services.events.url`) live in `greentic-config-types`; resolver applies precedence and exposes provenance.
- Events knobs (`events.reconnect`, `events.backoff`) get defaults (reconnect enabled, max_retries=50, backoff initial=250ms, max=30s, multiplier=2.0, jitter=true) and validation (offline + remote endpoint is rejected; backoff sanity enforced).
- No secrets in config; route auth material via your secrets backend.

## Environment variables

Only explicitly supported `GREENTIC_*` variables are mapped (stable, documented surface area):

| Env var | Config field |
| --- | --- |
| `GREENTIC_SCHEMA_VERSION` | `schema_version` |
| `GREENTIC_ENVIRONMENT_ENV_ID` | `environment.env_id` |
| `GREENTIC_ENVIRONMENT_DEPLOYMENT` | `environment.deployment` |
| `GREENTIC_ENVIRONMENT_CONNECTION` | `environment.connection` |
| `GREENTIC_ENVIRONMENT_REGION` | `environment.region` |
| `GREENTIC_PATHS_GREENTIC_ROOT` | `paths.greentic_root` |
| `GREENTIC_PATHS_STATE_DIR` | `paths.state_dir` |
| `GREENTIC_PATHS_CACHE_DIR` | `paths.cache_dir` |
| `GREENTIC_PATHS_LOGS_DIR` | `paths.logs_dir` |
| `GREENTIC_SERVICES_EVENTS_URL` | `services.events.url` |
| `GREENTIC_RUNTIME_MAX_CONCURRENCY` | `runtime.max_concurrency` |
| `GREENTIC_RUNTIME_TASK_TIMEOUT_MS` | `runtime.task_timeout_ms` |
| `GREENTIC_RUNTIME_SHUTDOWN_GRACE_MS` | `runtime.shutdown_grace_ms` |
| `GREENTIC_TELEMETRY_ENABLED` | `telemetry.enabled` |
| `GREENTIC_TELEMETRY_EXPORTER` | `telemetry.exporter` |
| `GREENTIC_TELEMETRY_ENDPOINT` | `telemetry.endpoint` |
| `GREENTIC_TELEMETRY_SAMPLING` | `telemetry.sampling` |
| `GREENTIC_NETWORK_PROXY_URL` | `network.proxy_url` |
| `GREENTIC_NETWORK_TLS_MODE` | `network.tls_mode` |
| `GREENTIC_NETWORK_CONNECT_TIMEOUT_MS` | `network.connect_timeout_ms` |
| `GREENTIC_NETWORK_READ_TIMEOUT_MS` | `network.read_timeout_ms` |
| `GREENTIC_SECRETS_KIND` | `secrets.kind` |
| `GREENTIC_SECRETS_REFERENCE` | `secrets.reference` |
| `GREENTIC_DEV_DEFAULT_ENV` | `dev.default_env` |
| `GREENTIC_DEV_DEFAULT_TENANT` | `dev.default_tenant` |
| `GREENTIC_DEV_DEFAULT_TEAM` | `dev.default_team` |
| `GREENTIC_EVENTS_RECONNECT_ENABLED` | `events.reconnect.enabled` |
| `GREENTIC_EVENTS_RECONNECT_MAX_RETRIES` | `events.reconnect.max_retries` |
| `GREENTIC_EVENTS_BACKOFF_INITIAL_MS` | `events.backoff.initial_ms` |
| `GREENTIC_EVENTS_BACKOFF_MAX_MS` | `events.backoff.max_ms` |
| `GREENTIC_EVENTS_BACKOFF_MULTIPLIER` | `events.backoff.multiplier` |
| `GREENTIC_EVENTS_BACKOFF_JITTER` | `events.backoff.jitter` |

## Status
- Schema and loader implemented; CLI binary behind the `cli` feature (`greentic-config show|explain|validate`).
- Defaults are non-secret and filesystem/network safe.

## Deployer defaults
- `deployer.base_domain` defaults to `deploy.greentic.ai` and is used when generating deployment URLs / routing domains.
- Provenance is tracked like other fields; override via config layers instead of baking domains into deployer code.
