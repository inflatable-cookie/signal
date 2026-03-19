# 043 MIDI 2.0, MPE, And Richer Controller-Expression Contract

Status: complete
Owner: core-product
Updated: 2026-03-18
Related contracts: `docs/contracts/023-generic-midi-note-expression-and-plugin-event-model-contract.md`, `docs/contracts/042-external-midi-endpoint-graph-and-device-identity-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the widened Signal-owned controller-expression boundary for `g07.012` so
later runtime, plugin, and device work can deepen one shared vocabulary for
MIDI 2.0-adjacent expression, MPE posture, and richer per-note or controller
meaning instead of reopening adapter-private packet semantics or host-local
controller taxonomies.

## Authority hierarchy

Widened controller-expression meaning has one authority chain:

1. `docs/contracts/023-generic-midi-note-expression-and-plugin-event-model-contract.md`
   remains the authority for the existing bounded generic event vocabulary,
   block-local timing, note identity, and current three-byte MIDI transport
   posture
2. `docs/contracts/042-external-midi-endpoint-graph-and-device-identity-contract.md`
   remains the authority for external MIDI device, endpoint, route, and
   lifecycle meaning once widened controller-expression traffic originates from
   external hardware or virtual device paths
3. plugin adapters and host/backend integrations may own raw packet evidence
   for:
   - backend-native MIDI 2.0 or UMP detail
   - MPE zone setup and backend-local controller grouping evidence
   - per-format packet encodings, capability flags, and transport quirks
4. `signal-runtime` must own the canonical consumer-visible meaning for:
   - widened controller-expression families
   - portable versus guarded expression capability claims
   - note-scoped versus channel-scoped expression meaning
   - runtime, plugin, device, and supervisor/export delivery
5. host crates and adapters may broker raw evidence into runtime-owned
   receipts, but they must not become the authority for:
   - a second controller-expression taxonomy detached from shared runtime DTOs
   - product-local expressive-controller naming or browser semantics
   - adapter-private MPE or MIDI 2.0 packet models as the consumer boundary

If a widened expression claim cannot be explained through `023`, `042`, and
runtime-owned receipts, it is not yet part of the reusable Signal contract.

## Existing anchors

Batch 12.1 freezes this contract on top of the current shared event and device
surface family:

- `PluginEvent`
- `EventPacket`
- `RuntimePluginEventSnapshot`
- `RuntimeExternalMidiEndpointGraphSnapshot`
- `RuntimeObservationReport`
- `RuntimeSupervisorReport`
- `RuntimePluginDiscoveredTypeRecord.processing_contract.supports_note_expression`
- existing plugin and hardware capability coverage surfaces

Batch 12.1 does not claim those anchors already expose full MIDI 2.0 or MPE
depth. It freezes how later DTOs and proofs must widen from them instead of
inventing a second host-private expressive-event shell.

## Shared vocabulary

### Richer controller expression

`richer controller expression` means runtime-owned expressive event meaning
beyond the current bounded three-byte MIDI and first note-expression baseline.

Batch 12.1 freezes the first widened families as:

- per-note pitch expression
- per-note pressure or aftertouch expression
- per-note timbre or brightness-oriented expression
- note-release and articulation-oriented expression
- richer channel-scoped controller-expression lanes

These are Signal vocabulary families, not a promise that every adapter already
realizes every family.

### MPE posture

`MPE posture` means the runtime-owned answer for whether widened controller
expression is being interpreted through a bounded channel-zoned expressive
model compatible with MPE-style note ownership.

This contract freezes MPE posture as shared runtime meaning rather than a
plugin-format or host-private setup convenience.

### MIDI 2.0 posture

`MIDI 2.0 posture` means whether widened controller-expression evidence is
portable, guarded, or still deferred when viewed through Signal's shared
runtime contract.

Batch 12.1 does not promise full UMP transport or profile/configuration depth.
It freezes how future work must classify that posture instead of leaking raw
packet dialects into consumer code.

### Expression capability

`expression capability` means the bounded runtime-owned claim that a plugin,
device, or route can consume, emit, or preserve one widened controller-
expression family.

Capability must stay separate from raw packet support. A backend or adapter may
understand more detail than Signal currently promotes into the shared contract.

### Guarded widening

`guarded widening` means a widened controller-expression family can appear in
the shared contract with explicit constraints or fallback rather than a false
portable claim.

Guarded widening must be typed and runtime-owned. It must not become an
implied gap reconstructed from missing adapter fields.

## Rules

### Rule 1: widened expression builds on the generic event contract

`g07.012` must deepen the existing generic event boundary from `023`. It must
not create a second event language for MIDI 2.0, MPE, or richer controller
expression.

### Rule 2: note, channel, and device meaning stay aligned

If Signal promotes a widened controller-expression family, plugin-facing,
runtime-facing, and external-device-facing meaning must stay on one shared
vocabulary instead of splitting into adapter-private or device-private terms.

### Rule 3: capability claims stay runtime-owned

Hosts and adapters may surface raw evidence, but portable versus guarded
controller-expression capability claims must remain runtime-owned on shared
receipts.

### Rule 4: raw packet dialects stay advisory unless promoted

UMP framing, MIDI 2.0 packet words, profile negotiation, zone setup details,
and adapter-private event records remain advisory until later batches promote
them into the shared contract.

### Rule 5: fallback and unsupported depth must stay typed

If one widened expression family is unavailable, downgraded, or preserved only
through a guarded path, the shared answer must remain explicit through
runtime-owned typed receipts instead of product-local heuristics.

### Rule 6: later control-surface work must widen from this contract

Future control-surface transport, mapping, feedback, and scripting milestones
must reuse this widened controller-expression vocabulary instead of reopening
event ownership.

## Deferred scope

Batch 12.1 intentionally does not claim:

- full MIDI 2.0 UMP transport or packet schema depth
- profile configuration, property exchange, or discovery negotiation depth
- exhaustive MPE zone editing or controller assignment policy
- SysEx, NRPN, RPN, notation, or score-editing workflows
- product-local expressive-controller setup UX
- control-surface mapping, transport feedback, or scripting semantics

Those belong to later `g07.012`, `g07.013`, and follow-on control-surface
milestones.

## Batch 12.1 outcome

Batch 12.1 freezes the widened controller-expression contract:

- Signal now has one explicit runtime-owned target for MIDI 2.0-adjacent,
  MPE-aware, and richer controller-expression meaning instead of falling back
  to adapter-private packet models
- the authority line is explicit: raw packet and backend detail stay advisory,
  while runtime-owned receipts must remain canonical for consumer-facing
  expressive-event truth
- generic event and external MIDI endpoint meaning stay the anchors, which
  prevents later runtime work from reopening a second device or event shell
- Batch 12.2 can now focus on materializing the first widened runtime and
  adapter receipts instead of reopening what expressive-controller meaning
  belongs to Signal

## Batch 12.2 outcome

Batch 12.2 materializes the first bounded widened controller-expression
receipts on shared Signal-owned surfaces:

- `signal-plugin` now breaks note-expression evidence into pressure, timbre,
  and tuning families through `EventPacketSummary` instead of one opaque
  widened-expression bucket
- `signal-runtime` now exposes those families, plus runtime-owned `MPE` and
  `MIDI 2.0` posture, through `RuntimePluginEventSnapshot`
- external MIDI capability summaries now carry explicit guarded or unsupported
  widened-expression posture through typed capability flags and
  `RuntimeControllerExpressionMidi2Posture`

This keeps widened controller-expression meaning attached to shared runtime and
plugin DTOs instead of adapter-private packet or capability shells. Batch 12.3
can now focus on proving those widened receipts remain consumable through the
public runtime, supervisor, and stable host-edge boundary.

## Batch 12.3 outcome

Batch 12.3 closes the widened controller-expression proof seam:

- public runtime consumers now have a focused proof that widened
  note-expression family totals, `MPE` posture, `MIDI 2.0` posture, and
  external-device capability posture remain consumable through shared runtime
  DTOs
- both stable host edges now prove they forward the same widened
  controller-expression receipts instead of rebuilding host-local packet or
  capability meaning
- `signal-supervisor-tools` and Effigy now expose a machine-readable rerun seam
  for this boundary through `signal.runtime.controller-expression-boundary` and
  `acceptance:controller-expression-boundary`

`g07.012` is therefore closed as a bounded controller-expression milestone.
Later control-surface work can widen from this proof seam instead of reopening
event ownership.

## Next Task

Continue `g07.013` with Batch 13.1 by freezing the runtime-owned
control-surface transport, mapping, feedback, and capability contract on top
of the now-closed external MIDI endpoint and widened controller-expression
boundaries.
