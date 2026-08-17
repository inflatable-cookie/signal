# Production Host-Assembly Integration

Status: active
Owner: core-product
Updated: 2026-08-17
Roadmap: `docs/roadmaps/g11/001-production-host-assembly-wiring.md`
Contracts: `072`, `009`, `014`

## Purpose

Map the production consumer path from `signal-host-local` through
`signal-plugin-bridge` to render-plane plugin execution. This file exists so
planners and agents do not confuse **shipped hosting substrate** with **missing
host-assembly wiring**.

## Authority chain

```text
Consumer (Pulse / Loophole)
  -> LocalRuntimeHost (Contract 009 host edge)
       -> SignalRuntime placement, lifecycle, supervisor receipts
       -> bridge backend factory (this milestone adds)
            -> signal-plugin-bridge (InProcess | DedicatedSandbox)
                 -> adapter hosting (CLAP | VST3 | AU | LV2)
                      -> optional signal-plugin-sandbox broker child
  -> RenderPluginProcessor on render-plane stages
```

Rules:

- runtime owns placement outcomes, lifecycle class, and supervisor meaning
- host assembly orchestrates scans, broker sessions, and backend construction
- bridge owns audio-thread `process()` behavior per isolation tier
- adapters own format-specific bring-up; they do not become a parallel consumer
  contract

## Shipped substrate (not g11.001 scope)

Already implemented and tested outside the host assembly:

| Layer | Crate(s) | Proof today |
| --- | --- | --- |
| Format hosting | `signal-plugin-clap`, `-vst3`, `-au`, `-lv2` | adapter + bridge unit tests |
| Sandbox broker | `signal-plugin-sandbox` | `tests/plugin_hosting/*` |
| Bridge backends | `signal-plugin-bridge` | in-process + shm tests per format |
| Render handle | `signal-render-plane` | offline + test plugin stages |

Contract `072` is the baseline authority for this table.

## v1 integration scope (g11.001)

`g11.001` closes the gap between the table above and the Pulse-facing host
assembly.

### Supported isolation tiers in v1

| Tier | Bridge backend | Host responsibility |
| --- | --- | --- |
| `InProcess` | `InProcess*Processor` | load from discovered type; direct FFI on audio thread |
| `DedicatedSandbox` | `ShmPluginProcessor` | bind to broker child; shm round-trip with bounded wait |
| `SharedSandbox` | none in v1 | typed rejection until `g11.002` |

### Host assembly work breakdown

1. **Backend factory on `LocalRuntimeHost`**
   Frozen signature for `g11.001` Batch 1.2:

   ```rust
   pub fn prepare_plugin_processor(
       &mut self,
       plugin_type_id: &str,
       tier: signal_plugin::PluginIsolationTier,
   ) -> Result<signal_render_plane::RenderPluginProcessor, signal_runtime::RuntimeError>;
   ```

   - `InProcess` → matching `InProcess*Processor::load_and_activate`
   - `DedicatedSandbox` → `ShmPluginProcessor::attach` from the broker lease
   - `SharedSandbox` → `UnsupportedCapability` / `shared_sandbox_unimplemented`
   - unknown type or missing lease → `ResourceUnavailable`

2. **Render-plane wiring**
   - hand `RenderPluginProcessor` handles to render-plane stages from the host
     assembly rather than test-only construction
   - start with one offline proof path, then public host-edge proof

3. **Doc and receipt truth**
   - remove "discovery only" / "no instantiation" language where it understates
     current wiring gaps
   - keep runtime receipts authoritative on placement and lifecycle

### Explicit non-goals

- SharedSandbox multiplexing (`g11.002`)
- product browser or workflow UX
- graph successor, device depth, or stretch work
- rebuilding adapter hosting

## Current seam (accurate as of 2026-08-17)

`LocalRuntimeHost` today:

- scans real plugin roots when asked (`start_plugin_scan`)
- records discovered types and sandbox lifecycle receipts
- can exercise broker attach/run/teardown for proof paths
- does **not** yet construct bridge processors as the canonical consumer path into
  render-plane execution

That is an **integration** gap, not missing hosting capability.

## Proof surfaces required before g11.001 closes

- targeted `signal-host-local` public host-edge tests using real bridge backends
- at least one offline render-plane stage driven from the host assembly
- `effigy validate` on the touched workspace surface
- architecture/inventory/front-door updates recorded in batch logs

## SharedSandbox follow-on

Contract `014` already defines shared-boundary semantics. Implementation is
tracked in `docs/roadmaps/g11/002-shared-sandbox-tier.md`. No separate research
program is required — only product pull after `g11.001` closes.

## Next Task

Execute
`docs/roadmaps/g11/batch-cards/001-g11-001-bridge-backend-factory.md`.
