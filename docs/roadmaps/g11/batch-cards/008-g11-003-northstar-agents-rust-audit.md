# 008 - g11.003 Northstar AGENTS And Rust Audit

Status: ready
Owner: core-product
Created: 2026-08-31
Updated: 2026-08-31
Master spec refs: none (baseline-routed maintenance lane)
Roadmap refs: g11.003
Governing refs: AGENTS.md, CLAUDE.md, docs/contracts/001-working-rules.md, docs/contracts/rust-quality-profile.json, docs/contracts/rust-quality-deviations.json, docs/architecture/system-architecture.md, docs/architecture/system-inventory.md, docs/architecture/product-guardrails.md, docs/roadmaps/g11/003-northstar-instruction-and-rust-quality-audit.md
Auto-start next card: no
Depends on: 007-g11-002-continuity-proof-and-closeout.md

## Objective

Run one repository-scope Northstar instruction and strict Rust audit, apply only
pre-authorized finding-first repairs, and return a reviewable PR with complete
preservation and validation evidence.

## Scope

- review `AGENTS.md` as a complete reader journey and `CLAUDE.md` as its exact
  bridge; optimize only when the reader need and preserved boundary are clear
- audit the root Cargo workspace and all 28 member crates at repository scope
- use the installed Northstar Rust tool bootstrap, strict projection, audit
  recorder, pinned stopslop scanner, and evidence collector
- inventory public API, panic/error paths, unsafe/FFI, async/concurrency,
  realtime paths, Cargo targets/features, and exact-forwarder candidates
- partition the workspace into disjoint architecture-aligned units; finish one
  assessment before any repair in that unit
- update this card and write the closeout log with the finalized audit result

Out of scope: product features, architecture replacement, compatibility shims,
dependency or toolchain changes, public API/error-policy decisions, ordinary
unsafe repair, CI workflow edits, release work, and threshold-led god-file
splitting.

## Ordered Work

1. Capture clean Git state, advisory AGENTS measurements, Cargo inventory,
   profile/deviations, MSRV 1.95, current toolchain, and repository contracts.
2. Build the AGENTS section-intent map and record retain/rewrite/reorder/move/
   remove/investigate dispositions before editing the instruction surfaces.
3. Bootstrap and verify the Northstar Rust audit tool and stopslop 0.5.1.
   Inspect, plan, and initialize repository scope before source assessment or
   mutation. Give every discovered package, target, feature, risk boundary, and
   dirty file an owned or excluded disposition.
4. Assess each unit in separate correctness, architecture, and human-quality
   passes. Record all rule verdicts and complete exact-forwarder ledger before
   planning any repair.
5. Apply only recorder-authorized `review_required` repair plans. Extend scope
   before touching a caller, test, doc, or contract outside the unit. Preserve
   report-only, operator-decision, read-only, and excluded files byte-for-byte.
6. Collect exact evidence, complete every unit, finalize the recorder, rerun
   AGENTS measurements, run repository QA, and reconcile card/log/front doors.
7. Falsify the diff against the review oracle, push the worker branch, and open
   a PR to `main`. Do not merge.

## Acceptance Criteria

- [ ] every AGENTS section has a human disposition and its reader need survives
- [ ] `CLAUDE.md` is exactly the required bridge unless a real Claude-only rule
  is evidenced
- [ ] the Rust audit is initialized before source mutation and finalized once
- [ ] all 28 crates and their discovered targets/features have a recorded unit
  or explicit exclusion; public API and risk surfaces are named
- [ ] every approved Rust rule has one verdict per unit and all assessment
  dimensions are attested
- [ ] every stopslop/manual exact-forwarder candidate has a retain or
  report-only disposition and independent `RUST-READ-001` assessment
- [ ] source changes map only to prior `review_required` plans and passing
  immutable evidence IDs; protected files pass preservation checks
- [ ] Rust 1.95 evidence is separate from the pinned current toolchain result
- [ ] closeout names findings, repairs, retained limitations, audit ID,
  catalogue hash, changed files, evidence, and honest remaining stops
- [ ] `effigy qa`, `effigy qa:docs`, `effigy qa:northstar`, and
  `git diff --check` are recorded with actual results

## Review Oracle

Use the milestone oracle. In review, try the five counterexamples there first,
then sample at least one unit from each architecture family: production audio,
DSP, analysis, control plane/IPC, plugin/FFI, and hardware/host edge. Reconcile
the finalized recorder's changed-file union with the Git diff exactly.

## Evidence Required

- advisory AGENTS before/after output and section-intent disposition
- finalized audit `report.md` and `result.json` in Git metadata, referenced by
  audit ID and hashes from the closeout log
- evidence records for compiler, lint, docs, tests, graph where used, and
  scanner; unavailable or unrun classes remain explicit limitations
- `docs/logs/2026-08/31-g11-003-northstar-agents-rust-audit-closeout.md`

## Stop Conditions

- profile, deviations, package inventory, MSRV, toolchain, or recorder state
  cannot be resolved consistently
- a public contract, foreign error policy, unsafe boundary, realtime invariant,
  compatibility policy, dependency, or version floor must change
- a missing sibling/external authority is necessary to judge or repair a seam
- scope extension would overlap another unit or validation changes the plan

## Next Task

Run this card in the dedicated worker lane. When the PR is ready, stop for
orchestrator exact-head review; do not start another card.
