# 045 Advanced Hardware Extensibility And Scripting-Safe Device Policy Contract

Status: complete
Owner: core-product
Updated: 2026-03-18
Related contracts: `docs/contracts/042-external-midi-endpoint-graph-and-device-identity-contract.md`, `docs/contracts/044-control-surface-transport-mapping-and-feedback-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first runtime-owned advanced hardware extensibility, scripting-safe
device policy, and guarded feedback boundary for `g07.014` so later device
extensibility work can deepen one shared Signal vocabulary instead of reopening
host-local scripting shells, backend-private hardware exceptions, or product-
specific controller policy.

## Authority hierarchy

Advanced hardware extensibility has one authority chain:

1. `docs/contracts/042-external-midi-endpoint-graph-and-device-identity-contract.md`
   remains the authority for bounded external MIDI device identity, endpoint
   graph, route, capability, and lifecycle meaning whenever advanced hardware
   access is MIDI-backed or MIDI-adjacent
2. `docs/contracts/044-control-surface-transport-mapping-and-feedback-contract.md`
   remains the authority for runtime-owned control-surface transport posture,
   mapping posture, feedback readiness, and bounded capability meaning
3. host/backend integration layers may own raw hardware evidence for:
   - device handles, attach or detach callbacks, and backend-native sessions
   - vendor packet dialects, display pages, LED protocols, haptic and motor
     channels, or backend-native extension APIs
   - low-level scheduling and delivery details for raw device traffic
4. `signal-runtime` must own the canonical consumer-visible meaning for:
   - advanced device capability classes and guarded extensibility posture
   - scripting-safe access policy and denied or guarded hardware operations
   - reusable feedback-channel policy that composes with the closed
     control-surface baseline
   - supervisor, observation, and stable host-edge export
5. future adapters or host crates may broker raw evidence into runtime-owned
   receipts, but they must not become the authority for:
   - arbitrary scripting execution against private host device tables
   - a second advanced-hardware taxonomy detached from runtime DTOs
   - product-local extension policy as the consumer boundary

If an advanced hardware claim cannot be explained through `042`, `044`, and
runtime-owned receipts, it is not yet part of the reusable Signal contract.

## Existing anchors

Batch 14.1 freezes this contract on top of the current shared hardware and
control-surface surface family:

- `RuntimeExternalMidiEndpointGraphSnapshot`
- `RuntimeControlSurfaceSnapshot`
- `RuntimeControlSurfaceCapabilitySummary`
- `RuntimeObservationReport`
- `RuntimeSupervisorReport`
- stable host-edge `supervisor_report()` export

Batch 14.1 does not claim those anchors already expose true scripting-safe
extensibility depth. It freezes how later DTOs and proofs must widen from them
instead of inventing a separate host-private hardware policy shell.

## Shared vocabulary

### Advanced hardware capability

`advanced hardware capability` means the bounded runtime-owned claim that a
device can participate in guarded non-baseline behavior such as display output,
motor or haptic feedback, paged surface feedback, macro-bank switching, or
other promoted device-class behaviors.

This capability is broader than the closed control-surface baseline, but it
must still compose through that baseline instead of replacing it.

### Scripting-safe device policy

`scripting-safe device policy` means the runtime-owned answer for which device
interactions may be requested through reusable extension surfaces and which
remain denied, guarded, or deferred.

This contract explicitly freezes scripting-safe policy as reusable Signal
meaning rather than product-local plugin, script, or host console behavior.

### Guarded feedback channel

`guarded feedback channel` means a runtime-owned feedback path whose existence
is acknowledged on the shared surface but whose exact hardware-native payload,
layout, or delivery semantics remain device- or backend-specific.

Guarded feedback channels are explicit capability and policy outcomes, not a
promise that every raw vendor feedback surface is already portable.

### Extension posture

`extension posture` means the runtime-owned claim about whether advanced device
interaction is:

- portable on the shared Signal surface
- guarded behind runtime policy and capability checks
- context-only and visible for diagnosis but not user-extensible
- denied on the shared Signal surface
- unsupported until later promotion

### Device action class

`device action class` means the bounded reusable category for advanced device
operations such as:

- guarded feedback emission
- display or text output
- motor or haptic output
- bank or page navigation
- macro or scene recall triggers
- device-state observation

Device action classes must remain typed and runtime-owned.

## Rules

### Rule 1: advanced hardware work must widen from the closed control-surface baseline

`g07.014` must widen from the closed external MIDI and control-surface
contracts. It must not create a second controller or hardware device shell.

### Rule 2: scripting-safe policy must stay runtime-owned

Host integrations may broker raw hardware evidence, but reusable extension and
device-policy meaning must remain canonical on shared runtime receipts.

### Rule 3: no arbitrary device scripting is implied

This contract may define portable, guarded, context-only, denied, or
unsupported posture, but it does not grant arbitrary runtime scripting
execution against raw hardware channels.

### Rule 4: guarded feedback must stay explicit

If advanced feedback is device-specific, partial, or unavailable, the shared
answer must remain explicit through runtime-owned typed receipts instead of
host-local heuristics or silent fallback.

### Rule 5: product workflow and UI remain out of scope

This contract may freeze reusable device policy and action classes, but it must
not absorb product-local scripting editors, controller-setup UI, preset
management, or user workflow semantics.

### Rule 6: later extensibility work must widen from this policy boundary

Future advanced hardware, guarded scripting, or richer feedback milestones must
reuse this boundary instead of reopening hardware access ownership.

## Deferred scope

Batch 14.1 intentionally does not claim:

- runtime realization of advanced device capability receipts yet
- arbitrary device scripting or user-authored executable extension logic
- exhaustive vendor protocol parity
- display-layout, animation, motor, or haptic composition semantics
- product-local controller setup or scripting UI
- broad remote-control or network-device orchestration

Those belong to later `g07.014` batches and follow-on acceptance work.

## Batch 14.1 outcome

Batch 14.1 freezes the first bounded advanced hardware extensibility contract:

- Signal now has one explicit runtime-owned target for advanced device
  capability classes, scripting-safe policy posture, guarded feedback
  channels, and typed device action classes instead of host-local extension
  logic
- external MIDI endpoint and control-surface meaning remain the anchors, which
  prevents later advanced hardware work from reopening a second hardware shell
  or bypassing runtime-owned controller policy
- Batch 14.2 can now materialize the first credible advanced hardware runtime
  depth instead of reopening what extensibility and scripting-safe semantics
  belong to Signal

## Batch 14.2 outcome

Batch 14.2 materializes the first bounded runtime-owned advanced-hardware
receipt family on top of the closed external MIDI and control-surface
substrate:

- `signal-runtime` now owns advanced-hardware graph state, scripting-safe
  device-policy posture, guarded feedback-channel posture, and typed action
  classes instead of leaving those outcomes implicit in host-local controller
  logic
- observation, supervisor, and stable host-edge surfaces now carry the same
  advanced-hardware snapshot family, including explicit `Unavailable`, `Empty`,
  `Guarded`, and `Ready` outcomes
- the baseline remains deliberately bounded: guarded display feedback and
  navigation semantics are runtime-visible now, while richer vendor protocol,
  motor, haptic, and executable scripting depth remain deferred

Batch 14.3 can now prove the widened receipt family stays consumable through
shared runtime, supervisor, and stable host-edge surfaces without reopening
host-local hardware ownership.

## Batch 14.3 outcome

Batch 14.3 closes the bounded advanced-hardware proof seam:

- public runtime now proves `RuntimeAdvancedHardwareSnapshot` remains
  consumable through shared runtime reports without host-local hardware or
  controller-policy reconstruction
- both stable host edges now prove they forward the same advanced-hardware
  graph state, scripting-safe device policy posture, guarded feedback-channel
  posture, and typed action-class truth
- `signal-supervisor-tools` now exposes
  `signal.runtime.advanced-hardware-boundary`, and Effigy now owns
  `acceptance:advanced-hardware-boundary` as the repo-owned rerun lane

This closes `g07.014` as the bounded advanced-hardware extensibility and
scripting-safe device-policy contract. Richer vendor protocol, display,
motor, haptic, and executable scripting depth remain later work.

## Next Task

Continue `g07.015` with Batch 15.1 by freezing the sample-domain
time-stretch engine contract on top of the closed media, analysis, and
routing surfaces before runtime stretch realization widens.
