# 080 Production Readiness Grade And Generation Release Gate Contract

Status: active
Owner: core-product
Updated: 2026-04-10
Related contracts: `docs/contracts/003-crate-maturity-and-public-runtime-boundary-baseline.md`, `docs/contracts/011-shared-downstream-conformance-and-release-acceptance-automation-contract.md`, `docs/contracts/071-generation-closeout-and-downstream-workflow-readiness-gate-contract.md`, `docs/contracts/079-interactive-demo-binary-and-crate-capability-proof-contract.md`
Related architecture: `docs/architecture/system-inventory.md`

## Purpose

Freeze the rule that `g09` does not close merely because audit-remediation work
and demo proof landed. `g09` only closes when the existing Signal crates have a
repo-owned production-readiness verdict for their intended role, the remaining
gaps are explicit, and the final generation gate is driven by shared evidence
instead of optimism or thread memory.

## Authority hierarchy

Production-readiness for `g09` has one authority chain:

1. `docs/contracts/003-crate-maturity-and-public-runtime-boundary-baseline.md`
   defines the current crate-role vocabulary: `public`,
   `consumer-facing but unstable`, and `internal`
2. the post-audit `g09` contracts (`072` through `079`) define the technical
   implementation and proof depth that those crates may claim today
3. `signal-runtime`, shared hosts, adapters, hardware crates, and
   `signal-supervisor-tools` own the typed receipts, descriptors, and proof
   surfaces that a readiness claim must summarize
4. Effigy tasks own the runnable gate policy that distinguishes:
   - required production-readiness evidence
   - advisory depth that improves confidence but does not yet block the gate
   - explicitly deferred scope that keeps `g09` open or forces a later queue
5. roadmap and strict-lane planning surfaces may sequence the work, but they do
   not override the contract-valid evidence chain above

If a crate or generation readiness claim cannot be explained through crate-role
policy, closed or active contracts, typed receipts, demo or acceptance proof,
and repo-owned Effigy tasks, it is not yet part of the production-readiness
boundary.

## Shared vocabulary

### Production-ready grade

`production-ready grade` means a repo-owned verdict that a crate is ready for
its intended production role inside Signal.

This is not a blanket semver promise or a product-launch verdict. It is a
bounded Signal-owned answer to whether the crate's current role is backed by
real implementation, proof, and explicit deferred scope.

### Role-correct readiness

`role-correct readiness` means readiness is evaluated against the crate's
intended role:

- `public` crates must be stable enough for direct downstream use through their
  documented public surface
- `consumer-facing but unstable` crates must be production-credible for shared
  integration while still allowed to evolve structurally
- `internal` crates must be production-credible as internal dependencies
  without being promoted to direct downstream contracts

### Required readiness evidence

`required readiness evidence` means proof that must remain green for a crate or
for `g09` to claim production-ready grade.

### Advisory readiness evidence

`advisory readiness evidence` means deeper confidence work that improves the
verdict but does not yet block the bounded `g09` closeout gate.

### Deferred readiness scope

`deferred readiness scope` means known work still needed before a crate or the
generation may claim production-ready grade. Deferred scope must stay explicit
rather than being implied by the absence of a proof lane.

## Readiness rubric

The reopened `g09` gate uses three explicit verdicts.

### Verdict 1: production-ready for role

Use this verdict when:

- the crate's intended role from contract `003` is still correct
- the core behavior for that role is implemented for real rather than scaffolded
- required proof for that role already exists in repo-owned tasks, receipts,
  demos, or focused validation
- any remaining limitations are advisory depth or explicitly out of role, not
  blockers for the intended production use

### Verdict 2: production-capable but blocked

Use this verdict when:

- the crate is materially implemented and may already be useful
- but one or more required gate conditions still fail:
  - no trustworthy production-readiness gate surface exists yet
  - the required validation/proof chain is still broken, missing, or too thin
  - an explicit technical or operational gap still contradicts a
    production-ready claim for the crate's intended role

This verdict keeps the crate inside the active `g09` burn-down set.

### Verdict 3: explicitly deferred or not ready

Use this verdict when:

- the crate's current role still depends on work not yet promoted into the
  active gate, or
- the remaining gap is intentionally deferred beyond the reopened `g09` scope

This verdict must name the deferred scope directly instead of implying it
through silence.

## Rules

### Rule 1: g09 closeout is stricter than audit-remediation completion

`g09` does not close only because the audit findings were addressed and demos
exist. It closes only when the existing crates have production-ready verdicts
for their intended role or are explicitly named as blocking deferred scope.

### Rule 2: readiness is per crate and per role

A single generation-level verdict must be explainable through per-crate or
per-family readiness classifications, not just one global feeling of maturity.

### Rule 3: proof must stay repo-owned

Readiness claims must compose from repo-owned tasks, descriptors, receipts,
tests, demos, or logs. External CI folklore or operator memory is not enough.

### Rule 4: deferred scope must remain explicit

If any existing crate is not yet production-ready for its intended role, that
gap must remain visible in the roadmap, logs, and final gate instead of being
smuggled past closeout.

### Rule 5: generation closeout gate stays additive

The final `g09` release/readiness gate must build on already-implemented
contracts and proof surfaces. It may summarize them, but it must not invent
new runtime behavior or readiness semantics at closeout time.

## Required proof surfaces

- one crate-by-crate or family-by-family readiness inventory for the current
  workspace
- one explicit readiness rubric tied back to crate roles from contract `003`
- one final `g09` gate that distinguishes required, advisory, and deferred
  readiness evidence
- explicit deferred-scope and next-generation posture when `g09` still cannot
  close

## Current gate baseline

The reopened `g09` gate uses one additive baseline until a later batch promotes
more of the workspace into the required lane.

### Required evidence

Required evidence is the minimum repo-owned proof set that must stay healthy for
the reopened gate itself to remain trustworthy:

- `effigy health`
- `effigy validate`
- `effigy qa:docs`
- `effigy qa:northstar`
- `effigy release gates`
- the focused boundary, demo, and receipt families that already justify any
  crate or crate-family marked `production-ready for role`

Required does not mean "run every acceptance task every time." It means the
generation may only promote a crate or family to production-ready when a
focused runnable proof chain exists, the repo-wide docs and build front doors
are healthy, and the workspace-wide compile surface no longer hides obvious
drift.

### Advisory evidence

Advisory evidence is shared and runnable depth that strengthens the gate but is
not yet the hard blocking baseline for the reopened generation:

- broader acceptance families under `effigy acceptance:*` that already validate
  specific runtime, host, plugin, hardware, DSP, graph, or analysis boundaries
- live demo launch tasks under `effigy demo:*`
- descriptor exports from `signal-supervisor-tools` that explain the same proof
  families in machine-readable form

Advisory evidence may later be promoted into the required lane, but only once
it is stable enough to function as a predictable release gate instead of a broad
confidence sweep.

### Deferred evidence

Deferred evidence is known and useful, but it does not yet block the reopened
`g09` verdict at the gate baseline:

- none at the repo-wide gate layer after the workspace validate repair

Remaining deferred scope now lives at the crate or family level: a crate may
still stay blocked because its focused proof chain or operational verdict is too
thin, but the workspace-wide validate surface is no longer itself deferred.

## Deferred scope

This contract does not itself implement the gate or fill every readiness gap.
It only freezes the rule that `g09` remains open until that work is completed
or explicitly re-scoped.

## 2026-07-27 Gate Baseline Reconciliation

The required evidence list named `effigy demo:coverage-matrix`, which does not
exist in this repository's task manifest. A gate baseline that names a task
nobody can run is not a gate, so it is replaced by `effigy release gates`, the
runnable gate set introduced for the `0.1.0` tag.

`effigy release gates` runs `fmt`, `lint`, `test`, `validate`, and `docs`. Two
of those are new: this workspace had no `cargo fmt --check` or `cargo clippy`
task at all before the release lane, so formatting and lint drift were unguarded
by any repo-owned command.

The `lint` gate deliberately does not deny warnings. The workspace carries `14`
known clippy warnings recorded as `g10.038` follow-up work, so denying would
block every release on pre-existing debt. As written the gate catches new clippy
errors only, and tightening it to `-D warnings` remains open.

## Next Task

Use this contract when reopening or auditing a future readiness gate after
next-generation planning.
