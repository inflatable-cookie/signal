# Papercuts wave 30 — sandbox broker child PID

handoff: single-repository-upstream-repair
status: ready-to-launch
owner: Tom / papercuts orchestrator
created: 2026-08-31
updated: 2026-08-31
handoff_path: /Users/tom/Dev/projects/signal/docs/handoffs/20260831-221016-papercuts-wave30-sandbox-child-pid.md
base_required: pushed-main
tags: [coordination, handoff, worker, pr, papercuts, signal]

## What This Thread Is Doing

Close the Signal-side source of the cross-repository papercut tracked at
`/Users/tom/Dev/projects/loophole/PAPERCUTS.md`:
“`SandboxBrokerClientSession` exposes no child pid”. The session already owns
the spawned `std::process::Child`, but consumers cannot obtain its stable PID
for crash evidence or `sandbox_pid` reporting without an external `ps` probe.

Expose the child PID through the public `SandboxBrokerClientSession` API and
add the smallest focused proof that the accessor reports the spawned child’s
PID. This is observability only; it must not change process lifecycle or wire
behavior.

## Current State

- **Repository:** `/Users/tom/Dev/projects/signal`
- **Planning base:** `3b203bf60fa9477d0d5cbfcc1145ff20b2f18df0`
- **Worker branch:** `worker/papercuts-wave30-sandbox-child-pid`
- **Canonical tracker:** Loophole `PAPERCUTS.md`, source line 391
- **Primary type:** `crates/signal-runtime/src/sandbox_broker_support/types/session.rs`
- **Existing process owner:** `SandboxBrokerClientSession.child: Child`

Signal has no separate active orchestrator or strict implementation lane.
This handoff is the authority for this bounded papercut lane.

## Boundaries

- Signal runtime API, focused tests, one evidence log, and no other package.
- Return the PID with the platform-neutral type already used by
  `std::process::Child::id()`; do not invent a new process identity type.
- Preserve `kill`, `is_alive`, `shutdown`, timeout cleanup, and all broker
  protocol semantics exactly.
- Do not edit Loophole, Pulse, Chorus, Longhorn, Poodle, `.github/workflows/`,
  release configuration, or package pins.
- Do not add a duplicate Signal `PAPERCUTS.md` entry. The canonical open
  tracker is the Loophole cross-repository entry; the orchestrator will close
  it only after this upstream repair is merged and downstream proof passes.
- Do not fold in the separate Cargo broker-packaging diagnosis from wave 29,
  broker lifecycle changes, crash-event schema changes, or host integration.

## Required Work

1. Read this handoff, `AGENTS.md`, `docs/README.md`, and the canonical
   Loophole tracker entry.
2. Confirm this is the Signal worker worktree, not `main`, and that `HEAD`
   contains this handoff at the planned base.
3. Locate the smallest existing test seam for `SandboxBrokerClientSession`.
   Prove the returned value is the spawned child PID without relying on
   `ps`, timing, or a platform-specific process API.
4. Add the documented public accessor and focused regression proof. Keep the
   implementation a direct read of the owned `Child` identity.
5. Record exact files, proof, and downstream follow-up in one timestamped
   evidence log. Leave the Loophole tracker open in this PR.

## Validation

Use Effigy selectors where they cover the path. At minimum, run the focused
Signal runtime test(s), the relevant compile/check selector, `effigy qa:docs`,
and `git diff --check`. Do not run release mutations or a broad workspace
suite merely to prove a one-method API addition.

## Completion Protocol

- Keep the diff Signal-only and bounded to this tracker.
- Commit and push the worker branch.
- Open a PR against `main`; do not merge from the worker lane.
- Report the exact head, PR URL, changed files, focused proof, and any
  platform caveat to the papercuts orchestrator.

The orchestrator will independently review the exact head. After merge, it
will close the Loophole tracking entry in a separate bounded documentation
closeout, leaving the unresolved broker-packaging decision open.
