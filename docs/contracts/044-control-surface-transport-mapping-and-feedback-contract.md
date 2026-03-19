# 044 Control-Surface Transport, Mapping, And Feedback Contract

Status: complete
Owner: core-product
Updated: 2026-03-18
Related contracts: `docs/contracts/042-external-midi-endpoint-graph-and-device-identity-contract.md`, `docs/contracts/043-midi-2-0-mpe-and-richer-controller-expression-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first runtime-owned control-surface transport, mapping, feedback,
and capability boundary for `g07.013` so later runtime and hardware work can
deepen one shared Signal vocabulary for control surfaces instead of reopening
host-local device tables, adapter-private feedback policy, or product-specific
mapping semantics.

## Authority hierarchy

Control-surface meaning has one authority chain:

1. `docs/contracts/042-external-midi-endpoint-graph-and-device-identity-contract.md`
   remains the authority for bounded external MIDI device, endpoint, route,
   and lifecycle meaning whenever a control surface is attached through MIDI or
   MIDI-adjacent transport
2. `docs/contracts/043-midi-2-0-mpe-and-richer-controller-expression-contract.md`
   remains the authority for widened controller-expression families, `MPE`
   posture, and `MIDI 2.0` posture once control-surface input or feedback uses
   that widened event vocabulary
3. host/backend integration layers may own raw transport evidence for:
   - backend-native device handles, sessions, and attach or detach callbacks
   - hardware-specific LED, display, pad, motor, haptic, or transport detail
   - packet dialects, vendor extensions, and backend-local protocol quirks
4. `signal-runtime` must own the canonical consumer-visible meaning for:
   - control-surface device identity and transport posture
   - control-surface feedback readiness and guarded capability claims
   - reusable mapping-relevant runtime vocabulary
   - supervisor, observation, and stable host-edge export
5. host crates and future adapters may broker raw evidence into runtime-owned
   receipts, but they must not become the authority for:
   - a second control-surface taxonomy detached from runtime DTOs
   - product-local mapping workflow semantics
   - hardware-private feedback models as the consumer boundary

If a control-surface claim cannot be explained through `042`, `043`, and
runtime-owned receipts, it is not yet part of the reusable Signal contract.

## Existing anchors

Batch 13.1 freezes this contract on top of the current shared device and event
surface family:

- `RuntimeExternalMidiEndpointGraphSnapshot`
- `RuntimeExternalMidiEndpointCapabilitySummary`
- `RuntimePluginEventSnapshot`
- `RuntimeObservationReport`
- `RuntimeSupervisorReport`
- stable host-edge `supervisor_report()` export

Batch 13.1 does not claim those anchors already expose true control-surface
transport or feedback depth. It freezes how later DTOs and proofs must widen
from them instead of inventing a separate host-private controller shell.

## Shared vocabulary

### Control-surface device

`control-surface device` means a runtime-owned device identity that is used
for controller input, transport control, feedback output, or guarded mapping-
relevant interaction.

This is broader than a generic MIDI endpoint, but it must still compose
through the closed external MIDI endpoint contract when transport is MIDI-
backed.

### Control-surface transport

`control-surface transport` means the bounded runtime-owned answer for how a
control-surface device exchanges control or feedback traffic with Signal.

Batch 13.1 freezes transport posture as shared meaning rather than a
backend-private session detail.

### Mapping posture

`mapping posture` means the runtime-owned claim about whether a control-
surface signal is:

- portable for reusable mapping
- guarded by device- or protocol-specific constraints
- feedback-only or observe-only
- unsupported on the shared Signal surface

This contract explicitly separates mapping posture from product-local mapping
workflow or UI policy.

### Feedback readiness

`feedback readiness` means the bounded runtime-owned answer for whether a
control-surface device can receive shared feedback output such as transport
state, meter-adjacent cues, parameter indication, or guarded display/update
signals.

Feedback readiness is a runtime-owned capability claim, not a promise that
every hardware-native feedback channel is already portable.

### Capability family

`capability family` means the bounded reusable claim that a control surface can
participate in:

- transport control
- parameter or controller input
- feedback output
- guarded richer controller-expression delivery
- guarded device-identity or layout-specific mapping hints

Capability families must remain typed and runtime-owned.

## Rules

### Rule 1: control-surface work must build on external MIDI and widened event truth

`g07.013` must widen from the closed external MIDI endpoint and widened
controller-expression contracts. It must not create a second device or event
language for control surfaces.

### Rule 2: transport and feedback meaning stay runtime-owned

Host integrations may broker raw transport evidence, but reusable control-
surface transport and feedback meaning must remain canonical on shared runtime
receipts.

### Rule 3: mapping posture must stay separate from product workflow

This contract may define portable versus guarded mapping posture, but it must
not absorb product-local mapping UI, editing workflow, or user-preset policy.

### Rule 4: feedback depth must be typed when guarded or unavailable

If shared feedback output is unavailable, feedback-only, or device-specific,
the shared answer must stay explicit through runtime-owned typed receipts
instead of host-local heuristics.

### Rule 5: raw protocol detail stays advisory unless promoted

Vendor protocol pages, display layouts, LED color models, motor control detail,
pad grids, or backend-native extension packets remain advisory until later
batches promote them into the shared contract.

### Rule 6: later extensibility work must widen from this boundary

Future advanced device extensibility, scripting-safe policy, or control-surface
mapping milestones must reuse this boundary instead of reopening transport or
feedback ownership.

## Deferred scope

Batch 13.1 intentionally does not claim:

- full control-surface runtime realization yet
- product-specific mapping UI or workflow
- device scripting or arbitrary extension execution
- exhaustive vendor protocol parity
- feedback animation or display-layout policy
- richer haptic, motor, or display composition semantics

Those belong to later `g07.013`, `g07.014`, and follow-on acceptance work.

## Batch 13.1 outcome

Batch 13.1 freezes the first bounded control-surface contract:

- Signal now has one explicit runtime-owned target for control-surface device,
  transport, mapping posture, feedback readiness, and bounded capability
  meaning instead of falling back to host-local controller integration logic
- external MIDI endpoint and widened controller-expression meaning remain the
  anchors, which prevents later control-surface work from reopening a second
  transport or event shell
- Batch 13.2 can now materialize the first runtime-owned control-surface
  baseline instead of reopening what control-surface semantics belong to Signal

## Batch 13.2 outcome

Batch 13.2 materializes the first runtime-owned control-surface baseline on top
of this contract:

- `signal-runtime` now owns explicit control-surface graph state, transport
  posture, mapping posture, feedback readiness, and bounded capability receipts
  derived from the external MIDI endpoint graph instead of host-local
  controller policy
- observation, supervisor, and stable host-edge export now carry the same
  control-surface snapshot family, including explicit unavailable, empty, and
  guarded outcomes
- widened-expression capability posture now composes directly with the closed
  `g07.012` controller-expression boundary instead of creating a second
  control-surface capability model

Batch 13.2 still does not claim the consumer-boundary proof seam is closed.
Machine-readable boundary proof and repo-owned acceptance evidence remain Batch
13.3 work.

## Batch 13.3 outcome

Batch 13.3 closes the bounded control-surface proof seam:

- public runtime now proves control-surface graph state, transport posture,
  mapping posture, feedback readiness, and bounded capability truth through
  shared runtime receipts
- both stable host edges now prove they forward the same control-surface
  baseline instead of rebuilding host-local controller policy
- `signal-supervisor-tools` now exposes the machine-readable
  `signal.runtime.control-surface-boundary` descriptor, and Effigy now owns
  `acceptance:control-surface-boundary` as the repo-owned rerun lane

This contract is now closed as the bounded control-surface baseline. Richer
hardware extensibility, vendor protocol depth, and scripting-safe policy widen
from `g07.014`.

## Next Task

Continue `g07.014` with Batch 14.1 by freezing the runtime-owned advanced
hardware extensibility, scripting-safe device policy, and guarded feedback
contract on top of the now-closed control-surface baseline.
