---
title: g01.008 host-local recovery stabilization after hardware contract freeze
status: complete
owner: core-product
created: 2026-03-11
updated: 2026-03-11
tags: [signal, hardware, host, runtime, roadmap, g01, g01.008]
---

## Summary

Stabilized the `g01.008` hardware-contract tranche after moving
`signal-host-local` onto negotiated device metadata. The follow-up work fixed
the real regression at the runtime/host seam: recovered transport sessions were
still classified as `RecoveryOverlap`, which blocked repeated watchdog
recovery and left the host-local diagnostics/tests out of sync with the newer
runtime continuity model.

## What landed

- added a runtime-owned transport transition that promotes a recovered session
  from overlap admission into steady-state admission once recovery succeeds
- updated `signal-host-local` recovery paths to call that promotion after
  successful restart so later recovery cycles are measured against the real
  active session state instead of a stale overlap classification
- reconciled host-local timeout and watchdog expectations with the current
  runtime observation model:
  - broader valid prework retirement reasons
  - lower final block sequences after steady-state promotion
  - updated automation tail values and gesture counts
  - updated continuity-segment expectations for repeated recovery paths
  - explicit transport-fault diagnostic counts on recovery-heavy soak paths

## Contract outcome

- `signal-runtime` now owns a first-class distinction between:
  - overlap admission during recovery attach
  - steady-state admission after the replacement becomes the active session
- `signal-host-local` recovery tests now match runtime-owned diagnostics instead
  of preserving assumptions from the pre-promotion transport model
- the hardware-contract freeze remains intact; this tranche only stabilized the
  recovery/runtime seam that the new negotiated host setup exposed

## Validation

- `cargo test -p signal-runtime`
- `cargo test -p signal-hardware`
- `cargo test -p signal-hardware-coreaudio`
- `cargo test -p signal-host-local`
- `effigy health`
- `effigy validate`
- `effigy test`
- touched-file `git diff --check`

## Next

Take the first `008.2` execution batch: add the initial CoreAudio-backed host
stream path plus bounded host/runtime buffer-transfer rules in
`signal-host-local`, then prove one topology-aware engine path runs through
that boundary without flattening node/lane/bus structure into callback-local
glue.
