# 011 Shared Downstream Conformance And Release-Acceptance Automation Contract

Status: active
Owner: core-product
Updated: 2026-03-13
Related contracts: `docs/contracts/003-crate-maturity-and-public-runtime-boundary-baseline.md`, `docs/contracts/005-runtime-work-orchestration-and-deferred-service-policy.md`, `docs/contracts/008-backend-neutral-plugin-capability-and-adapter-breadth-contract.md`, `docs/contracts/009-shared-host-convenience-api-and-consumer-edge-contract.md`, `docs/contracts/010-publication-grade-packaging-manifest-and-release-receipt-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first shared downstream conformance and release-acceptance
automation contract for `g05.004` so broader consumer confidence can grow from
repo-owned tasks, receipts, and descriptors instead of app-local orchestration
or downstream-specific CI policy.

## Authority hierarchy

Shared downstream automation has one authority chain:

1. Signal-owned contracts define what the workspace is allowed to claim:
   - runtime/export public boundary and schema promises
   - backend-neutral plugin breadth promises
   - shared host-edge stability promises
   - publication packaging and release-receipt promises
2. `signal-runtime` owns the typed profiling, soak, and supervisor receipt
   family that broader automation may inspect:
   - `RuntimeProfilingReceipt`
   - `RuntimeSoakReceipt`
   - `RuntimeObservationReport`
   - `RuntimeSupervisorReport`
3. `signal-supervisor-tools` owns the machine-readable consumer and release
   descriptors that explain the runnable automation boundary:
   - `--describe-conformance-matrix`
   - `--describe-host-edge-boundary`
   - `--describe-release-boundary`
   - `--describe-packaging-manifest`
4. Effigy tasks own the runnable automation policy:
   - which checks are mandatory for a shared release claim
   - which broader soak or conformance paths are optional depth
   - how those checks are grouped for maintainers and consumers
   - which current widened checks actually block a release claim versus remain
     deferred advisory depth
5. downstream consumers may invoke or archive the automation outputs, but they
   must not become the authority for what Signal considers mandatory release
   acceptance versus optional deeper confidence work

If a downstream check cannot be explained through Signal-owned contracts,
receipts, descriptors, or Effigy tasks, it is not yet part of the shared
automation boundary.

## Automation tiers

This milestone freezes a two-tier automation vocabulary.

### Mandatory release acceptance

Mandatory release acceptance is the bounded fast-path that must stay green for
Signal to claim the current shared consumer and release boundary.

The mandatory tier includes:

- `effigy acceptance:plugin-backend-breadth`
- `effigy acceptance:host-edge-consumer`
- `effigy acceptance:conformance`
- `effigy acceptance:release-boundary`
- `effigy acceptance:packaging-manifest`
- `effigy acceptance:release-packaging-consumer`
- `effigy acceptance:downstream-release`
- `effigy acceptance:downstream-gate`

Mandatory release acceptance may compose earlier tasks, but it must remain:

- repo-owned rather than downstream-owned
- bounded enough for maintainers to run deliberately and regularly
- aligned with explicit contracts rather than ad hoc scenario taste
- inspectable through typed receipts or machine-readable descriptors wherever
  possible

### Optional soak and confidence depth

Optional depth includes broader or longer-running shared checks that improve
confidence but are not yet required for every release claim.

The optional tier currently includes:

- `effigy acceptance:analysis`
- `signal-supervisor-tools` scenario runs that exercise `soak` or `mixed`
  watchdog/fault paths
- broader runtime profiling and soak receipt inspection beyond the current
  bounded release fast-path
- future longer-running downstream acceptance bundles that remain shared and
  repo-owned

Optional depth must still stay inside the Signal-owned contract boundary, but
it is allowed to be slower, broader, or more scenario-heavy than the mandatory
release path.

## Automation promises

The first downstream automation contract keeps four promises.

### Mandatory and optional depth stay distinct

Signal must not blur broader soak confidence into a hidden release gate. If a
check is required for the current shared release claim, it belongs in the
mandatory tier and should be surfaced as such through Effigy tasks and
descriptors. If it is broader, slower, or more exploratory, it should stay
optional until later policy explicitly promotes it.

### Automation stays additive over typed boundaries

Automation may exercise runtime, host-edge, backend-breadth, and packaging
surfaces together, but it must not create a new semantic authority. The checks
remain proofs over existing contracts and typed receipts, not substitutes for
them.

### Shared automation is repo-owned, not app-owned

Signal may support downstream consumers by publishing shared tasks and
descriptors, but it must not absorb consumer-specific CI ownership, product
workflow scripting, or private release wrappers into the canonical automation
boundary.

### Broader checks should prefer typed outputs

As automation deepens, broader checks should keep yielding typed receipts,
machine-readable descriptors, or structured summary outputs instead of only
log-scraping or human-only console review.

### Fail-gate policy must stay explicit

The first fail-gate policy uses three states:

- `required`: the check blocks the current shared release claim when it fails
- `advisory`: the check is shared and runnable, but it does not yet block the
  bounded release path
- `deferred`: the check is known and useful, but it is not yet stable enough to
  promote into either the required or advisory fast path

The current repo-owned fail-gate descriptor and task are:

- `cargo run -p signal-supervisor-tools -- --describe-downstream-fail-gates --format=json`
- `effigy acceptance:downstream-gate`

The first explicit policy is:

- `effigy acceptance:downstream-release` is `required`
- `effigy acceptance:downstream-depth` is `advisory`
- `cargo run -p signal-supervisor-tools -- --format=json server soak` is
  currently `deferred` because the recovery-overlap attach limit still trips
  that broader fixture
- `effigy acceptance:analysis` remains `deferred` for release gating
  even though it is shared and runnable

## Canonical automation order

Consumers and maintainers should inspect or run downstream automation in this
order:

1. run the mandatory bounded release-acceptance tier for shared boundary claims
2. inspect machine-readable descriptors that explain what those mandatory tasks
   prove
3. use typed profiling, soak, and supervisor receipts when broader scenario
   depth is needed
4. run optional soak/depth automation only when the bounded release tier is
   already explicit and healthy

This keeps release claims narrow and predictable while still leaving room for
shared broader confidence work.

## Deferred automation breadth

This Batch 4.1 contract intentionally defers:

- fail-gate policy for promoting optional depth into mandatory release gates
- long-duration soak schedules or repeated-run threshold policy
- downstream-specific CI ownership or environment matrices
- fleet, farm, or benchmark-cluster orchestration
- generation-closeout composition for `g05`, which belongs to `g05.005`

Those areas may build on this contract later, but they are not part of the
first shared downstream automation baseline.

## Current baseline surfaces

The current repo-owned baseline that this contract builds on is:

- `cargo run -p signal-supervisor-tools -- --describe-conformance-matrix --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-host-edge-boundary --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-release-boundary --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-packaging-manifest --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-downstream-automation --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-downstream-fail-gates --format=json`
- `effigy acceptance:plugin-backend-breadth`
- `effigy acceptance:host-edge-consumer`
- `effigy acceptance:conformance`
- `effigy acceptance:release-boundary`
- `effigy acceptance:packaging-manifest`
- `effigy acceptance:release-packaging-consumer`
- `effigy acceptance:downstream-release`
- `effigy acceptance:downstream-depth`
- `effigy acceptance:downstream-automation`
- `effigy acceptance:downstream-gate`
- `effigy acceptance:analysis`

## Next Task

COMPLETE. The shared downstream automation boundary is closed as part of the
completed `g05` generation. Promote
`docs/roadmaps/backlog/post-g05-publication-promotion-and-shared-acceptance-depth.md`
only when maintainers choose to open the post-`g05` generation.
