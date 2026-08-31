---
title: Signal Northstar AGENTS and Rust audit worker handoff
kind: northstar-handoff
handoff_mode: worker-pr-loop
worker_mode: implementation
dispatch_authority: orchestrator
handoff: single-file-path-only
status: ready-to-launch
owner: Tom / Northstar orchestrator
created: 2026-08-31
updated: 2026-08-31
handoff_path: /Users/tom/Dev/projects/signal/docs/handoffs/20260831-232427-northstar-agents-rust-audit.md
base_required: pushed-main
tags: [coordination, handoff, worker, pr, audit]
---

## What This Thread Was Doing

The operator selected Signal for the next project-by-project Northstar AGENTS
and language-quality audit while an independent Nucleus audit continues. The
orchestrator confirmed Signal has no live project orchestrator, resolved its
Rust-only owned language surface, and opened one bounded `g11.003` maintenance
lane.

This dispatches card `008` as one worker lane. No transcript or second prompt
is part of the authority chain.

## Why It Matters

Signal's always-loaded instructions need to stay useful after recent host and
sandbox work, and the complete 28-crate workspace needs one finding-first audit
against its realtime, plugin, public API, FFI, and Rust 1.95 boundaries. The
result must be trustworthy maintenance evidence, not threshold-driven churn.

## Current State

- **Repository:** `/Users/tom/Dev/projects/signal`
- **Planning branch:** `main`
- **Planning base commit:** `becc558a22e0b88961b1966d8db89efb4a05a138`
- **Pushed main verification:** local `HEAD` and `origin/main` both resolved to
  that SHA before this handoff was created
- **Planning checkout:** clean before this handoff file was created
- **Worker mode:** implementation worker dispatched by the orchestrator; this
  handoff activates the worker-only worktree preflight
- **Planning artifacts included at the base:** `g11.003`, card `008`, and the
  opening log
- **Worker branch:** `worker/northstar-agents-rust-audit`
- **Worker worktree:** Paseo launcher-managed worktree first
- **Worktree creation:** Paseo `branch-off` from pushed `origin/main`
- **Required sibling worktree links:** none
- **Active spec lane:** none; this is baseline-routed maintenance
- **Roadmap milestone:**
  `docs/roadmaps/g11/003-northstar-instruction-and-rust-quality-audit.md`
- **Ready cards, in order:**
  `docs/roadmaps/g11/batch-cards/008-g11-003-northstar-agents-rust-audit.md`
- **Allowed runway:** card `008` only
- **Remaining card budget:** one card, then PR and stop
- **Dispatch topology:** parallel with the independent Nucleus repository audit
- **Parallel safety:** different repository, branch, worktree, planning spine,
  and PR; no shared mutable scope
- **Canonical refs:** `AGENTS.md`; `CLAUDE.md`;
  `docs/architecture/system-architecture.md`;
  `docs/architecture/system-inventory.md`;
  `docs/architecture/product-guardrails.md`;
  `docs/contracts/001-working-rules.md`;
  `docs/contracts/rust-quality-profile.json`;
  `docs/contracts/rust-quality-deviations.json`
- **Review oracle:** milestone `g11.003` and card `008`
- **Model capability profile:** frontier worker with high reasoning, selected
  from current Paseo profile notes at dispatch
- **Tool/runtime restrictions:** use the installed Northstar AGENTS review and
  Rust explicit-audit modes; initialize the recorder before source mutation;
  do not install project dependencies, change toolchains/dependencies, blanket
  format/fix, edit workflows, or run release mutations
- **Required validation:** finalized Rust recorder evidence; advisory AGENTS
  before/after evidence; focused repair evidence; `effigy qa`;
  `effigy qa:docs`; `effigy qa:northstar`; `git diff --check`
- **PR base/head:** `main` ← worker branch
- **PR URL:** pending
- **Review state:** awaiting worker PR
- **Merge path:** orchestrator after accepted review of the current head and
  passing required checks

## Boundaries

Please keep this run inside card `008`.

- **In scope:** full AGENTS/CLAUDE reader-journey review; repository-scope Rust
  audit; recorder-authorized `review_required` repairs; audit/card/log/front-
  door closeout
- **Out of scope:** product features, new architecture, public API or foreign
  error policy changes, ordinary unsafe/FFI repair, realtime contract changes,
  compatibility shims, dependency/toolchain updates, CI workflow edits, release
  work, and threshold-led god-file splitting
- **Outcome:** audit-and-repair where the recorder grants authority; honest
  report-only or operator-decision limitations everywhere else
- Do not widen the roadmap, open `g12`, choose a product backlog item, or touch
  the independent Nucleus lane.
- Work only in the launcher-selected clean worker worktree. Never edit the
  planning checkout or another Signal worktree.
- Do not merge. Merge belongs to this orchestrator after exact-head review.

## Important Context

- Signal is baseline-routed. `g11.001` and `g11.002` are complete; this lane
  does not reopen a strict spec.
- The root package file is tooling-only. Signal owns no TypeScript package for
  this audit; do not manufacture a TypeScript lane.
- The Rust profile already declares strict repository scope and Rust 1.95.
  Current toolchain 1.97.1 is separate evidence, not the floor proof.
- `effigy doctor` currently reports the existing god-file threshold baseline,
  a stale graph index, and attention-marker warnings. Treat them as leads or
  limitations, not automatic repair authority.
- The installed AGENTS advisory measured 111 non-blank lines and an exact
  Claude bridge. Its leads are context-cost evidence, not prose verdicts.
- `docs/triage/20260829-224753-stale-next-task-pointers.md` remains a separate
  open docs-currentness item. Do not absorb it merely because the audit reads
  those surfaces.
- Signal has no local Chorus checkout. If an IPC judgment truly requires the
  external Chorus guardrail and Signal's own contracts do not settle it, record
  the limitation and stop that repair rather than inventing or fetching policy.
- Report after scope/recorder initialization, after each coherent assessed and
  repaired architecture family, and at PR readiness.
- **Report to:** the orchestrator through Paseo notifications.

## Suggested Next Move

Run the worktree preflight before broad reads. Then load card `008`, the named
canonical refs, and the installed Northstar AGENTS-review and Rust explicit-
audit contracts. Freeze repository scope and initialize the recorder before
assessing or editing Rust. Work architecture family by architecture family and
keep report-only surfaces byte-identical.

## Completion Protocol

### Before you start

1. This handoff's `worker_mode: implementation` and
   `dispatch_authority: orchestrator` activate worker mode. Before broad reads,
   run `git rev-parse --show-toplevel`, `git branch --show-current`,
   `git status --porcelain`, and `git worktree list --porcelain`.
2. If the current root is a registered worktree, status is empty, and the
   branch is not `main`, accept it as launcher-provided. Record its actual
   root/branch and do not create another worktree because its generated path or
   branch differs from the placeholders above.
3. If the launcher supplied a dirty or `main` worktree, stop and report it.
   Only a manual fallback may read `.agents.local.env` and require
   `AGENTS_WORKTREE_CONTAINER_DIR`; never use `/tmp`, guess a path, or clean,
   reset, stash over, or discard existing state.
4. From the selected worktree, record this repository-relative path:
   `docs/handoffs/20260831-232427-northstar-agents-rust-audit.md`. Run
   `GIT_SSH_COMMAND="ssh -o ConnectTimeout=10 -o BatchMode=yes" git fetch origin`.
   Confirm `HEAD == origin/main`, confirm planning base
   `becc558a22e0b88961b1966d8db89efb4a05a138` is an ancestor, and load the
   tracked handoff from `HEAD`. If it differs from the absolute dispatch file,
   stop. The tracked blob is canonical.
5. Required sibling links are `none`; skip link setup.
6. Read card `008`, `AGENTS.md`, `PAPERCUTS.md`, and the named canonical refs.
7. Run the repo's cheap orientation checks and record what actually ran.

### While you work

- Follow the installed Northstar AGENTS review and Rust explicit-audit modes,
  including their complete setup, recorder, scanner, evidence, preservation,
  and papercut contracts.
- Assess before editing. A source repair needs a prior finding, derived
  `review_required` authority, and bounded repair plan in the recorder.
- Run three distinct passes per unit: correctness/assurance, architecture, and
  human quality. Record one verdict for every approved rule and a total
  exact-forwarder ledger.
- Extend recorder ownership before touching an outside caller, test, doc, or
  contract. Preserve report-only, operator-decision, read-only, and excluded
  files byte-for-byte.
- Report meaningful chunks through Paseo with changed files, evidence actually
  run, remaining units, limitations, and blockers.
- Stop and return a planning choice if scope, contract, API/error policy,
  unsafe/FFI, realtime, compatibility, dependency, or MSRV authority changes.

### When the assigned runway is complete

1. Complete every unit and finalize the recorder once. Run the card's final
   validation and record actual results, warnings, unavailable classes, and
   limitations.
2. Falsify the diff against every milestone/card counterexample. Reconcile the
   recorder's exact changed-file union with Git, sample each architecture
   family, and verify instruction boundaries survived any rewrite.
3. Update card `008`, the `g11.003` milestone, front doors, and
   `docs/logs/2026-08/31-g11-003-northstar-agents-rust-audit-closeout.md`.
4. Push the selected worker branch and open a reviewable PR against current
   `main`.
5. The PR body must link the milestone, card, closeout, changed surfaces,
   recorder evidence, validation, retained limitations, and operator stops.
6. Report the exact head SHA and PR URL through Paseo. Do not merge.

### Review and merge path

The orchestrator will inspect the exact PR head independently and record its
verdict on the PR. If changes are requested, it will post every blocking
finding there and explicitly notify this same worker through Paseo. Repair only
those in-bounds findings on this branch, validate, push, and notify again.

When the exact reviewed head is current, required checks pass, the PR is
mergeable into `main`, and no stricter rule or explicit operator pause applies,
the orchestrator merges without another approval prompt.

- **Closeout refs:** card `008`; milestone `g11.003`; closeout log;
  `docs/README.md`; `docs/roadmaps/README.md`; `docs/roadmaps/g11/README.md`;
  `docs/logs/README.md`

### Handoff closeout

Leave the card, milestone, log, front doors, audit limitations, and single next
task honest. Card `008` ends at PR readiness; it does not authorize another
audit or product lane.
