# 001 - g11.001 Bridge Backend Factory

Status: ready
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
- unsupported layout from the existing bridge constructors → `InvalidRequest` or
  the bridge error mapped into `RuntimeError`
- missing broker lease for `DedicatedSandbox` → `ResourceUnavailable`

## Scope

- `crates/signal-host-local` plus its `Cargo.toml` dependencies
- add `signal-plugin-bridge` and `signal-plugin-lv2` deps as needed
- own an LV2 adapter on `LocalRuntimeHost` the same way CLAP/AU/VST3 are owned
  today, using existing `signal-plugin-lv2` discovery/hosting surfaces
- keep runtime placement, lifecycle, and supervisor receipts authoritative
- unit or host-crate tests that the factory constructs processors and rejects
  `SharedSandbox`

Do not:

- drive render-plane offline stages (card 002)
- add public host-edge e2e proof beyond factory construction (card 003)
- implement SharedSandbox multiplexing (`g11.002`)
- rebuild adapter hosting, change Contract `072`/`014` semantics, or touch
  stretch / Loophole / Chorus surfaces

## Steps

1. Add `signal-plugin-bridge` (and LV2 if missing) to `signal-host-local`.
2. Store discovered LV2 types beside the existing CLAP/AU/VST3 maps.
3. Implement `prepare_plugin_processor` with the routing table above.
4. Reuse existing broker-session plumbing for `DedicatedSandbox`; do not invent
   a second sandbox protocol.
5. Add focused host-crate tests: one in-process construction per format that
   already has a fixture/compiler path, plus a SharedSandbox rejection test.
6. Update `LocalRuntimeHost` crate docs so they no longer say the host never
   instantiates plugins, if that sentence is now false.
7. Write the batch log and point Next Task at card 002.

## Acceptance Criteria

- `prepare_plugin_processor` exists on `LocalRuntimeHost` with the frozen
  signature
- CLAP, VST3, AU, and LV2 each have an in-process construction path through
  that method
- `DedicatedSandbox` attaches `ShmPluginProcessor` from a real broker lease
  rather than a fake handle
- `SharedSandbox` is a typed rejection with `shared_sandbox_unimplemented`
- runtime-owned receipts remain the placement/lifecycle authority
- focused `signal-host-local` tests cover construction and the SharedSandbox
  rejection
- no render-plane consumer wiring or public host-edge e2e is required on this
  card

## Validation

- `effigy qa:docs` if docs change
- targeted `cargo test -p signal-host-local` (or `effigy test` equivalent)
- do not run the full workspace suite unless a host-local change forces it

## Evidence Required

- batch log under `docs/logs/YYYY-MM/`
- validation actually run
- note any format whose fixture path had to be skipped and why

## Stop Conditions

- constructing a processor would require a new sandbox protocol or a new
  isolation tier
- LV2 host ownership cannot reuse existing `signal-plugin-lv2` surfaces and
  becomes a new architecture
- the frozen signature is insufficient and a different consumer API is needed
- work spills into render-plane plan compilation or Pulse-facing workflow

## Next Task

If this card closes cleanly, auto-start
`docs/roadmaps/g11/batch-cards/002-g11-001-render-plane-consumer-wiring.md`.
