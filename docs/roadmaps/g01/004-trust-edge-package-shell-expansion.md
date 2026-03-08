# Roadmap g01.004: Trust-Edge Package Shell Expansion

Status: active
Owner: core-product
Created: 2026-03-08
Depends on: g01.003
Vision tags: RT, RES, AUTH
Target envelope: expand the initial Rust workspace shell with the first
trust-edge package set so runtime-host, plugin-sandbox, hardware, and shared
control/message boundaries exist as real workspace packages rather than docs
only.

## Problem

Signal's core workspace shells existed, but the trust-edge boundaries still
lived only in architecture docs. That left the runtime-host split incomplete in
implementation terms:

1. plugin and hardware boundaries had stable names but no real package homes,
2. host binaries could not point at trust-edge crates directly,
3. runtime-host and sandbox ownership rules were still easier to drift on than
   the shared DSP and analysis crates.

## Goals

- add the first trust-edge package shells:
  - `signal-ipc`
  - `signal-plugin`
  - `signal-plugin-clap`
  - `signal-plugin-sandbox`
  - `signal-hardware`
  - `signal-hardware-coreaudio`
- wire those packages into the root Cargo workspace
- make host ownership boundaries visible in `signal-host-local` and
  `signal-host-server`
- update Signal-owned docs so the package map and active roadmap reflect the
  real workspace state

## Non-Goals

- implementing real CLAP hosting or CoreAudio I/O in this batch
- defining the final shared-memory sandbox layout in code
- replacing the legacy C++ tree in this batch

## Execution Plan

### 004.1 Workspace expansion

- [x] Add the trust-edge packages to the Cargo workspace.
- [x] Create minimal manifests and source files for each package.

### 004.2 Host-boundary wiring

- [x] Point `signal-host-local` at the plugin and hardware shells.
- [x] Point `signal-host-server` at the plugin and hardware shells.
- [x] Keep the code intentionally small but explicit about ownership boundaries.

### 004.3 Documentation alignment

- [x] Update the package map to reflect the current workspace state.
- [x] Add this milestone to the active roadmap index.
- [x] Update the repo/docs entry points to point at the trust-edge expansion.

## Acceptance Signals

1. `cargo check --workspace` succeeds with the new trust-edge packages present.
2. A contributor can see where plugin, sandbox, hardware, and shared
   control/message code belong without inferring it from Loophole docs.
3. The host binaries reference the trust-edge packages clearly enough that the
   runtime-host ownership split is visible in code.

## Next Task

Decide whether the payload-only debug policy is now sufficiently frozen to
leave this export boundary alone for a while, or whether there is a concrete
inspection need strong enough to justify a second explicit debug section.
