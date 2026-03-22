# 065 Live External MIDI Device Ownership And Backend Parity Contract

Status: complete
Owner: core-product
Updated: 2026-03-22
Related contracts: `docs/contracts/042-external-midi-endpoint-graph-and-device-identity-contract.md`, `docs/contracts/043-midi-2-0-mpe-and-richer-controller-expression-contract.md`, `docs/contracts/052-live-linux-audio-backend-ownership-and-session-lifecycle-contract.md`, `docs/contracts/054-pipewire-and-alsa-session-role-device-claim-and-stream-policy-parity-contract.md`, `docs/contracts/064-asset-session-transform-persistence-retention-and-cache-placement-policy-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first runtime-owned live external MIDI device ownership and backend
parity boundary for `g08.014` so later live endpoint work can deepen one shared
Signal contract for ownership, attach state, backend parity, and guarded
continuity instead of reopening host-local device picks, backend-native port
policy, or product-local live MIDI workflow shells.

## Authority hierarchy

Live external MIDI device ownership and backend parity have one authority
chain:

1. `docs/contracts/042-external-midi-endpoint-graph-and-device-identity-contract.md`
   remains the authority for:
   - external MIDI device identity
   - endpoint identity, graph membership, capability, and route meaning
   - the rule that later live ownership work must widen one shared external
     MIDI graph instead of inventing a second endpoint shell
2. `docs/contracts/043-midi-2-0-mpe-and-richer-controller-expression-contract.md`
   remains the authority for:
   - richer controller-expression meaning
   - widened endpoint capability relevance once live devices participate in
     expressive transport
3. `docs/contracts/052-live-linux-audio-backend-ownership-and-session-lifecycle-contract.md`
   remains the authority for:
   - live backend ownership posture, guarded continuity, restart, and terminal
     runtime boundaries
   - the rule that live MIDI ownership must compose with one shared live
     runtime lifecycle instead of a device-specific reconnect shell
4. `docs/contracts/054-pipewire-and-alsa-session-role-device-claim-and-stream-policy-parity-contract.md`
   remains the authority for:
   - backend parity posture and guarded parity rules
   - the rule that backend differences must collapse into typed parity answers
     instead of leaking backend-native policy into consumers
5. `docs/contracts/064-asset-session-transform-persistence-retention-and-cache-placement-policy-contract.md`
   remains the authority for:
   - bounded persistence, retention, and cache-placement ownership
   - the rule that later live device continuity hints stay additive on top of
     runtime-owned persistence policy instead of opening a second session
     ledger
6. `signal-runtime` must own the canonical consumer-visible meaning for:
   - live external MIDI ownership posture
   - attach and detach continuity class
   - backend parity class and guarded parity outcome
   - observation, supervisor, and stable host-edge export
7. backend adapters and host crates may broker raw transport evidence for:
   - backend-native client IDs, port handles, subscribe state, and hotplug
     notifications
   - backend-local ownership or reservation evidence
   - attach, detach, or reconnect callbacks
   but they must not become the authority for:
   - a second live external MIDI ownership taxonomy
   - backend-private parity meaning as the consumer boundary
   - host-local device picker or session-manager policy as shared truth

If a live external MIDI ownership or parity claim cannot be explained through
`042`, `043`, `052`, `054`, `064`, and runtime-owned receipts, it is not yet
part of the shared Signal contract.

## Existing anchors

Batch 14.1 freezes this contract on top of the current shared runtime, host,
and device surface family:

- `RuntimeExternalMidiEndpointGraphSnapshot`
- `RuntimeObservationReport`
- `RuntimeSupervisorReport`
- `RuntimeInterruptionSummary`
- `RuntimeHostHardwareSummary`
- `RuntimeHostIoSummary`
- stable host-edge `supervisor_report()` export

Batch 14.1 does not claim these anchors already expose realized live ownership
or backend-parity receipts. It freezes how later DTOs, receipts, and proofs
must widen from this shared surface family instead of inventing a second
backend-private or host-private live MIDI ownership shell.

## Shared vocabulary

### Live external MIDI ownership

`live external MIDI ownership` means the runtime-owned bounded answer for
whether Signal currently holds, expects, or has lost the live relationship to
one external MIDI device or endpoint set that matters to active runtime
behavior.

This is not a backend-native reservation handle, not a host-local device pick,
and not a product-local live setup record.

### Ownership posture

`ownership posture` means the bounded category of live external MIDI ownership
behavior Signal is currently using.

Batch 14.1 freezes the concept, not final implementation breadth, around:

- no live external MIDI ownership
- runtime-declared live ownership
- guarded live ownership
- backend-advisory live ownership
- unavailable live ownership

### Attach continuity

`attach continuity` means the runtime-owned answer for whether live external
MIDI ownership is attached, interrupted but resumable, restarted but
recoverable, or terminal from the consumer point of view.

This must compose with the closed interruption and live-backend lifecycle
seams instead of opening a device-only reconnect model.

### Backend parity

`backend parity` means the bounded runtime-owned answer for whether the active
live external MIDI ownership semantics are equivalent, guarded, or unavailable
across the backend families Signal currently supports.

This is not a promise that every backend has identical native APIs. It is the
typed consumer-facing collapse of those differences.

### Guarded parity outcome

`guarded parity outcome` means the runtime-owned result when live ownership is
projected through backend-specific capability or lifecycle limits.

This outcome must remain explicit and typed rather than inferred from missing
fields, host logs, or backend-native warnings.

## Rules

### Rule 1: live ownership meaning must stay runtime-owned

Hosts, adapters, and products must not define their own reusable live
external MIDI ownership taxonomy for shared consumers.

### Rule 2: endpoint identity stays anchored in the closed external MIDI graph

Later live ownership work must widen from `042`. It must not introduce a
second device or endpoint identity shell detached from the existing runtime
graph.

### Rule 3: backend parity stays additive on the closed live backend seams

Backend-specific attach or reservation detail may inform classification, but
shared parity meaning must collapse through typed runtime-owned answers rather
than backend-native policy tables.

### Rule 4: continuity must compose with shared interruption and lifecycle

Attach, detach, guarded rebind, restart, and terminal loss answers must align
with the closed runtime interruption and live backend lifecycle model instead
of creating a MIDI-only recovery shell.

### Rule 5: host and backend detail stay advisory

Backend-native client IDs, subscriptions, session-manager evidence, and
host-local device-pick hints may feed runtime classification later, but shared
consumers must not depend on them for stable live ownership truth.

### Rule 6: product-local workflow depth stays out of scope

This contract freezes reusable runtime and adapter meaning only. It does not
freeze controller workflow UX, MIDI device browser UX, live performance scene
policy, or backend-specific session-manager scripting.

## Deferred scope

Batch 14.1 intentionally leaves these out:

- concrete runtime DTOs for live external MIDI ownership and backend parity
- public runtime, supervisor, and host-edge proof surfaces
- richer live MIDI session-manager or reservation depth
- product-local device browser, mapping, or rehearsal workflow
- deeper remote collaboration, cloud handoff, or multi-user endpoint policy

## Batch 14.1 outcome

Batch 14.1 freezes the first reusable live external MIDI ownership and backend
parity contract for Signal:

- live external MIDI ownership, attach continuity, and backend parity now have
  one explicit Signal-owned authority line
- later runtime realization is forced to compose with the closed external MIDI
  graph, controller-expression, live backend lifecycle, backend parity, and
  transform-persistence seams instead of reopening host-local device picks or
  backend-private policy
- Batch 14.2 can now focus on materializing the first bounded receipt family
  instead of reopening what live external MIDI ownership or parity means

## Batch 14.2 outcome

Batch 14.2 materializes the first runtime-owned live external MIDI ownership
and backend-parity receipt family on the existing external MIDI seam:

- `signal-runtime` now exposes bounded live ownership posture, attach
  continuity, backend parity, and guarded parity outcome on
  `RuntimeExternalMidiEndpointGraphSnapshot`
- the same live ownership and parity truth now flows through public runtime
  surfaces and stable local or server host-edge export without a backend-local
  endpoint policy shell
- the widened receipt family stays additive on top of the closed external MIDI
  graph, live backend lifecycle, backend-parity, and transform-persistence
  contracts instead of opening a second live-MIDI report model

## Batch 14.3 outcome

Batch 14.3 closes the bounded live external MIDI consumer seam by widening
the existing shared external MIDI boundary instead of opening a second
live-MIDI-only acceptance lane:

- `signal-supervisor-tools` now points
  `signal.runtime.external-midi-boundary` at this `065` contract instead of
  the older `042` contract
- the machine-readable boundary explicitly describes `live_ownership`,
  `ownership_posture`, `attach_continuity`, `backend_parity`, and
  `guarded_parity_outcome` alongside the earlier external MIDI graph anchors
- the repo-owned proof path remains
  `effigy acceptance:external-midi-boundary`, so runtime, supervisor, and
  both stable host edges continue to close on one shared seam without a
  backend-local endpoint policy shell

## Next Task

Continue `g08.015` with Batch 15.1 by freezing the shared cross-backend
device protocol and live workflow acceptance contract on top of the closed
live external MIDI ownership seam.
