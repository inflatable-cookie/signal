# 056 Complex Plugin Pin-Matrix And Dynamic Bus Negotiation Contract

Status: complete
Owner: core-product
Updated: 2026-03-19
Related contracts: `docs/contracts/034-multi-bus-graph-execution-and-auxiliary-topology-contract.md`, `docs/contracts/035-plugin-complex-io-topology-and-multi-output-instrument-contract.md`, `docs/contracts/039-linux-cross-adapter-plugin-parity-and-sandbox-policy-contract.md`, `docs/contracts/055-lv2-worker-urid-patch-and-extension-negotiation-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first runtime-owned complex plugin pin-matrix and dynamic
bus-negotiation boundary for `g08.005` so later Linux-native plugin routing,
format breadth, and richer graph execution can deepen from one shared Signal
authority line instead of drifting into adapter-private port graphs,
host-local bus rules, or format-specific negotiation policy.

## Authority hierarchy

Complex plugin pin-matrix and dynamic bus-negotiation depth has one authority
chain:

1. `035` remains the authority for bounded complex plugin-I/O topology,
   multi-output instrument meaning, bus-capable FX class, and attachment
   policy:
   - this milestone layers richer pin-matrix and dynamic bus-negotiation
     meaning on top of that topology baseline instead of replacing it
2. `034` remains the authority for bus-role, auxiliary-path, connection
   identity, and fallback meaning:
   - pin-matrix and dynamic bus negotiation must reuse the closed multi-bus
     routing substrate rather than invent a plugin-only routing shell
3. `039` remains the authority for Linux cross-adapter parity and sandbox
   policy:
   - this milestone may widen guarded versus portable plugin-routing answers,
     but it must not redefine parity bands or sandbox rules
4. `055` remains the authority for bounded LV2 worker, URID, patch, and
   extension-negotiation meaning:
   - LV2-specific bus or pin detail may inform evidence, but it must not
     become the shared authority for dynamic bus negotiation
5. `signal-plugin` and adapter crates may report raw port, bus, and
   negotiation evidence, but they must not redefine shared pin-matrix meaning
   once runtime-owned receipts exist
6. `signal-runtime` owns the canonical shared interpretation for:
   - pin-group identity and matrix posture
   - dynamic bus-negotiation posture and fallback outcome
   - observation, supervisor, and stable host-edge export delivery
7. host crates may broker adapter evidence into runtime-owned receipts, but
   they must not become the authority for:
   - competing pin-matrix taxonomies
   - host-private bus activation conclusions
   - consumer-visible negotiation summaries

If a pin-matrix or dynamic bus-negotiation claim cannot be explained through
`034`, `035`, `039`, `055`, adapter-private realization, and runtime-owned
receipts, it is not yet part of the reusable Signal contract.

## Existing anchors

This contract builds on the current shared plugin, routing, and Linux surface
family:

- `PluginFormat`
- `RuntimePluginDiscoveredTypeRecord`
- `RuntimePluginComplexIoSummary`
- `RuntimePluginDiscoverySnapshot`
- `RuntimePluginLifecycleSnapshot`
- `RuntimePluginChainSnapshot`
- `RuntimeExecutionTopologySummary`
- `RuntimeOfflineRenderPreview`
- `RuntimeObservationReport`
- `RuntimeSupervisorReport`

Batch 5.1 does not claim those anchors already provide explicit pin-matrix or
dynamic bus-negotiation truth. It freezes how later DTOs and proofs must widen
from this existing runtime-owned family instead of inventing a plugin-format
or host-local negotiation shell.

## Shared vocabulary

### Pin-group identity

`pin-group identity` means the bounded runtime-owned answer for one meaningful
plugin-facing input or output group inside a richer attachment surface.

Batch 5.1 freezes this bounded family:

- `PrimaryProgramPath`
- `SecondaryProgramPath`
- `AuxReturnPath`
- `SidechainPath`
- `AnalysisPath`
- `InactiveDeclaredPath`

This is shared Signal routing meaning, not raw format-native bus names or pin
numbers.

### Pin-matrix posture

`pin-matrix posture` means the bounded shared interpretation of how much of a
plugin's declared pin surface is currently represented through the reusable
Signal path.

Batch 5.1 freezes this bounded family:

- `Simple`
- `Declared`
- `Negotiated`
- `Guarded`
- `Unavailable`

This is a runtime-owned posture, not a full dump of every adapter-private pin
or channel-map detail.

### Dynamic bus-negotiation posture

`dynamic bus-negotiation posture` means the bounded runtime-owned answer for
whether the plugin's bus surface can be activated, narrowed, widened, or
rebound in a way Signal can safely expose through shared receipts.

Batch 5.1 freezes this bounded family:

- `Static`
- `Negotiated`
- `Guarded`
- `Unavailable`

This must not become host-local knowledge inferred from callback behavior or
private adapter state machines.

### Negotiation fallback outcome

`negotiation fallback outcome` means the runtime-owned result when a declared
pin-matrix or dynamic bus surface cannot be activated as requested.

Batch 5.1 freezes this bounded family:

- `CollapseToDeclaredBaseline`
- `DeactivateOptionalPath`
- `RoutePrimaryOnly`
- `GuardedDegradation`
- `TerminalNegotiationFailure`

Later batches may widen this family, but they must remain additive and
runtime-owned.

## Rules

### Rule 1: pin-matrix meaning layers on top of the closed complex-I/O baseline

`035` remains the authority for complex plugin-I/O topology. This milestone
widens pin-matrix and dynamic bus-negotiation meaning on top of that baseline
instead of reopening the original topology model.

### Rule 2: shared negotiation truth stays runtime-owned

Hosts and adapter crates may supply evidence, but the canonical shared answer
must remain on one runtime-owned receipt family reused by observation,
supervisor export, and stable host-edge surfaces.

### Rule 3: multi-bus routing substrate remains authoritative

Dynamic bus negotiation must reuse the closed multi-bus and auxiliary-topology
meaning from `034` instead of inventing plugin-only send, return, or attach
classes.

### Rule 4: adapter-private pin names remain advisory

Format-native pin labels, bus indices, and channel-map details may remain
adapter-private. Shared runtime meaning must reduce them to bounded pin-group
identity, posture, and fallback receipts.

### Rule 5: product routing UX stays out of scope

This contract freezes reusable runtime and adapter meaning only. It does not
freeze mixer-strip presentation, patchbay editing, product-local pin-matrix UX,
or host-private routing policy.

## Deferred scope

Batch 5.1 intentionally does not claim:

- full format-specific pin or port schema export
- arbitrary channel-map editing or product pin-matrix UX
- final runtime execution receipts for every negotiated bus transition
- immersive, object, or renderer-specific bus negotiation semantics
- Linux daemon, callback-thread, or distro-policy guarantees
- broader acceptance or failure-injection depth, which belongs to later `g08`
  batches

## Batch 5.1 outcome

Batch 5.1 freezes the first runtime-owned pin-matrix and dynamic
bus-negotiation authority line for Signal:

- pin-group identity, pin-matrix posture, dynamic bus-negotiation posture, and
  negotiation fallback outcome are now explicit Signal-owned vocabulary
- richer plugin routing can now widen on top of the closed complex-I/O,
  multi-bus, Linux parity, and LV2 extension seams instead of reopening bus
  policy per format
- Batch 5.2 now has one bounded contract target for runtime-owned pin-matrix
  and dynamic bus-negotiation receipts before consumer proof widens in
  Batch 5.3

## Batch 5.2 outcome

Batch 5.2 materializes the first reusable runtime-owned pin-matrix and dynamic
bus-negotiation receipt family.

- `RuntimePluginPinMatrixSnapshot` now provides one shared authority line for
  pin-group identity, pin-matrix posture, dynamic bus-negotiation posture, and
  bounded fallback outcome
- runtime-owned complex-I/O discovery, lifecycle, and plugin-chain evidence
  now compose into declared, negotiated, guarded, and unavailable routing
  answers without adapter-private reclassification
- stable host-edge export now reuses the same runtime-owned pin-matrix seam
  instead of reconstructing richer bus activation posture from host-local
  plugin detail

## Batch 5.3 outcome

Batch 5.3 closes the bounded pin-matrix and dynamic bus-negotiation consumer
seam.

- the existing `signal.runtime.complex-io-boundary` descriptor now points at
  this contract instead of the older baseline-only complex plugin-I/O contract
- the repo-owned acceptance lane continues to reuse the existing focused
  runtime and host-edge proofs, but now describes the widened
  `plugin_pin_matrix_snapshot` seam directly
- the machine-readable supervisor boundary now presents complex plugin-I/O,
  pin-group identity, pin-matrix posture, and dynamic bus-negotiation posture
  as one bounded shared proof surface

## Next Task

Open `g08.006` with Batch 6.1 by freezing the first runtime-owned immersive
object rendering and room-policy contract on top of the closed plugin-routing,
LV2 extension, Linux parity, and live backend seams.
