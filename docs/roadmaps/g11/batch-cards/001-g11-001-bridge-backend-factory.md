# 001 - g11.001 Bridge Backend Factory

Status: complete
Owner: core-product
Updated: 2026-08-17
Master spec refs: none (baseline-routed; no active strict spec)
Roadmap refs: g11.001
Governing refs: docs/contracts/001-working-rules.md, docs/contracts/009-shared-host-convenience-api-and-consumer-edge-contract.md, docs/contracts/072-real-plugin-hosting-discovery-and-sandbox-execution-contract.md, docs/architecture/production-host-assembly-integration.md, docs/roadmaps/g11/001-production-host-assembly-wiring.md
Auto-start next card: yes

## Objective

Add a host-owned bridge backend factory on `LocalRuntimeHost` that constructs a
`RenderPluginProcessor` from a previously scanned plugin type and a runtime-owned
`PluginIsolationTier`.

## Frozen API

```rust
impl LocalRuntimeHost {
    pub fn prepare_plugin_processor(
        &mut self,
        plugin_type_id: &str,
        tier: signal_plugin::PluginIsolationTier,
    ) -> Result<signal_render_plane::RenderPluginProcessor, signal_runtime::RuntimeError>;
}
```

Routing:

- `InProcess` — construct the matching `InProcess*Processor` via existing
  `load_and_activate` on `signal-plugin-bridge`, wrap in `RenderPluginProcessor`
- `DedicatedSandbox` — require or ensure a broker session for that type, then
  `ShmPluginProcessor::attach` from the session lease (`region_id`, `shm_path`,
  bytes, frames, channels, sample rate)
- `SharedSandbox` — return `RuntimeErrorKind::UnsupportedCapability` whose
  message contains the stable token `shared_sandbox_unimplemented`

Typed failures, no panics:

- unknown / unscanned `plugin_type_id` → `ResourceUnavailable`
- unsupported layout from the existing bridge constructors → `InvalidRequest`
- load/open failure from the existing bridge constructors → `ResourceUnavailable`
- missing broker lease for `DedicatedSandbox` → `ResourceUnavailable`

## Scope

Closed. Factory, LV2 host ownership, and focused construction tests landed in
`signal-host-local`.

## Acceptance Criteria

- [x] `prepare_plugin_processor` exists on `LocalRuntimeHost` with the frozen
  signature
- [x] CLAP, VST3, AU, and LV2 each have an in-process construction path through
  that method
- [x] `DedicatedSandbox` attaches `ShmPluginProcessor` from a real broker lease
  rather than a fake handle
- [x] `SharedSandbox` is a typed rejection with `shared_sandbox_unimplemented`
- [x] runtime-owned receipts remain the placement/lifecycle authority
- [x] focused `signal-host-local` tests cover construction and the SharedSandbox
  rejection
- [x] no render-plane consumer wiring or public host-edge e2e is required on this
  card

## Validation

- `cargo test -p signal-host-local`

## Evidence Required

- batch log: `docs/logs/2026-08/17-g11-001-batch-1-2-bridge-backend-factory.md`
- AU in-process construction uses a scanned load-key that resolves stock
  AUDelay through the system registry. AU has no compiled fixture compiler;
  temp bundles are not AudioComponent-visible.

## Stop Conditions

None fired.

## Next Task

Execute
`docs/roadmaps/g11/batch-cards/002-g11-001-render-plane-consumer-wiring.md`.
