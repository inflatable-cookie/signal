# Roadmap g01.002: Package Map and Runtime Entrypoint Naming

Status: complete
Owner: core-product
Created: 2026-03-08
Depends on: g01.001
Vision tags: RES, RT, AUTH
Target envelope: replace Finch-shaped provisional crate names with a broader
Signal-owned package map that can support DSP, analysis, runtime, plugin, and
hardware subjects without immediate renaming churn.

## Problem

The migrated research corpus still carried crate examples shaped around Finch's
analysis slice. That is too narrow for Signal's actual scope, which must cover
foundations, DSP, analysis, graph/runtime, plugin hosting, hardware, and host
assemblies.

Without a broader naming proposal:

1. research will keep targeting names that feel app-local or overly narrow,
2. generic buckets like `signal-core` will attract unrelated responsibilities,
3. runtime-host and trust-edge crates will not have a stable naming pattern.

## Goals

- Freeze a broader package naming pattern for Signal.
- Prefer layer-and-domain naming such as `signal-analysis-rhythm` and
  `signal-dsp-spectral` over app-shaped names such as `signal-beat`.
- Define the first concrete host assembly names.
- Align active research artifacts to the chosen package family.

## Non-Goals

- Creating the actual Cargo workspace in this batch.
- Freezing every future platform-backend crate name.
- Implementing the first Rust crates in this batch.

## Execution Plan

### 002.1 Package-map proposal

- [x] Add a dedicated architecture doc for the package map.
- [x] Define naming principles and anti-patterns.
- [x] Freeze the first recommended package family.

### 002.2 Research alignment

- [x] Update the research master index to use the broader names.
- [x] Update source hubs, value tracks, and algorithm specs that still used the
  older provisional crate names.
- [x] Align Essentia migration notes to the broader package family.

### 002.3 Host assembly naming

- [x] Freeze the first host names:
  - `signal-host-local`
  - `signal-host-server`
  - `signal-plugin-sandbox`

## Acceptance Signals

1. A contributor can answer "what should this new crate be called?" from one
   Signal-owned doc.
2. The analysis crates are named broadly enough to survive beyond Finch's first
   feature slice.
3. The runtime host and sandbox names are stable enough to use in follow-on
   planning and implementation docs.

## Next Task

Expand the real workspace shell with the first trust-edge package set so plugin,
hardware, sandbox, and shared control/message boundaries become concrete
workspace members.
