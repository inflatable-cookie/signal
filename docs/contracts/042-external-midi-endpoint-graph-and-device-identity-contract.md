# 042 External MIDI Endpoint Graph And Device-Identity Contract

Status: complete
Owner: core-product
Updated: 2026-03-18
Related contracts: `docs/contracts/006-runtime-hardware-portability-and-clock-domain-contract.md`, `docs/contracts/013-recording-continuity-midi-capture-and-checkpoint-contract.md`, `docs/contracts/023-generic-midi-note-expression-and-plugin-event-model-contract.md`, `docs/contracts/027-external-io-monitoring-tap-point-and-loopback-measurement-contract.md`, `docs/contracts/041-linux-backend-clocking-duplex-and-endpoint-topology-parity-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`, `docs/architecture/package-map.md`

## Purpose

Freeze the first runtime-owned external MIDI endpoint and device-identity
boundary for `g07.011` so later runtime baseline work can deepen one shared
Signal vocabulary for MIDI devices, endpoints, capabilities, lifecycle, and
routing instead of pushing that meaning back into host-local device tables,
backend-private port graphs, or product-specific MIDI browser models.

## Authority hierarchy

External MIDI endpoint meaning has one authority chain:

1. host/backend integration layers own raw transport evidence for:
   - backend-native device and endpoint handles
   - backend-local names, port numbers, client IDs, and session detail
   - attach, detach, and route-change notifications
   - raw transport health and availability evidence
2. `docs/contracts/023-generic-midi-note-expression-and-plugin-event-model-contract.md`
   remains the authority for bounded MIDI event meaning once external MIDI
   traffic enters Signal's shared event vocabulary
3. `signal-runtime` must own the canonical consumer-visible meaning for:
   - external MIDI device identity
   - endpoint identity, direction, and grouping
   - endpoint graph membership and route visibility
   - endpoint capability and guarded portability bands
   - discovery, lifecycle, and observation/export delivery
4. host crates may broker enumeration, transport callbacks, and route-change
   evidence into runtime-owned receipts, but they must not become the
   authority for:
   - reusable external MIDI device or endpoint taxonomy
   - portable versus backend-private endpoint capability claims
   - product-facing route or browser semantics
   - a second MIDI lifecycle model detached from shared runtime receipts

If an external MIDI device, endpoint, or route claim cannot be explained
through additive host evidence, `023`, and runtime-owned receipts, it is not
yet part of the reusable Signal contract.

## Existing anchors

Batch 11.1 freezes this contract on top of the current shared runtime and
hardware surface family:

- `AudioDeviceDescriptor`
- `HardwareDiagnosticsSnapshot`
- `RuntimeHostHardwareSummary`
- `RuntimeHostIoSummary`
- `RuntimeExternalIoSnapshot`
- `RuntimeHostObservationReport`
- `RuntimeHostSupervisorReport`
- `RuntimeObservationReport`
- `RuntimeSupervisorReport`
- `PluginEvent::Midi`
- `EventPacket`
- `RuntimeRecordingCaptureKind::Midi`

Batch 11.1 does not claim those anchors already expose a full external MIDI
endpoint graph. It freezes how later DTOs, receipts, and proofs must widen
from this shared surface family instead of inventing a second host-private or
product-private MIDI device shell.

## Shared vocabulary

### External MIDI device identity

`external MIDI device identity` means the runtime-owned stable identity Signal
assigns to one discovered MIDI-capable device or virtual transport peer.

It must stay separate from backend-native handles and friendly names. Backend
names, client IDs, or port numbers may contribute evidence, but the reusable
consumer contract must remain runtime-owned.

### External MIDI endpoint

`external MIDI endpoint` means one bounded ingress, egress, or duplex-capable
connection point associated with a discovered external MIDI device identity.

An endpoint is not a product-local UI tree node. It is the smallest shared
runtime-owned object on which Signal can make claims about direction,
availability, capability, and route attachment.

### Endpoint graph

`endpoint graph` means the runtime-owned topology of discovered external MIDI
devices and endpoints, including bounded attachment or routing relationships
that matter to consumers.

This graph must stay intentionally coarse. It is not a promise of exposing raw
backend client graphs, patchbay semantics, or every backend-local transport
edge.

### Endpoint capability

`endpoint capability` means the bounded runtime-owned answer for what an
external MIDI endpoint can credibly do through the shared Signal contract.

Batch 11.1 freezes the first capability families as:

- bounded MIDI event input
- bounded MIDI event output
- transport or clock signal participation
- note or controller-oriented event transport
- guarded future-control-surface relevance

These are shared contract families, not a promise that every backend already
exposes the same depth.

### Endpoint lifecycle

`endpoint lifecycle` means the runtime-owned availability and health posture of
an external MIDI device or endpoint across discovery, attachment, guarded
availability, disappearance, and later recovery.

Later work may promote richer health detail, but this contract freezes that
hosts must not invent a separate MIDI-only lifecycle shell outside shared
runtime receipts.

### Endpoint route

`endpoint route` means a bounded runtime-owned relationship between an external
MIDI endpoint and a Signal-facing consumer path such as event intake, capture,
monitoring, or future control-surface attachment.

Route meaning must stay runtime-owned even when backend-native APIs expose
their own patchbay or subscription detail.

## Rules

### Rule 1: runtime owns external MIDI identity and graph meaning

Consumers must not depend on backend-native port tables, client IDs, or
product-local browser models to determine what device or endpoint Signal is
referring to. Shared identity and graph meaning must be runtime-owned.

### Rule 2: MIDI event meaning reuses the closed generic event contract

External MIDI endpoint work may widen discovery and routing, but it must reuse
the bounded MIDI event vocabulary from `023` instead of inventing a second
transport-specific event model.

### Rule 3: endpoint lifecycle composes with shared hardware and supervision

Attach, detach, guarded availability, and failure posture must compose with the
existing shared hardware and supervision model rather than becoming a
backend-private MIDI reconnect taxonomy.

### Rule 4: backend-private graph detail stays advisory

Raw ALSA sequencer, JACK MIDI, CoreMIDI, or other backend-native graph detail
may inform runtime classification later, but shared consumers must not depend
on that detail for stable endpoint identity or route truth.

### Rule 5: route meaning stays bounded and typed

If Signal claims an endpoint is routed, capturable, observable, or unavailable,
that answer must appear through typed runtime-owned receipts instead of missing
fields, host-private logs, or product-local heuristics.

### Rule 6: later control-surface and MIDI 2.0 work must deepen this contract

Future controller-expression, control-surface, MIDI 2.0, MPE, and scripting
milestones may widen device or route detail, but they must build on this
contract instead of reopening endpoint identity ownership.

## Deferred scope

Batch 11.1 intentionally does not claim:

- a concrete runtime DTO family yet
- live external MIDI host ownership across every backend
- MIDI 2.0, MPE, SysEx, NRPN, or richer controller dialect depth
- control-surface mapping, feedback, or scripting policy
- product-local MIDI device browser, setup, or repair UX
- exhaustive backend-native patchbay or graph visualization detail

Those belong to later `g07.011` and follow-on MIDI/control-surface milestones.

## Batch 11.1 outcome

Batch 11.1 freezes the first bounded external MIDI endpoint contract:

- Signal now has one explicit runtime-owned target for external MIDI device
  identity, endpoint identity, capability, lifecycle, and routing meaning
- the authority line is explicit: backend and host layers provide evidence,
  while runtime-owned receipts must remain canonical for consumer-facing MIDI
  endpoint truth
- generic MIDI event meaning stays anchored in `023`, which prevents later
  runtime endpoint work from drifting into transport-private event semantics
- Batch 11.2 can now focus on materializing runtime-owned endpoint receipts and
  shared host export instead of reopening what a discovered MIDI device or
  endpoint means

## Batch 11.2 outcome

Batch 11.2 materializes the first runtime-owned external MIDI endpoint graph
baseline on top of this contract:

- `signal-runtime` now owns typed external MIDI graph, device, endpoint,
  capability, and route receipts on `RuntimeObservationReport` and
  `RuntimeSupervisorReport` instead of leaving external MIDI identity and graph
  meaning implicit
- runtime capture now defaults to explicit `Unavailable` external MIDI state
  when no host context is present, while local and server hosts both project a
  shared `Empty` graph baseline through stable host export instead of
  inventing host-private MIDI device tables
- compact, multiline, and JSON report surfaces now all carry the same runtime-
  owned external MIDI receipt family, which narrows the remaining work to the
  consumer-facing proof seam rather than more internal DTO design

Batch 11.2 still does not close the public proof seam. That is now the
remaining Batch 11.3 job rather than hidden runtime-shaping work.

## Batch 11.3 outcome

Batch 11.3 closes the bounded external MIDI consumer boundary:

- public runtime proof now shows the same runtime-owned external MIDI graph
  receipt family remains consumable through shared observation and supervisor
  reports with explicit `Unavailable` and `Empty` outcomes
- both stable host edges now prove they forward runtime-owned external MIDI
  graph truth instead of rebuilding device identity, capability, or route
  meaning in host-local code
- `signal-supervisor-tools` now exposes the machine-readable
  `signal.runtime.external-midi-boundary` descriptor, and Effigy owns
  `acceptance:external-midi-boundary` as the repo-owned rerun lane

That completes the bounded `g07.011` contract depth. Richer MIDI 2.0, MPE,
controller-expression, and later control-surface transport work must now widen
from this closed external MIDI endpoint baseline rather than reopening device
identity ownership.

## Next Task

Continue `g07.012` with Batch 12.1 by freezing the widened MIDI 2.0, MPE, and
richer controller-expression contract on top of the now-closed external MIDI
endpoint graph and generic event boundaries.
