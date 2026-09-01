---
title: Papercuts wave 41 LocalRuntimeHost Send boundary worker handoff
kind: northstar-handoff
handoff_mode: worker-pr-loop
worker_mode: implementation
dispatch_authority: orchestrator
handoff: single-file-path-only
status: ready-to-launch
owner: Tom / papercuts orchestrator
created: 2026-09-01
updated: 2026-09-01
handoff_path: /Users/tom/Dev/projects/signal/docs/handoffs/20260901-114200-papercuts-wave41-local-host-send-boundary.md
base_required: pushed-main
tags: [coordination, handoff, worker, pr, papercuts, signal, host, send]
---

## What This Thread Is Doing

Repair the Signal regression exposed by downstream Loophole revalidation after
Signal PR 17 introduced `LocalRuntimeHost::with_hardware`.

`LocalRuntimeHost` now stores `Box<dyn HardwareBackend>`, but Loophole's
existing `LiveHost` must implement `pulse_authority::TransportDriver: Send`.
The erased backend therefore makes `pulse-signal-link` fail to compile at its
existing `TransportDriver for LiveHost` implementation. Restore the existing
thread-safety boundary with the smallest safe Signal-owned change, then close
the matching Signal papercut and report the downstream prerequisite.

## Why It Matters

The headless injection seam is useful, but it cannot break a shipped consumer's
compile boundary. The fix must make the host movable across the existing
authority boundary without weakening `TransportDriver: Send`, using `unsafe`
impls, or changing runtime behavior.

## Current State

- **Repository:** `/Users/tom/Dev/projects/signal`
- **Planning branch:** `main`
- **Planning base commit:** `e93cc3ff404036673c1b3ffff7e1bfcfa4213d95`
- **Pushed-main verification:** local `main` and `origin/main` matched this
  exact commit before this handoff was created.
- **Regression source:** Signal PR 17 merge; the injected host field is
  `hardware: Box<dyn HardwareBackend>`.
- **Downstream reproducer:** Loophole `cargo check -p pulse-signal-link`
  fails because `dyn HardwareBackend` is not `Send`, preventing `LiveHost`
  from satisfying `TransportDriver: Send`.
- **Canonical Signal refs:** `docs/README.md`; `AGENTS.md`; Contract 001;
  `crates/signal-hardware/src/backend_contract.rs`;
  `crates/signal-host-local/src/host.rs` and its host tests.
- **Worker branch:** `worker/papercuts-wave41-local-host-send-boundary`.
- **Worker worktree:** launcher first; manual fallback only through the
  completion preflight and configured worktree container.
- **Required sibling, read-only:** Loophole is the downstream reproducer at
  `/Users/tom/Dev/projects/loophole`; do not edit it, change its pins, or use
  its uncommitted broker draft. Any downstream compile check is evidence only.
- **Active roadmap lane:** none. Signal `g11` has no ready roadmap card; this
  is explicit papercut/regression maintenance and must not open a new
  generation or alter roadmap status.
- **Worker class:** bounded non-frontier Rust maintenance with focused API and
  compile regression proof. The reasoning is local and the consequence is
  material, so the worker stays economical while the orchestrator applies the
  stricter material-risk review gate.
- **Remaining budget:** one regression repair, one PR.
- **Dispatch topology:** one Signal worker. Do not start another lane against
  the same host/backend boundary or `PAPERCUTS.md`.

## Boundaries

- **In scope:** Signal's hardware/host Send boundary, the smallest focused
  compile or trait-bound regression proof, one evidence log, and the matching
  Signal `PAPERCUTS.md` closeout. If no matching entry exists, add one in the
  worker PR and resolve it there with the observed regression and repair.
- **Out of scope:** Loophole edits, broker contract work, Longhorn, Poodle,
  package pins, `Cargo.lock`, release assets, workflows, IPC/protocol changes,
  audio callback behavior, runtime lifecycle, or unrelated papercuts.
- Preserve `LocalRuntimeHost::new` on the real cpal path and
  `with_hardware` as the headless injection seam. Preserve the existing
  `TransportDriver: Send` contract.
- Prefer a narrow `Send` requirement at the host's erased backend boundary if
  that is sufficient. If the shared `HardwareBackend` trait itself must gain a
  supertrait, prove all in-tree implementors and explain the public contract
  impact in the evidence. Do not make a non-Send backend appear safe with an
  `unsafe impl`, mutex wrapper that changes semantics, or thread-affinity
  workaround.
- Do not broaden this into a redesign of hardware ownership, callback
  threading, or `TransportDriver`.

## Review Oracle

1. A clean Signal checkout builds the host boundary and its existing in-tree
   hardware implementations with the repaired Send contract.
2. A downstream Loophole checkout using Signal at or after this exact merge
   can compile `pulse-signal-link` at the existing `TransportDriver for
   LiveHost` implementation. This check is read-only downstream evidence; do
   not edit Loophole in this lane.
3. Existing real-host and injected-simulated-host tests retain their behavior.
   No `LocalRuntimeHost::new` production path is removed or replaced.
4. No unsafe Send/Sync assertion, protocol change, package pin, lockfile
   churn, or unrelated Signal tracker closure appears in the PR.

If the existing host or hardware contracts do not justify a safe Send bound,
stop with the precise ownership/design blocker and do not invent one. Do not
reopen the already-set Signal broker option-2 decision.

## Required Worktree Preflight

1. Before broad reads, run `git rev-parse --show-toplevel`,
   `git branch --show-current`, `git status --porcelain`, and
   `git worktree list --porcelain`. Accept only the assigned clean,
   non-`main` launcher worktree; never edit the planning checkout.
2. Fetch origin non-interactively. Confirm selected `HEAD == origin/main`, the
   planning base is an ancestor, and the tracked copy of this handoff matches
   the absolute dispatch file.
3. Read `AGENTS.md`, `PAPERCUTS.md`, `docs/README.md`, Contract 001, the active
   roadmap front door, the `HardwareBackend` trait, the host implementation,
   and the existing host tests before editing.
4. Confirm the downstream Loophole reproducer and its Signal sibling resolve
   to the named clean checkouts. Use explicit working directories for any
   cross-repo Effigy/Git command; do not change either checkout.
5. Use Effigy selectors where they cover Signal validation. Do not run release
   mutations or edit `.github/workflows/`.

## Required Work

1. Reproduce the downstream `HardwareBackend`/`TransportDriver: Send` failure
   or a minimal equivalent from the current Signal source before editing.
2. Determine the smallest safe type-bound repair. Keep the public surface
   explicit and document why built-in and injected backends satisfy it.
3. Add focused Signal proof for the repaired Send boundary and retain the
   existing real and simulated host boot coverage. Do not replace a compile
   guarantee with a comment-only assertion.
4. Add one timestamped evidence log naming the reproducer, chosen bound,
   implementors checked, downstream compile proof, and unchanged boundaries.
5. Update only the matching Signal `PAPERCUTS.md` entry after proof is green.
6. Push one PR against the current `main`; do not merge from the worker lane.

## Required Validation

- Focused `signal-hardware` and `signal-host-local` tests covering the
  implementors and injected host path.
- `cargo check -p signal-host-local` and a strict clippy check for changed
  crates where practical.
- Downstream read-only `cargo check -p pulse-signal-link` against this Signal
  worktree or merged-compatible sibling, with incidental lockfile changes
  reverted and not committed.
- `effigy fmt:rust:check`, `effigy qa:docs`, `effigy qa:northstar`, and
  `git diff --check`.
- Record any skipped broad suite with a concrete reason. Do not run release or
  distribution mutations.

## Completion Protocol

When complete, commit and push the bounded change, open one PR, and report:

- exact PR URL and exact tested head;
- changed files and proof that only this Send-boundary entry was closed;
- the before/after compile evidence;
- implementors and public-boundary reasoning;
- exact validation commands/results;
- downstream Signal prerequisite and any sibling caveat;
- any stop condition or skipped suite.

The orchestrator will independently review the exact head with material-risk
scrutiny, record the verdict on the PR, and merge after required checks and
mergeability pass. The Loophole broker-contract worker then resumes only after
this Signal prerequisite is merged.

## Handoff Closeout

Until the upstream repair is reviewed and merged, the Signal papercut and the
downstream Loophole broker-contract draft remain open. This lane must not close
the Loophole tracker or modify its worktree.
