---
handoff: single-repository-upstream-repair
status: ready-to-launch
owner: Tom / papercuts orchestrator
created: 2026-09-01
updated: 2026-09-01
handoff_path: /Users/tom/Dev/projects/signal/docs/handoffs/20260901-110725-papercuts-wave39-sandbox-broker-prebuilt-contract.md
base_required: pushed-main
handoff_mode: worker-pr-loop
worker_mode: implementation
dispatch_authority: orchestrator
tags: [coordination, handoff, worker, pr, papercuts, signal, broker, provisioning]
---

# Papercuts wave 39 — sandbox broker prebuilt boundary

## What this thread is doing

Implement the selected option-2 resolution for the cross-repository papercut
tracked at `/Users/tom/Dev/projects/loophole/PAPERCUTS.md`:
“Signal sandbox broker binary is not consumable as a Cargo dependency”.

The user has chosen the honest stable-Cargo boundary: Signal does not expose
the broker executable through a normal Cargo dependency. Consumers receive a
compatible prebuilt broker through `SIGNAL_PLUGIN_SANDBOX_BROKER_COMMAND`.
Signal must make that contract and its provisioning path explicit enough that
CI, local development, and product integration do not compile the broker from
a Signal checkout during the first consumer run.

The earlier diagnosis at `3393ce11` remains valid and should be preserved as
evidence. This is a new implementation lane, not a retry of the rejected
empty-library experiment.

## Current state

- **Repository:** `/Users/tom/Dev/projects/signal`
- **Planning base:** `f357b56e7a470ea1cac3a51553fac3eb779102aa`
- **Primary package:** `crates/signal-plugin-sandbox`
- **Canonical decision:** `docs/triage/2026-09-01-sandbox-broker-prebuilt-contract.md`
- **Consumer runbook:** `docs/reference/consuming-signal.md`
- **Canonical tracker:** Loophole `PAPERCUTS.md`; leave it open in this PR

Signal has no separate active product roadmap card for this maintenance
decision. This handoff is the execution authority for the bounded papercut
lane.

## Required outcome

1. Make the option-2 contract explicit in the canonical consuming-Signal
   runbook:
   - a consumer supplies `SIGNAL_PLUGIN_SANDBOX_BROKER_COMMAND` pointing to a
     compatible executable;
   - broker arguments and working directory remain explicit configuration;
   - missing configuration fails fast with an actionable diagnostic;
   - consumer startup never invokes Cargo or builds Signal source implicitly.
2. Add one Signal-owned, reproducible developer/CI provisioning entry point
   using the repository's existing Effigy/script conventions. It may build or
   retrieve the broker before the consumer run, but it must produce or report
   an absolute executable path suitable for
   `SIGNAL_PLUGIN_SANDBOX_BROKER_COMMAND` and must not be hidden inside
   consumer startup.
3. Add focused proof for the provisioner/runbook boundary. Prove at least that
   the produced path is usable by the broker consumer and that the missing-env
   failure remains clear. Keep any target/profile/cache inputs explicit; do
   not imply that one host's binary is portable to another host.
4. Record the exact implementation, changed files, commands, and remaining
   platform/provenance limits in one timestamped evidence log.

If the repository's current task conventions make a provisioning helper
unsafe or impossible without release assets, stop with a precise diagnosis
and explain the smallest additional Signal-owned packaging decision required.
Do not substitute another empty library, `CARGO_BIN_EXE_*`, or on-demand
consumer-side Cargo build and call it a repair.

## Boundaries

- Signal source, docs, scripts/Effigy task configuration, focused tests, and
  one evidence log only.
- Preserve the broker wire protocol, process lifecycle, realtime guarantees,
  and existing `SIGNAL_PLUGIN_SANDBOX_BROKER_*` environment surface.
- Do not edit Loophole, Pulse, Chorus, Longhorn, Poodle, package pins, or
  release assets/installers.
- Do not edit `.github/workflows/` or run release mutations in this lane.
- Do not add a Signal `PAPERCUTS.md`; Loophole remains the canonical tracker.
- Do not close the Loophole tracker from this worker PR. A separate Loophole
  revalidation/closeout follows only after this repair merges.

## Required startup checks

1. Read this handoff, `AGENTS.md`, `docs/README.md`, the option-2 decision
   note, and `docs/reference/consuming-signal.md`.
2. Confirm the worktree is the worker branch, not Signal `main`, and base it
   on the pushed planning commit above. Fetch current `origin/main` first if
   the existing diagnosis workspace is behind it.
3. Inspect the existing Effigy selectors and scripts before choosing the
   provisioning surface. Keep the implementation aligned with the existing
   runbook and task conventions.

## Validation

Use Effigy selectors where they cover the path. At minimum, run the focused
broker/provisioning proof, the relevant compile or package check,
`effigy qa:docs`, and `git diff --check`. Avoid a broad workspace suite merely
to manufacture evidence.

## Completion protocol

- Keep the diff Signal-only and bounded to this decision.
- Commit and push the worker branch.
- Open a PR against `main` only when the option-2 contract is real and
  demonstrated; do not merge from the worker lane.
- Report the exact head, PR URL or diagnosis, changed files, focused proof,
  and any remaining consumer limitation to the papercuts orchestrator.

The orchestrator will independently review the exact head. If it merges, the
Loophole orchestrator will separately revalidate the prebuilt path and close
the cross-repository tracker only when its acceptance criterion is met.
