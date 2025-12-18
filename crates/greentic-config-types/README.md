# greentic-config-types

Schema-only types for Greentic host configuration. This crate defines the canonical configuration structures reused by runners, deployers, CLIs, and services.

## Scope
- Pure data structures with serde support.
- No filesystem, environment variable, or network IO.
- No secret material: only backend selection references are captured.
- Reuses `greentic-types` for shared identifiers (e.g., `EnvId`, `DeploymentCtx`, `ConnectionKind`).
- Shared services/events configuration: service endpoints/transports live in `services`, reconnect/backoff knobs in `events`.
- Deployer defaults (e.g., `deployer.base_domain`) live here so deploy tooling stays out of code defaults.

## Notes
- Durations use millisecond suffixes (`*_ms`) with `u64` values.
- Enums are case-insensitive when deserializing.
- `ConfigVersion` is a string schema version (default `"1"`).
- Do not put tokens/passwords/API keys in config; route auth via your secrets backend.

## Services and events configuration

`services.*` is for service endpoint/transport selection (HTTP/NATS/noop) and non-secret routing metadata.

`network.*` is cross-cutting client/network behavior (proxy/TLS/timeouts) and should not be duplicated per-service.

Admin endpoints are disabled by default and must be explicitly enabled via `runtime.admin_endpoints`.

```toml
[services]
runner = { kind = "http", url = "https://runner.greentic.local", headers = { "x-routing-key" = "tenant-1" } }
deployer = { kind = "nats", url = "nats://nats.greentic.local:4222", subject_prefix = "greentic" }
metadata = { kind = "noop" }

[services.events]
url = "https://events.greentic.local"
# headers are non-secret metadata only and MUST NOT include secrets
# (e.g. Authorization, Cookie, Set-Cookie).
headers = { "x-routing-key" = "tenant-1" }

[events.reconnect]
enabled = true
max_retries = 25

[events.backoff]
initial_ms = 250
max_ms = 30000
multiplier = 2.0
jitter = true

[runtime.admin_endpoints]
# disabled by default
secrets_explain_enabled = true
```

## Deployer defaults

```toml
[deployer]
# Default domain used when generating deployment URLs / routing domains.
base_domain = "deploy.greentic.ai"

[deployer.provider]
# Optional strategy hints; non-secret.
provider_kind = "aws"
region = "us-west-2"
```
