# 072 Real Plugin Hosting, Discovery, And Sandbox Execution Contract

Status: draft
Owner: core-product
Updated: 2026-04-08
Related contracts: `docs/contracts/007-plugin-backend-and-host-neutral-delegation-contract.md`, `docs/contracts/008-backend-neutral-plugin-capability-and-adapter-breadth-contract.md`, `docs/contracts/014-plugin-isolation-policy-transport-rebind-and-shared-sandbox-continuity-contract.md`, `docs/contracts/020-vst3-adapter-baseline-and-runtime-owned-lifecycle-contract.md`, `docs/contracts/021-au-adapter-baseline-and-runtime-owned-lifecycle-contract.md`, `docs/contracts/038-lv2-adapter-baseline-and-linux-native-plugin-lifecycle-contract.md`
Related architecture: `docs/architecture/system-architecture.md`, `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the post-audit contract for turning Signal's plugin-hosting stack from
fixture-backed scaffolding into real filesystem-backed discovery, plugin
instantiation, sandbox execution, and runtime-owned lifecycle truth.

## Authority hierarchy

1. `signal-plugin` owns the format-neutral plugin vocabulary, lifecycle states,
   transport contracts, placement semantics, and consumer-visible fault meaning.
2. `signal-runtime` owns discovery receipts, runtime lifecycle interpretation,
   chain continuity, and supervisor/export proof.
3. `signal-plugin-vst3`, `signal-plugin-au`, and `signal-plugin-lv2` own
   format-specific realization:
   - scan roots and traversal
   - descriptor and capability extraction
   - plugin process bring-up and teardown
   - host bridge and backend-native callback adaptation
4. `signal-plugin-sandbox` owns the long-lived sandbox process, not a demo-only
   harness.
5. host crates may orchestrate scans and broker sandbox transport, but they do
   not own plugin capability meaning, lifecycle classification, or recovery
   semantics.

## Required shared guarantees

### Guarantee 1: discovery must be real, inspectable, and filesystem-backed

Signal must scan real plugin roots and record the provenance of discovered
modules, bundles, manifests, or components in runtime-owned receipts.

### Guarantee 2: adapter realization must end in one shared lifecycle model

Format-specific bring-up detail may differ, but VST3, AU, and LV2 instances
must collapse into one Signal-owned lifecycle and continuity vocabulary.

### Guarantee 3: sandbox execution must be a hardened process boundary

The sandbox process must support real request handling, bounded startup and
shutdown, transport attachment, crash attribution, and degraded recovery
without relying on synthetic lifecycle simulation.

### Guarantee 4: shared proof must cover discovery and execution separately

The repo must prove both:

- adapter discovery correctness
- sandboxed execution and lifecycle correctness

without requiring downstream consumers to reconstruct behavior from adapter
internals.

## Current repo mapping

The current implementation posture is narrower than the contract surface and
must be treated explicitly.

### Shared runtime and host posture

- `signal-runtime` already owns scan request recording, discovered-type
  snapshots, parity summaries, and consumer-visible discovery receipts.
- `signal-host-local` and `signal-host-server` already route plugin scan
  requests into runtime-owned receipts, but they currently feed those receipts
  from adapter and demo helpers rather than real platform discovery.
- both host crates still carry demo runtime assemblies and discovery helpers
  intended for bounded proof, not for production plugin-hosting truth.

### Adapter posture today

- `signal-plugin-vst3`
  - exposes default scan roots per platform
  - discovery still returns fixture-backed plugin records from
    `src/fixtures.rs`
  - session planning still only wraps discovered fixture metadata
- `signal-plugin-au`
  - exposes default macOS scan roots
  - discovery still returns fixture-backed AudioUnit records from
    `src/fixtures.rs`
  - session planning still only wraps discovered fixture metadata
- `signal-plugin-lv2`
  - exposes default Linux scan roots
  - discovery still returns fixture-backed bundle and manifest records from
    `src/fixtures.rs`
  - session planning still only wraps discovered fixture metadata
- `signal-plugin-sandbox`
  - current `main.rs` is still a synthetic CLAP lifecycle and block-processing
    shell, not a long-lived request-serving broker

### Audit matrix

| Surface | Real scan roots | Real filesystem traversal | Real instantiation | Real sandbox broker | Runtime-owned proof |
| --- | --- | --- | --- | --- | --- |
| `signal-plugin-vst3` | yes | no | no | shared scaffold only | bounded |
| `signal-plugin-au` | yes | no | no | shared scaffold only | bounded |
| `signal-plugin-lv2` | yes | no | no | shared scaffold only | bounded |
| `signal-plugin-sandbox` | n/a | n/a | CLAP-only synthetic | no | no |
| host-local scan path | request routing only | no | no | demo-driven | bounded |
| host-server scan path | request routing only | no | no | demo-driven | bounded |

This contract exists to replace that matrix with real discovery and execution,
not to restate the scaffold as a finished implementation.

## Deferred scope

- plugin-editor UI embedding
- product-local browser shells
- vendor certification matrices
- unsupported backend-private extension depth that has not yet been promoted
  into shared DTOs

## Rules

### Rule 1: fixture discovery may remain only as test fixtures

Fixture catalogs and synthetic scan responses may still support tests, but they
must not remain the implementation path for production discovery.

### Rule 2: sandbox faults must resolve to typed runtime receipts

Adapter or sandbox failures must surface as runtime-owned fault and continuity
receipts, not panic text or host-local wrapper states.

### Rule 3: adapter depth must stay format-local until promoted

Module traversal, class-factory details, AudioComponent metadata, and LV2
manifest specifics remain adapter-private unless explicitly promoted into
shared DTOs.

### Rule 4: demo assemblies cannot remain the production authority

Host demo assemblies and demo-only discovery helpers may stay as proof tools,
but they must not continue to answer the canonical plugin-hosting path once real
discovery and sandbox execution land.

## Required proof surfaces

- focused adapter discovery tests per format
- focused sandbox lifecycle tests per format
- stable host-edge receipts showing runtime-owned discovery and execution truth
- at least one interactive demo path per major plugin family under contract
  `079`

## Next Task

Use this contract to drive the `g09` plugin-hosting milestones: shared sandbox
substrate first, then VST3, AU, and LV2 realization roadmaps on top.
