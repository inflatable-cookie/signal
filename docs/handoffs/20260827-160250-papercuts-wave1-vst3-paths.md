---
title: Papercuts wave 1 VST3 path resolution worker handoff
kind: northstar-handoff
handoff_mode: worker-pr-loop
worker_mode: implementation
dispatch_authority: orchestrator
handoff: single-file-path-only
status: ready-to-launch
owner: Tom / papercuts orchestrator
created: 2026-08-27
updated: 2026-08-27
handoff_path: /Users/tom/Dev/projects/signal/docs/handoffs/20260827-160250-papercuts-wave1-vst3-paths.md
base_required: pushed-main
tags: [coordination, handoff, worker, pr, papercuts]
---

## What This Thread Was Doing

A papercuts sweep found `resolve_module_binary_path` compiled only off
macOS, so Windows `Contents/{x86_64,arm64}-win` layout tests could not run
on this machine.

The operator approved wave 1. You are the Signal implementation worker for
this one-item compile-gate lane.

Leave SharedSandbox stop/start, cross-repo handoff path resolution, and
plugin-hosting docs currentness alone.

## Why It Matters

Platform-parameterized Windows VST3 layout tests need
`cfg(any(test, not(target_os = "macos")))` just to compile the path
helper. Path resolution is pure filesystem math and should be available
on every host. dlopen/hosting can stay cfg-gated.

## Current State

- **Repository:** `/Users/tom/Dev/projects/signal`
- **Planning branch:** `main`
- **Planning base commit:** `5ad14ae4b08a9ba5a84eb418e9b4ffdd3607048f`
- **Pushed main verification:** local `HEAD` and `origin/main` both resolved
  to that SHA before this handoff was created.
- **Planning checkout:** clean before this handoff file was created.
- **Worker mode:** implementation worker dispatched by the orchestrator; this
  handoff activates the worker-only worktree preflight.
- **Planning artifacts included at the base:** `PAPERCUTS.md`; this handoff.
- **Worker branch:** `worker/papercuts-wave1-vst3-paths`
- **Worker worktree:** prefer the launcher-provided clean dedicated
  worktree. Named manual fallback:
  `/Users/tom/Dev/worktrees/signal-papercuts-wave1-vst3-paths`
- **Worktree creation command:** only when the startup preflight permits the
  manual fallback:
  `git worktree add /Users/tom/Dev/worktrees/signal-papercuts-wave1-vst3-paths -b worker/papercuts-wave1-vst3-paths origin/main`
- **Worker worktree policy:** first use a clean, dedicated, non-`main`
  registered launcher worktree. `.agents.local.env` had
  `AGENTS_WORKTREE_CONTAINER_DIR=/Users/tom/Dev/worktrees`.
- **Active spec lane:** none for this papercuts lane.
- **Roadmap milestone:** none. Do not continue g11 shared-sandbox or
  Windows discovery cards.
- **Ready work items, in order:**
  1. VST3 module binary resolution is cfg-gated off macOS
- **Allowed runway:** that one item only, one PR.
- **Remaining card budget:** one papercut.
- **Dispatch topology:** serial inside this repo; parallel with other
  wave-1 repos.
- **Parallel safety check:** no shared files with other wave-1 workers.
  Do not edit Loophole or Soundcheck.
- **Canonical refs:** `AGENTS.md`; `PAPERCUTS.md`;
  `crates/signal-plugin-vst3/src/vst3_host_adapter/introspection/paths.rs`.
- **Model capability profile:** capable coding model, medium reasoning.
- **Tool/runtime restrictions:** use Effigy selectors. Do not load real
  VST3 modules on this lane.
- **Required validation:** Windows layout tests compile and run on macOS
  without wrapping the helper in `cfg(any(test, not(target_os = "macos")))`.
  Hosting/dlopen stays cfg-gated. Focused `signal-plugin-vst3` tests.
- **PR base/head:** current pushed `main` / selected worker branch
- **PR URL:** pending
- **Review state:** awaiting orchestrator review after worker completion
- **Merge authorisation:** absent; do not merge

## Boundaries

- **In scope:** keep path resolution compiled on every host; leave
  dlopen/hosting cfg-gated. Close the papercut.
- **Out of scope:** SharedSandbox add-while-processing; cross-repo
  handoff lookup; `.agents.local.env` seeding (already present here);
  stale plugin-hosting docs; live plugin hosting proof.
- `libloading::Library` is currently `cfg(not(target_os = "macos"))` in
  `paths.rs`. Do not pull that into the always-compiled path helper.
- Do not merge the PR.

## Important Context

- **Planning lineage:** operator-authorized papercuts wave 1, 2026-08-27.
- **Surface:** `resolve_module_binary_path` in
  `crates/signal-plugin-vst3/src/vst3_host_adapter/introspection/paths.rs`
  is `#[cfg(any(test, not(target_os = "macos")))]`. Callers in hosting
  wire/module/events/parameters already import it.
- **Report after:** the cfg split and focused tests; then PR.
- **Report to:** the operator, who will relay progress to the orchestrator.

## Suggested Next Move

Read this file from the top. Run the worktree-safety preflight. Use the
launcher worktree if it is clean, dedicated, and not `main`.

Then read `paths.rs` and lift `resolve_module_binary_path` out of the
macOS cfg gate without moving `libloading` with it.

## Completion Protocol

### Before you start

1. Read this handoff. Then run `git rev-parse --show-toplevel`,
   `git branch --show-current`, `git status --porcelain`, and
   `git worktree list --porcelain`.
2. If the current root is a registered worktree, status is empty, and the
   branch is not `main`, accept it and record the actual path/branch.
3. Only if that context is unusable, use the named worktree, then
   `.agents.local.env`. Never use `/tmp`. Never clean a dirty checkout.
4. Confirm `HEAD == origin/main`, confirm
   `git merge-base --is-ancestor 5ad14ae4b08a9ba5a84eb418e9b4ffdd3607048f HEAD`,
   and confirm this handoff exists in `HEAD`.
5. Read `AGENTS.md`, `PAPERCUTS.md`, and `paths.rs`.

### While you work

- Keep the diff in VST3 path resolution and its tests.

### When the assigned runway is complete

1. Run focused `signal-plugin-vst3` tests, including Windows layout
   cases on this macOS host.
2. Close the papercut in `PAPERCUTS.md`.
3. Push the worker branch and open a PR against current pushed `main`.
4. Report the PR URL. Do not merge.

### Review and merge path

Awaiting orchestrator review after the PR exists. Merge is
operator-authorised only.

- **Closeout refs:** `PAPERCUTS.md`; this handoff; the PR.

### Handoff closeout

If hosting callers cannot compile on macOS after the lift, keep those
callers cfg-gated and only expose the pure path function.
