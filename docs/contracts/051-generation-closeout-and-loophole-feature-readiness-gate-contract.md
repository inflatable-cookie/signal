# 051 Generation Closeout And Loophole Feature-Readiness Gate Contract

Status: complete
Owner: core-product
Updated: 2026-03-19
Related contracts: `docs/contracts/050-multichannel-linux-time-stretch-and-control-surface-acceptance-contract.md`, `docs/contracts/032-canonical-multichannel-layout-and-channel-role-contract.md`, `docs/contracts/033-sidechain-routing-and-secondary-input-execution-contract.md`, `docs/contracts/034-multi-bus-graph-execution-and-auxiliary-topology-contract.md`, `docs/contracts/037-surround-bed-object-and-mix-policy-expansion-contract.md`, `docs/contracts/039-linux-cross-adapter-plugin-parity-and-sandbox-policy-contract.md`, `docs/contracts/040-linux-audio-backend-portability-across-alsa-jack-and-pipewire-contract.md`, `docs/contracts/041-linux-backend-clocking-duplex-and-endpoint-topology-parity-contract.md`, `docs/contracts/042-external-midi-endpoint-graph-and-device-identity-contract.md`, `docs/contracts/044-control-surface-transport-mapping-and-feedback-contract.md`, `docs/contracts/045-advanced-hardware-extensibility-and-scripting-safe-device-policy-contract.md`, `docs/contracts/046-sample-domain-time-stretch-engine-contract.md`, `docs/contracts/047-warp-marker-transient-anchor-and-tempo-assist-analysis-contract.md`, `docs/contracts/048-post-warp-render-cache-and-transform-artifact-contract.md`, `docs/contracts/049-low-latency-audition-scrub-and-preview-transform-service-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the bounded `g07.020` closeout policy so the final generation verdict,
Loophole-facing feature-readiness claims, and deferred post-`g07` posture stay
repo-owned, typed, and additive over the now-closed `g07.019` integrated
acceptance lane.

## Authority hierarchy

`g07` closeout has one authority chain:

1. closed `g07` contracts define the bounded routing, Linux, controller, and
   stretch claims that may be promoted at generation closeout
2. `signal-runtime` owns the typed receipts those claims must summarize:
   - `RuntimeObservationReport`
   - `RuntimeSupervisorReport`
   - `RuntimeExecutionTopologySummary`
   - `RuntimeExternalIoSnapshot`
   - `RuntimeExternalMidiEndpointGraphSnapshot`
   - `RuntimeControlSurfaceSnapshot`
   - `RuntimeAdvancedHardwareSnapshot`
   - `RuntimeStretchEngineSnapshot`
   - `RuntimeMarkerAnalysisSnapshot`
   - `RuntimeTransformArtifactSnapshot`
   - `RuntimePreviewTransformServiceSnapshot`
3. shared host crates may contribute bounded local or server proof surfaces,
   but they do not own closeout meaning
4. `signal-supervisor-tools` owns the machine-readable `g07` closeout and
   readiness descriptors that explain:
   - which evidence bundles count toward the final gate
   - which evidence is required, advisory, or deferred
   - which Loophole-facing feature-readiness claims are supported, guarded, or
     still deferred
5. Effigy tasks own the runnable grouping policy for:
   - the required integrated `g07` acceptance lane
   - the final `g07` closeout gate
   - any advisory rerun lanes promoted into the repo-owned closeout surface
6. downstream consumers such as Loophole may archive or consume the resulting
   evidence, but they must not redefine the canonical Signal closeout bar

If a `g07` closeout claim cannot be explained through closed contracts, typed
runtime receipts, supervisor-tools descriptors, and repo-owned Effigy tasks, it
is not yet part of the reusable closeout boundary.

## Existing anchors

This contract builds on the bounded integrated evidence already closed in
`g07.019`:

- `effigy acceptance:g07-integrated-acceptance-lane`
- `cargo run -p signal-supervisor-tools -- --describe-g07-acceptance-lane --format=json`
- `cargo test -p signal-supervisor-tools export_json_carries_cross_family_g07_acceptance_evidence`

It also builds on the already-closed bounded boundary tasks composed by that
lane, especially:

- multichannel, sidechain, multi-bus, and spatial routing boundaries
- Linux plugin parity, Linux audio backend portability, and Linux backend
  clock-topology parity boundaries
- external MIDI, controller-expression, control-surface, and advanced-hardware
  boundaries
- stretch, marker-analysis, transform-artifact, and preview-transform
  boundaries

Batch 20.1 does not claim the final closeout gate is implemented. It freezes
the policy the later closeout surface must obey.

## Shared vocabulary

### Generation closeout gate

`generation closeout gate` means the final repo-owned rule set that decides
whether `g07` has enough reusable evidence to claim closeout and hand the next
work either to backlog hardening, a new active generation, or a clearly named
deferred queue.

### Loophole-facing feature readiness

`Loophole-facing feature readiness` means Signal's reusable answer to whether
`g07` materially improved Loophole's remaining routing, Linux, controller, and
sample-domain media needs through shared runtime substrate, not whether
Loophole is product-launch ready.

### Required closeout evidence

`required closeout evidence` means evidence that must remain green for `g07` to
claim closeout.

### Advisory closeout evidence

`advisory closeout evidence` means shared evidence that materially improves the
closeout decision but does not yet block closeout.

### Deferred closeout evidence

`deferred closeout evidence` means known useful evidence that remains outside
the closeout gate because it is not yet bounded, stable, portable, or worth
promoting into the shared fast path.

## Closeout evidence families

Batch 20.1 freezes four closeout evidence families.

### Family 1: Integrated acceptance substrate

This family proves the widened `g07` surface still composes in one required
cross-family lane:

- `effigy acceptance:g07-integrated-acceptance-lane`
- the machine-readable grouped `g07` acceptance descriptor
- the focused cross-family export proof inside `signal-supervisor-tools`

This family is always `required`.

### Family 2: Closeout descriptor and gate coherence

This family proves the generation closeout itself is inspectable:

- one machine-readable `g07` closeout descriptor
- one repo-owned Effigy closeout task
- one explicit required/advisory/deferred record

This family becomes `required` once implemented in Batch 20.2.

### Family 3: Loophole-facing readiness summary

This family explains whether `g07` moved Loophole forward on the pressures that
motivated the generation:

- routing, multichannel, sidechain, multi-bus, and bounded spatial readiness
- Linux plugin and backend portability readiness
- external MIDI, controller-expression, control-surface, and advanced-hardware
  readiness
- stretch, marker-analysis, transform-artifact, and preview-service readiness

This family must be machine-readable and explicit, but Batch 20.1 keeps its
final review posture for Batch 20.3.

### Family 4: Explicit deferred and backlog posture

This family records what `g07` intentionally does not claim at closeout:

- richer complex plugin-I/O breadth that remained advisory
- deeper LV2 worker, UI, patch, and URID scope
- fuller Linux live backend ownership breadth
- richer object rendering, room policy, or immersive deployment depth
- broader preview-browser or control-surface product-local workflows

This family is `required` as a visibility rule, but not as a success criterion
for the widened substrate itself.

## Required versus advisory versus deferred policy

Batch 20.1 freezes the following policy.

### Required

The final `g07` closeout gate must require:

- the bounded integrated acceptance lane from `g07.019`
- one machine-readable `g07` closeout descriptor and Effigy gate task
- explicit Loophole-facing readiness output tied back to reusable Signal
  evidence instead of product-local judgment
- explicit deferred and backlog posture so `g07` does not close ambiguously

### Advisory

The final gate may report but not block on:

- the advisory `complex-io` and `lv2` breadth still kept visible in
  `g07.019`
- repeated-run confidence passes over the bounded integrated lane
- broader local/server permutations or richer environment mixes that remain
  useful but not yet required

### Deferred

The final gate must keep explicitly deferred:

- exhaustive adapter, plugin, and environment certification matrices
- live backend-native Linux ownership breadth beyond the bounded proof seam
- richer immersive rendering, device-protocol, or preview-browser workflows
- Loophole product-launch readiness beyond reusable Signal substrate evidence

## Rules

### Rule 1: closeout remains additive over closed contracts

`g07` closeout may summarize the generation, but it must not invent new
semantic authority beyond what the closed milestone contracts and typed
receipts already support.

### Rule 2: readiness must stay explicit but narrow

Loophole-facing readiness must answer whether reusable runtime and feature
substrate improved, not whether Loophole is globally ready to ship.

### Rule 3: integrated acceptance remains the non-negotiable fast path

The final `g07` closeout gate must build on the bounded integrated lane from
`g07.019`; it must not bypass that lane with prose-only judgment or ad hoc
scenario bundles.

### Rule 4: deferred scope must stay visible

Unstable, product-local, or overly broad closeout depth must be recorded
explicitly rather than quietly omitted or smuggled into the required gate.

### Rule 5: final promotion stays repo-owned

The canonical closeout gate must remain in shared Signal descriptors and Effigy
tasks, not private CI or product-local scripts.

## Deferred scope

Batch 20.1 intentionally leaves these out:

- the concrete `g07` closeout descriptor and task implementation
- the final Loophole-facing readiness verdict
- any post-`g07` backlog or next-generation activation decision
- exact advisory rerun counts or environment inventories

## Batch 20.1 outcome

Batch 20.1 freezes the final policy shape for `g07` closeout:

- Signal now has one authority line for integrated acceptance, closeout-gate
  meaning, Loophole-facing feature readiness, and explicit deferred scope
- required, advisory, and deferred closeout evidence is explicit instead of
  collapsing the end of `g07` into a vague final review
- the `g07.019` integrated lane is now fixed as the non-negotiable base of the
  final gate
- later `g07.020` batches can now implement one machine-readable closeout
  surface and one explicit readiness verdict without reopening the policy
  question

## Batch 20.2 outcome

Batch 20.2 materializes the policy from Batch 20.1 into repo-owned shared
surfaces:

- `signal-supervisor-tools` now emits one machine-readable `g07` closeout
  descriptor through `--describe-generation-closeout`
- Effigy now owns one runnable `acceptance:g07-closeout` task that composes the
  grouped `g07` acceptance lane, the closeout descriptor proof, descriptor
  export, and repo validation
- the closeout descriptor now reports provisional Loophole-facing readiness
  areas and explicit residual risk while keeping the final promotion verdict
  deferred to Batch 20.3

This tranche does not close `g07`. It only guarantees the final review will
land on one typed, repo-owned gate instead of a manual summary.

## Batch 20.3 outcome

Batch 20.3 closes `g07` with one explicit repo-owned verdict:

- the machine-readable `g07` closeout descriptor now records `promote-g08`
  instead of a provisional review posture
- the readiness areas now resolve to sufficient-for-promotion across routing,
  Linux breadth, controller substrate, and sample-domain transform services
- explicit residual scope remains visible and is handed forward into the newly
  active `g08` queue instead of being hidden inside the closeout decision

This completes the bounded `g07` closeout contract. The verdict remains a
reusable Signal substrate verdict, not a Loophole product-launch verdict.

## Next Task

Continue `g08.001` with Batch 1.1 by freezing the runtime-owned live Linux
audio backend ownership and session-lifecycle contract before deeper ALSA,
JACK, and PipeWire runtime realization widens.
