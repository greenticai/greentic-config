# greentic-config-types

Schema-only types for Greentic host configuration. This crate defines the canonical configuration structures reused by runners, deployers, CLIs, and services.

## Scope
- Pure data structures with serde support.
- No filesystem, environment variable, or network IO.
- No secret material: only backend selection references are captured.
- Reuses `greentic-types` for shared identifiers (e.g., `EnvId`, `DeploymentCtx`, `ConnectionKind`).

## Notes
- Durations use millisecond suffixes (`*_ms`) with `u64` values.
- Enums are case-insensitive when deserializing.
- `ConfigVersion` is a string schema version (default `"1"`).

