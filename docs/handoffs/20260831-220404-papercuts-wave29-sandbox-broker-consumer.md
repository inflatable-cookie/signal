# Papercuts wave 29 — sandbox broker consumer boundary

handoff: single-repository-upstream-repair
status: ready-to-launch
owner: Tom / papercuts orchestrator
created: 2026-08-31
updated: 2026-08-31
handoff_path: /Users/tom/Dev/projects/signal/docs/handoffs/20260831-220404-papercuts-wave29-sandbox-broker-consumer.md
base_required: pushed-main
tags: [coordination, handoff, worker, pr, papercuts, signal]

## What This Thread Is Doing

Resolve the Signal-side source of the cross-repository papercut tracked at
`/Users/tom/Dev/projects/loophole/PAPERCUTS.md`:
“Signal sandbox broker binary is not consumable as a Cargo dependency”. The
current `signal-plugin-sandbox` package is bin-only. Consumer tests either
provide `SIGNAL_PLUGIN_SANDBOX_BROKER_COMMAND` or invoke Cargo against the
Signal checkout, which makes the first plugin-isolation run compile the broker
on demand.

First reproduce the exact Cargo behavior with the smallest existing consumer
surface. Then determine whether a small Signal-owned package/API repair makes
the broker reliably consumable. Do not assume that adding an empty library
target makes a dependency binary available: prove the proposed mechanism.

## Current State

- **Repository:** `/Users/tom/Dev/projects/signal`
- **Planning base:** `eecd0605009e62f8dbe0f1a0403162b592a09a92`
- **Worker branch:** `worker/papercuts-wave29-sandbox-broker-consumer`
- **Canonical tracker:** Loophole `PAPERCUTS.md`, source line 377
- **Primary package:** `crates/signal-plugin-sandbox`
- **Consumer surfaces:** `crates/signal-runtime` sandbox support and the
  `pulse-signal-link` tests named by the tracker

Signal has no separate active orchestrator or strict implementation lane.
This handoff is the authority for this bounded papercut lane.

## Boundaries

- Signal source, tests, package documentation, and one evidence log only.
- Preserve the broker wire protocol, process lifecycle, realtime guarantees,
  and existing environment-variable escape hatch.
- Do not edit Loophole, Pulse, Chorus, Longhorn, Poodle, `.github/workflows/`,
  release configuration, or package pins.
- Do not add a duplicate Signal `PAPERCUTS.md` entry. The canonical open
  tracker is the Loophole cross-repository entry; the orchestrator will close
  that tracker only after the upstream repair is merged and revalidated.
- Do not claim a Cargo consumer fix unless a clean consumer proof demonstrates
  it. If the proposed library-target/helper/script shape cannot provide a
  stable executable boundary, stop with a precise diagnosis and leave the
  tracker open; do not widen this lane into a packaging or release redesign.

## Required Work

1. Read this handoff, `AGENTS.md`, `docs/README.md`, and the canonical
   Loophole tracker entry.
2. Confirm this is the Signal worker worktree, not `main`, and that `HEAD`
   contains this handoff at the planned base.
3. Reproduce the current limitation using the smallest relevant consumer or
   fixture. Distinguish Cargo package/dependency behavior from local workspace
   convenience and from the environment-variable path.
4. If a minimal Signal-owned repair is real, implement it with focused proof
   that an external consumer can use the resulting broker boundary without an
   on-demand source checkout build.
5. Record the exact limitation or repair, changed files, and proof in one
   timestamped evidence log. Leave the Loophole tracker open in this PR.

## Validation

Use Effigy selectors where they cover the path. At minimum, run the focused
consumer/package tests, the relevant compile check, `effigy qa:docs`, and
`git diff --check` for an implementation. If the result is diagnosis-only,
validate the reproduction and document why no safe PR was opened. Do not run
release mutations or a broad workspace suite merely to manufacture evidence.

## Completion Protocol

- Keep the diff Signal-only and bounded to this tracker.
- Commit and push the worker branch.
- Open a PR against `main` only when a valid repair exists; do not merge from
  the worker lane.
- Report the exact head, PR URL or diagnosis, changed files, focused proof,
  and any remaining consumer limitation to the papercuts orchestrator.

The orchestrator will independently review the exact head. If a repair lands,
it will then close the Loophole tracking entry in a separate bounded closeout.
