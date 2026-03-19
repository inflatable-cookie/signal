# 053 JACK Transport, Graph, And Backend-Native Coordination Contract

Status: complete
Owner: core-product
Updated: 2026-03-19
Related contracts: `docs/contracts/052-live-linux-audio-backend-ownership-and-session-lifecycle-contract.md`, `docs/contracts/040-linux-audio-backend-portability-across-alsa-jack-and-pipewire-contract.md`, `docs/contracts/041-linux-backend-clocking-duplex-and-endpoint-topology-parity-contract.md`, `docs/contracts/025-device-supervision-restart-state-machine-and-fault-boundary-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first JACK-specific coordination boundary for `g08.002` so later
runtime work can deepen JACK transport, graph attachment, and backend-native
coordination on one shared runtime-owned seam instead of reopening host-private
client, callback, or daemon policy.

## Authority hierarchy

JACK coordination has one authority chain:

1. `052` remains the authority for live Linux backend ownership, session role,
   device-claim posture, and guarded ownership fallback
2. `040` remains the authority for Linux backend identity and portability-band
   meaning
3. `041` remains the authority for Linux-facing clocking, duplex, and
   endpoint-topology parity meaning
4. `025` remains the authority for recovering, exhausted, and faulted hardware
   outcomes when JACK coordination crosses from bounded runtime ownership into
   restart or failure state
5. `signal-hardware` may own backend-neutral live evidence for:
   - client attachment and graph-presence evidence
   - transport availability and observed transport state
   - callback-path or graph-managed execution evidence
   - bounded restart and detach evidence that later contributes to runtime
     coordination receipts
6. `signal-runtime` owns the canonical JACK interpretation for:
   - transport posture
   - graph attachment and connection coordination posture
   - client role and bounded transport participation
   - observation, supervisor, and stable host-edge export delivery
7. host crates may broker JACK client creation, port graph events, callback
   lifecycle, and transport notifications into runtime-owned receipts, but they
   must not become the authority for:
   - competing JACK-only transport taxonomies
   - host-private graph attachment truth
   - daemon-local reconnect or session-manager policy exposed as shared
     consumer meaning

If a JACK transport or graph claim cannot be explained through `052`, `040`,
`041`, `025`, `signal-hardware`, and runtime-owned receipts, it is not yet
part of the reusable Signal contract.

## Existing anchors

This contract builds on the current shared Linux and hardware surface family:

- `HardwareBackendIdentity`
- `LinuxAudioBackendKind`
- `HardwareLifecycleOwnership`
- `HardwareDiagnosticsSnapshot`
- `RuntimeHostHardwareSummary`
- `RuntimeHostClockingSummary`
- `RuntimeHostIoSummary`
- `RuntimeExternalIoSnapshot`
- `RuntimeLinuxBackendSessionSnapshot`
- `RuntimeSupervisionSnapshot`
- `RuntimeFaultStatusSnapshot`
- `RuntimeInterruptionSummary`

Batch 2.1 does not claim those anchors already provide JACK-native transport or
graph coordination. It freezes how later DTOs and proofs must deepen from this
shared surface family instead of inventing a second JACK-only control shell.

## Shared vocabulary

### JACK transport posture

`JACK transport posture` means the bounded runtime-owned answer for whether the
current JACK session is:

- transport-unavailable
- detached from transport
- following an external JACK transport timeline
- leading or owning bounded transport progression
- guarded because JACK transport exists but runtime control is constrained

Consumers must not infer this from raw JACK callback state, daemon handles, or
host-private transport wrappers.

### JACK graph coordination

`JACK graph coordination` means the bounded runtime-owned interpretation of how
Signal participates in the JACK graph:

- not attached
- attached with stable graph membership
- attached with guarded connection or port-shape negotiation
- interrupted or recovering while graph presence is retained or reacquired
- released or unavailable

This is shared coordination meaning, not a direct export of JACK daemon
objects, client handles, or callback registration details.

### JACK client role

`JACK client role` means the bounded shared interpretation of what the active
Signal JACK client is doing:

- primary live audio I/O participant
- monitoring-capable or preview-capable participant
- transport follower
- fallback continuation path
- unavailable or non-JACK

Client role is additive over the closed live Linux ownership seam. It must not
become a second Linux-only routing taxonomy.

### Backend-native guarded coordination

`backend-native guarded coordination` means JACK coordination exists, but the
current Signal-owned answer is constrained because:

- transport is present but not runtime-led
- graph attachment is backend-managed or externally coordinated
- callback or graph recovery is in progress
- Signal can observe coordination posture but not claim direct control

Guarded coordination must stay typed on shared runtime receipts rather than
disappearing into host-local JACK callback policy.

## Bounded JACK coordination matrix

Batch 2.1 freezes the first bounded JACK-native coordination matrix.

| Capability family | JACK baseline | Notes |
| --- | --- | --- |
| Backend identity and live ownership posture | reused | Reused from `040` and `052` |
| Transport posture through shared runtime receipts | guarded | Frozen here before concrete runtime realization exists |
| Graph attachment and connection coordination posture | guarded | Must stay runtime-owned instead of host-private callback policy |
| Client role and guarded transport participation | guarded | Reuses shared session-role meaning instead of a JACK-only shell |
| Callback-thread, session-manager, and daemon detail | private | Still backend-private until later promotion |
| Non-JACK transport claims | unsupported | Outside this milestone |

The matrix is intentionally guarded-first. Batch 2.1 freezes one JACK
coordination target before later runtime work proves how much of that depth is
already realized.

## Rules

### Rule 1: JACK coordination layers on top of live ownership

`052` owns whether Signal has a live Linux session and what bounded ownership
posture it is in. This contract adds JACK transport and graph coordination on
top of that ownership instead of creating a competing JACK-only lifecycle shell.

### Rule 2: transport and graph answers stay runtime-owned

JACK callbacks, graph notifications, and transport events may inform the
answer, but the canonical transport and graph coordination state must remain on
one runtime-owned receipt family.

### Rule 3: restart and loss still compose through supervision

If JACK graph or transport coordination crosses into recovering, exhausted, or
faulted hardware state, the answer must still compose through `025` instead of
growing a JACK-private restart taxonomy.

### Rule 4: backend-native detail stays advisory until promoted

The following remain JACK-private until later batches promote them:

- raw JACK client names and daemon registration policy
- port IDs, connection lists, and session-manager objects
- transport callback sequencing and callback-thread detail
- daemon startup, packaging, and distro-specific behavior

Consumers must not depend on those details for shared JACK coordination claims.

### Rule 5: unavailable or guarded JACK coordination must stay typed

If Signal cannot directly own JACK transport or graph coordination, that answer
must land through shared runtime-owned guarded or unavailable receipts, not
missing fields or host-local callback matrices.

## Deferred scope

Batch 2.1 intentionally does not claim:

- full JACK transport realization
- full port graph or connection mutation ownership
- session-manager integration or daemon policy depth
- product-local transport UI, graph browser, or repair UX
- PipeWire or ALSA parity work, which belongs to later `g08` milestones

Those belong to later `g08` Linux and workflow queues.

## Batch 2.1 outcome

Batch 2.1 freezes the bounded JACK coordination contract:

- JACK transport, graph attachment, client role, and guarded coordination now
  share one explicit Signal-owned vocabulary instead of implicit future
  host-private behavior
- later runtime work must compose JACK-native detail through the closed live
  Linux ownership, hardware, and supervision seams instead of inventing a new
  backend-only shell
- Batch 2.2 now has one bounded target for runtime-owned JACK transport and
  graph receipts before concrete server-host realization widens

## Batch 2.2 outcome

Batch 2.2 materializes the first runtime-owned JACK coordination receipt
family on top of this contract:

- `signal-runtime` now owns typed JACK transport posture, graph coordination
  state, client role, and guarded coordination through one
  `RuntimeJackCoordinationSnapshot`
- that snapshot derives from shared host-I/O and transport-session evidence,
  so JACK coordination stays additive over the closed live Linux ownership seam
  instead of becoming a host-private callback or daemon shell
- stable host edges now export explicit bounded answers on the same seam:
  `NotJack` on local host and a guarded simulated JACK graph baseline on
  server host
- Batch 2.3 remains the proof step that must show this widened seam stays
  consumable through public runtime, supervisor, and stable host-edge
  consumer surfaces

## Batch 2.3 outcome

Batch 2.3 closes the bounded JACK coordination consumer seam:

- public runtime now proves JACK transport posture, graph coordination,
  client role, and guarded state through one downstream-style observation and
  supervisor boundary instead of host-private callback reconstruction
- both stable host edges now prove the same seam stays explicit:
  `NotJack` on local host and a bounded guarded JACK graph baseline on server
  host
- `signal-supervisor-tools` and Effigy now expose one repo-owned
  `jack-coordination-boundary` descriptor and acceptance task before `g08`
  moves into broader PipeWire and ALSA stream-policy parity work
- the closed proof seam stays intentionally bounded: real JACK daemon
  integration, callback-thread ownership, and session-manager depth remain
  deferred

## Next Task

Continue `g08.003` with Batch 3.1 by freezing runtime-owned PipeWire and ALSA
session-role, device-claim, and stream-policy parity meaning on top of the
closed live Linux ownership and JACK coordination seams.
