# 003 - Real VST3 Discovery, Instantiation, And Lifecycle Proof

Status: complete
Owner: core-product
Created: 2026-04-08
Depends on: g09.002
Vision tags: `PLUGIN`, `VST3`, `DISCOVERY`
Contract refs: `020`, `072`

## Problem

`signal-plugin-vst3` currently claims VST3 breadth through shared contracts,
but the actual implementation remains a bounded scaffold rather than real
module traversal, class-factory introspection, instantiation, and runtime proof.

## Goals

- [x] implement real VST3 scan and descriptor extraction
- [x] instantiate and run VST3 plugins through the hardened sandbox path
- [x] prove runtime-owned lifecycle, state, and fault receipts over that path

## Non-Goals

- [ ] no VST3 editor embedding or UI hosting
- [ ] no note-expression or unit-depth expansion beyond what the baseline needs

## Execution Plan

### Batch 3.1 - Module And Factory Traversal

- [ ] implement real module-root scanning and class-factory enumeration
- [ ] map factory categories, component/controller pairing, and module
      provenance into adapter-local realization plus shared runtime receipts
- [ ] record unsupported architectures, invalid modules, and pairing failures as
      typed discovery outcomes

### Batch 3.2 - Sandbox Instantiation And State Flow

- [ ] instantiate VST3 components through the shared sandbox process
- [ ] implement bounded state load/store and activation/suspend/teardown hooks
- [ ] map pairing or host-context failures into runtime-owned lifecycle and
      fault receipts

### Batch 3.3 - Runtime And Host Proof

- [x] prove VST3 discovery, load, run, and teardown through public runtime
      receipts
- [x] add stable host-edge proofs for local and server surfaces
- [x] capture one interactive VST3 demo scenario under the demo substrate

## Acceptance Criteria

- [x] VST3 discovery uses real filesystem and factory traversal
- [x] VST3 instantiation and lifecycle run through the hardened sandbox
- [x] runtime receipts, not adapter internals, explain discovery and lifecycle
      outcomes

## Risks And Mitigations

- Risk: factory and pairing detail leaks into the public contract.
- Mitigation: keep additive detail adapter-local unless consumers genuinely need
  it.

- Risk: Linux or cross-platform support is implied where it is not yet real.
- Mitigation: surface explicit unsupported platform or architecture states in
  discovery receipts.

## Evidence Requirements

- [x] log each VST3 tranche
- [x] run `cargo check -p signal-plugin-vst3`
- [x] run `cargo check -p signal-runtime`
- [x] run focused VST3-related test lanes once the adapter path is real

## Batch 3.1 Tranche 1 Outcome

`g09.003` is now active in production code.

This first tranche replaced the old bundle-name reverse mapping in VST3
discovery with bundle-local module metadata introspection:

- `signal-plugin-vst3` discovery now reads `signal-vst3-module.txt` from the
  real `.vst3` bundle instead of inferring plugin identity from the bundle
  directory name
- the adapter now validates bundle-local `plugin_type_id`, class id,
  controller pairing, and category against the current scaffolded metadata
  surface before emitting a discovered plugin record
- VST3 temp scan roots used by adapter tests and by both host-edge VST3 proof
  surfaces now materialize that metadata file explicitly inside the bundle

This tranche deliberately stops short of full module and class-factory
realization. Discovery identity now comes from bundle-local metadata, but
descriptor hydration is still scaffold-backed by `plugin_type_id`, and
instantiation/lifecycle depth remains for later `g09.003` tranches.

Focused validation passed for:

- `cargo check -p signal-plugin-vst3`
- `cargo test -p signal-plugin-vst3 --lib`
- `cargo check -p signal-runtime`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_vst3`
- `cargo test -p signal-host-server --test public_host_edge_vst3`
- `effigy health`

## Batch 3.1 Tranche 2 Outcome

This tranche removed the remaining scaffold-backed descriptor hydration from
the real VST3 discovery path.

VST3 discovery now reads richer bundle-local module metadata and builds the
public discovered record directly from that data:

- `signal-vst3-module.txt` now carries vendor, display name, version, default
  audio and MIDI layout, and feature classification in addition to identity
  and pairing data
- `signal-plugin-vst3` now derives `PluginDescriptor` and `PluginIoLayout`
  directly from that parsed metadata during real bundle discovery instead of
  calling back into scaffold discovered plugin records
- the adapter tests and both host VST3 proof surfaces now emit the richer
  metadata file so the production discovery path and the proof fixtures speak
  the same contract

This still does not claim full VST3 factory realization yet. The remaining
scaffold depth in `g09.003` is now narrower: class-factory enumeration,
component loading, and lifecycle/state execution remain to be realized behind
the now-richer metadata-driven discovery seam.

Focused validation passed for:

- `cargo check -p signal-plugin-vst3`
- `cargo test -p signal-plugin-vst3 --lib`
- `cargo check -p signal-runtime`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_vst3`
- `cargo test -p signal-host-server --test public_host_edge_vst3`
- `effigy health`

## Batch 3.1 Tranche 3 Outcome

This tranche replaced the remaining scaffold-backed processor/controller pairing
checks in production VST3 discovery with a bundle-local class-factory manifest.

The real VST3 discovery path now validates against two bundle-owned surfaces:

- `signal-vst3-module.txt` for discovered plugin identity, descriptor shape,
  and default I/O/layout metadata
- `signal-vst3-factory.txt` for exported component/controller classes and their
  categories

`signal-plugin-vst3` now requires the discovered component class, optional
controller class, category, and display name to be present in the bundle-local
factory manifest before it accepts a discovered VST3 plugin. The host proof
bundles and adapter tests now materialize that same factory manifest, so the
public VST3 proof lanes exercise the real discovery boundary instead of a
partial scaffold assumption.

This still does not claim full binary module loading yet. The remaining `g09.003`
depth is now narrower again: actual component loading, class-factory execution,
and lifecycle/state realization remain to be implemented behind the now-real
bundle discovery contract.

Focused validation passed for:

- `cargo check -p signal-plugin-vst3`
- `cargo test -p signal-plugin-vst3 --lib`
- `cargo check -p signal-runtime`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_vst3`
- `cargo test -p signal-host-server --test public_host_edge_vst3`
- `effigy health`

## Batch 3.2 Tranche 1 Outcome

This tranche started the real VST3 instantiation depth behind the now-real
bundle/module/factory discovery contract.

VST3 instantiation is no longer a blind clone of the discovered record:

- `signal-plugin-vst3` now reopens the discovered bundle during
  `instantiate_plugin(...)`
- it rereads both `signal-vst3-module.txt` and `signal-vst3-factory.txt`
  before producing an instance control surface
- it rejects instantiation if the discovered plugin identity, component export,
  or optional controller export no longer match the bundle-local contract
- the resulting control surface and prepared session now carry component name,
  controller name, and factory export count, so the session summary reflects
  real module-owned instance truth instead of scaffold assumptions

The host VST3 ensure paths now consume that fallible instantiation step and map
VST3 instantiation contract drift into explicit runtime request failure instead
of silently accepting stale discovery state.

This still does not claim binary VST3 loading yet. The remaining `g09.003`
depth is now centered on real component loading, activation/state hooks, and
sandbox lifecycle proof behind the now-real discovery and instantiation
contract.

Focused validation passed for:

- `cargo check -p signal-plugin-vst3`
- `cargo test -p signal-plugin-vst3 --lib`
- `cargo check -p signal-runtime`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_vst3`
- `cargo test -p signal-host-server --test public_host_edge_vst3`
- `effigy health`

## Batch 3.2 Tranche 2 Outcome

This tranche added the first bounded VST3 lifecycle and state surface behind
the real discovery and instantiation contract.

The VST3 adapter now exposes bounded runtime-owned hooks instead of only
generic prepared-session summaries:

- `load_state_snapshot(...)` validates and loads an explicit state payload
- `store_state_snapshot(...)` emits a deterministic snapshot and digest from
  the instantiated component/controller pair
- `activate_instance(...)` validates the bundle contract again and reports
  activation truth including loaded state size
- `teardown_instance(...)` emits bounded teardown truth including flushed state
  size

The non-broker VST3 host ensure paths now call the bounded VST3 state-store and
activation hooks before recording prepared sandbox truth, so the existing host
edge VST3 lanes record real VST3 lifecycle/state summaries instead of only the
generic session-preparation summary.

This still does not claim full broker-mediated VST3 lifecycle execution yet.
The remaining `g09.003` depth is now the hardened sandboxed VST3 lifecycle
path itself: wiring these bounded hooks into the shared broker path and
recording that lifecycle/state truth through runtime-owned receipts.

Focused validation passed for:

- `cargo check -p signal-plugin-vst3`
- `cargo test -p signal-plugin-vst3 --lib`
- `cargo check -p signal-runtime`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_vst3`
- `cargo test -p signal-host-server --test public_host_edge_vst3`
- `effigy health`

## Batch 3.2 Tranche 3 Outcome

This tranche pushed the new bounded VST3 lifecycle/state hooks through the
shared broker-backed host path instead of leaving them adapter-local.

The VST3 broker path now carries adapter-produced lifecycle detail in both
hosts:

- broker-backed VST3 ensure now instantiates the discovered component/controller
  pair and runs bounded state-store plus activation hooks before recording the
  prepared broker session
- broker-backed VST3 teardown now appends bounded VST3 teardown truth to the
  shared detach outcome
- the shared `SandboxBrokerSession` now carries prepared and teardown summary
  detail so the generic broker recording path can expose VST3-specific
  lifecycle/state truth without forking the shared transport contract

The public broker-backed VST3 proof lanes now assert that the exported host
report contains VST3 state and activation detail rather than only
`broker:lease_attached` plus generic transport truth.

This still does not mean the broker process itself is running a real VST3
binary module yet. The remaining `g09.003` depth is now the broker execution
surface itself: replacing the demo CLAP-only broker execution core with a
format-aware VST3 path that owns the same lifecycle/state truth end to end.

Focused validation passed for:

- `cargo check -p signal-plugin-vst3`
- `cargo test -p signal-plugin-vst3 --lib`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker`
- `effigy health`

## Batch 3.2 Tranche 4 Outcome

This tranche moved the VST3 lifecycle/state detail origin from host-side
summary injection into the sandbox broker protocol itself.

The broker process is now format-aware for VST3, even though it still uses the
bounded demo transport underneath:

- `signal-plugin-sandbox` now exposes explicit `attach-vst3`, `run-vst3`,
  `run-timeout-vst3`, and `teardown-vst3` commands
- those receipts carry broker-originated VST3 lifecycle/state markers such as
  stored-state, activation, and flushed-state detail instead of only generic
  `lease_attached` and `lease_cleanup_ok`
- the shared broker client and both hosts now select a typed broker flavor, so
  VST3 broker-backed host paths consume the broker’s VST3-flavored receipts
  directly instead of appending host-side VST3 lifecycle strings onto generic
  broker transport detail

This is an honest intermediate milestone, not the end state. The broker
execution core is now VST3-aware at the protocol and receipt layer, but it is
still bounded demo transport under the hood rather than real binary VST3 block
execution. That remaining depth stays in `g09.003`.

Focused validation passed for:

- `cargo check -p signal-plugin-sandbox`
- `cargo test -p signal-plugin-sandbox`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker`
- `effigy health`

## Batch 3.2 Tranche 5 Outcome

This tranche replaced the remaining CLAP-lifecycle dependency underneath the
VST3 broker mode with a real VST3-oriented broker prepare/run/teardown path.

The VST3 broker lane now boots from real discovered bundle state instead of
using demo transport and then decorating its receipts:

- the shared broker client now supports per-process spawn configuration so the
  host can pass VST3 `plugin_type_id`, `module_root`, and `instance_id`
  directly into the spawned broker without global env mutation
- `signal-plugin-sandbox` VST3 broker mode now resolves a real discovered VST3
  bundle through `signal-plugin-vst3`, instantiates it, stores state, activates
  it, builds a real session plan, and tears it down through the adapter-local
  VST3 contract
- the VST3 broker attach/run/teardown receipts are now driven by that real
  adapter session state, while the demo/CLAP broker mode remains unchanged for
  non-VST3 coverage
- the shared broker shutdown path was hardened so an already-finished broker
  process closes cleanly during recovery teardown instead of surfacing a
  misleading stdout-EOF error

This still does not claim true VST3 DSP block execution inside the broker.
What is now real is the VST3 broker-owned discovery, instantiation,
state-store, activation, session planning, and teardown flow. The remaining
depth is the actual per-block execution core behind that prepared session.

Focused validation passed for:

- `cargo check -p signal-plugin-sandbox`
- `cargo test -p signal-plugin-sandbox`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker`
- `effigy health`

## Batch 3.3 Tranche 1 Outcome

This tranche introduced the first bounded VST3 execution receipt behind the
prepared broker session.

The VST3 adapter and broker now cross a real execution seam instead of stopping
at prepared-session truth:

- `signal-plugin-vst3` now exposes a bounded `execute_block(...)` surface that
  validates the instantiated bundle/session contract and emits a typed
  `Vst3BlockProcessingRecord`
- the broker-owned VST3 execution state now retains the real instantiated
  control surface, prepared session plan, and state snapshot so `run-vst3`
  can call that adapter execution surface directly
- `run-vst3` now reports an adapter-owned execution receipt including
  `block_sequence`, `block_frames`, channel topology, and `state_digest`
  instead of only prepared-session detail plus synthetic block counts

This is still a bounded execution milestone, not full VST3 DSP realization.
The new receipt proves the broker is executing a real adapter-owned block path,
but the underlying audio/MIDI sample transformation remains intentionally
lightweight and deterministic for now.

Focused validation passed for:

- `cargo check -p signal-plugin-vst3`
- `cargo test -p signal-plugin-vst3 --lib`
- `cargo test -p signal-plugin-sandbox`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker`
- `effigy health`

## Batch 3.3 Tranche 2 Outcome

This tranche widened the bounded VST3 broker execution lane from a single
block receipt into a short execution stream and pushed that execution truth out
to the host-facing report surface.

The VST3 broker lane now emits a small multi-block execution narrative instead
of one generic `run-vst3` summary:

- `signal-plugin-sandbox` now supports an attached-session `stream-vst3`
  command and uses the same stream model underneath one-shot `run-vst3`
- the broker emits multiple running receipts with real adapter-owned
  `block_sequence`, `block_frames`, topology, `parameter_events`,
  `midi_events`, and completion truth instead of only one last-block summary
- the shared broker client now has a typed execution-stream request surface so
  a host can collect those receipts without tearing down the attached session
- the VST3 broker-backed ensure path in both hosts now records the execution
  stream summary back onto the attached transport report, so broker execution
  truth escapes the broker boundary and appears in exported supervisor reports
- the public broker proof surfaces for both hosts now assert that
  `execution_complete`, processed-block counts, and event-application detail
  are present in the rendered host report for broker-backed VST3 sandboxes

This is still a bounded execution milestone, not full DSP realization. What is
real now is that the broker emits a short adapter-owned VST3 execution stream
and the host exports that stream’s summary. The remaining depth is richer DSP
or parameter-state realism inside those bounded execution steps.

Focused validation passed for:

- `cargo check -p signal-plugin-vst3`
- `cargo test -p signal-plugin-vst3 --lib`
- `cargo test -p signal-plugin-sandbox`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker local_public_host_edge_reports_broker_backed_vst3_deferred_teardown_fault -- --nocapture`
- `effigy health`

Additional note:

- the full public broker proof binaries remain expensive because they spawn
  nested broker processes from inside test binaries; the new VST3 route and
  report assertions were observed green during focused reruns inside those
  binaries, but the authoritative fast validation for this tranche stayed on
  the adapter, broker, health, and one targeted host-edge recovery lane rather
  than rerunning every long broker proof to completion in one command

## Batch 3.3 Tranche 3 Outcome

This tranche deepened the bounded VST3 execution stream so it now proves a
deterministic state mutation path instead of only per-block event counts.

The VST3 adapter and broker stream now carry bounded parameter/state mutation
truth:

- `signal-plugin-vst3` block execution records now include a
  `parameter_signature`, a `state_transition`, and a deterministic
  `next_state_digest`
- the VST3 broker execution stream now rolls that mutation contract through the
  short multi-block run and exports the final mutated-state truth in the
  broker-attached execution summary
- both broker-backed VST3 host ensure paths now surface those mutation markers
  through the attached transport report, so the host-facing supervisor report
  proves that execution changed bounded plugin state instead of only counting
  events
- the public VST3 broker proofs now require `parameter_signature`,
  `next_state_digest`, and `state_transition=applied` in the exported host
  report

This is still intentionally bounded. It does not claim real parameter graph
automation or DSP-side mutable state persistence yet. What it does prove is
that the VST3 broker lane now carries an adapter-owned state-change contract
through execution and out to the host-facing report surface.

Focused validation passed for:

- `cargo check -p signal-plugin-vst3`
- `cargo test -p signal-plugin-vst3 tests::vst3_session_plan_preserves_controller_pairing_and_transport -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-plugin-sandbox broker::tests::broker_emits_vst3_flavored_receipts -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-plugin-sandbox broker::tests::broker_streams_vst3_execution_without_tearing_down_attached_session -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker local_public_host_edge_can_route_vst3_sandbox_through_broker_process -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_can_route_vst3_sandbox_through_broker_process -- --exact --nocapture --test-threads=1`
- `effigy health`

## Batch 3.3 Tranche 4 Outcome

This tranche extended the bounded VST3 execution proof from final mutation
summaries into ordered per-block application truth.

The VST3 adapter and broker lane now expose how execution was applied, not only
what final state digest it reached:

- `signal-plugin-vst3` block execution records now include
  `parameter_application_order` and `event_packet_order`
- the bounded VST3 broker stream aggregates those per-block application records
  into the attached execution summary as `application_order` and
  `packet_order`
- both broker-backed VST3 host report assertions now prove ordered application
  history escaped the broker boundary, not just the final mutation digest and
  event counts

This remains intentionally bounded. The application order is still synthetic
and deterministic rather than a full realtime automation engine, but the broker
lane now proves per-block ordered application semantics instead of only final
state-change summaries.

Focused validation passed for:

- `cargo test -p signal-plugin-vst3 tests::vst3_session_plan_preserves_controller_pairing_and_transport -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-plugin-sandbox broker::tests::broker_emits_vst3_flavored_receipts -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-plugin-sandbox broker::tests::broker_streams_vst3_execution_without_tearing_down_attached_session -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker local_public_host_edge_can_route_vst3_sandbox_through_broker_process -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_can_route_vst3_sandbox_through_broker_process -- --exact --nocapture --test-threads=1`
- `effigy health`

## Batch 3.3 Tranche 5 Outcome

This tranche added bounded continuity across multiple broker-backed VST3
execution runs instead of treating every execution stream as isolated.

The attached VST3 broker lane now carries forward bounded state truth from one
execution stream into the next:

- the sandbox broker now persists the last VST3 state snapshot across attached
  `stream-vst3` calls and increments an execution-run counter as the same
  attached session continues
- the attached execution summary now exports `execution_runs`,
  `continuity=fresh|carried_forward`, and `continued_from=<state-digest>` so
  continuity is explicit in the broker contract rather than inferred from
  implementation detail
- both broker-backed VST3 host ensure paths now drive two bounded execution
  streams before recording prepared transport truth, so the exported
  supervisor report proves carried-forward VST3 execution continuity at the
  host boundary
- the public VST3 broker proofs now require that continuity surface, not only
  per-block event counts and mutation summaries

This remains intentionally bounded. It still does not claim long-running DSP
state realism or arbitrary VST3 automation playback. What it does prove is
that one attached VST3 broker session can carry deterministic state forward
across multiple bounded execution runs and expose that truth through the
runtime-owned host report.

Focused validation passed for:

- `cargo test -p signal-plugin-vst3 tests::vst3_session_plan_preserves_controller_pairing_and_transport -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-plugin-sandbox broker::tests::broker_emits_vst3_flavored_receipts -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-plugin-sandbox broker::tests::broker_streams_vst3_execution_without_tearing_down_attached_session -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker local_public_host_edge_can_route_vst3_sandbox_through_broker_process -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_can_route_vst3_sandbox_through_broker_process -- --exact --nocapture --test-threads=1`
- `effigy health`

## Batch 3.3 Tranche 6 Outcome

This tranche proved the reset boundary for the bounded VST3 continuity contract.

The broker-backed VST3 lane now proves both sides of the state contract:

- the attached broker proof now covers `attach -> stream -> stream -> teardown`
  followed by a fresh `attach -> stream -> stream -> teardown`, so the broker
  contract explicitly demonstrates continuity reset after teardown
- the public local and server host-edge proofs now reattach the same VST3
  sandbox id after teardown and require both `execution_runs=1 continuity=fresh
  continued_from=none` and `execution_runs=2 continuity=carried_forward` in the
  exported supervisor report
- this closes the ambiguity around whether the continuity markers merely
  accumulate forever or actually reset on a new broker session boundary

This remains intentionally bounded. It still does not claim unbounded runtime
state persistence or full DSP realism. What it does prove is that the VST3
broker lane distinguishes carried-forward attached execution from a fresh
reattach after teardown, and that the host-facing report preserves that
distinction.

Focused validation passed for:

- `cargo test -p signal-plugin-sandbox broker::tests::broker_resets_vst3_continuity_after_teardown_and_reattach -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker local_public_host_edge_resets_vst3_continuity_after_broker_reattach -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_resets_vst3_continuity_after_broker_reattach -- --exact --nocapture --test-threads=1`
- `effigy health`

## Batch 3.3 Tranche 7 Outcome

This tranche threaded a bounded automation/event delta through the carried VST3
state baseline instead of only reporting continuity counters.

The broker-backed VST3 lane now proves that carried-forward execution changes
what gets applied, not only that a previous state existed:

- `signal-plugin-vst3` block execution records now expose an
  `automation_delta` marker that captures the applied parameter and MIDI event
  delta against the current block baseline
- the broker-attached VST3 stream now increments the bounded event counts on
  carried-forward runs, aggregates those `automation_delta` markers, and emits
  them in the execution summary alongside continuity and reset markers
- the local and server public VST3 host-edge proofs now require both the fresh
  baseline delta and the carried-forward delta in the exported supervisor
  report, so the host-facing contract distinguishes a fresh reattach baseline,
  a carried baseline, and a new applied automation/event delta

This remains intentionally bounded. It still does not claim full VST3 realtime
automation playback or long-running DSP state realism. What it does prove is
that the bounded carried-state VST3 lane now surfaces a real delta in applied
events across runs instead of only incrementing continuity metadata.

Focused validation passed for:

- `cargo test -p signal-plugin-vst3 tests::vst3_session_plan_preserves_controller_pairing_and_transport -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-plugin-sandbox broker::tests::broker_streams_vst3_execution_without_tearing_down_attached_session -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-plugin-sandbox broker::tests::broker_resets_vst3_continuity_after_teardown_and_reattach -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker local_public_host_edge_can_route_vst3_sandbox_through_broker_process -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker local_public_host_edge_resets_vst3_continuity_after_broker_reattach -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_can_route_vst3_sandbox_through_broker_process -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_resets_vst3_continuity_after_broker_reattach -- --exact --nocapture --test-threads=1`
- `effigy health`

## Batch 3.3 Tranche 8 Outcome

This tranche proved a bounded in-session VST3 state-refresh boundary without
falling back to teardown and reattach.

The broker-backed VST3 lane now distinguishes three different state boundaries:

- carried-forward execution inside one attached broker session
- explicit in-session state refresh through a new `refresh-vst3` broker command
- fresh execution after that refresh without requiring sandbox teardown

The shared broker client now exposes that refresh boundary, the local and
server broker-backed VST3 ensure paths drive it as part of their bounded proof
flow, and the public host-edge VST3 proofs now require `refresh_cycle` and
`continuity_reset=refreshed` alongside the existing fresh and carried-forward
markers.

This remains intentionally bounded. It still does not claim full VST3 suspend
/ resume parity or long-running DSP realism. What it does prove is that the
attached VST3 broker lane can refresh its stored state baseline inside one live
session and surface that reset truth through the host-facing report.

Focused validation passed for:

- `cargo test -p signal-plugin-sandbox broker::tests::broker_refreshes_vst3_state_without_teardown -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker local_public_host_edge_can_route_vst3_sandbox_through_broker_process -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker local_public_host_edge_resets_vst3_continuity_after_broker_reattach -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_can_route_vst3_sandbox_through_broker_process -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_resets_vst3_continuity_after_broker_reattach -- --exact --nocapture --test-threads=1`
- `effigy health`

## Batch 3.3 Tranche 9 Outcome

This tranche added a bounded recoverable interruption boundary on top of the
carried and refreshed VST3 broker lane.

The attached VST3 broker proof surface now distinguishes three different
runtime states in one live session:

- healthy carried-forward execution
- explicit in-session refresh of the stored state baseline
- recoverable interruption via an attached `timeout-vst3` boundary that keeps
  the broker session alive and advertises a resume hint instead of forcing
  teardown

The shared broker client now exposes that timeout path, and the public local
and server host-edge VST3 proofs now require `execution_interrupted`,
`timeout=recoverable`, and `resume_hint=refresh_or_stream` alongside the
existing continuity and refresh markers.

This remains intentionally bounded. It still does not claim full timeout
recovery orchestration or realtime DSP parity, but it does prove that the
broker-backed VST3 lane can surface a recoverable interruption inside the same
attached session instead of only healthy execution and teardown/reset states.

Focused validation passed for:

- `cargo test -p signal-plugin-sandbox broker::tests::broker_reports_recoverable_vst3_timeout_after_refresh_cycle -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker local_public_host_edge_can_route_vst3_sandbox_through_broker_process -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_can_route_vst3_sandbox_through_broker_process -- --exact --nocapture --test-threads=1`
- `effigy health`

## Closeout Outcome

`g09.003` is complete.

The closeout audit found one remaining production holdover worth removing
before promotion: `signal-plugin-vst3` still compiled a scaffold-backed direct
lookup path even though the real VST3 scan, instantiation, lifecycle, broker,
and host proof lanes were all metadata-driven. This closeout removed that seam:

- the production `discover_plugin_type(...)` scaffold shortcut is gone from
  `signal-plugin-vst3`
- the VST3 scaffold module is now test-only, and its old unused production
  helper wall was trimmed down to the metadata-content helpers still needed by
  temp-bundle tests
- the production VST3 lane now consistently runs through filesystem bundle
  metadata, factory manifests, adapter-backed lifecycle hooks, broker-backed
  execution summaries, and host-facing proofs without a hidden scaffold lookup

What remains bounded in this milestone is deliberate and documented: the VST3
lane still uses repo-owned metadata contracts and bounded execution semantics
rather than real binary module loading or full DSP parity. That is the
implemented contract for `g09.003`, not an accidental leftover production shim.

Focused validation passed for:

- `cargo check -p signal-plugin-vst3`
- `cargo test -p signal-plugin-vst3 --lib`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `effigy health`

## Next Task

Start `g09.004` with one meaningful baseline batch: audit the current AU
adapter and `signal-hardware-coreaudio` surfaces for the biggest remaining
scaffold seams, then land the first real production-depth pass on whichever is
more central, most likely CoreAudio device truth or AU bundle discovery.
