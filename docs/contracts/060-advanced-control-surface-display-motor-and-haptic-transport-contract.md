# 060 Advanced Control-Surface Display, Motor, And Haptic Transport Contract

Status: complete
Owner: core-product
Updated: 2026-03-21
Related contracts: `docs/contracts/043-midi-2-0-mpe-and-richer-controller-expression-contract.md`, `docs/contracts/044-control-surface-transport-mapping-and-feedback-contract.md`, `docs/contracts/045-advanced-hardware-extensibility-and-scripting-safe-device-policy-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first reusable Signal-owned display, motor, and haptic transport
boundary so later advanced control-surface feedback work can deepen one shared
runtime vocabulary instead of reopening host-local feedback bridges,
vendor-private display page models, or product-local controller UX as the
shared authority.

## Authority hierarchy

Advanced control-surface feedback meaning has one authority chain:

1. `docs/contracts/043-midi-2-0-mpe-and-richer-controller-expression-contract.md`
   remains the authority for widened controller-expression families whenever
   advanced display, motor, or haptic transport composes with richer event or
   feedback lanes
2. `docs/contracts/044-control-surface-transport-mapping-and-feedback-contract.md`
   remains the authority for control-surface transport posture, mapping
   posture, feedback readiness, and the baseline reusable control-surface
   feedback model
3. `docs/contracts/045-advanced-hardware-extensibility-and-scripting-safe-device-policy-contract.md`
   remains the authority for advanced hardware capability classes,
   scripting-safe device policy, guarded feedback channels, and action-class
   posture that this contract must widen rather than replace
4. host/backend integration layers may own raw device evidence for:
   - vendor display page, strip, segment, or text payload dialects
   - motor-fader, encoder-ring, tension, haptic, or tactile transport packets
   - backend-native scheduling, batching, and delivery quirks for device
     feedback traffic
5. `signal-runtime` must own the canonical consumer-visible meaning for:
   - advanced display posture and bounded display content class
   - motor transport posture and bounded motion authority
   - haptic transport posture and bounded tactile outcome
   - observation, supervisor, and stable host-edge export
6. hosts, device adapters, and future protocol layers may broker raw evidence
   into runtime-owned receipts, but they must not become the authority for:
   - a second display, motor, or haptic taxonomy detached from runtime DTOs
   - product-local controller-page or workflow semantics
   - vendor-private payload schemas as the consumer boundary

If an advanced control-surface feedback claim cannot be explained through
`043`, `044`, `045`, and runtime-owned receipts, it is not yet part of the
shared Signal contract.

## Existing anchors

Batch 9.1 freezes this contract on top of the currently closed control-surface
and advanced-hardware baseline instead of pretending richer display, motor, or
haptic depth already exists:

- `docs/contracts/043-midi-2-0-mpe-and-richer-controller-expression-contract.md`
  - widened event and expression meaning that later display or haptic feedback
    may need to compose with
- `docs/contracts/044-control-surface-transport-mapping-and-feedback-contract.md`
  - baseline control-surface transport, mapping posture, and feedback
    readiness meaning that richer feedback work must widen rather than reopen
- `docs/contracts/045-advanced-hardware-extensibility-and-scripting-safe-device-policy-contract.md`
  - advanced hardware capability, scripting-safe device policy, guarded
    feedback channel, and action-class meaning that richer display, motor, and
    haptic transport must layer on top of
- `crates/signal-runtime/src/interfaces.rs`
  - the current `RuntimeControlSurfaceSnapshot`,
    `RuntimeControlSurfaceCapabilitySummary`, and
    `RuntimeAdvancedHardwareSnapshot` seams that later batches must widen
- `crates/signal-runtime/src/runtime.rs`
  - the current bounded control-surface and advanced-hardware projection path
    that defines the baseline this contract expands from

This contract does not claim richer display pages, motor control, or haptic
transport are already realized. It freezes the meaning later runtime and host
work must obey.

## Shared vocabulary

### Display transport posture

A `display transport posture` is the runtime-owned description of how strongly
Signal can express reusable display output for the current advanced
control-surface device.

Batch 9.1 freezes this bounded family:

- `NotPresent`
- `GuardedDisplay`
- `TextOnlyDisplay`
- `PageAwareDisplay`
- `UnavailableDisplay`

This posture is stronger than raw display capability presence. It says whether
Signal only knows a guarded display channel exists, can provide reusable text,
or can reason about a bounded page-aware display surface.

### Display content class

A `display content class` is the runtime-owned category of display meaning the
shared surface can talk about without leaking vendor-private payload layout.

Batch 9.1 freezes this bounded family:

- `NoDisplayContent`
- `StatusText`
- `ParameterValueText`
- `MeterBridgeText`
- `PagedStatusView`
- `GuardedVendorDisplay`

Display content class is not a vendor page schema and not a product-local
controller UI. It describes reusable Signal-owned display intent.

### Motor transport posture

A `motor transport posture` is the runtime-owned answer for how strongly
Signal can participate in reusable motorized surface transport.

Batch 9.1 freezes this bounded family:

- `NoMotorTransport`
- `GuardedMotorTransport`
- `PositionMotorTransport`
- `BankAwareMotorTransport`
- `UnavailableMotorTransport`

This posture separates bounded reusable motion semantics from raw
device-specific servo or tension protocols.

### Haptic transport posture

A `haptic transport posture` is the runtime-owned answer for how strongly
Signal can participate in reusable haptic or tactile feedback transport.

Batch 9.1 freezes this bounded family:

- `NoHapticTransport`
- `GuardedHapticTransport`
- `CueOnlyHapticTransport`
- `StateAwareHapticTransport`
- `UnavailableHapticTransport`

This keeps haptic meaning typed and reusable without claiming any one
vendor-specific haptic dialect is already shared.

### Feedback authority

`feedback authority` means where the currently active display, motor, or
haptic transport meaning is allowed to come from.

Batch 9.1 freezes this bounded family:

- `RuntimeDefault`
- `RuntimeDeclared`
- `HostForwarded`
- `DeviceAdvisory`

This keeps the shared ownership line explicit: hosts and devices may forward
detail, but the shared feedback boundary must still reduce to one runtime-owned
authority answer.

### Feedback outcome

A `feedback outcome` is the runtime-owned result when display, motor, or
haptic transport intent is projected onto the currently available control-
surface and advanced-hardware substrate.

Batch 9.1 freezes this bounded family:

- `PreserveDeclaredFeedback`
- `CollapseToGuardedFeedback`
- `ObserveOnlyFeedback`
- `BypassFeedbackTransport`
- `TerminalFeedbackFailure`

This outcome is distinct from baseline feedback readiness. It explains what
happened at the widened advanced feedback boundary once richer transport is in
scope.

## Rules

### Rule 1: richer feedback meaning must stay runtime-owned

Display posture, display content class, motor transport posture, haptic
transport posture, feedback authority, and feedback outcome belong to
runtime-owned receipts, not to host-local feedback bridges, vendor-private page
schemas, or product-local controller UX.

### Rule 2: richer feedback must compose with the closed control-surface baseline

This contract widens the closed controller-expression, control-surface, and
advanced-hardware seams. It must not replace feedback readiness, guarded
feedback-channel truth, or action-class posture with a second device-private
taxonomy.

### Rule 3: raw vendor payload detail stays advisory

Display-page schemas, segment maps, LED color planes, motor servo detail,
haptic waveform payloads, and device-native batching rules may exist
internally, but the shared boundary must remain grounded in runtime-owned
display, motor, haptic, authority, and outcome meaning.

### Rule 4: live feedback and observation surfaces must converge

Later richer feedback work may stage rollout, but it must not create one
display or haptic model for live device delivery, another for observation, and
a third for supervisor export.

### Rule 5: product-local controller workflow stays out of scope

This contract freezes reusable runtime and adapter meaning only. It does not
freeze controller page design, workflow choreography, device editor UX, preset
management, or product-local surface scripting.

## Deferred scope

Batch 9.1 intentionally leaves these out:

- final runtime execution, delivery, and observation receipts for display,
  motor, and haptic transport
- public runtime, supervisor, and host-edge proof surfaces
- concrete vendor display layouts, motor control payloads, or haptic waveform
  schemas
- product-local controller page workflow, device editors, or preset UX

## Batch 9.1 outcome

Batch 9.1 freezes the first reusable advanced control-surface feedback
authority line for Signal:

- display transport posture, display content class, motor transport posture,
  haptic transport posture, feedback authority, and feedback outcome are now
  explicit Signal-owned vocabulary
- later runtime execution can widen from the closed controller-expression,
  control-surface, and advanced-hardware seams instead of inventing a parallel
  vendor-display or device-feedback policy shell
- unsupported or device-private feedback paths are now required to explain
  themselves through bounded transport and outcome meaning rather than
  product-local controller UX or host-local protocol glue

## Batch 9.2 outcome

Batch 9.2 turns this contract into a real runtime seam:

- `signal-runtime` now owns typed display posture, display content class,
  motor posture, haptic posture, feedback authority, and feedback outcome on
  the existing `RuntimeAdvancedHardwareSnapshot` family
- the same advanced-hardware seam now carries aggregate display, motor, and
  haptic transport counts instead of leaving richer feedback depth implicit in
  action-class flags
- the stable local and server host edges now export the same bounded advanced
  display, motor, and haptic answers as the runtime-owned advanced-hardware
  seam

The bounded baseline is intentionally conservative:

- the current guarded control-surface path reports `GuardedDisplay`,
  `GuardedVendorDisplay`, `NoMotorTransport`, `NoHapticTransport`,
  `RuntimeDefault`, and `CollapseToGuardedFeedback`
- empty advanced-hardware paths still surface zero display, motor, and haptic
  transport counts instead of fabricating feedback depth
- Batch 9.3 still has to prove the widened seam through the shared supervisor
  consumer boundary before this milestone is complete

## Batch 9.3 outcome

Batch 9.3 closes the widened consumer boundary without inventing a second
device-feedback proof seam:

- `signal-supervisor-tools` now points the existing
  `signal.runtime.advanced-hardware-boundary` descriptor at this contract
  instead of the earlier advanced-hardware baseline-only seam
- the shared supervisor descriptor now names display, motor, and haptic
  transport counts plus device-level posture and bounded feedback outcome
  anchors on the same runtime-owned snapshot family
- the existing `effigy acceptance:advanced-hardware-boundary` lane now proves
  the same bounded advanced control-feedback answers across public runtime,
  stable host-edge, and supervisor surfaces

This closes the bounded contract meaningfully:

- later control-surface workflow and acceptance work now has one explicit
  runtime-owned advanced feedback authority line to build on
- vendor-private payload schemas and host-local feedback bridges are no longer
  needed to inspect the current guarded display baseline
- page-aware display depth, real motor transport, real haptic transport, and
  fuller controller workflow remain intentionally deferred

## Next Task

Continue `g08.010` with Batch 10.1 by freezing the first runtime-owned
control-surface scene mapping, feedback pages, and safe action graph contract
on top of the closed controller-expression, control-surface, advanced
feedback, and advanced-hardware seams.
