# Roadmap g01.003: Rust Workspace Shell Bootstrap

Status: complete
Owner: core-product
Created: 2026-03-08
Depends on: g01.002
Vision tags: RT, RES, AUTH
Target envelope: turn the frozen Signal package names into a real Rust
workspace shell without disturbing the legacy C++ implementation island.

## Problem

Signal had package names and docs direction, but no actual Rust workspace
surface. That kept every package boundary notional and prevented downstream
consumers or follow-on implementation batches from targeting real manifests.

## Goals

- add a root Cargo workspace manifest
- create the first package directories and manifests for the core shared crates
- create thin local/server host entrypoints
- keep the shell intentionally small so later implementation batches can fill in
  behavior without renaming churn

## Non-Goals

- implementing real DSP algorithms in this batch
- replacing the legacy C++ runtime in this batch
- adding plugin or hardware crates before the core workspace is stable

## Execution Plan

### 003.1 Workspace root

- [x] Add root `Cargo.toml` workspace manifest.
- [x] Add Rust build output ignore rules.

### 003.2 Core library shells

- [x] Add `signal-primitives`.
- [x] Add `signal-dsp`.
- [x] Add `signal-dsp-spectral`.
- [x] Add `signal-analysis`.
- [x] Add `signal-analysis-rhythm`.
- [x] Add `signal-analysis-tonal`.
- [x] Add `signal-analysis-loudness`.
- [x] Add `signal-graph`.
- [x] Add `signal-runtime`.

### 003.3 Host entrypoint shells

- [x] Add `signal-host-local`.
- [x] Add `signal-host-server`.

## Acceptance Signals

1. `cargo check --workspace` succeeds in Signal.
2. The workspace shell matches the package-map naming without requiring
   immediate reshaping.
3. Follow-on implementation can target real package directories instead of
   placeholder docs.

## Next Task

Expand the workspace with the first trust-edge package shells so the runtime-host,
plugin-sandbox, hardware, and shared control/message boundaries become concrete
implementation targets.
