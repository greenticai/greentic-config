# greentic-config

Enterprise configuration loader for Greentic hosts. This crate resolves configuration from defaults, user-level config, project config, environment variables, and CLI overrides with clear precedence and provenance tracking.

## Features
- Loads canonical schema from `greentic-config-types` (no secrets).
- Precedence: CLI > env > project > user > defaults.
- Provenance tracking for every resolved field.
- Validation hooks for dev-only fields, path sanity, and unsafe combinations.
- Explain output (string/JSON) for operators.

## Status
- Schema and loader implemented; CLI binary behind the `cli` feature (`greentic-config show|explain|validate`).
- Defaults are non-secret and filesystem/network safe.

