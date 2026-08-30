---
title: Papercuts wave 16 rust-quality setup closeout worker handoff
kind: northstar-handoff
handoff_mode: worker-pr-loop
worker_mode: implementation
dispatch_authority: orchestrator
handoff: single-file-path-only
status: ready-to-launch
owner: Tom / papercuts orchestrator
created: 2026-08-30
updated: 2026-08-30
handoff_path: /Users/tom/Dev/projects/signal/docs/handoffs/20260830-165200-papercuts-wave16-rustc-setup-closeout.md
base_required: pushed-main
tags: [coordination, handoff, worker, pr, papercuts]
---

## What This Thread Was Doing

Northstar PR 9 landed: `rust-quality:setup` canonicalizes an absolute
scope that is the target root or a subdirectory of it. This repo still
lists the copy.

You are the Signal implementation worker. Prove that setup against
sibling Northstar and close the copy. Leave SharedSandbox, cross-repo
handoff paths, and `.agents.local.env` seeding alone.

## Why It Matters

An open copy still sends the next worker into a setup failure that
already shipped.

## Current State

- **Repository:** `/Users/tom/Dev/projects/signal`
- **Planning branch:** `main`
- **Planning base commit:** `c0adc75184f736cf9c16f79a5b05f9c9c463ae65`
- **Pushed main verification:** local `HEAD` and `origin/main` both resolved
  to that SHA before this handoff was created.
- **Planning checkout:** clean before this handoff file was created.
- **Worker mode:** implementation worker dispatched by the orchestrator.
- **Worker branch:** `worker/papercuts-wave16-rustc-setup-closeout`
- **Worker worktree:** launcher first.
  `AGENTS_WORKTREE_CONTAINER_DIR=/Users/tom/Dev/worktrees`.
- **Required sibling worktree links:**
  - `northstar` from `/Users/tom/Dev/projects/northstar` as `../northstar`
  Create when absent; reuse only a symlink that already resolves to that
  source; stop on any other existing path; never overwrite.
- **Ready work items, in order:**
  1. Rust quality setup scope is repository-relative — close if sibling
     Northstar `77dcda9fa20e9d63977eb3488b0738ea0391f0bb` (PR 9) accepts
     `northstar/rust-quality:setup apply <this-worktree-abs>
     <this-worktree-abs>` (absolute target and absolute scope that is
     the target root). Cite that SHA. If the installed skill is older,
     run from inside the sibling Northstar checkout (do not
     `cargo`/`effigy` it from a Signal cwd that would apply the wrong
     catalog). Do not re-implement setup here. Apply is idempotent; do
     not rewrite `AGENTS.md` unless the apply actually changes it.
- **Out of scope:** SharedSandbox live add-while-processing; cross-repo
  handoff path resolution; creating or rewriting `.agents.local.env`;
  editing Northstar.
- **Canonical refs:** `PAPERCUTS.md`; Northstar PR 9
  (`77dcda9fa20e9d63977eb3488b0738ea0391f0bb`);
  `skills/northstar/scripts/rust-quality-setup.rhai`.
- **Required validation:** the apply command above exits 0 against that
  SHA. Close the papercut with the SHA you actually ran.
- **PR URL:** pending
- **Merge authorisation:** absent; do not merge

## Boundaries

- Prove Northstar 9, then close the copy. Do not edit Northstar. Do not
  merge.

## Important Context

- Wave 15 closed the AGENTS-audit selector. This is the remaining
  consumer copy of the setup command.
- **Report to:** the operator.

## Suggested Next Move

Read this file from the top. Run the worktree-safety preflight. After
the committed `HEAD` handoff checks out, create the Northstar sibling
link, then prove setup.

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
   `docs/handoffs/20260830-165200-papercuts-wave16-rustc-setup-closeout.md`.
   Confirm `HEAD == origin/main`, ancestor
   `c0adc75184f736cf9c16f79a5b05f9c9c463ae65`, and that relative path in
   `HEAD`. Load
   `git show HEAD:docs/handoffs/20260830-165200-papercuts-wave16-rustc-setup-closeout.md`.
   If the absolute dispatch file differs, stop. The `HEAD` copy is
   canonical.
5. Then create the sibling links from that tracked list. Canonicalize
   source and destination. Create when absent; reuse only a correct
   symlink; stop on conflict; never overwrite. Do not skip a listed
   catalog member.
6. Read `AGENTS.md` and `PAPERCUTS.md`.

### When the assigned runway is complete

1. Update `PAPERCUTS.md`. Push a PR. Do not merge.

### Review and merge path

Awaiting orchestrator review. Merge is operator-authorised only.

- **Closeout refs:** `PAPERCUTS.md`; this handoff; the PR.

### Handoff closeout

If sibling Northstar is older than PR 9, keep the copy open with the
SHA you actually ran.
