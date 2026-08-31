# 003 - Northstar Instruction And Rust Quality Audit

Status: active
Owner: core-product
Created: 2026-08-31
Updated: 2026-08-31
Depends on: g11.002
Vision tags: `QUALITY`, `MAINTAINABILITY`, `REALTIME`
Governing refs: `AGENTS.md`, `docs/contracts/001-working-rules.md`, `docs/contracts/rust-quality-profile.json`, `docs/contracts/rust-quality-deviations.json`, `docs/architecture/system-architecture.md`, `docs/architecture/product-guardrails.md`

## Problem

Signal's instruction surface was refreshed before the latest host and sandbox
work, and its Rust workspace has not been assessed as one repository-scope
Northstar explicit audit. The repository needs a current reader-journey review
and a finding-first Rust audit without turning maintenance into a product or
architecture rewrite.

## Goals

- audit and, where justified, tighten `AGENTS.md` and the root `CLAUDE.md`
  bridge while preserving Signal's project-specific boundaries
- assess every owned Rust package under Northstar's strict explicit-audit
  projection at the declared Rust 1.95 MSRV
- repair only recorded `review_required` findings inside the audit recorder's
  owned units
- leave report-only, unsafe, public-contract, foreign-error, version-policy,
  and other operator decisions visible rather than silently changing them
- finish with one reviewable PR and deterministic audit evidence

## Non-Goals

- opening `g12` or selecting a product backlog item
- redesigning realtime, plugin, IPC, hardware, or public API contracts
- blanket formatting, blanket lint fixing, dependency upgrades, or god-file
  demolition by threshold
- treating the AGENTS checker, Clippy, stopslop, or line counts as prose or
  architecture verdicts
- changing `.github/workflows/` or running release mutations

## Execution Plan

### Batch 3.1 - AGENTS And Rust Repository Audit

Status: ready

- review the full instruction reader journey and exact Claude bridge
- initialize the Rust audit recorder before source mutation
- partition all workspace crates into disjoint architecture-aligned units
- run correctness, architecture, and human-quality assessments for every unit
- apply only bounded recorder-authorized repairs, then finalize evidence and
  reconcile the planning/log surfaces

Card: `docs/roadmaps/g11/batch-cards/008-g11-003-northstar-agents-rust-audit.md`.

## Acceptance Criteria

- the AGENTS disposition covers every section and records preserved intent,
  before/after measurements, and the bridge result
- the Rust recorder covers the complete Cargo package/target/feature inventory,
  public APIs, unsafe/FFI, async/concurrency, realtime, and foreign-boundary risk
- every normative Rust rule has a verdict for every audit unit; every exact
  forwarder candidate has an explicit retain or report-only disposition
- only `review_required` repairs authorized before mutation change source;
  report-only and operator-decision surfaces remain unchanged
- Rust 1.95 floor evidence and repository-native current-toolchain validation
  are recorded honestly, including unavailable or warning-bearing evidence
- docs, card, log, and front doors agree on the result and next state

## Review Oracle

Invariant: the audit may improve instructions and recorder-authorized Rust
quality without weakening realtime safety, plugin isolation, public API/error
semantics, MSRV, or the audit's finding-first evidence chain.

Smallest adversarial counterexamples:

- a source edit appears before its unit assessment and repair plan
- an unsafe/FFI or public-contract finding is repaired under ordinary audit
  authority
- a passing newest-toolchain run is presented as proof of Rust 1.95 support
- one workspace crate, public surface, exact forwarder, or excluded file has no
  recorded disposition
- AGENTS becomes shorter by dropping a safety, authority, worktree, or
  completion boundary

Expected response: the worker stops before the unauthorized mutation or the
review rejects the PR. Required proof is the finalized recorder report,
changed-file attribution, preservation hashes, exact command evidence, AGENTS
section map, and clean final diff.

## Evidence Requirements

- opening and closeout log under `docs/logs/2026-08/`
- finalized Northstar Rust audit report from repository Git metadata, with the
  audit ID and catalogue hash recorded in the closeout log
- AGENTS advisory measurement plus human section disposition
- focused evidence for every repair and `effigy qa`, `effigy qa:docs`, and
  `effigy qa:northstar` results

## Stop Conditions

- the recorder cannot resolve or preserve repository scope
- a repair needs a new public API, foreign error, realtime, unsafe/FFI,
  compatibility, dependency, or version-policy decision
- a missing external contract prevents an honest assessment
- validation changes the plan or exposes work outside this maintenance lane

## Next Task

Execute card `008` in the dedicated worker lane. Stop after its PR is ready for
orchestrator review; do not infer another product or maintenance batch.
