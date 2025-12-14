# Repository Overview

## 1. High-Level Purpose
- Workspace providing Greentic configuration capabilities.
- `greentic-config-types`: canonical schema types (no IO, no secrets) for host configuration, leveraging `greentic-types`.
- `greentic-config`: loader/resolver implementing precedence, validation, provenance, and explain/reporting, with optional CLI.

## 2. Main Components and Functionality
- **Path:** `crates/greentic-config-types`
  - **Role:** Schema-only config types with serde support.
  - **Key functionality:** Defines `GreenticConfig` (root), `ConfigVersion`, `EnvironmentConfig`, `PathsConfig`, `RuntimeConfig`, `TelemetryConfig`, `NetworkConfig`, `SecretsBackendRefConfig`, `DevConfig`, provenance helpers.
  - **Notes:** Uses `greentic-types` for `EnvId`, `DeploymentCtx`, `ConnectionKind`; durations standardized as `_ms` millisecond fields; telemetry exporter enum `otlp|stdout|none`; TLS mode enum `disabled|system|strict`.

- **Path:** `crates/greentic-config`
  - **Role:** Config loader/resolver with precedence and validation.
  - **Key functionality:** 
    - Root discovery (`paths.rs`) via nearest `.greentic/`, `.git/`, or `Cargo.toml`; default paths under `<root>/.greentic`.
    - Layered loading (`loaders.rs`): defaults, user config (`~/.config/greentic/config.toml`), project config (`.greentic/config.toml`), env vars (`GREENTIC_*`), CLI overrides; supports TOML/JSON.
    - Deep merge + provenance (`merge.rs`): precedence CLI > env > project > user > defaults; per-field provenance map.
    - Validation (`validate.rs`): dev fields gated by env unless `allow_dev`, absolute path enforcement, telemetry sampling bounds, warnings for missing optional dev team or enabled-without-exporter.
    - Explain (`explain.rs`): produces text + JSON report with provenance and warnings.
    - Public API: `ConfigResolver::new/with_project_root/with_cli_overrides/allow_dev/load`, `ResolvedConfig`, `explain`.
    - Optional CLI (`src/bin/greentic-config.rs`, feature `cli`): `show`, `explain`, `validate`.
- **Tooling/CI:**
  - `ci/local_check.sh` runs fmt/clippy/build/test with optional package and tarpaulin coverage (disabled by default via env flags).
  - GitHub workflows: `ci.yml` (fmt/clippy/build/test), `coverage.yml` (tarpaulin), `check-package.yml` (cargo package dry-run), `auto-tag.yml` (crate tags on version bumps), `publish.yml` (publish changed crates).
  - Supporting script: `scripts/version-tools.sh` for crate version/tag helpers.

## 3. Work In Progress, TODOs, and Stubs
- None detected (no TODO/FIXME markers present).

## 4. Broken, Failing, or Conflicting Areas
- None observed. `cargo test` passes for both crates (after fetching dependencies).
- `ci/local_check.sh` passes (package/coverage steps intentionally skipped unless enabled).

## 5. Notes for Future Work
- Expand validation rules for unsafe combinations (e.g., offline + cloud endpoints) when `ConnectionKind` semantics are finalized.
- Broaden provenance/explain coverage to include all fields.
- Consider feature flags for platform-specific path resolution if needed.
