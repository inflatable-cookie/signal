# 014 - Production Readiness Grade And Generation Release Gate

Status: active
Owner: core-product
Created: 2026-04-10
Depends on: g09.013
Vision tags: `READINESS`, `RELEASE`, `CLOSEOUT`
Contract refs: `003`, `011`, `080`

## Problem

`g09` cannot honestly close on audit-remediation and demo proof alone if the
existing Signal crates are not yet production-ready for their intended role.
The generation needs one explicit readiness rubric, one gap inventory, and one
final gate that says which crates are truly ready, which still need work, and
what remains intentionally deferred.

## Goals

- [ ] define a role-correct production-readiness rubric for the existing crates
- [ ] inventory every active crate against that rubric using repo-owned proof
      and explicit gaps
- [ ] burn down the blocking gaps or reclassify them explicitly
- [ ] close `g09` only when the generation can support a repo-owned
      production-ready verdict

## Non-Goals

- [ ] no semver or crates.io publication policy redesign
- [ ] no downstream product-launch verdict
- [ ] no new feature-expansion queue disguised as readiness work

## Execution Plan

### Batch 14.1 - Readiness Rubric And Gap Inventory

- [ ] define the production-ready grading rubric tied to crate roles from
      contract `003`
- [ ] classify every active crate with required, advisory, and deferred
      readiness evidence
- [ ] identify which crates are already acceptable, which are blocked, and
      which need more proof or implementation

### Batch 14.2 - Blocking Gap Burn-Down

- [ ] group the blocking crates into a few high-leverage implementation or
      proof batches
- [ ] keep the burn-down tied to concrete receipts, validation, and role-correct
      readiness claims instead of aesthetic cleanup

#### Batch 14.2 Tranche 1 Outcome

- defined the first repo-owned `g09` production-readiness gate baseline
- froze the current gate into three evidence classes:
  - `required`
    - `effigy health`
    - `effigy validate`
    - `effigy qa:docs`
    - `effigy qa:northstar`
    - `effigy demo:coverage-matrix`
    - focused runnable proof families for any crate or family promoted as
      `production-ready for role`
  - `advisory`
    - broader `effigy acceptance:*` families
    - live demo launch tasks beyond the coverage matrix
    - machine-readable descriptor exports that explain the same boundary claims
  - `deferred`
    - none at the repo-wide gate layer after the validate repair
- recorded the validate wall as an explicit blocker to be repaired before the
  next crate-family verdicts could become honest

#### Batch 14.2 Tranche 2 Ready Posture

- the next highest-leverage seam is to repair the workspace validate surface
- that batch will fix the stale split test-module tree and related host-test
  import drift so `effigy validate` can become a trustworthy gate input
- later burn-down batches can then promote or reject blocked crate groups
  against a cleaner required validation baseline

#### Batch 14.2 Tranche 2 Outcome

- repaired the stale split test-module and host-test import drift in
  `signal-host-local` and `signal-host-server`
- restored both `cargo test --workspace --no-run` and `effigy validate` as
  trustworthy runnable gate surfaces
- promoted `effigy validate` from deferred evidence into the required gate
  baseline
- kept the remaining repo state honest: warnings still exist, but the
  workspace-wide validate wall is no longer broken or deferred

#### Batch 14.2 Tranche 3 Ready Posture

- the next highest-leverage seam is the plugin, broker, and IPC family verdict
- that batch will use the repaired gate to decide which plugin adapters,
  broker surfaces, and transport surfaces can now be promoted to
  `production-ready for role`
- later batches can then focus the remaining runtime, host, and hardware
  families against a gate that is both explicit and runnable

#### Batch 14.2 Tranche 3 Outcome

- promoted the plugin abstraction and adapter family to
  `production-ready for role`:
  - `signal-plugin`
  - `signal-plugin-clap`
  - `signal-plugin-vst3`
  - `signal-plugin-au`
  - `signal-plugin-lv2`
  - `signal-ipc`
- kept `signal-plugin-sandbox` blocked on one explicit remaining operational
  gap: there is still no repo-owned long-lived broker production verdict beyond
  the bounded lifecycle, receipt, and demo surfaces already in place
- narrowed the remaining reopened `g09` burn-down to the
  runtime/host/hardware/supervisor family plus the broker-operational verdict
  that still spans `signal-plugin-sandbox`

#### Batch 14.2 Tranche 4 Ready Posture

- the next highest-leverage seam is the remaining operational family verdict
- that batch will classify `signal-runtime`, `signal-host-local`,
  `signal-host-server`, `signal-hardware`, `signal-hardware-coreaudio`,
  `signal-supervisor-tools`, and the still-blocked `signal-plugin-sandbox`
- that should expose whether one final burn-down seam remains after the
  runtime/host/hardware/broker family is judged against the repaired gate

### Batch 14.3 - Final Release Gate

- [ ] define the repo-owned `g09` production-readiness gate
- [ ] record the final generation verdict and any truly deferred post-`g09`
      scope explicitly

## Acceptance Criteria

- [ ] every active crate has an explicit production-readiness posture
- [ ] blocking gaps are inventory-backed rather than implied
- [ ] `g09` closes only on a repo-owned production-ready verdict or an explicit
      decision to keep the generation open

## Evidence Requirements

- [ ] log each meaningful readiness batch
- [ ] run the validation actually used to justify readiness claims
- [ ] keep the crate inventory and deferred-scope record current as the gate
      evolves

## Batch 14.1 Outcome

Batch 14.1 reopens `g09` on one explicit production-readiness rubric and the
first per-crate verdict inventory.

### Readiness rubric

- `production-ready for role`
  - the crate's intended role is still correct
  - real implementation exists for that role
  - required proof for that role already exists through repo-owned validation,
    receipts, or demos
- `production-capable but blocked`
  - the crate is materially implemented, but a required gate condition still
    fails: missing or broken proof, missing release/readiness gate wiring, or a
    remaining technical/operational blocker
- `explicitly deferred or not ready`
  - the crate still depends on scope intentionally outside the reopened `g09`
    gate

### Per-crate readiness inventory

#### Production-ready for role

- `signal-primitives`
- `signal-dsp`
- `signal-dsp-resample`
- `signal-dsp-spectral`
- `signal-analysis`
- `signal-analysis-rhythm`
- `signal-analysis-tonal`
- `signal-analysis-loudness`
- `signal-analysis-character`
- `signal-analysis-embed`
- `signal-graph`
- `signal-ipc`
- `signal-plugin`
- `signal-plugin-clap`
- `signal-plugin-vst3`
- `signal-plugin-au`
- `signal-plugin-lv2`

#### Production-capable but blocked

- `signal-runtime`
  - blocked on the missing repo-owned production-readiness gate and release
    verdict for its public/runtime role
- `signal-supervisor-tools`
  - blocked on the same missing release/readiness gate surface it must help
    describe
- `signal-host-local`
  - blocked on release-grade gate and required validation posture, not on the
    old CLAP bootstrap gap
- `signal-host-server`
  - blocked on release-grade gate and required validation posture
- `signal-hardware`
  - blocked on explicit role-correct production verdict for backend-neutral
    hardware posture
- `signal-hardware-coreaudio`
  - blocked on production-grade backend gate depth beyond the bounded macOS
    proof lane
- `signal-plugin-sandbox`
  - blocked on an explicit long-lived broker production verdict rather than the
    bounded lifecycle, receipt, and demo posture alone

#### Explicitly deferred or not ready

- none inside the existing crate set for Batch 14.1
- deferred post-`g09` scope still exists for `signal.demo.plugin.capability-browser`,
  but that is a demo-surface gap rather than a separate workspace crate verdict

### First blocking-gap groups

- group A: release gate and required/advisory/deferred evidence wiring for the
  reopened `g09` verdict
- group B: plugin and broker edge production-depth proof for
  `signal-plugin*`, `signal-plugin-sandbox`, and `signal-ipc`
- group C: host/runtime/hardware operational readiness verdict for
  `signal-runtime`, `signal-host-*`, `signal-hardware`, and
  `signal-hardware-coreaudio`

### Batch 14.2 Tranche 1 Ready Posture

- the next highest-leverage seam is group A: the release-gate baseline
- it will define the runnable required/advisory/deferred production-readiness
  evidence set and repair any gate-surface or proof-wiring drift needed for an
  honest repo-owned verdict
- later burn-down batches can then narrow the still-blocked plugin and
  host/runtime/hardware groups against a stable gate instead of an implied one

## Next Task

Continue the reopened strict `g09` lane from
`docs/specs/batch-cards/039-g09-014-runtime-host-hardware-broker-operational-verdict.md`.
