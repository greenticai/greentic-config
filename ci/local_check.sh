#!/usr/bin/env bash
set -euo pipefail

# Usage:
#   LOCAL_CHECK_ONLINE=1 LOCAL_CHECK_STRICT=1 LOCAL_CHECK_VERBOSE=1 LOCAL_CHECK_COVERAGE=1 LOCAL_CHECK_PACKAGE=1 ci/local_check.sh
# Defaults: offline (LOCAL_CHECK_ONLINE=0), coverage/package disabled, non-strict, non-verbose.

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

LOCAL_CHECK_ONLINE="${LOCAL_CHECK_ONLINE:-0}"
LOCAL_CHECK_STRICT="${LOCAL_CHECK_STRICT:-0}"
LOCAL_CHECK_VERBOSE="${LOCAL_CHECK_VERBOSE:-0}"
LOCAL_CHECK_COVERAGE="${LOCAL_CHECK_COVERAGE:-0}"
LOCAL_CHECK_PACKAGE="${LOCAL_CHECK_PACKAGE:-0}"
if [[ -n "${LOCAL_CHECKPACKAGE:-}" && "$LOCAL_CHECK_PACKAGE" == "0" ]]; then
  LOCAL_CHECK_PACKAGE="$LOCAL_CHECKPACKAGE"
fi

if [[ "$LOCAL_CHECK_VERBOSE" == "1" ]]; then
  set -x
fi

export RUST_BACKTRACE=1

SKIPPED_STEPS=()

step() {
  printf "\n▶ %s\n" "$*"
}

have() {
  command -v "$1" >/dev/null 2>&1
}

need() {
  local tool="$1"
  if have "$tool"; then
    return 0
  fi
  echo "[miss] $tool"
  if [[ "$LOCAL_CHECK_STRICT" == "1" ]]; then
    echo "[fail] Missing required tool '$tool' (LOCAL_CHECK_STRICT=1)" >&2
    exit 1
  fi
  return 1
}

ensure_core_tool() {
  local tool="$1"
  if ! need "$tool"; then
    echo "[fail] '$tool' is required for local CI checks" >&2
    exit 1
  fi
}

ensure_tools() {
  local tool
  for tool in "$@"; do
    need "$tool" || return 1
  done
  return 0
}

run_or_skip() {
  local desc="$1"
  shift
  if "$@"; then
    return 0
  fi
  SKIPPED_STEPS+=("$desc")
  echo "[skip] $desc"
  return 1
}

show_tool_version() {
  local cmd="$1"
  local reason="$2"
  shift 2
  if have "$cmd"; then
    "$@" || true
  else
    echo "[warn] $cmd not available${reason:+ ($reason)}"
  fi
}

print_tool_versions() {
  step "Toolchain versions"
  show_tool_version cargo "" cargo --version
  show_tool_version rustc "" rustc --version
  show_tool_version rustfmt "required for cargo fmt" rustfmt --version
  show_tool_version cargo-clippy "install via 'rustup component add clippy' to run clippy" cargo clippy --version
  show_tool_version jq "needed for cargo package dry-runs" jq --version
  show_tool_version cargo-tarpaulin "set LOCAL_CHECK_COVERAGE=1 to use" cargo tarpaulin --version
}

ensure_core_tool cargo
ensure_core_tool rustc

print_tool_versions

run_fmt() {
  step "cargo fmt --all -- --check"
  cargo fmt --all -- --check
}

if run_or_skip "cargo fmt --all -- --check (requires cargo & rustfmt)" ensure_tools cargo rustfmt; then
  run_fmt
fi

run_clippy() {
  step "cargo clippy --workspace --all-targets --all-features -- -D warnings"
  cargo clippy --workspace --all-targets --all-features -- -D warnings
}

if run_or_skip "cargo clippy --workspace --all-targets (requires cargo-clippy component)" ensure_tools cargo cargo-clippy; then
  run_clippy
fi

run_build() {
  step "cargo build --workspace --all-features --locked"
  cargo build --workspace --all-features --locked
}

if run_or_skip "cargo build --workspace --all-features --locked" ensure_tools cargo; then
  run_build
fi

run_tests() {
  step "cargo test --workspace --all-features --locked -- --nocapture"
  cargo test --workspace --all-features --locked -- --nocapture
}

if run_or_skip "cargo test --workspace --all-features --locked" ensure_tools cargo; then
  run_tests
fi

package_publishable_crates() {
  step "cargo package (dry-run) for publishable crates"
  local pkg_list=""
  if have python3; then
    pkg_list="$(python3 <<'PY' 2>/dev/null
import json, subprocess, sys
data = json.loads(subprocess.check_output(["cargo", "metadata", "--format-version", "1", "--no-deps"]))
publishable = {}
for pkg in data["packages"]:
    if pkg.get("source") is not None:
        continue
    publish = pkg.get("publish")
    if publish == ["false"]:
        continue
    publishable[pkg["id"]] = pkg["name"]
if not publishable:
    sys.exit(0)
graph = {pkg_id: set() for pkg_id in publishable}
resolve = data.get("resolve", {})
for node in resolve.get("nodes", []):
    node_id = node["id"]
    if node_id not in publishable:
        continue
    for dep in node.get("deps", []):
        dep_id = dep["pkg"]
        if dep_id in publishable:
            graph[node_id].add(dep_id)
order = []
temp = set()
perm = set()
def visit(node_id):
    if node_id in perm:
        return
    if node_id in temp:
        raise SystemExit("cycle detected in workspace dependency graph")
    temp.add(node_id)
    for dep_id in sorted(graph[node_id], key=lambda pid: publishable[pid]):
        visit(dep_id)
    temp.remove(node_id)
    perm.add(node_id)
    order.append(node_id)
for node_id in sorted(graph, key=lambda pid: publishable[pid]):
    visit(node_id)
print("\n".join(publishable[node_id] for node_id in order))
PY
)"
  fi
  if [[ -z "$pkg_list" ]]; then
    if have jq; then
      pkg_list="$(
        cargo metadata --format-version 1 \
          | jq -r '.packages[] | select(.publish != ["false"] and (.source == null)) | .name' \
          | sort -u
      )"
    fi
  fi
  if [[ -z "$pkg_list" ]]; then
    echo "No publishable crates detected"
    return
  fi
  if [[ "$LOCAL_CHECK_ONLINE" == "1" ]]; then
    if need curl && ! curl -sSf --max-time 5 https://index.crates.io/config.json >/dev/null 2>&1; then
      local msg="cargo package dry-run (crates.io unreachable)"
      if [[ "$LOCAL_CHECK_STRICT" == "1" ]]; then
        echo "[fail] $msg" >&2
        exit 1
      fi
      SKIPPED_STEPS+=("$msg")
      echo "[skip] $msg"
      return
    fi
  fi
  local -a failed_pkgs=()
  local -a pkg_args=(--allow-dirty)
  if [[ "$LOCAL_CHECK_ONLINE" != "1" ]]; then
    pkg_args+=(--offline)
  fi
  local pkg
  while IFS= read -r pkg; do
    [[ -z "$pkg" ]] && continue
    echo "→ Packaging $pkg"
    if ! cargo package -p "$pkg" "${pkg_args[@]}"; then
      echo "[warn] cargo package failed for $pkg"
      failed_pkgs+=("$pkg")
    fi
  done <<<"$pkg_list"
  if [[ "${#failed_pkgs[@]}" -ne 0 ]]; then
    if [[ "$LOCAL_CHECK_STRICT" == "1" ]]; then
      echo "[fail] cargo package dry-run failed for: ${failed_pkgs[*]}" >&2
      exit 1
    fi
    local summary="cargo package dry-run (failed for: ${failed_pkgs[*]})"
    SKIPPED_STEPS+=("$summary")
    echo "[skip] $summary"
  fi
}

should_run_package() {
  if [[ "$LOCAL_CHECK_PACKAGE" == "1" || "$LOCAL_CHECK_STRICT" == "1" ]]; then
    return 0
  fi
  echo "[info] Set LOCAL_CHECK_PACKAGE=1 (or LOCAL_CHECK_STRICT=1) to run cargo package dry-runs"
  return 1
}

if run_or_skip "cargo package dry-run (requires jq or python3 and LOCAL_CHECK_PACKAGE=1)" \
  should_run_package && ensure_tools cargo; then
  package_publishable_crates
fi

run_tarpaulin() {
  step "cargo tarpaulin --workspace --all-features"
  cargo tarpaulin \
    --workspace \
    --all-features \
    --timeout 600 \
    --out Lcov \
    --output-dir coverage
}

should_run_tarpaulin() {
  if [[ "$LOCAL_CHECK_COVERAGE" == "1" || "$LOCAL_CHECK_STRICT" == "1" ]]; then
    return 0
  fi
  echo "[info] Set LOCAL_CHECK_COVERAGE=1 (or LOCAL_CHECK_STRICT=1) to run coverage locally"
  return 1
}

if run_or_skip "cargo tarpaulin coverage (requires cargo-tarpaulin and LOCAL_CHECK_COVERAGE=1)" \
  should_run_tarpaulin && ensure_tools cargo cargo-tarpaulin; then
  run_tarpaulin
fi

printf "\nAll requested checks finished.\n"
if [[ "${#SKIPPED_STEPS[@]}" -gt 0 ]]; then
  printf "Skipped steps:\n"
  printf " - %s\n" "${SKIPPED_STEPS[@]}"
else
  printf "No steps were skipped.\n"
fi
