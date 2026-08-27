---
title: Papercuts wave 3 docs currentness worker handoff
kind: northstar-handoff
handoff_mode: worker-pr-loop
worker_mode: implementation
dispatch_authority: orchestrator
handoff: single-file-path-only
status: ready-for-review
owner: Tom / papercuts orchestrator
created: 2026-08-27
updated: 2026-08-27
handoff_path: /Users/tom/Dev/projects/signal/docs/handoffs/20260827-210100-papercuts-wave3-docs-currentness.md
base_required: pushed-main
tags: [coordination, handoff, worker, pr, papercuts]
---

## What This Thread Was Doing

Wave 1 compiled VST3 path resolution on every host. Remaining Signal
papercuts are docs currentness: plugin hosting is still advertised as
missing after g09/g11/g12 shipped it, and Northstar refresh found stale
Next Task pointers. The operator approved papercuts wave 3.

You are the Signal implementation worker for this docs lane. Leave
SharedSandbox add-while-processing and T3 `.agents.local.env` seeding
alone (the latter already exists on this machine).

## Why It Matters

Refresh/atlas work reopened a finished hosting lane. Stale Next Task
pointers reopen finished cards.

## Current State

- **Repository:** `/Users/tom/Dev/projects/signal`
- **Planning branch:** `main`
- **Planning base commit:** `dba1347cde8f3fd93e33be8ab4524a6d97d68e43`
- **Pushed main verification:** local `HEAD` and `origin/main` both resolved
  to that SHA before this handoff was created.
- **Planning checkout:** clean before this handoff file was created.
- **Worker mode:** implementation worker dispatched by the orchestrator.
- **Worker branch:** `worker/papercuts-wave3-docs-currentness`
- **Worker worktree:** `/Users/tom/Dev/worktrees/signal-papercuts-wave3-docs-currentness`
- **Ready work items, in order:**
  1. Stale plugin-hosting docs misled planning — closed (already aligned; verified)
  2. Northstar refresh found stale Next Task pointers — closed (front doors aligned)
- **Out of scope:** SharedSandbox live add-while-processing (v1
  non-goal); cross-repo handoff path resolution; creating
  `.agents.local.env` (already seeded here).
- **Canonical refs:** `PAPERCUTS.md`; `docs/architecture`;
  `docs/roadmaps/backlog`; `docs/contracts/072`;
  `docs/roadmaps/README.md` and generation front doors.
- **Required validation:** hosting docs/backlog/contract 072 no longer
  say hosting is missing; Next Task pointers match the live roadmap
  front door. Cheap docs / `qa:northstar` if present.
- **PR URL:** pending
- **Merge authorisation:** absent; do not merge

## Boundaries

- Align Contract 072, backlog, architecture, and strategic runway with
  shipped CLAP/VST3/AU/LV2 hosting. Do not invent new hosting depth.
- Do not merge.

## Important Context

- The papercut was filed after demolition docs lagged g09/g11/g12.
- **Report to:** the operator.

## Suggested Next Move

Read this file, run the worktree preflight, then grep hosting-missing
claims and Next Task pointers against the live roadmap.

## Completion Protocol

### Before you start

1. Read this handoff. Run the four git identity commands.
2. Accept a clean dedicated non-`main` registered worktree.
3. Confirm `HEAD == origin/main` and ancestor
   `dba1347cde8f3fd93e33be8ab4524a6d97d68e43`.
4. Confirm this handoff exists in `HEAD`.

### When the assigned runway is complete

1. Update `PAPERCUTS.md`. Push a PR. Do not merge.

### Review and merge path

Awaiting orchestrator review. Merge is operator-authorised only.

- **Closeout refs:** `PAPERCUTS.md`; this handoff; the PR.

### Handoff closeout

If hosting docs are already aligned on this SHA, close with evidence.
