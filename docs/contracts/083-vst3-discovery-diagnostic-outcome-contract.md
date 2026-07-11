# 083 VST3 Discovery Diagnostic Outcome Contract

Status: active
Owner: core-product
Updated: 2026-07-10
Related contracts: `docs/contracts/016-runtime-fault-cause-attribution-and-diagnostic-receipt-contract.md`, `docs/contracts/020-vst3-adapter-baseline-and-runtime-owned-lifecycle-contract.md`
Consumer evidence: `../soundcheck/docs/logs/2026-07/10-201500-g04-native-product-reality-audit.md`

## Purpose

Freeze the adapter-owned outcome boundary for offline VST3 discovery. A bundle
that times out, crashes its helper, or produces invalid data must not collapse
into the same empty list as a valid bundle containing no eligible classes.

This contract is consumer-driven reusable Signal work. It does not reopen or
redirect Signal's active `g10.029` time-stretch lane.

## Authority

- `signal-plugin-vst3` owns VST3 bundle traversal, helper execution, watchdog,
  descriptor projection, and bundle-level diagnostic classification.
- shared hosts/consumers may aggregate diagnostics but must not infer failure
  kind from an empty descriptor list.
- Soundcheck and soundcheck-library own their scan-job/product interpretation,
  not VST3-native classification.

## Discovery Batch

The detailed VST3 discovery entry point returns one batch containing:

- all successfully discovered plugin descriptors
- zero or more bundle diagnostics

Each attempted bundle yields successful descriptors, one diagnostic, or both.
One failed bundle never prevents later bundles from being attempted.

The existing descriptor-only convenience entry point may project the batch's
successful descriptors for active Signal callers. It must delegate to the
detailed path so discovery behavior cannot diverge.

## Diagnostic Shape

Each diagnostic contains:

- kind: `timed_out`, `helper_failed`, or `invalid_data`
- bundle path
- bounded stable detail
- elapsed milliseconds

Classification rules:

- `io::ErrorKind::TimedOut` -> `timed_out`
- non-success helper termination/spawn/process failure -> `helper_failed`
- malformed snapshot, invalid bundle metadata, or unsupported result decoding
  -> `invalid_data`

Do not include raw helper stderr, arbitrary file contents, secrets, backtraces,
or unbounded nested errors.

## Watchdog And Cleanup

- default helper timeout remains ten seconds
- timeout kills and waits for the child before returning
- abnormal exit is waited/reaped
- pipes are consumed only after child completion
- fixtures must prove no child remains after timeout/failure

## Compatibility Posture

Signal is pre-1.0. Do not add aliases or shims for hypothetical consumers.
Retain the descriptor-only method only for verified active callers and implement
it as a projection of the detailed batch.

## Acceptance Gate

- valid multi-class bundle preserves all descriptors
- deterministic timeout produces one `timed_out` diagnostic and no leaked child
- deterministic abnormal exit produces `helper_failed`
- malformed output produces `invalid_data`
- later bundles still run after any earlier failure
- existing Signal VST3 tests and repo health remain green

## Next Task

Soundcheck card 058 implements this narrow boundary without changing runtime hosting.
