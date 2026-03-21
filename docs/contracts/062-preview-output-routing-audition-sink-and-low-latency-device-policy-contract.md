# 062 Preview-Output Routing, Audition Sink, And Low-Latency Device Policy Contract

Status: active
Owner: core-product
Updated: 2026-03-21
Related contracts: `docs/contracts/027-external-io-monitoring-tap-point-and-loopback-measurement-contract.md`, `docs/contracts/028-media-indexing-waveform-analysis-and-preview-service-contract.md`, `docs/contracts/044-control-surface-transport-mapping-and-feedback-contract.md`, `docs/contracts/045-advanced-hardware-extensibility-and-scripting-safe-device-policy-contract.md`, `docs/contracts/049-low-latency-audition-scrub-and-preview-transform-service-contract.md`, `docs/contracts/061-control-surface-scene-mapping-feedback-pages-and-safe-action-graph-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first runtime-owned preview-output routing, audition-sink
ownership, and low-latency device-policy boundary so later preview-browser,
audition, and workflow depth widens one shared Signal contract instead of
reopening browser-local preview buses, host-local device picks, or app-local
audition shells as the authority.

## Authority hierarchy

Preview-output routing, audition-sink ownership, and low-latency device policy
have one authority chain:

1. `docs/contracts/028-media-indexing-waveform-analysis-and-preview-service-contract.md`
   remains the authority for:
   - media identity, preview readiness, waveform readiness, and analysis-ready
     service meaning
   - the rule that preview playback claims must stay grounded in runtime-owned
     media-service truth
2. `docs/contracts/049-low-latency-audition-scrub-and-preview-transform-service-contract.md`
   remains the authority for:
   - low-latency audition and scrub preview scope
   - preview-transform service class, readiness, degraded state, fallback, and
     artifact alignment
   - the rule that later preview routing must widen from one shared preview
     vocabulary instead of inventing a second preview engine
3. `docs/contracts/027-external-io-monitoring-tap-point-and-loopback-measurement-contract.md`
   remains the authority for:
   - external-I/O role naming, monitoring-path meaning, tap-point truth, and
     loopback posture
   - the rule that device-facing path identity and monitor-path semantics must
     stay runtime-owned instead of host-local routing prose
4. `docs/contracts/044-control-surface-transport-mapping-and-feedback-contract.md`,
   `docs/contracts/045-advanced-hardware-extensibility-and-scripting-safe-device-policy-contract.md`,
   and `docs/contracts/061-control-surface-scene-mapping-feedback-pages-and-safe-action-graph-contract.md`
   remain the authority for:
   - transport-triggered preview or audition requests that originate from
     shared controller and workflow seams
   - the rule that low-latency device policy must compose with closed
     controller and advanced-hardware truth instead of replacing it
5. `signal-runtime` must own the canonical consumer-visible meaning for:
   - preview-output routing posture
   - audition-sink ownership and bounded sink class
   - low-latency device-policy class, authority, and fallback outcome
   - observation, supervisor, render-preview, and stable host-edge export
6. host crates may broker raw device identities, negotiated outputs, and
   bounded sink evidence into runtime-owned receipts, but they must not become
   the authority for:
   - a second preview-route or audition-sink taxonomy
   - host-local device-pick policy as the consumer boundary
   - browser-local preview buses, cue paths, or app-local audition shells

If a preview-output or audition-sink claim cannot be explained through the
closed preview-transform, media-service, external-I/O, controller, and
advanced-hardware seams plus runtime-owned receipts, it is not yet part of the
shared Signal contract.

## Existing anchors

Batch 11.1 freezes this contract on top of the currently closed preview,
external-I/O, and controller-workflow seams:

- `RuntimePreviewTransformServiceSnapshot`
- `RuntimeMediaServiceSnapshot`
- `RuntimeExternalIoSnapshot`
- `RuntimeControlSurfaceSnapshot`
- `RuntimeAdvancedHardwareSnapshot`
- `RuntimeObservationReport`
- `RuntimeSupervisorReport`
- `RuntimeOfflineRenderContractPreview`
- stable host-edge `supervisor_report()` export

Batch 11.1 does not claim these anchors already expose realized preview-device
routing or audition-sink ownership. It freezes how later DTOs and proofs must
widen from them instead of inventing a separate host-private or browser-local
preview routing model.

## Shared vocabulary

### Preview-output routing

`preview-output routing` means the runtime-owned bounded answer for where a
preview or audition request is intended to leave Signal relative to the
currently available program, monitor, external-output, or guarded preview
paths.

This is not a browser-local player route, not a host-private output picker,
and not a product-specific cue-bus taxonomy.

### Audition sink

`audition sink` means the runtime-owned consumer-visible sink Signal is using,
or would use, for bounded preview playback.

Batch 11.1 freezes only bounded sink meaning such as:

- program-aligned preview
- monitor-aligned preview
- guarded dedicated preview sink
- unavailable or unresolved preview sink

It does not freeze full hardware endpoint UI, user presets, or product-local
cue workflow.

### Audition-sink ownership

`audition-sink ownership` means where the currently active sink decision is
allowed to come from.

Batch 11.1 freezes the ownership line conceptually as:

- runtime default
- runtime declared
- host forwarded
- device advisory

The shared boundary must answer this through runtime-owned meaning, even if a
host brokers raw evidence.

### Low-latency device policy

`low-latency device policy` means the runtime-owned bounded answer for how
Signal is allowed to prefer, preserve, guard, or degrade device-facing preview
delivery when low-latency audition is in scope.

This policy is distinct from preview-transform readiness. It explains how the
currently available preview route and sink are being constrained at the device
boundary.

### Device-policy class

`device-policy class` means the bounded category of device-facing preview
policy Signal is applying.

Batch 11.1 freezes the concept, not final implementation breadth, around:

- no dedicated preview-device policy
- program-aligned policy
- monitor-aligned policy
- guarded low-latency policy
- unavailable preview-device policy

### Device-policy outcome

`device-policy outcome` means the runtime-owned result when low-latency
preview intent is projected onto the currently available preview route, sink,
and external-I/O state.

This outcome is separate from preview-transform fallback. It answers what
happened at the device boundary once preview delivery left the transform
service seam.

## Rules

### Rule 1: preview route and sink meaning must stay runtime-owned

Hosts, browsers, and products must not define their own route, sink, or
device-policy taxonomy for shared consumers.

### Rule 2: preview routing must compose with preview-transform and external-I/O truth

Later preview routing work must widen from the closed preview-transform and
external-I/O seams. It must not invent a second low-latency preview engine or
a host-private monitor-path model.

### Rule 3: controller-triggered preview depth must remain additive

If preview or audition requests originate from control-surface workflow, they
must still reduce to one shared preview-route, sink-ownership, and device-
policy answer instead of opening a controller-private preview shell.

### Rule 4: host-local device selection stays advisory

Hosts may provide raw negotiated outputs, bounded preferred devices, or
availability evidence, but the shared consumer-facing sink and policy answers
must stay typed and runtime-owned.

### Rule 5: browser and product-local workflow stay out of scope

This contract freezes reusable runtime and adapter meaning only. It does not
freeze browser routing UX, preview-panel design, cue workflow, or end-user
device picker semantics.

## Deferred scope

Batch 11.1 intentionally leaves these out:

- realized runtime receipts for preview-route, sink, and device-policy depth
- public runtime, supervisor, and host-edge proof surfaces
- full preview-browser queueing or remote audition transport
- user-facing device selection, sink presets, or per-product routing UX
- richer hardware-cue or monitor-mix policy beyond bounded sink classes

## Batch 11.1 outcome

Batch 11.1 freezes the first reusable preview-device contract for Signal:

- preview-output routing, audition-sink ownership, and low-latency device
  policy now have one explicit Signal-owned authority line
- later runtime realization is forced to compose with the closed preview-
  transform, media-service, external-I/O, controller, and advanced-hardware
  seams instead of reopening host-local output picks or browser-local preview
  buses
- Batch 11.2 can now focus on materializing the first bounded receipt family
  instead of reopening which preview-device semantics belong to Signal

## Next Task

Continue `g08.011` with Batch 11.2 by materializing the first runtime-owned
preview-output routing, audition-sink ownership, and low-latency device-policy
receipts, then align stable host-edge export to the same bounded model.
