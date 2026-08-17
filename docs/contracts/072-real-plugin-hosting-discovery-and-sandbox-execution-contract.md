# 072 Real Plugin Hosting, Discovery, And Sandbox Execution Contract

Status: active
Owner: core-product
Updated: 2026-08-17
Related contracts: `docs/contracts/007-plugin-backend-and-host-neutral-delegation-contract.md`, `docs/contracts/008-backend-neutral-plugin-capability-and-adapter-breadth-contract.md`, `docs/contracts/014-plugin-isolation-policy-transport-rebind-and-shared-sandbox-continuity-contract.md`, `docs/contracts/020-vst3-adapter-baseline-and-runtime-owned-lifecycle-contract.md`, `docs/contracts/021-au-adapter-baseline-and-runtime-owned-lifecycle-contract.md`, `docs/contracts/038-lv2-adapter-baseline-and-linux-native-plugin-lifecycle-contract.md`
Related architecture: `docs/architecture/system-architecture.md`, `docs/architecture/graph-runtime-feature-reference.md`, `docs/architecture/system-inventory.md`

## Purpose

Freeze the post-audit contract for real filesystem-backed discovery, plugin
instantiation, sandbox execution, and runtime-owned lifecycle truth. This
contract drove the `g09` plugin-hosting program and remains the authority for
what "real hosting" means in Signal.

## Authority hierarchy

1. `signal-plugin` owns the format-neutral plugin vocabulary, lifecycle states,
   transport contracts, placement semantics, and consumer-visible fault meaning.
2. `signal-runtime` owns discovery receipts, runtime lifecycle interpretation,
   chain continuity, and supervisor/export proof.
3. `signal-plugin-clap`, `signal-plugin-vst3`, `signal-plugin-au`, and
   `signal-plugin-lv2` own format-specific realization:
   - scan roots and traversal
   - descriptor and capability extraction
   - plugin process bring-up and teardown
   - host bridge and backend-native callback adaptation
4. `signal-plugin-sandbox` owns the long-lived sandbox broker process, not a
   demo-only harness.
5. `signal-plugin-bridge` owns the render-plane-facing processing backends
   behind one placement-agnostic handle.
6. host crates may orchestrate scans and broker sandbox transport, but they do
   not own plugin capability meaning, lifecycle classification, or recovery
   semantics.

## Required shared guarantees

### Guarantee 1: discovery must be real, inspectable, and filesystem-backed

Signal must scan real plugin roots and record the provenance of discovered
modules, bundles, manifests, or components in runtime-owned receipts.

### Guarantee 2: adapter realization must end in one shared lifecycle model

Format-specific bring-up detail may differ, but CLAP, VST3, AU, and LV2
instances must collapse into one Signal-owned lifecycle and continuity
vocabulary.

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

The baseline hosting program is implemented. Treat the April 2026 scaffold
matrix below as historical evidence only — it described pre-`g09` posture.

### Shared runtime and host posture

- `signal-runtime` owns scan request recording, discovered-type snapshots,
  parity summaries, and consumer-visible discovery receipts.
- `signal-host-local` routes plugin scan requests into runtime-owned receipts
  and carries bounded demo assemblies for proof paths.
- demo helpers and fixture catalogs may remain for stable automation, but they
  must not be mistaken for the only hosting path.

### Adapter and bridge posture today

- `signal-plugin-clap`
  - real library loading, factory entry, lifecycle, `process()`, events,
    state, and GUI hosting
- `signal-plugin-vst3`
  - real scan roots, out-of-process scan helper, lifecycle, `process()`,
    parameter/event translation, and GUI hosting
- `signal-plugin-au`
  - macOS registry and plist discovery, lifecycle, `AudioUnitRender` pull,
    parameter inventory, and GUI hosting
- `signal-plugin-lv2`
  - manifest scanning, lifecycle, `run`, and parameter hosting
- `signal-plugin-sandbox`
  - long-lived broker child with shared-memory block transport and crash
    isolation proofs for CLAP, VST3, AU, and LV2
- `signal-plugin-bridge`
  - **InProcess** and **DedicatedSandbox** tiers behind
    `RenderPluginProcessor`; **SharedSandbox** tier is modeled but unimplemented
    in v1

### Current matrix

| Surface | Real discovery | Real instantiation | In-process `process()` | Dedicated sandbox | Render-plane handle |
| --- | --- | --- | --- | --- | --- |
| CLAP | yes | yes | yes | yes | yes |
| VST3 | yes | yes | yes | yes | yes |
| AU | yes | yes | yes | yes | yes |
| LV2 | yes | yes | yes | yes | yes |

### Remaining gaps (not "hosting missing")

| Gap | Meaning |
| --- | --- |
| SharedSandbox tier | one broker, many plugins — deferred |
| Production host-assembly wiring | `signal-host-local` does not yet wire bridge backends into the Pulse-facing consumer path end to end |
| Product browser/workflow shells | downstream-app UX remains outside Signal unless promoted |

Fixture catalogs and compile-time test fixtures remain valid proof tools. They
must not be described as the production hosting implementation.

## Deferred scope

- SharedSandbox multi-plugin broker tier
- product-local browser shells and workflow UX
- vendor certification matrices
- unsupported backend-private extension depth that has not yet been promoted
  into shared DTOs

## Rules

### Rule 1: fixture catalogs are test and proof tools only

Fixture catalogs and synthetic scan responses may support tests and bounded
demos, but docs must not describe them as the production hosting path when
real adapters and bridge backends exist.

### Rule 2: sandbox faults must resolve to typed runtime receipts

Adapter or sandbox failures must surface as runtime-owned fault and continuity
receipts, not panic text or host-local wrapper states.

### Rule 3: adapter depth must stay format-local until promoted

Module traversal, class-factory details, AudioComponent metadata, and LV2
manifest specifics remain adapter-private unless explicitly promoted into
shared DTOs.

### Rule 4: demo assemblies cannot remain the production authority

Host demo assemblies and demo-only discovery helpers may stay as proof tools,
but the canonical hosting story is the adapter + bridge + sandbox stack above.

## Required proof surfaces

- focused adapter discovery tests per format
- focused sandbox lifecycle tests per format (`crates/signal-plugin-sandbox/tests/plugin_hosting/`)
- bridge in-process and shm tests per format (`crates/signal-plugin-bridge/`)
- stable host-edge receipts showing runtime-owned discovery and execution truth
- render-plane offline/plugin-stage proofs where applicable

## Next Task

Use this contract when extending hosting depth — SharedSandbox tier, production
host-assembly wiring, or promoted product workflow — not when deciding whether
Signal can host CLAP/VST3/AU/LV2 at all. That baseline is already shipped.
