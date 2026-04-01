# Semver Fix Report

## Scope
- Crate: `greentic-config-types`
- File changed: `crates/greentic-config-types/src/lib.rs`

## Reported violations
1. `enum_no_repr_variant_discriminant_changed`
- `TelemetryExporterKind::None` discriminant changed from `2` to `5`.

2. `enum_variant_added`
- Added variants on exhaustive enum `TelemetryExporterKind`: `Gcp`, `Azure`, `Aws`.

## Fixes applied
1. Added `#[non_exhaustive]` to `TelemetryExporterKind`.
- Resolves the exhaustive-enum variant addition compatibility issue.

2. Added explicit discriminants to `TelemetryExporterKind` variants to preserve prior numeric value for `None`.
- Set:
  - `Otlp = 0`
  - `Stdout = 1`
  - `None = 2` (preserved old value)
  - `Gcp = 3`
  - `Azure = 4`
  - `Aws = 5`
- Resolves discriminant-change compatibility issue.

## Behavioral impact
- No logic/behavior changes were made.
- Changes are metadata/API-surface compatibility adjustments only.

## Match exhaustiveness check
- Searched `greentic-config-types` for `match` usage on `TelemetryExporterKind`.
- No exhaustive `match` statements on this enum were found in this crate, so no wildcard arm changes were required.
