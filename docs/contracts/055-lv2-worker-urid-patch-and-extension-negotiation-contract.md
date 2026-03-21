# 055 LV2 Worker, URID, Patch, And Extension-Negotiation Contract

Status: complete
Owner: core-product
Updated: 2026-03-19
Related contracts: `docs/contracts/038-lv2-adapter-baseline-and-linux-native-plugin-lifecycle-contract.md`, `docs/contracts/039-linux-cross-adapter-plugin-parity-and-sandbox-policy-contract.md`, `docs/contracts/052-live-linux-audio-backend-ownership-and-session-lifecycle-contract.md`, `docs/contracts/053-jack-transport-graph-and-backend-native-coordination-contract.md`, `docs/contracts/054-pipewire-and-alsa-session-role-device-claim-and-stream-policy-parity-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first runtime-owned LV2 worker, URID, patch, and
extension-negotiation boundary for `g08.004` so later Linux-native LV2 depth
can widen shared runtime meaning without reopening adapter-private feature
tables, host-local negotiation policy, or backend-native callback ownership as
if they were already portable.

## Authority hierarchy

LV2 extension-negotiation depth has one authority chain:

1. `038` remains the authority for the bounded LV2 adapter baseline:
   - LV2 backend identity
   - Linux-native discovery and lifecycle alignment
   - bounded shared capability and continuity mapping
2. `039` remains the authority for Linux cross-adapter parity and sandbox
   policy:
   - guarded versus private LV2 breadth
   - Linux parity bands and shared-sandbox policy rules
   - the rule that extension depth must be promoted additively instead of
     inferred from adapter existence
3. `052`, `053`, and `054` remain the authority for live Linux backend
   ownership, JACK coordination, and PipeWire/ALSA parity:
   - this milestone must not reopen host-audio session ownership, daemon
     policy, or backend callback truth as LV2 extension semantics
4. `signal-plugin` owns backend-neutral plugin vocabulary and the bounded
   feature-class, lifecycle, and continuity meaning already shared across
   formats
5. a Signal-owned LV2 adapter layer such as `signal-plugin-lv2` owns
   protocol-native realization detail for:
   - worker interface discovery and invocation plumbing
   - URID map or unmap negotiation and adapter-private caching
   - patch, atom, and property URI handling
   - extension feature-table traversal and Linux-native host-feature wiring
6. `signal-runtime` owns the canonical shared interpretation for:
   - whether worker participation is absent, available, required, guarded, or
     unavailable
   - whether URID negotiation is portable enough for shared runtime
     continuity, patch exchange, or delegated execution
   - whether patch exchange is supported, guarded, unavailable, or deferred
   - whether extension negotiation succeeded, is guarded, or failed in a way
     consumers should see through shared receipts
   - observation, supervisor, and stable host-edge export delivery
7. host crates may broker adapter evidence, Linux-native host features, or
   plugin-instance hints into runtime-owned receipts, but they must not become
   the authority for:
   - competing LV2 worker or URID taxonomies
   - host-private patch support conclusions
   - consumer-visible extension-negotiation summaries

If an LV2 worker, URID, patch, or extension-negotiation claim cannot be
explained through `038`, `039`, `052`, `053`, `054`, adapter-private
realization, and runtime-owned receipts, it is not yet part of the reusable
Signal contract.

## Existing anchors

This contract builds on the current shared plugin and Linux surface family:

- `PluginFormat`
- `RuntimePluginDiscoverySnapshot`
- `RuntimePluginScanReceipt`
- `RuntimePluginDiscoveredTypeRecord`
- `RuntimePluginCapabilityCoverageSummary`
- `RuntimePluginLifecycleSnapshot`
- `RuntimePluginChainSnapshot`
- `RuntimePluginRecallSnapshot`
- `RuntimeInterruptionSummary`
- `RuntimeDegradationSummary`
- `RuntimeOfflinePluginExecutionBoundary`
- `RuntimeObservationReport`
- `RuntimeSupervisorReport`

Batch 4.1 does not claim those anchors already provide explicit worker, URID,
patch, or extension-negotiation truth. It freezes how later DTOs and proofs
must deepen from this existing runtime-owned family instead of inventing an
LV2-only host policy shell.

## Shared vocabulary

### Worker posture

`worker posture` means the bounded runtime-owned answer for how an LV2 plugin
relates to worker participation:

- not LV2
- worker absent
- worker optional and available
- worker required and available
- worker guarded
- worker unavailable or unsupported

This is shared consumer meaning, not a raw export of adapter callback hooks,
thread handles, or Linux host-feature tables.

### URID negotiation posture

`URID negotiation posture` means the bounded shared interpretation of whether
the LV2 runtime path has enough URI-to-ID support to make shared runtime
surfaces reliable:

- not required
- negotiated
- adapter-guarded
- unavailable

The contract does not freeze every URI map or cache strategy. It freezes one
shared answer for whether URID support is sufficient for the current Signal
owned runtime path.

### Patch exchange posture

`patch exchange posture` means the bounded runtime-owned answer for whether the
plugin can participate in shared patch or property style state exchange:

- absent
- supported
- guarded
- unavailable

This must not become host-local knowledge derived from port graphs, atom
buffers, or private property ledgers.

### Extension-negotiation summary

`extension-negotiation summary` means the bounded shared answer for whether
LV2-specific extension capability needed by the current runtime path was:

- not needed
- negotiated successfully
- only partially satisfied
- unavailable

This is shared runtime meaning, not a full dump of every LV2 feature URI or
adapter-native negotiation branch.

## Bounded LV2 extension matrix

Batch 4.1 freezes the first bounded LV2 extension matrix.

| Capability family | Baseline band | Notes |
| --- | --- | --- |
| Worker posture | guarded | Shared runtime answer is required before callback or helper detail matters |
| URID negotiation posture | guarded | Shared runtime answer is about usable negotiation, not raw URI tables |
| Patch exchange posture | guarded | Must stay shared even when adapter-private atom handling varies |
| Extension-negotiation summary | guarded | One bounded shared summary, not a full feature dump |
| Atom buffer formats, object payload schemas, property catalogs | private | Still adapter-private unless later promoted additively |
| UI, external-editor, time-position, state-path, and custom extension semantics | private | Not part of this baseline |
| Linux daemon, callback-thread, and backend session policy | unsupported | Remains owned by `052`, `053`, and `054` |

The matrix is intentionally guarded-first. Batch 4.1 freezes one shared target
before runtime-owned receipt depth proves how much of that target is already
realized.

## Rules

### Rule 1: LV2 extension depth layers on top of the closed LV2 baseline

`038` remains the authority for the Linux-native LV2 baseline. This milestone
widens worker, URID, patch, and extension-negotiation meaning on top of that
baseline instead of creating a second LV2 lifecycle or discovery model.

### Rule 2: adapter-private negotiation stays adapter-private until Signal promotes it

Worker hook plumbing, URID cache internals, patch port traversal, and feature
URI tables may remain adapter-private. Only the bounded consumer answer gets
promoted here.

### Rule 3: shared negotiation truth stays runtime-owned

Hosts may supply evidence, but the canonical shared answer must remain on one
runtime-owned receipt family reused by observation, supervisor export, and
stable host-edge surfaces.

### Rule 4: guarded and unavailable extension outcomes must stay typed

If Signal cannot claim full worker, URID, patch, or extension-negotiation
support, the answer must land through shared guarded or unavailable receipts,
not missing fields, host-local warnings, or adapter-private notes.

### Rule 5: Linux host-backend ownership is not reclassified as LV2 negotiation

Backend session ownership, JACK coordination, and PipeWire or ALSA parity stay
owned by `052`, `053`, and `054`. This milestone may reuse those seams, but it
must not blur backend-native live-ownership policy into LV2 extension meaning.

## Deferred scope

Batch 4.1 intentionally does not claim:

- full LV2 atom schema or object-model parity
- every custom extension URI as shared consumer data
- LV2 UI, external editor, or product-local extension UX
- full worker scheduling or realtime/offline helper execution depth
- distro-specific Linux packaging, portal, or daemon guarantees
- broader acceptance or failure-injection depth, which belongs to later `g08`
  milestones

Those remain later runtime, Linux, or workflow queues.

## Batch 4.1 outcome

Batch 4.1 freezes the bounded LV2 extension-negotiation contract:

- worker, URID, patch, and extension-negotiation depth now have one explicit
  Signal-owned authority line instead of staying deferred prose on the earlier
  LV2 baseline
- the contract now makes the separation explicit between adapter-private LV2
  realization and the shared runtime-owned answers consumers may eventually
  rely on
- Batch 4.2 now has a bounded target for the first runtime-owned receipt
  family without reopening Linux backend ownership or host-local negotiation
  policy

## Batch 4.2 outcome

Batch 4.2 materializes the first reusable runtime-owned LV2 extension receipt
family.

- `RuntimeLv2ExtensionCapabilitySummary` and `RuntimeLv2ExtensionSnapshot` now
  provide one shared authority line for worker posture, URID negotiation
  posture, patch exchange posture, and extension-negotiation state
- runtime-owned discovery and lifecycle evidence now compose into guarded,
  negotiated, and unavailable LV2 extension answers without adapter-private
  reclassification
- stable host-edge export now reuses the same runtime-owned LV2 extension seam
  instead of reconstructing worker, URID, or patch support from host-local
  feature summaries

## Batch 4.3 outcome

Batch 4.3 closes the bounded LV2 extension consumer seam.

- the existing `signal.runtime.lv2-boundary` descriptor now points at this
  contract instead of the older baseline-only LV2 contract
- the repo-owned acceptance lane now requires public runtime proof plus stable
  local and server host-edge proofs for the same runtime-owned LV2 extension
  snapshot
- the machine-readable supervisor boundary now describes worker posture, URID
  negotiation posture, patch exchange posture, and extension-negotiation state
  as one bounded shared proof surface

## Next Task

Open `g08.005` with Batch 5.1 by freezing the first runtime-owned complex
plugin pin-matrix and dynamic bus-negotiation contract on top of the closed
LV2 extension, Linux parity, and live backend seams.
