# 052 Live Linux Audio Backend Ownership And Session Lifecycle Contract

Status: active
Owner: core-product
Updated: 2026-03-19
Related contracts: `docs/contracts/040-linux-audio-backend-portability-across-alsa-jack-and-pipewire-contract.md`, `docs/contracts/041-linux-backend-clocking-duplex-and-endpoint-topology-parity-contract.md`, `docs/contracts/025-device-supervision-restart-state-machine-and-fault-boundary-contract.md`, `docs/contracts/027-external-io-monitoring-tap-point-and-loopback-measurement-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first live Linux audio backend ownership boundary for `g08.001` so
later ALSA, JACK, and PipeWire runtime work can deepen one shared session and
device-lifecycle vocabulary without reopening backend-private daemon policy,
server-local ownership stories, or host-only reconnect heuristics.

## Authority hierarchy

Live Linux backend ownership has one authority chain:

1. `040` remains the authority for Linux backend identity, portability band,
   guarded fallback posture, and bounded backend-family classification
2. `041` remains the authority for Linux-facing clocking, duplex, and
   endpoint-topology parity meaning
3. `025` remains the authority for recovering, exhausted, and faulted hardware
   outcomes when a live backend session crosses from guarded ownership into
   restart or failure state
4. `signal-hardware` owns backend-neutral live hardware evidence for:
   - device claim intent and negotiated stream ownership
   - session lifecycle state and ownership handoff
   - backend-managed versus host-driven callback or graph attachment posture
   - backend diagnostics that may later contribute to restart or loss receipts
5. `signal-runtime` owns the canonical live Linux interpretation for:
   - active backend session ownership
   - stream attachment, start, running, interruption, release, and reacquire posture
   - bounded session role and ownership fallback state
   - observation, supervisor, and stable host-edge export delivery
6. host crates may broker ALSA handles, JACK graph callbacks, PipeWire node or
   stream events, and reconnect attempts into runtime-owned receipts, but they
   must not become the authority for:
   - competing backend-specific session lifecycle taxonomies
   - host-private device-claim or stream-ownership truth
   - Linux-only attach or reconnect matrices outside shared runtime receipts

If a live Linux backend ownership claim cannot be explained through `040`,
`041`, `025`, `signal-hardware`, and runtime-owned receipts, it is not yet
part of the reusable Signal contract.

## Existing anchors

This contract builds on the current shared hardware and runtime surface family:

- `HardwareBackendIdentity`
- `LinuxAudioBackendKind`
- `HardwareLifecycleOwnership`
- `HardwareLifecycleContract`
- `AudioDeviceDescriptor`
- `HardwareDiagnosticsSnapshot`
- `HardwareStreamConfig`
- `RuntimeHostHardwareSummary`
- `RuntimeHostClockingSummary`
- `RuntimeHostIoSummary`
- `RuntimeExternalIoSnapshot`
- `RuntimeSupervisionSnapshot`
- `RuntimeFaultStatusSnapshot`
- `RuntimeInterruptionSummary`
- `RuntimeDegradationSummary`

Batch 1.1 does not claim those anchors already provide live ALSA, JACK, or
PipeWire ownership. It freezes how later DTOs and proofs must deepen from this
shared surface family instead of inventing a second Linux-only session shell.

## Shared vocabulary

### Live backend ownership

`live backend ownership` means the runtime-owned answer to which side currently
owns an active Linux audio session:

- Signal runtime
- a host-brokered callback path
- a backend-managed graph or daemon path
- an explicitly unavailable or released session

Ownership must land through shared runtime receipts. Consumers must not infer
it from backend crate names, daemon handles, thread models, or host-private
state machines.

### Session lifecycle

`session lifecycle` means the bounded runtime-owned progression of a live Linux
backend path through:

- prepared or claimable
- attached
- running
- interrupted or detached
- recovering
- released
- unavailable

These are shared ownership states, not backend-native daemon stories. ALSA,
JACK, and PipeWire may realize them differently, but the consumer answer must
stay on one shared runtime seam.

### Device claim posture

`device claim posture` means whether Signal has:

- not yet claimed a backend path
- claimed it exclusively or directly
- attached to a backend-managed shared graph
- lost or released the claim

This contract does not yet freeze every backend-native claim mode. It freezes
one shared consumer answer for whether the live backend session is runtime-owned,
backend-managed, or explicitly unavailable.

### Session role

`session role` means the bounded shared interpretation of why a live Linux
backend session exists:

- primary runtime audio I/O
- monitoring or preview-capable attachment
- offline-unavailable
- fallback or degraded continuation

Session role is additive over the existing external-I/O and monitoring
contracts. It must not become a second Linux-only routing taxonomy.

### Ownership fallback

`ownership fallback` means a live Linux backend is present, but the current
Signal-owned answer is guarded because:

- backend-managed graph ownership limits direct runtime control
- attach or reacquire is in progress
- the session is running in degraded or recovering mode
- direct live ownership is not available on the current host path

Guarded ownership must stay typed on shared runtime receipts rather than
disappearing into host-local reconnect or daemon logic.

## Live Linux backend matrix

Batch 1.1 freezes the first bounded live ownership matrix.

| Capability family | ALSA | JACK | PipeWire | Notes |
| --- | --- | --- | --- | --- |
| Backend identity and portability band | portable | portable | portable | Reused from `040` |
| Live session ownership answer through shared runtime receipts | guarded | guarded | guarded | Live ownership is frozen before concrete runtime realization exists |
| Session lifecycle and reacquire posture | guarded | guarded | guarded | Must compose with `025` rather than backend-private reconnect logic |
| Session role and ownership fallback | guarded | guarded | guarded | Must reuse shared external-I/O meaning instead of Linux-only route shells |
| Backend-native daemon, graph, or node detail | private | private | private | Still backend-private until later promotion |
| Non-Linux ownership claims | unsupported | unsupported | unsupported | Outside this queue |

The matrix is intentionally guarded-first. Batch 1.1 freezes one live
ownership target before later runtime work proves how much of that depth is
already realized.

## Rules

### Rule 1: live ownership layers on top of portability, not around it

`040` owns backend identity and guarded portability posture. This contract adds
live session ownership on top of that identity rather than replacing it with a
new Linux-only lifecycle shell.

### Rule 2: session lifecycle stays runtime-owned

ALSA callbacks, JACK graph attachment, and PipeWire stream or node events may
inform the answer, but the canonical session lifecycle must remain on one
runtime-owned receipt family.

### Rule 3: restart and failure compose through the closed supervision seam

If live backend ownership crosses into recovering, exhausted, or faulted
hardware state, the answer must still compose through `025` instead of a
backend-native reconnect or daemon incident taxonomy.

### Rule 4: backend-native detail stays advisory until promoted

The following remain backend-private until later batches promote them:

- ALSA card, PCM, and reservation detail
- JACK client, port graph, transport, or callback-thread detail
- PipeWire node, stream, session-manager, or portal detail
- distro-specific daemon startup, policy, or packaging behavior

Consumers must not depend on those details for shared live ownership claims.

### Rule 5: unavailable or guarded ownership must stay typed

If Signal cannot directly own a live ALSA, JACK, or PipeWire session, that
answer must land through shared runtime-owned guarded or unavailable receipts,
not missing fields or host-local Linux capability tables.

## Deferred scope

Batch 1.1 intentionally does not claim:

- full live ALSA, JACK, or PipeWire runtime realization
- backend-native graph, node, or transport coordination depth
- distro certification or packaging guarantees
- user-facing Linux device setup, repair, or browser UX
- external MIDI, control-surface, or preview-device ownership breadth

Those belong to later `g08` Linux, device, and workflow milestones.

## Batch 1.1 outcome

Batch 1.1 freezes the bounded live Linux backend ownership contract:

- ALSA, JACK, and PipeWire now share one explicit Signal-owned session and
  ownership vocabulary instead of implicit future backend-private behavior
- live attach, running, recovery, release, and unavailable posture are now
  explicitly required to compose through shared hardware, supervision, and
  external-I/O receipts
- Batch 1.2 can now materialize runtime-owned live backend ownership receipts
  against one bounded contract instead of reopening Linux session authority

## Batch 1.2 outcome

Batch 1.2 materializes the first shared receipt family for this contract:

- `signal-runtime` now owns `RuntimeLinuxBackendSessionSnapshot` and the
  bounded ownership, lifecycle, device-claim, session-role, and
  ownership-fallback enums that define the reusable live Linux answer
- the first derivation path composes from `RuntimeHostIoSummary` rather than
  backend-native daemon or reconnect state, keeping the authority line inside
  shared runtime-owned receipts
- non-Linux hosts now answer this seam explicitly as `NotLinux`
- the stable server host edge now exports a bounded simulated PipeWire
  backend-managed live-session baseline instead of leaving Linux ownership
  completely absent

This still does not claim full live ALSA, JACK, or PipeWire ownership depth.
It freezes and realizes the first reusable DTO family that later `g08` Linux
milestones must deepen.

## Batch 1.3 outcome

Batch 1.3 closes the public consumer seam for this contract:

- public runtime now proves live Linux session ownership, lifecycle,
  device-claim, role, and guarded fallback through shared observation and
  supervisor surfaces
- the stable local host edge proves non-Linux hosts answer this contract
  explicitly as `NotLinux`
- the stable server host edge proves the bounded PipeWire-style live-session
  baseline remains runtime-owned instead of collapsing into server-local Linux
  ownership logic
- `signal-supervisor-tools` and Effigy now expose one
  `linux-live-ownership-boundary` descriptor and acceptance task so downstream
  consumers can verify this seam without reading backend-private host code

This closes the bounded `g08.001` ownership proof seam. It does not yet claim
real ALSA, JACK, or PipeWire daemon coordination depth, transport integration,
or backend-native recovery behavior.

## Next Task

Continue `g08.002` with Batch 2.2 by materializing the first runtime-owned
JACK transport, graph, client-role, and guarded-coordination receipt family
across runtime, supervision, and stable host-edge surfaces.
