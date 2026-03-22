# 071 Generation Closeout And Downstream Workflow Readiness Gate Contract

Status: complete
Owner: core-product
Updated: 2026-03-22
Related contracts: `docs/contracts/066-cross-backend-device-protocol-and-live-workflow-acceptance-contract.md`, `docs/contracts/067-live-linux-backend-acceptance-and-failure-injection-contract.md`, `docs/contracts/068-immersive-render-and-monitoring-acceptance-contract.md`, `docs/contracts/069-control-surface-and-preview-workflow-acceptance-contract.md`, `docs/contracts/070-integrated-live-ownership-and-workflow-acceptance-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the bounded `g08.020` generation closeout and downstream workflow
readiness policy so the final `g08` verdict stays repo-owned, typed, and
additive over the now-closed `g08.019` integrated live-ownership and workflow
acceptance seam.

## Authority hierarchy

`g08` closeout has one authority chain:

1. closed `g08` contracts define the bounded live Linux, plugin protocol,
   immersive, device-workflow, preview-workflow, and integrated acceptance
   claims that may be promoted at generation closeout
2. `signal-runtime` owns the typed receipts those claims must summarize:
   - `RuntimeObservationReport`
   - `RuntimeSupervisorReport`
   - `RuntimeExecutionTopologySummary`
   - `RuntimeLinuxBackendSessionSnapshot`
   - `RuntimeJackCoordinationSnapshot`
   - `RuntimePipeWireAlsaParitySnapshot`
   - `RuntimeExternalMidiEndpointGraphSnapshot`
   - `RuntimeControlSurfaceSnapshot`
   - `RuntimeAdvancedHardwareSnapshot`
   - `RuntimePreviewTransformServiceSnapshot`
   - the richer spatial and plugin-chain receipt families already frozen by
     `g08`
3. shared host crates may contribute bounded local and server proof surfaces,
   but they do not own closeout meaning
4. `signal-supervisor-tools` owns the machine-readable `g08` closeout and
   downstream workflow readiness descriptors that explain:
   - which evidence bundles count toward the final gate
   - which evidence is required, advisory, or deferred
   - which downstream workflow readiness claims are supported, guarded, or
     still deferred
5. Effigy tasks own the runnable grouping policy for:
   - the required integrated `g08.019` acceptance lane
   - the final `g08` closeout gate
   - any advisory rerun lanes promoted into the repo-owned closeout surface
6. downstream consumers may archive or consume the resulting evidence, but
   they must not redefine the canonical Signal closeout bar

If a `g08` closeout claim cannot be explained through closed contracts, typed
runtime receipts, supervisor-tools descriptors, and repo-owned Effigy tasks,
it is not yet part of the reusable closeout boundary.

## Existing anchors

This contract builds on the bounded integrated evidence already closed in
`g08.019`:

- `effigy acceptance:integrated-live-ownership-and-workflow-acceptance-lane`
- `cargo run -p signal-supervisor-tools -- --describe-integrated-live-workflow-acceptance-lane --format=json`
- `cargo test -p signal-supervisor-tools export_json_carries_cross_family_integrated_live_workflow_acceptance_evidence`

It also builds on the grouped acceptance seams composed by that lane:

- Linux live ownership, JACK coordination, PipeWire/ALSA parity, and
  clock-topology continuity
- device workflow, controller-expression breadth, and advanced hardware or
  control-surface workflow posture
- immersive room-policy, deployment-monitoring, and renderer-export posture
- preview-device policy and preview-workflow posture

Batch 20.1 does not claim the final closeout gate is implemented. It freezes
the policy the later closeout surface must obey.

## Shared vocabulary

### Generation closeout gate

`generation closeout gate` means the final repo-owned rule set that decides
whether `g08` has enough reusable evidence to claim closeout and hand the next
work either to backlog hardening, a new active generation, or a clearly named
deferred queue.

### Downstream workflow readiness

`downstream workflow readiness` means Signal's reusable answer to whether
`g08` materially improved downstream live ownership, immersive monitoring,
device workflow, and preview workflow needs through shared runtime substrate,
not whether any product is globally ready to ship.

### Required closeout evidence

`required closeout evidence` means evidence that must remain green for `g08`
to claim closeout.

### Advisory closeout evidence

`advisory closeout evidence` means shared evidence that materially improves
the closeout decision but does not yet block closeout.

### Deferred closeout evidence

`deferred closeout evidence` means known useful evidence that remains outside
the closeout gate because it is not yet bounded, stable, portable, or worth
promoting into the shared fast path.

## Closeout evidence families

Batch 20.1 freezes four closeout evidence families.

### Family 1: Integrated acceptance substrate

This family proves the widened `g08` surface still composes in one required
cross-family lane:

- `effigy acceptance:integrated-live-ownership-and-workflow-acceptance-lane`
- the machine-readable grouped integrated acceptance descriptor
- the focused cross-family export proof inside `signal-supervisor-tools`

This family is always `required`.

### Family 2: Closeout descriptor and gate coherence

This family proves the generation closeout itself is inspectable:

- one machine-readable `g08` closeout descriptor
- one repo-owned Effigy closeout task
- one explicit required, advisory, and deferred record

This family becomes `required` once implemented in Batch 20.2.

### Family 3: Downstream workflow readiness summary

This family explains whether `g08` moved downstream reusable workflow forward
on the pressures that motivated the generation:

- live Linux ownership, backend-native coordination, and guarded continuity
  readiness
- immersive room-policy, deployment-monitoring, and renderer-export readiness
- external MIDI, control-surface, advanced hardware, and grouped device
  workflow readiness
- preview-device policy, queue posture, audition continuity, and transform
  scheduling readiness

This family must be machine-readable and explicit, but Batch 20.1 keeps its
final review posture for Batch 20.3.

### Family 4: Explicit deferred and backlog posture

This family records what `g08` intentionally does not claim at closeout:

- exhaustive distro, daemon, renderer-vendor, device-vendor, and browser UX
  certification matrices
- deeper LV2, product-local controller page, browser queue editor, or
  immersive console workflows
- richer repeated-run or environment-specific acceptance depth
- any downstream launch or distribution verdict beyond reusable Signal
  substrate evidence

This family is `required` as a visibility rule, but not as a success criterion
for the widened substrate itself.

## Required versus advisory versus deferred policy

Batch 20.1 freezes the following policy.

### Required

The final `g08` closeout gate must require:

- the bounded integrated acceptance lane from `g08.019`
- one machine-readable `g08` closeout descriptor and Effigy gate task
- explicit downstream workflow readiness output tied back to reusable Signal
  evidence instead of product-local judgment
- explicit deferred and backlog posture so `g08` does not close ambiguously

### Advisory

The final gate may report but not block on:

- broader repeated-run confidence passes over the bounded integrated lane
- richer local/server permutations or environment mixes that remain useful but
  not yet required
- additional downstream workflow scenarios that remain informative but not yet
  stable enough for the fast path

### Deferred

The final gate must keep explicitly deferred:

- exhaustive adapter, backend, device, renderer, and environment
  certification matrices
- product-local controller, browser, or immersive workflow shells
- downstream launch or distribution readiness beyond reusable Signal evidence
- post-`g08` queue activation decisions that are not yet grounded in the
  closeout verdict

## Rules

### Rule 1: closeout remains additive over closed contracts

`g08` closeout may summarize the generation, but it must not invent new
semantic authority beyond what the closed milestone contracts and typed
receipts already support.

### Rule 2: readiness must stay explicit but narrow

Downstream workflow readiness must answer whether reusable runtime and
workflow substrate improved, not whether any downstream product is globally
ready to ship.

### Rule 3: integrated acceptance remains the non-negotiable fast path

The final `g08` closeout gate must build on the bounded integrated lane from
`g08.019`; it must not bypass that lane with prose-only judgment or ad hoc
scenario bundles.

### Rule 4: deferred scope must stay visible

Unstable, product-local, or overly broad closeout depth must be recorded
explicitly rather than quietly omitted or smuggled into the required gate.

### Rule 5: final promotion stays repo-owned

The canonical closeout gate must remain in shared Signal descriptors and
Effigy tasks, not private CI or product-local scripts.

## Deferred scope

Batch 20.1 intentionally leaves these out:

- the concrete `g08` closeout descriptor and task implementation
- the final downstream workflow readiness verdict
- any post-`g08` backlog or next-generation activation decision
- exact advisory rerun counts or environment inventories

## Batch 20.1 outcome

Batch 20.1 freezes the final policy shape for `g08` closeout:

- Signal now has one authority line for integrated acceptance, closeout-gate
  meaning, downstream workflow readiness, and explicit deferred scope
- required, advisory, and deferred closeout evidence is explicit instead of
  collapsing the end of `g08` into a vague final review
- the `g08.019` integrated lane is now fixed as the non-negotiable base of
  the final gate
- later `g08.020` batches can now implement one machine-readable closeout
  surface and one explicit readiness verdict without reopening the policy
  question

## Batch 20.2 outcome

- `signal-supervisor-tools` now emits one machine-readable `g08` closeout
  descriptor through `--describe-generation-closeout`
- Effigy now owns one runnable `acceptance:g08-closeout` task that composes
  the closed `g08.019` integrated lane, the closeout descriptor proof, the
  descriptor export, and repo validation
- the closeout descriptor now reports provisional downstream workflow
  readiness areas and explicit residual risk while keeping the final closeout
  verdict deferred to Batch 20.3

This tranche does not close `g08`. It guarantees the final review will land on
one typed, repo-owned gate instead of a manual summary.

## Batch 20.3 outcome

- the machine-readable `g08` closeout descriptor now records the final bounded
  closeout verdict instead of a review-active placeholder
- the closeout surface now points at
  `docs/roadmaps/backlog/post-g08-repeated-run-environment-matrices-and-downstream-workflow-depth.md`
  as the explicit post-`g08` candidate queue instead of a self-referential
  placeholder
- `g08` now closes with one repo-owned answer: bounded downstream workflow
  readiness is sufficient for closeout, while broader repeated-run confidence,
  environment matrices, and product-local workflows remain explicit backlog or
  deferred scope

## Next Task

COMPLETE. `g08` is closed. Promote
`docs/roadmaps/backlog/post-g08-repeated-run-environment-matrices-and-downstream-workflow-depth.md`
only when maintainers choose to open the post-`g08` generation.
