---
title: Papercuts wave 15 AGENTS-audit selector worker handoff
kind: northstar-handoff
handoff_mode: worker-pr-loop
worker_mode: implementation
dispatch_authority: orchestrator
handoff: single-file-path-only
status: ready-to-launch
owner: Tom / papercuts orchestrator
created: 2026-08-30
updated: 2026-08-30
handoff_path: /Users/tom/Dev/projects/signal/docs/handoffs/20260830-154700-papercuts-wave15-agents-audit.md
base_required: pushed-main
tags: [coordination, handoff, worker, pr, papercuts]
---

## What This Thread Was Doing

`effigy check:agent-instructions` is not defined here, so the standard
AGENTS review command fails before the audit. The consumer-safe
installed-Northstar fallback exists, but it is not visible from this
repo's task surface.

You are the Signal implementation worker. Make the AGENTS audit
reachable. Leave rust-quality setup (Northstar owns that command),
SharedSandbox, cross-repo handoff paths, and `.agents.local.env` seeding
alone.

## Why It Matters

The next AGENTS review stops on routing, not on the instruction file.

## Current State

- **Repository:** `/Users/tom/Dev/projects/signal`
- **Planning branch:** `main`
- **Planning base commit:** `0f4745e88fbeaea78e9058e27126baa6ca433f0c`
- **Pushed main verification:** local `HEAD` and `origin/main` both resolved
  to that SHA before this handoff was created.
- **Planning checkout:** clean before this handoff file was created.
- **Worker mode:** implementation worker dispatched by the orchestrator.
- **Worker branch:** `worker/papercuts-wave15-agents-audit`
- **Worker worktree:** launcher first.
  `AGENTS_WORKTREE_CONTAINER_DIR=/Users/tom/Dev/worktrees`.
- **Required sibling worktree links:** `none`
- **Ready work items, in order:**
  1. Consumer AGENTS audit is not exposed through the local catalog —
     add a target-local read-only `check:agent-instructions` alias, or
     document the installed-Northstar fallback on `AGENTS.md` so the
     exact command is discoverable. Do not copy Northstar's Rhai into
     this repo. Do not invent a second audit. Existing
     `qa:docs:agent-defaults` is a different check (forbidden `--repo
     .`); keep it. Fallback shape when documenting:
     `effigy --repo <installed-northstar> northstar/check:agent-instructions <this-repo>`.
- **Out of scope:** rust-quality setup relative-scope (Northstar wave
  15); SharedSandbox live add-while-processing; cross-repo handoff path
  resolution; creating or rewriting `.agents.local.env` (already seeded
  here).
- **Canonical refs:** `PAPERCUTS.md`; `AGENTS.md`;
  `docs/effigy.tasks.docs.toml`; Northstar
  `references/modes/agent-instruction-review.md` consumer fallback.
- **Required validation:** `effigy check:agent-instructions` succeeds
  from this repo, or `AGENTS.md` names the installed-Northstar command
  and that command runs. `effigy qa:docs:agent-defaults` still passes.
- **PR URL:** pending
- **Merge authorisation:** absent; do not merge

## Boundaries

- Selector or docs only. Do not copy Northstar Rhai. Do not merge.

## Important Context

- Longhorn wave 14 PR 16 chose the documented fallback rather than a
  local alias. Either choice is in bounds here.
- **Report to:** the operator.

## Suggested Next Move

Read this file from the top. Run the worktree-safety preflight. After
the committed `HEAD` handoff checks out, skip sibling links (`none`),
then make the AGENTS audit reachable.

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
   `docs/handoffs/20260830-154700-papercuts-wave15-agents-audit.md`.
   Confirm `HEAD == origin/main`, ancestor
   `0f4745e88fbeaea78e9058e27126baa6ca433f0c`, and that relative path in
   `HEAD`. Load
   `git show HEAD:docs/handoffs/20260830-154700-papercuts-wave15-agents-audit.md`.
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

Leave the rust-quality, SharedSandbox, and handoff-path papercuts open.
