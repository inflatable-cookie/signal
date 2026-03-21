# 054 PipeWire And ALSA Session-Role, Device-Claim, And Stream-Policy Parity Contract

Status: complete
Owner: core-product
Updated: 2026-03-19
Related contracts: `docs/contracts/052-live-linux-audio-backend-ownership-and-session-lifecycle-contract.md`, `docs/contracts/053-jack-transport-graph-and-backend-native-coordination-contract.md`, `docs/contracts/040-linux-audio-backend-portability-across-alsa-jack-and-pipewire-contract.md`, `docs/contracts/041-linux-backend-clocking-duplex-and-endpoint-topology-parity-contract.md`, `docs/contracts/025-device-supervision-restart-state-machine-and-fault-boundary-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first PipeWire and ALSA parity boundary for `g08.003` so later live
Linux work can deepen one shared runtime-owned answer for session role,
device-claim posture, and stream-policy parity without reopening backend-
private daemon, stream, or callback policy.

## Authority hierarchy

PipeWire and ALSA parity has one authority chain:

1. `052` remains the authority for live Linux backend ownership, lifecycle,
   device-claim posture, session role, and guarded ownership fallback
2. `053` remains the authority for JACK-specific transport, graph, client-role,
   and guarded coordination meaning; this milestone must not reopen JACK-only
   coordination as a generic Linux parity shell
3. `040` remains the authority for Linux backend identity and portability-band
   meaning
4. `041` remains the authority for Linux-facing clocking, duplex, and
   endpoint-topology parity meaning
5. `025` remains the authority for recovering, exhausted, and faulted hardware
   outcomes once PipeWire or ALSA parity crosses into restart or failure state
6. `signal-hardware` may own backend-neutral live evidence for:
   - host-driven versus backend-managed stream ownership
   - negotiated stream format, block, and channel intent
   - restart policy and bounded transfer-policy inputs
   - backend diagnostics that later contribute to parity receipts
7. `signal-runtime` owns the canonical PipeWire and ALSA interpretation for:
   - session-role parity across direct and backend-managed paths
   - device-claim posture parity across direct-claim, shared-graph, lost, and
     released states
   - stream-policy parity across lifecycle ownership, restart policy, transfer
     policy, and guarded fallback
   - observation, supervisor, and stable host-edge export delivery
8. host crates may broker ALSA callback evidence, PipeWire node or stream
   events, and backend-specific restart hints into runtime-owned receipts, but
   they must not become the authority for:
   - competing ALSA-only or PipeWire-only session-role taxonomies
   - host-private device-claim truth
   - daemon-local stream-policy matrices exported as shared consumer meaning

If a PipeWire or ALSA parity claim cannot be explained through `052`, `053`,
`040`, `041`, `025`, `signal-hardware`, and runtime-owned receipts, it is not
yet part of the reusable Signal contract.

## Existing anchors

This contract builds on the current shared Linux and runtime surface family:

- `HardwareBackendIdentity`
- `LinuxAudioBackendKind`
- `HardwareLifecycleOwnership`
- `HardwareRestartPolicy`
- `RuntimeHostAudioTransferPolicy`
- `RuntimeHostClockingSummary`
- `RuntimeHostIoSummary`
- `RuntimeExternalIoSnapshot`
- `RuntimeLinuxBackendSessionSnapshot`
- `RuntimeDeviceSupervisionSnapshot`
- `RuntimeFaultStatusSnapshot`
- `RuntimeInterruptionSummary`

Batch 3.1 does not claim those anchors already provide full PipeWire and ALSA
parity. It freezes how later DTOs and proofs must deepen from this shared
surface family instead of inventing separate daemon or callback policy shells.

## Shared vocabulary

### Session-role parity

`session-role parity` means the bounded runtime-owned answer for why an active
PipeWire or ALSA path exists:

- primary runtime audio I/O
- monitoring-capable attachment
- offline-unavailable
- fallback or degraded continuation

Consumers must not infer this from backend crate names, daemon objects,
callback threads, or host-private stream wrappers.

### Device-claim parity

`device-claim parity` means the bounded shared interpretation of whether Signal
currently has:

- no claim
- a direct or exclusive claim
- participation in a backend-managed shared graph
- a lost or released claim

This contract does not freeze every backend-native reservation or node mode. It
freezes one shared consumer answer for how PipeWire and ALSA claims line up on
the same runtime-owned seam.

### Stream-policy parity

`stream-policy parity` means the bounded runtime-owned interpretation of how
the active PipeWire or ALSA path is being serviced:

- host-brokered callback or direct stream service
- backend-managed graph service
- restart responsibility and whether backend or host may recover the path
- transfer-policy limits that materially affect shared runtime execution
- guarded fallback when direct parity cannot be claimed

This is shared policy meaning, not a raw export of daemon properties, ALSA PCM
flags, PipeWire node params, or host-private callback scheduling logic.

### Guarded parity

`guarded parity` means PipeWire or ALSA is present, but the current
Signal-owned answer is constrained because:

- backend-managed service limits direct runtime control
- restart or reacquire is in progress
- aggregate or fallback clocking is active
- device-claim truth is present but stream-policy parity is only partial

Guarded parity must stay typed on shared runtime receipts rather than
disappearing into host-local daemon or callback policy.

## Bounded PipeWire and ALSA parity matrix

Batch 3.1 freezes the first bounded PipeWire and ALSA parity matrix.

| Capability family | ALSA baseline | PipeWire baseline | Notes |
| --- | --- | --- | --- |
| Backend identity and ownership posture | reused | reused | Reused from `040` and `052` |
| Session-role parity through shared runtime receipts | guarded | guarded | Frozen here before concrete runtime widening exists |
| Device-claim posture parity | guarded | guarded | Must stay runtime-owned instead of backend-private claim stories |
| Stream-policy parity through lifecycle ownership, restart policy, and transfer policy | guarded | guarded | Must compose through shared host-I/O and supervision seams |
| Daemon, node, reservation, and callback-thread detail | private | private | Still backend-private until later promotion |
| JACK-native transport or graph claims | unsupported | unsupported | Remains owned by `053` |

The matrix is intentionally guarded-first. Batch 3.1 freezes one parity target
before later runtime work proves how much depth is already realized.

## Rules

### Rule 1: PipeWire and ALSA parity layers on top of live ownership

`052` owns whether Signal has a live Linux session and what bounded ownership
posture it is in. This contract adds PipeWire and ALSA parity depth on top of
that ownership instead of replacing it with a second Linux-only lifecycle shell.

### Rule 2: parity answers stay runtime-owned

ALSA callback service, PipeWire node or stream events, and restart hints may
inform the answer, but the canonical parity state must remain on one runtime-
owned receipt family.

### Rule 3: stream policy must compose through shared host-I/O and supervision

Transfer policy, lifecycle ownership, restart policy, and guarded fallback must
compose through shared host-I/O, clocking, and supervision receipts instead of
becoming backend-private daemon or callback ledgers.

### Rule 4: JACK-specific coordination stays in `053`

This milestone may borrow the closed live-ownership and clocking seams, but it
must not reopen JACK transport or graph claims as if they were generic PipeWire
or ALSA parity terms.

### Rule 5: unavailable or guarded parity must stay typed

If Signal cannot claim full PipeWire or ALSA parity, that answer must land
through shared runtime-owned guarded or unavailable receipts, not missing
fields or host-local backend policy tables.

## Deferred scope

Batch 3.1 intentionally does not claim:

- full PipeWire node, session-manager, or portal depth
- full ALSA reservation, duplex, or device-broker policy depth
- distro-specific daemon startup, packaging, or environment guarantees
- user-facing Linux device browser, repair, or stream-policy UX
- broader acceptance or failure-injection depth, which belongs to later `g08`
  milestones

Those belong to later `g08` Linux and workflow queues.

## Batch 3.1 outcome

Batch 3.1 freezes the bounded PipeWire and ALSA parity contract:

- PipeWire and ALSA now share one explicit Signal-owned vocabulary for session
  role, device-claim posture, and stream-policy parity instead of implicit
  future backend-private behavior
- the authority line is now explicit: live ownership remains anchored in `052`,
  JACK-native coordination remains anchored in `053`, and stream-policy parity
  must compose through shared host-I/O, clocking, and supervision seams
- Batch 3.2 now has one bounded target for runtime-owned PipeWire and ALSA
  parity receipts instead of reopening backend-private authority during
  implementation

## Batch 3.2 outcome

Batch 3.2 widens this contract into runtime-owned DTOs and host-edge export:

- `signal-runtime` now owns a dedicated `RuntimePipeWireAlsaParitySnapshot`
  layered on top of the existing Linux session and host-I/O seams
- PipeWire and ALSA session-role, device-claim, stream-policy, and guarded
  parity are now typed on one shared runtime receipt instead of implicit host
  policy
- stable host edges now export the same parity family, so local and server
  hosts no longer need separate PipeWire or ALSA parity stories
- Batch 3.3 can now stay focused on downstream-style proof and acceptance
  surfacing instead of reopening the runtime-owned classification boundary

## Batch 3.3 outcome

Batch 3.3 closes the bounded consumer seam on top of that runtime-owned
receipt family:

- `signal-supervisor-tools` now exposes
  `signal.runtime.pipewire-alsa-parity-boundary` as the repo-owned descriptor
  for runtime, supervisor, and stable host-edge parity proof
- `effigy acceptance:pipewire-alsa-parity-boundary` now composes the public
  runtime proof, stable local/server host-edge proof, and descriptor proof
  into one reusable acceptance lane
- this contract is now complete: later Linux workflow and acceptance queues
  can consume one explicit PipeWire and ALSA parity boundary instead of
  reopening backend-local authority

## Next Task

Open `g08.004` with Batch 4.1 by freezing the first runtime-owned LV2 worker,
URID, patch, and extension-negotiation contract on top of the now-closed live
Linux ownership, JACK coordination, and PipeWire/ALSA parity seams.
