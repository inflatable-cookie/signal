# Papercuts wave 32 — headless LocalRuntimeHost hardware seam

handoff: single-repository-upstream-repair
status: ready-to-launch
owner: Tom / papercuts orchestrator
created: 2026-08-31
updated: 2026-08-31
handoff_path: /Users/tom/Dev/projects/signal/docs/handoffs/20260831-223724-papercuts-wave32-local-host-headless.md
base_required: pushed-main
tags: [coordination, handoff, worker, pr, papercuts, signal, hardware]

## What This Thread Is Doing

Close the source-side papercut tracked in Loophole's
`PAPERCUTS.md`: “`LocalRuntimeHost` cannot boot headless (no injectable
hardware seam)”. The local host currently constructs a concrete
`LocalHardwareBackend` that enumerates cpal devices during construction and
`boot_default()` fails with `DeviceUnavailable` when no output device exists.
Signal already ships the public `HardwareBackend` contract and
`SimulatedHardwareBackend`; the host does not currently expose an injection
seam over that contract.

Add the smallest supported seam so tests and headless consumers can construct
`LocalRuntimeHost` with an explicit hardware backend, while the existing
constructor keeps the real local/cpal behavior. Prove that an injected
simulated output can boot without depending on the machine's default output
device.

## Current State

- **Repository:** `/Users/tom/Dev/projects/signal`
- **Planning base:** `95a1978f6bae060dce60a0f96d223162964e4272`
- **Worker branch:** `worker/papercuts-wave32-local-host-headless`
- **Canonical tracker:** `/Users/tom/Dev/projects/loophole/PAPERCUTS.md`,
  the open `LocalRuntimeHost` entry around line 410
- **Host:** `crates/signal-host-local/src/host.rs`
- **Current hardware owner:** `crates/signal-host-local/src/host_support/hardware.rs`
- **Shared contract:** `crates/signal-hardware/src/backend_contract.rs`
- **Existing test seam:** `signal_hardware::SimulatedHardwareBackend`

Signal has no separate active project orchestrator or implementation lane.
This handoff is the authority for this bounded Signal papercut.

## Boundaries

- Signal host-local, hardware adapter/contract code only where required for the
  seam, focused tests, one evidence log, and no unrelated package.
- Preserve `LocalRuntimeHost::new` for the real local/cpal default. Add an
  explicit constructor or equivalent injection path; do not make production
  callers silently use simulated hardware.
- Prefer the existing `HardwareBackend` contract and
  `SimulatedHardwareBackend` rather than inventing a second test-only trait.
  If the current private `LocalHardwareBackend` needs a thin implementation or
  adapter to satisfy the contract, keep its real-device behavior unchanged.
- Preserve hardware negotiation, diagnostics, policy reporting, boot order,
  graph/plugin discovery, stream summary semantics, and runtime lifecycle.
- Do not open an actual audio callback stream as part of this seam; this host's
  current boot contract negotiates and reports the output stream.
- Do not edit Loophole, Longhorn, Poodle, Effigy, package pins, or release/CI
  configuration. The canonical Loophole tracker stays open in this upstream
  PR; the orchestrator will close it in a separate bounded documentation
  closeout after the source repair is merged and proved.
- Leave the separate broker-packaging diagnosis and all plugin sandbox
  papercuts untouched.

## Required Work

1. Read this handoff, `AGENTS.md`, the Signal docs front doors, and the
   canonical Loophole tracker entry. Confirm this is the Signal worker
   worktree and that its base contains this handoff.
2. Inspect all `LocalRuntimeHost` construction and hardware uses. Define the
   smallest public or crate-appropriate injection API that lets a caller pass
   an explicit `HardwareBackend` while keeping `new(runtime)` unchanged for
   the real backend.
3. Add a focused host-local regression using
   `SimulatedHardwareBackend::default_stereo_output` (or an equivalent
   explicit simulated device) that boots and reports a simulated output
   without enumerating or requiring cpal hardware. Assert the existing default
   path remains represented by its current tests.
4. Keep the implementation mechanical at the host/hardware boundary. Do not
   broaden this into a new cross-platform hardware backend, a callback
   runtime, or a redesign of device policy.
5. Record exact files, the public construction API, focused proof, and the
   unchanged real-device behavior in one timestamped evidence log. Leave the
   Loophole tracker open in this PR.
6. If the current contract cannot support a safe seam without a larger
   architecture decision, stop at a precise diagnosis and evidence rather
   than adding a speculative abstraction; report that no repair PR was opened.

## Validation

Use Effigy selectors where they cover the path. For a repair, at minimum run
the focused `signal-host-local` host tests, `cargo check -p signal-host-local`,
`effigy qa:docs`, `effigy qa:northstar`, and `git diff --check`. A diagnosis
must still include the smallest reproducer and the same documentation gates.
Do not run release mutations or a broad workspace suite merely to prove the
constructor seam.

## Completion Protocol

- Keep the diff Signal-only and bounded to this hardware seam.
- Commit and push the worker branch.
- If repaired, open one PR against `main`; do not merge from the worker lane.
- Report the exact head, PR URL (or diagnosis-only status), changed files,
  focused proof, and any platform caveat to the papercuts orchestrator.

The orchestrator will independently review the exact head. After a merged
repair and a clean downstream record, it will close the corresponding Loophole
tracker without changing the separate broker-packaging decision.
