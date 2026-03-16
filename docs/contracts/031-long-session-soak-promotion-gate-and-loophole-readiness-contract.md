# 031 Long-Session Soak, Promotion Gate, And Loophole-Readiness Contract

Status: complete
Owner: core-product
Updated: 2026-03-16
Related contracts: `docs/contracts/030-fault-injection-harness-and-multi-backend-acceptance-contract.md`, `docs/contracts/016-runtime-fault-cause-attribution-and-diagnostic-receipt-contract.md`, `docs/contracts/019-deferred-work-scheduler-priority-backpressure-and-cancellation-contract.md`, `docs/contracts/022-backend-capability-parity-linux-plugin-support-and-cross-adapter-conformance-contract.md`, `docs/contracts/025-device-supervision-restart-state-machine-and-fault-boundary-contract.md`, `docs/contracts/027-external-io-monitoring-tap-point-and-loopback-measurement-contract.md`, `docs/contracts/028-media-indexing-waveform-analysis-and-preview-service-contract.md`, `docs/contracts/029-analysis-metadata-extraction-and-library-service-contract.md`, `docs/contracts/030-fault-injection-harness-and-multi-backend-acceptance-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the bounded `g06.020` closeout policy so long-session soak evidence,
promotion-gate meaning, and Loophole-facing readiness claims stay repo-owned,
typed, and additive over the now-closed `g06.019` integrated acceptance lane.

## Authority hierarchy

`g06` closeout has one authority chain:

1. closed `g06` contracts define the bounded runtime, adapter, hardware,
   external-I/O, media-service, and analysis-service claims that may be
   promoted at generation closeout
2. `signal-runtime` owns the typed receipts those claims must summarize:
   - `RuntimeObservationReport`
   - `RuntimeSupervisorReport`
   - `RuntimeProfilingReceipt`
   - `RuntimeSoakReceipt`
   - `RuntimeFaultDiagnosticReceipt`
   - `RuntimePerformanceSnapshot`
   - `RuntimePerformanceTraceReceipt`
   - `RuntimeDeviceSupervisionSnapshot`
   - `RuntimeExternalIoSnapshot`
   - `RuntimeMediaServiceSnapshot`
   - `RuntimeMediaLibraryServiceSnapshot`
3. shared host crates may contribute bounded local or server scenario evidence,
   but they do not own closeout meaning
4. `signal-supervisor-tools` owns the machine-readable closeout and readiness
   descriptors that explain:
   - which evidence bundles count toward the closeout gate
   - which evidence is required, advisory, or deferred
   - which Loophole-facing readiness claims are supported, blocked, or deferred
5. Effigy tasks own the runnable grouping policy for:
   - required integrated acceptance
   - bounded long-session soak
   - the final `g06` closeout gate
6. downstream consumers such as Loophole may archive or consume the evidence,
   but they must not redefine the canonical Signal closeout bar

If a `g06` closeout claim cannot be explained through closed contracts, typed
runtime receipts, supervisor-tools descriptors, and repo-owned Effigy tasks, it
is not yet part of the reusable closeout boundary.

## Existing anchors

This contract builds on the bounded integrated evidence already closed in
`g06.019`:

- `effigy acceptance:integrated-acceptance-lane`
- `cargo run -p signal-supervisor-tools -- --describe-integrated-acceptance-lane --format=json`

It also builds on the already-closed bounded boundary tasks composed by that
lane, especially:

- interruption, fault-diagnostic, critical-path, and deferred-work policy
- plugin continuity and cross-adapter parity
- device supervision, clock topology, and external-I/O
- media-service and analysis-metadata boundaries

Batch 20.1 does not claim the final closeout gate is implemented. It freezes
the policy the later closeout surface must obey.

## Shared vocabulary

### Long-session soak

`long-session soak` means a bounded, repo-owned endurance pass that exercises
the widened `g06` runtime surface long enough to detect stability regressions
that the fast integrated lane will not catch, without turning closeout into an
unbounded certification matrix.

It is not a product-local burn-in ritual, remote farm policy, or indefinite
manual confidence exercise.

### Promotion gate

`promotion gate` means the final repo-owned rule set that decides whether `g06`
has enough reusable evidence to claim closeout and hand the next work either to
`g07` or back into backlog.

### Loophole-facing readiness

`Loophole-facing readiness` means Signal's reusable answer to whether `g06`
materially improved Loophole's remaining runtime-hardening and feature-breadth
needs through shared substrate, not whether Loophole is product-launch ready.

### Required closeout evidence

`required closeout evidence` means evidence that must remain green for `g06` to
claim closeout.

### Advisory closeout evidence

`advisory closeout evidence` means shared evidence that materially improves the
closeout decision but does not yet block closeout.

### Deferred closeout evidence

`deferred closeout evidence` means known useful evidence that remains outside
the closeout gate because it is not yet bounded, stable, or portable enough.

## Closeout evidence families

Batch 20.1 freezes four closeout evidence families.

### Family 1: Bounded integrated acceptance

This family proves the widened `g06` surface still composes in one required
cross-family lane:

- `effigy acceptance:integrated-acceptance-lane`
- the machine-readable integrated-acceptance descriptor
- the focused cross-family export proof inside `signal-supervisor-tools`

This family is always `required`.

### Family 2: Long-session soak confidence

This family proves the widened surface survives a bounded longer-running pass:

- soak receipts
- profiling receipts
- repeated or longer-running watchdog and recovery evidence
- bounded mixed runtime-service pressure evidence

Batch 20.1 freezes this family as split policy:

- one future bounded long-session soak lane may become `required`
- broader rerun counts, repeated confidence passes, and unstable overlap-heavy
  server recovery scenarios stay `advisory` or `deferred`

### Family 3: Closeout descriptor and gate coherence

This family proves the generation closeout itself is inspectable:

- one machine-readable `g06` closeout descriptor
- one repo-owned Effigy closeout task
- one explicit required/advisory/deferred record

This family becomes `required` once implemented in Batch 20.2.

### Family 4: Loophole-facing readiness summary

This family explains whether `g06` moved Loophole forward on the pressures that
motivated the generation:

- runtime recovery and diagnostics confidence
- execution instrumentation and scheduler-policy confidence
- widened plugin-format and portability breadth
- hardware, external-I/O, and media-service substrate readiness

This family must be machine-readable and explicit, but Batch 20.1 keeps its
final review posture for Batch 20.3.

## Required versus advisory versus deferred policy

Batch 20.1 freezes the following policy.

### Required

The final `g06` promotion gate must require:

- the bounded integrated acceptance lane from `g06.019`
- one bounded long-session soak task or descriptor family that is repo-owned
  and typed
- one machine-readable `g06` closeout descriptor and Effigy gate task
- explicit Loophole-facing readiness output tied back to reusable Signal
  evidence instead of product-local judgment

### Advisory

The final gate may report but not block on:

- wider rerun counts over the bounded soak lane
- advisory continuity lanes already kept visible in `g06.019`
- broader local mixed watchdog or media-service confidence passes
- extra backend breadth checks that remain useful but not yet required

### Deferred

The final gate must keep explicitly deferred:

- unstable broader server-host recovery-overlap scenarios that still trip the
  current attach-limit constraint
- remote or distributed soak orchestration
- exhaustive environment or plugin certification matrices
- Loophole product-launch readiness beyond reusable Signal substrate evidence

## Rules

### Rule 1: closeout remains additive over closed contracts

`g06` closeout may summarize the generation, but it must not invent new
semantic authority beyond what the closed milestone contracts and typed receipts
already support.

### Rule 2: soak must stay bounded

Long-session evidence must stay runnable and bounded. If a soak claim requires
open-ended duration, operator babysitting, or environment-specific ceremony, it
is outside the shared closeout gate.

### Rule 3: readiness must be explicit but narrow

Loophole-facing readiness must answer whether reusable runtime and feature
substrate improved, not whether Loophole is globally ready to ship.

### Rule 4: deferred evidence must stay visible

Unstable or costly closeout depth must be recorded explicitly rather than
quietly omitted or smuggled into the required gate.

### Rule 5: final promotion stays repo-owned

The canonical closeout gate must remain in shared Signal descriptors and Effigy
tasks, not private CI or product-local scripts.

## Deferred scope

Batch 20.1 intentionally leaves these out:

- exact soak duration, repetition count, or scenario inventory for the later
  bounded soak lane
- the concrete `g06` closeout descriptor/task implementation
- the final Loophole-facing readiness verdict
- any post-`g06` backlog or `g07` promotion decision

## Batch 20.1 outcome

Batch 20.1 freezes the final policy shape for `g06` closeout:

- Signal now has one authority line for bounded soak, promotion-gate, and
  Loophole-facing readiness meaning
- required, advisory, and deferred closeout evidence is explicit instead of
  collapsing everything into a vague final checklist
- the integrated acceptance lane is now fixed as the non-negotiable fast-path
  base of the final gate
- later `g06.020` batches can now implement one machine-readable closeout
  surface and one bounded soak lane without reopening the policy question

## Batch 20.2 Outcome

Batch 20.2 materializes the bounded closeout policy as repo-owned runnable
surfaces instead of leaving it as contract prose:

- `signal-supervisor-tools` now exposes a machine-readable
  `signal.g06.long-session-soak-lane` descriptor
- the generation-closeout descriptor is now aligned to `g06` rather than the
  stale earlier generation shape
- Effigy now owns `acceptance:g06-soak-lane` and `acceptance:g06-closeout`

The required closeout spine is now concrete:

1. `effigy acceptance:integrated-acceptance-lane`
2. `effigy acceptance:g06-soak-lane`
3. `effigy acceptance:g06-closeout`

Batch 20.2 also keeps the contract's bounded policy honest:

- required local soak evidence is now explicit and runnable
- advisory integrated lane context remains visible instead of being hidden
  behind the soak descriptor
- unstable broader `server soak` depth remains explicitly deferred because the
  current recovery-overlap attach-limit issue still blocks promotion into the
  required closeout gate
- Loophole-facing readiness remains pending review for Batch 20.3 rather than
  being silently implied by the existence of the closeout task

## Batch 20.3 Outcome

Batch 20.3 records the final verdict this contract was meant to support:
`g06` is now sufficiently hardened and broadened at the reusable Signal
substrate level to promote `g07` into the active generation.

That verdict is grounded in the closeout descriptor rather than prose:

- the `g06` closeout surface now reports a concrete `promote-g07` decision
- all four Loophole-facing readiness areas now resolve to
  `sufficient-for-promotion`
- residual unstable `server soak` and broader advisory rerun depth remain
  explicit deferred scope rather than blocking the next generation

This contract therefore closes with the policy, runnable gate, and final
readiness decision all aligned to the same repo-owned surfaces.

## Next Task

Continue `g07.001` with Batch 1.1 by freezing the canonical multichannel
layout and channel-role contract before widening sidechain, spatial, Linux, or
time-stretch implementation depth.
