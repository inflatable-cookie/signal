# 061 Control-Surface Scene Mapping, Feedback Pages, And Safe Action Graph Contract

Status: complete
Owner: core-product
Updated: 2026-03-21
Related contracts: `docs/contracts/043-midi-2-0-mpe-and-richer-controller-expression-contract.md`, `docs/contracts/044-control-surface-transport-mapping-and-feedback-contract.md`, `docs/contracts/045-advanced-hardware-extensibility-and-scripting-safe-device-policy-contract.md`, `docs/contracts/060-advanced-control-surface-display-motor-and-haptic-transport-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first reusable Signal-owned scene-mapping, feedback-page, and safe
action graph boundary so later control-surface workflow work can deepen one
shared runtime vocabulary instead of reopening controller-page assumptions,
unsafe device scripts, or host-local scene ledgers as the shared authority.

## Authority hierarchy

Control-surface workflow meaning has one authority chain:

1. `docs/contracts/043-midi-2-0-mpe-and-richer-controller-expression-contract.md`
   remains the authority for widened controller-expression families whenever
   scene mapping or action graphs compose with richer event lanes
2. `docs/contracts/044-control-surface-transport-mapping-and-feedback-contract.md`
   remains the authority for baseline control-surface transport, mapping
   posture, feedback readiness, and reusable device identity or capability
   meaning
3. `docs/contracts/045-advanced-hardware-extensibility-and-scripting-safe-device-policy-contract.md`
   remains the authority for scripting-safe device policy, guarded feedback
   channels, and typed action-class posture that scene mapping and safe action
   graphs must obey rather than bypass
4. `docs/contracts/060-advanced-control-surface-display-motor-and-haptic-transport-contract.md`
   remains the authority for richer display, motor, haptic, and bounded
   advanced feedback outcome meaning that this workflow layer must compose with
   instead of replacing
5. host/backend integration layers may own raw workflow evidence for:
   - vendor page banks, page identifiers, or strip layout internals
   - controller scene recall packets, unsafe macros, and backend-native action
     dispatch quirks
   - low-level scheduling, batching, and delivery details for scene or page
     transitions
6. `signal-runtime` must own the canonical consumer-visible meaning for:
   - scene-mapping posture and bounded scene identity
   - feedback-page posture and bounded page class
   - safe action graph posture and bounded action outcome
   - observation, supervisor, and stable host-edge export
7. hosts, device adapters, and future protocol layers may broker raw evidence
   into runtime-owned receipts, but they must not become the authority for:
   - a second scene or workflow taxonomy detached from runtime DTOs
   - product-local page editing or controller workflow semantics
   - unsafe executable device scripting as the shared consumer boundary

If a control-surface workflow claim cannot be explained through `043`, `044`,
`045`, `060`, and runtime-owned receipts, it is not yet part of the shared
Signal contract.

## Existing anchors

Batch 10.1 freezes this contract on top of the currently closed control-
surface and advanced-feedback seams instead of pretending richer controller
workflow depth already exists:

- `docs/contracts/043-midi-2-0-mpe-and-richer-controller-expression-contract.md`
  - widened controller-expression meaning that later scene mapping or action
    graphs may need to compose with
- `docs/contracts/044-control-surface-transport-mapping-and-feedback-contract.md`
  - baseline transport, mapping posture, feedback readiness, and reusable
    control-surface meaning that workflow depth must widen rather than reopen
- `docs/contracts/045-advanced-hardware-extensibility-and-scripting-safe-device-policy-contract.md`
  - scripting-safe device policy, guarded feedback channel, and action-class
    meaning that safe action graphs must obey
- `docs/contracts/060-advanced-control-surface-display-motor-and-haptic-transport-contract.md`
  - bounded display, motor, haptic, authority, and feedback outcome meaning
    that page and scene work must layer on top of
- `crates/signal-runtime/src/interfaces.rs`
  - the current `RuntimeControlSurfaceSnapshot` and
    `RuntimeAdvancedHardwareSnapshot` seams that later batches must widen
- `crates/signal-runtime/src/runtime.rs`
  - the current bounded control-surface and advanced-hardware projection path
    that defines the baseline this contract expands from

This contract does not claim scene mapping, feedback-page routing, or safe
action graphs are already realized. It freezes the meaning later runtime and
host work must obey.

## Shared vocabulary

### Scene-mapping posture

A `scene-mapping posture` is the runtime-owned description of how strongly
Signal can express reusable controller-scene mapping for the current device.

Batch 10.1 freezes this bounded family:

- `NoSceneMapping`
- `GuardedSceneMapping`
- `ContextualSceneMapping`
- `PortableSceneMapping`
- `UnavailableSceneMapping`

This posture separates reusable scene intent from vendor-private page banks or
product-local controller setup workflow.

### Feedback-page posture

A `feedback-page posture` is the runtime-owned answer for how strongly Signal
can reason about bounded feedback-page transitions or page-aware controller
views.

Batch 10.1 freezes this bounded family:

- `NoFeedbackPages`
- `GuardedFeedbackPages`
- `StatusFeedbackPages`
- `SceneAwareFeedbackPages`
- `UnavailableFeedbackPages`

This posture is stronger than raw display transport. It says whether Signal
can only acknowledge guarded page-like depth or can reason about bounded
scene-aware feedback pages.

### Feedback-page class

A `feedback-page class` is the runtime-owned category of page meaning the
shared surface can talk about without leaking vendor-private layout internals.

Batch 10.1 freezes this bounded family:

- `NoFeedbackPageClass`
- `StatusPage`
- `ParameterPage`
- `MeterPage`
- `ScenePage`
- `GuardedVendorPage`

Feedback-page class is not a vendor display schema and not a product-local
controller UI. It describes reusable Signal-owned page intent.

### Safe action graph posture

A `safe action graph posture` is the runtime-owned answer for how strongly
Signal can represent reusable controller-triggered actions through a bounded
safe action graph instead of raw device scripting.

Batch 10.1 freezes this bounded family:

- `NoSafeActionGraph`
- `GuardedSafeActionGraph`
- `TransportSafeActionGraph`
- `SceneSafeActionGraph`
- `UnavailableSafeActionGraph`

This posture separates reusable safe-action meaning from unsafe arbitrary
device scripts or host-local macro bridges.

### Action authority

`action authority` means where the currently active scene, page, or safe
action graph meaning is allowed to come from.

Batch 10.1 freezes this bounded family:

- `RuntimeDefault`
- `RuntimeDeclared`
- `HostForwarded`
- `DeviceAdvisory`

This keeps the shared ownership line explicit: hosts and devices may forward
detail, but the shared workflow boundary must still reduce to one runtime-owned
authority answer.

### Safe action outcome

A `safe action outcome` is the runtime-owned result when scene mapping,
feedback-page intent, and safe action graph requests are projected onto the
currently available control-surface and advanced-feedback substrate.

Batch 10.1 freezes this bounded family:

- `PreserveDeclaredAction`
- `CollapseToGuardedAction`
- `ObserveOnlyAction`
- `BypassUnsafeAction`
- `TerminalActionFailure`

This outcome is distinct from baseline transport or advanced feedback outcome.
It explains what happened at the control-surface workflow boundary once richer
scene and action depth is in scope.

## Rules

### Rule 1: workflow meaning must stay runtime-owned

Scene-mapping posture, feedback-page posture, feedback-page class, safe action
graph posture, action authority, and safe action outcome belong to
runtime-owned receipts, not to controller-page assumptions, host-local scene
ledgers, or unsafe device scripts.

### Rule 2: workflow depth must compose with the closed control-surface seams

This contract widens the closed controller-expression, control-surface,
advanced-hardware, and advanced-feedback seams. It must not replace mapping
posture, feedback readiness, scripting-safe policy, or advanced feedback
outcome with a second workflow-private taxonomy.

### Rule 3: raw page and script detail stays advisory

Vendor page identifiers, strip-bank schemas, unsafe script bodies, device
macro payloads, and backend-native action dispatch rules may exist internally,
but the shared boundary must remain grounded in runtime-owned scene, page,
safe-action, authority, and outcome meaning.

### Rule 4: live workflow and observation surfaces must converge

Later controller workflow work may stage rollout, but it must not create one
scene or action graph model for live device delivery, another for observation,
and a third for supervisor export.

### Rule 5: product-local controller workflow stays out of scope

This contract freezes reusable runtime and adapter meaning only. It does not
freeze controller page editing UX, end-user scene authoring workflow, preset
management, or product-local surface choreography.

### Rule 6: unsafe device scripting is not implied

This contract may define guarded or portable action posture, but it does not
grant arbitrary executable device scripting or unsafe direct device control on
the shared Signal surface.

## Deferred scope

Batch 10.1 intentionally leaves these out:

- final runtime execution, delivery, and observation receipts for scene
  mapping, feedback pages, and safe action graphs
- public runtime, supervisor, and host-edge proof surfaces
- concrete vendor page-bank schemas, scene packet layouts, or unsafe macro
  payloads
- product-local controller page editing, workflow choreography, or setup UI

## Batch 10.1 outcome

Batch 10.1 freezes the first reusable control-surface workflow authority line
for Signal:

- scene-mapping posture, feedback-page posture, feedback-page class, safe
  action graph posture, action authority, and safe action outcome are now
  explicit Signal-owned vocabulary
- later runtime execution can widen from the closed controller-expression,
  control-surface, advanced-hardware, and advanced-feedback seams instead of
  inventing a parallel controller workflow shell
- unsupported or unsafe workflow paths are now required to explain themselves
  through bounded scene or action meaning rather than product-local controller
  UX or host-local scripting glue

## Next Task

Open `g08.011` with Batch 11.1 by freezing the first runtime-owned preview-
output routing, audition-sink ownership, and low-latency device-policy
contract on top of the closed controller and workflow seams.

## Batch 10.2 outcome

Batch 10.2 materializes the first runtime-owned control-surface workflow
receipts on the existing advanced-hardware seam:

- `signal-runtime` now exposes bounded scene-mapping posture, feedback-page
  posture and class, safe action graph posture, action authority, and safe
  action outcome on `RuntimeAdvancedHardwareDeviceDescriptor`
- `RuntimeAdvancedHardwareSnapshot` now carries aggregate workflow counts so
  downstream consumers can inspect reusable scene, page, and safe-action
  depth without rebuilding device policy from raw action classes
- the same workflow truth now flows through public runtime surfaces and stable
  local or server host-edge export without a host-local controller workflow
  shell

This still keeps vendor page banks, device-private scripts, and product-local
controller choreography advisory only, but it turns the bounded Signal-owned
workflow contract into typed runtime evidence that Batch 10.3 can now prove at
the supervisor boundary.

## Batch 10.3 outcome

Batch 10.3 closes the bounded consumer seam on top of the Batch 10.2 runtime
receipt family:

- the existing `signal.runtime.advanced-hardware-boundary` now points at this
  control-surface workflow contract instead of the narrower advanced-feedback
  contract alone
- the machine-readable supervisor boundary explicitly describes runtime-owned
  scene-mapping, feedback-page, and safe action graph counts plus bounded
  per-device posture, authority, and safe-action outcome
- the repo-owned acceptance lane continues to reuse the focused public runtime
  and stable host-edge proofs, but now closes the control-surface workflow
  seam without a second controller-workflow acceptance shell

This leaves vendor-private page banks, executable scripting depth, and richer
device choreography out of scope while making the bounded workflow seam fully
consumable through shared runtime, supervisor, and host-edge surfaces.
