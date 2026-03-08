# 2026-03-08 19:45:00 UTC — Rust workspace moved under crates/

Status: completed
Owner: nucleus

## Summary

Moved the Rust workspace packages under `signal/crates/` instead of leaving
every package directory at the repository root.

## Why

- keeps the repository root focused on repo-level concerns
- makes the Rust workspace boundary explicit
- leaves more room for the legacy C++ tree, docs, and build surfaces to coexist
  during the transition period
- gives Finch and Loophole one stable, documented package root to target

## Changes

- moved all current Rust workspace packages into `crates/`
- updated the root `Cargo.toml` workspace members to use `crates/...` paths
- updated `README.md` to document the new repository layout
- updated `docs/architecture/package-map.md` to record `crates/` as the
  canonical workspace root
- updated `docs/research/source-hubs/002-signal-library-architecture.md` so
  consumer examples point at `signal/crates/...`

## Validation

- `cargo check --workspace`
- `git diff --check`

## Next Task

Update any downstream sibling-repo path references that still assume the old
flat Signal layout, then decide whether the legacy C++ implementation should
also move under a dedicated `cpp/` or `legacy/` root for the same reason.
