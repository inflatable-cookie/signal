---
title: Papercuts wave 18 dispatch closeout worker handoff
kind: northstar-handoff
handoff_mode: worker-pr-loop
worker_mode: implementation
dispatch_authority: orchestrator
handoff: single-file-path-only
status: ready-to-launch
owner: Tom / papercuts orchestrator
created: 2026-08-30
updated: 2026-08-30
handoff_path: /Users/tom/Dev/projects/signal/docs/handoffs/20260830-185600-papercuts-wave18-dispatch-closeout.md
base_required: pushed-main
tags: [coordination, handoff, worker, pr, papercuts]
---

## What This Thread Was Doing

Two leftover copies of dispatch protocol that already shipped.

Northstar PR 8 made the operator-facing dispatch path the owning repo's
**absolute** handoff. This repo still lists a Soundcheck-relative lookup
that first reported the Signal handoff absent.

`.agents.local.env` is gitignored and already seeded on this machine
with `AGENTS_WORKTREE_CONTAINER_DIR=/Users/tom/Dev/worktrees`. The open
entry still reads as if first dispatch would stop.

You are the Signal implementation worker. Prove both and close the
copies. Leave SharedSandbox alone.

## Why It Matters

The next worker still hunts a relative handoff and still thinks fallback
worktrees cannot be created here.

## Current State

- **Repository:** `/Users/tom/Dev/projects/signal`
- **Planning branch:** `main`
- **Planning base commit:** `26ca13b080b4cee8e60a274b3f8075eb89fd1b13`
- **Pushed main verification:** local `HEAD` and `origin/main` both resolved
  to that SHA before this handoff was created.
- **Planning checkout:** clean before this handoff file was created.
- **Worker mode:** implementation worker dispatched by the orchestrator.
- **Worker branch:** `worker/papercuts-wave18-dispatch-closeout`
- **Worker worktree:** launcher first.
  `AGENTS_WORKTREE_CONTAINER_DIR=/Users/tom/Dev/worktrees`.
- **Required sibling worktree links:** `none`
- **Ready work items, in order:**
  1. Cross-repo worker handoff paths need resolution — close if
     Northstar `1840c9f6d4f7127240622a09e462b06adc094971` (PR 8) binds
     dispatch to the owning repo's absolute handoff path. Cite that
     SHA. Put one line on `AGENTS.md` that operator-facing dispatch is
     that absolute path, not a Soundcheck-relative lookup. Do not add
     a cross-repo path resolver. Do not copy Soundcheck files here.
  2. Signal had no `.agents.local.env` at first orchestrator dispatch —
     close if `.agents.local.env` exists, is gitignored, and names
     `AGENTS_WORKTREE_CONTAINER_DIR=/Users/tom/Dev/worktrees`. Do not
     commit the file. Record that evidence in `PAPERCUTS.md`.
- **Out of scope:** SharedSandbox live add-while-processing; editing
  Northstar or Soundcheck; GitHub workflows.
- **Canonical refs:** `PAPERCUTS.md`; `AGENTS.md`; `.gitignore`
  (`.agents.local.env`); Northstar PR 8
  (`1840c9f6d4f7127240622a09e462b06adc094971`).
- **Required validation:** `AGENTS.md` names absolute-path dispatch.
  `git check-ignore -v .agents.local.env` shows the ignore. The env
  file is present locally and is not staged.
- **PR URL:** pending
- **Merge authorisation:** absent; do not merge

## Boundaries

- Close the two dispatch copies. Do not change SharedSandbox. Do not
  merge.

## Important Context

- **Report to:** the operator.

## Suggested Next Move

Read this file from the top. Run the worktree-safety preflight. After
the committed `HEAD` handoff checks out, skip sibling links (`none`),
then close the two copies.

## Completion Protocol

### Before you start

1. Read this handoff path. Its `worker_mode: implementation` and
   `dispatch_authority: orchestrator` metadata activate worker mode. Then
   run `git rev-parse --show-toplevel`, `git branch --show-current`,
   `git status --porcelain`, and `git worktree list --porcelain`.
2. If the current root is a registered worktree, status is empty, and the
   branch is not `main`, accept it. Record the actual path/branch.
3. If the launcher supplied a dirty or `main` worktree, stop and report
   it. Fallback container is
   `AGENTS_WORKTREE_CONTAINER_DIR=/Users/tom/Dev/worktrees`. Never use
   `/tmp`.
4. From the selected worktree, record the repository-relative path
   `docs/handoffs/20260830-185600-papercuts-wave18-dispatch-closeout.md`.
   Confirm `HEAD == origin/main`, ancestor
   `26ca13b080b4cee8e60a274b3f8075eb89fd1b13`, and that relative path in
   `HEAD`. Load
   `git show HEAD:docs/handoffs/20260830-185600-papercuts-wave18-dispatch-closeout.md`.
   If the absolute dispatch file differs, stop. The `HEAD` copy is
   canonical.
5. Required sibling list is `none`. Skip link setup.
6. Read `AGENTS.md` and `PAPERCUTS.md`.

### When the assigned runway is complete

1. Update `PAPERCUTS.md`. Push a PR. Do not merge.

### Review and merge path

Awaiting orchestrator review. Merge is operator-authorised only.

- **Closeout refs:** `PAPERCUTS.md`; this handoff; the PR.

### Handoff closeout

Leave SharedSandbox open.
