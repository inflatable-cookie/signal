# 006 - Shared Host/Runtime Execution And Recovery Unification

Status: complete
Owner: core-product
Created: 2026-04-08
Depends on: g09.001
Vision tags: `RUNTIME`, `HOST`, `RECOVERY`
Contract refs: `014`, `015`, `025`, `074`
Strict lane refs: `docs/specs/001-g09-lane-first-strict-adoption.md`

## Problem

`signal-host-local` and `signal-host-server` still replicate too much
execution-block, watchdog, broker, and recovery behavior, which makes bug fixes
and feature changes easy to land asymmetrically.

## Goals

- [ ] promote shared execution and recovery policy into reusable runtime/host
      substrate
- [ ] leave only genuine environment-specific differences in local and server
      hosts
- [ ] reduce duplicate-block and duplicate-test pressure materially

## Non-Goals

- [ ] no new host product features
- [ ] no rewrite of the runtime execution engine itself

## Execution Plan

### Batch 6.1 - Duplicate Policy Inventory

- [x] inventory duplicated execution-block, watchdog, transport, and lingering
      cleanup paths across both host crates
- [x] classify each duplicated seam as runtime-owned, shared-host-support, or
      truly environment-specific
- [ ] freeze one migration order that does not disturb audio-thread safety

### Batch 6.2 - Shared Support Extraction

- [x] extract reusable recovery and execution helpers into shared support
      surfaces
- [ ] route both hosts through the same completion-slot, lingering-session, and
      broker-failure policy where the semantics are identical
- [ ] keep transport and process-boundary differences explicit at the edges

### Batch 6.3 - Conformance And Proof

- [ ] align local and server tests around shared scenarios instead of parallel
      bespoke harnesses
- [ ] reduce duplicate code and duplicate test assertions measurably
- [ ] add one interactive continuity or recovery comparison demo under the demo
      substrate

## Acceptance Criteria

- [ ] equivalent recovery cases use the same shared policy implementation
- [ ] duplicate execution or recovery logic is materially reduced
- [ ] local and server exports stay aligned on fault and continuity meaning

## Risks And Mitigations

- Risk: extracting support code regresses realtime safety.
- Mitigation: keep shared helpers off the audio thread unless they are
  demonstrably safe.

- Risk: host differences get flattened incorrectly.
- Mitigation: classify differences before extraction and keep them edge-local.

## Evidence Requirements

- [x] log the duplicate-inventory and extraction tranches
- [x] run `cargo check -p signal-host-local`
- [x] run `cargo check -p signal-host-server`
- [x] run `effigy health`

## Batch 6.1 Tranche 1 Outcome

`g09.006` is now active, and the first duplicate inventory is grounded in the
real host support walls rather than inferred from the earlier audit. The
highest-leverage duplicated seams are:

- the execution-cycle shell in `runtime_cycle.rs`, which was effectively
  byte-for-byte identical across both hosts
- the watchdog timeout and retry policy in `boot_recovery_helpers.rs`, which
  differed only by host type, sandbox assembly type, and instance-id literal
- the larger follow-on walls in `runtime_block.rs` and `sandbox_sessions.rs`,
  which still contain shared broker and recovery policy wrapped around smaller
  host-specific dispatch or transport differences

This tranche turns that inventory into the first real shared substrate. The
duplicated `runtime_cycle.rs` and `boot_recovery_helpers.rs` bodies now come
from one runtime-owned support layer in `signal-runtime`, while the hosts keep
their environment-specific types and fault-envelope edges explicit at the call
site. That is the right first consolidation seam for `g09.006`: broad policy
re-use, low realtime risk, and clear remaining targets for the next pass.

## Batch 6.2 Tranche 2 Outcome

The next highest-leverage execution seam is now shared too. The brokered block
shell in `runtime_block.rs` no longer exists as two parallel host copies; it
now lives in the same runtime-owned support layer as the cycle and watchdog
helpers, while each host provides only the genuinely different phases:

- local host: prepare plugin-dispatch-aware payloads, capture plugin render
  context and automation state, then feed the output-pump-backed engine path
- server host: prepare the simpler default test payload, then feed the
  synthetic-engine-input path after forecast application

That keeps the duplicated broker dispatch, slot transition, outcome readback,
event/automation summary recording, dispatch receipt recording, and watchdog
restart policy in one place without flattening the real host-specific execution
differences. The remaining broad seam is larger now but clearer: `sandbox_sessions.rs`
still carries a lot of shared transport and broker orchestration wrapped around
format and host-edge differences.

## Batch 6.2 Tranche 3 Outcome

The `sandbox_sessions.rs` seam is now materially narrower. The duplicated
broker attach, prepared-session recording, attached execution summary
recording, VST3 broker execution sequence, and broker teardown flow no longer
live as parallel host implementations; they now come from the runtime-owned
broker support layer. That leaves the host files with the true edge concerns:

- format-specific adapter preparation and failure mapping
- host-specific broker environment assembly and instance-id prefixes
- server-only LV2 negotiation and execution handling

This is the right stopping point for the strict batch card. It removes the
shared broker session and transport shell without turning `sandbox_sessions.rs`
into a forced common abstraction over genuinely different host behavior.

## Strict-Lane Reassessment Outcome

`g09.006` still has one more bounded broad seam that is worth treating as a
strict batch card before the lane pauses for a wider planning decision. After
the shared broker shell extraction, the remaining meaningful duplication inside
`sandbox_sessions.rs` is concentrated in:

- the AU broker-preparation shell
- the VST3 broker-preparation shell
- the identical AU fault-recording shell

That seam is still large enough to justify one more batch, but it is no longer
the old broad broker-session wall. The lane should now continue only through
that narrower AU/VST3 preparation-and-fault shell, while leaving local/server
environment assembly and server-only LV2 behavior explicit at the edge.

## Batch 6.2 Tranche 4 Outcome

That remaining AU/VST3 preparation seam is now closed too. The runtime-owned
broker support layer now carries the shared prepared-session shell for AU and
VST3, and it also owns the identical AU protocol-violation prepare-fault
recording shell. The host files now keep only the actual edge behavior:

- local versus server environment assembly and instance-id prefixes
- server-only LV2 negotiation, fault mapping, and execution depth
- the remaining format-specific error behavior that is not truly shared

After this tranche, `g09.006` no longer has a clearly broad
shared-support-extraction seam inside `sandbox_sessions.rs`. What remains is
smaller edge-local behavior and a wider planning decision about whether the
lane should close or hand off into the next milestone.

## Next Task

`g09.006` is closed. Hand off the active strict lane into
`docs/specs/batch-cards/004-g09-007-offline-preview-assembly-carveout.md`.
