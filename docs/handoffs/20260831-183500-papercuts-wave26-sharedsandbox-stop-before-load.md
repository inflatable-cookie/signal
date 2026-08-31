---
title: Papercuts wave 26 SharedSandbox stop-before-load closeout
kind: northstar-handoff
handoff_mode: worker-pr-loop
worker_mode: implementation
dispatch_authority: orchestrator
handoff: single-file-path-only
status: ready-to-launch
owner: Tom / papercuts orchestrator
created: 2026-08-31
updated: 2026-08-31
handoff_path: /Users/tom/Dev/projects/signal/docs/handoffs/20260831-183500-papercuts-wave26-sharedsandbox-stop-before-load.md
base_required: pushed-main
tags: [coordination, handoff, worker, pr, papercuts]
---

## What This Thread Is Doing

Close the open Signal papercut `SharedSandbox sequential prepare must stop
before load` in `PAPERCUTS.md`.

The current `LocalRuntimeHost::prepare_shared_sandbox_processor` path appears
to stop the broker before `load-plugin-instance`, activate the new member,
then restart processing. The broker deliberately rejects lifecycle mutation
while its audio thread is live (`already_processing`). Establish whether the
current path and its tests prove that sequencing. If the proof is incomplete,
add the smallest focused regression test or implementation correction needed;
if it is already complete, make a docs/evidence-only closeout. Close only this
papercut entry.

## Why It Matters

The v1 shared broker cannot add a member while processing. A future change that
loads or activates directly on a running child would turn a valid sequential
prepare into a typed broker refusal or an unsafe lifecycle mutation.

## Current State

- **Repository:** `/Users/tom/Dev/projects/signal`
- **Planning branch:** `main`
- **Planning base commit:** `5a7a44949aad3f37d69ed5a42db8aa0d29400432`
- **Pushed main verification:** local `HEAD` and `origin/main` resolve to the
  planning base before this handoff.
- **Worker mode:** implementation worker dispatched by the papercuts
  orchestrator.
- **Worker branch:** `worker/papercuts-wave26-sharedsandbox-stop-before-load`
- **Required sibling worktree links:** none

## Boundaries

- Keep this lane inside Signal-owned SharedSandbox lifecycle sequencing.
- Do not add live add-while-processing, a broker pause protocol, a second
  audio-thread backend, or a new SharedSandbox product contract.
- Do not alter Chorus IPC meanings, release files, CI workflows, or sibling
  repositories.
- Do not touch the Loophole/Poodle papercut catalog entries; report any
  cross-repo finding instead.
- Preserve realtime safety: no allocation, blocking, locks, or unbounded work
  on audio-thread paths.

## Canonical References

- `PAPERCUTS.md` — the open entry to close
- `crates/signal-host-local/src/host_support/plugin_processor.rs` — host
  orchestration
- `crates/signal-plugin-sandbox/src/broker/lifecycle.rs` — broker refusal and
  lifecycle rules
- `tests/prepare_plugin_processor.rs` — existing SharedSandbox proof
- `docs/architecture/shared-sandbox-multiplexing.md`
- `docs/roadmaps/g11/README.md`

## Required Validation

- Run the narrow SharedSandbox host/broker tests that cover the changed path.
- Run `effigy qa:docs` and `effigy qa:northstar` after docs changes.
- Run `git diff --check`.
- Record exact commands and results in a new `docs/logs/` evidence note.
- Keep the PR diff limited to the focused proof/fix, the single PAPERCUTS
  closeout, and its evidence log.

## Completion Protocol

Open a PR from the worker branch and report the exact implementation head,
changed files, evidence, and any pre-existing failures. Do not merge from the
worker lane; the papercuts orchestrator reviews the exact head and merges when
the gate passes.

If the sequencing is already fully proved and no meaningful change is needed,
close the entry with the evidence note rather than inventing a refactor.

## Suggested Next Move

Read this handoff from the top, inspect the current implementation and focused
tests, then execute the smallest bounded proof or correction.
