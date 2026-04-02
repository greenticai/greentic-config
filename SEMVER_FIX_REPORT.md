# Semver Fix Report

## Scope
- Crate: `greentic-config-types`
- Version in tree: `0.4.15`
- Violation analyzed: `enum_marked_non_exhaustive`

## Violation
- `TelemetryExporterKind` in `crates/greentic-config-types/src/lib.rs:351` was marked `#[non_exhaustive]`.
- This is a semver-breaking API change because downstream exhaustive matches would stop compiling.

## Fix Applied
- Removed `#[non_exhaustive]` from `TelemetryExporterKind`.
- File changed: `crates/greentic-config-types/src/lib.rs`

## Why This Is Minimal and Safe
- No logic or runtime behavior changed.
- No enum variants, discriminants, serialization attributes, or defaults were modified.
- Public API behavior was restored to baseline compatibility by removing only the breaking attribute.

## Match Arm Catch-All Check
- No new `#[non_exhaustive]` was introduced in this fix, so no wildcard match-arm updates were required.

## Validation Status
- Attempted to run:
  - `cargo semver-checks check-release --package greentic-config-types`
- Could not complete validation in this CI environment due to network restrictions (`Could not resolve host: index.crates.io`).
