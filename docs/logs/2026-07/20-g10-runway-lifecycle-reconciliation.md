# g10 Runway Lifecycle Reconciliation

Date: 2026-07-20
Status: complete; routing conclusion superseded by later operator correction
Generation: `g10`

## Result

Reconciled three stale milestone states after the stretch lanes closed:

- `g10.001` is complete. Generation adoption and the remediation runway landed.
- `g10.003` is complete. The legacy `signal-hardware-coreaudio` crate and
  executable `system_profiler` path are absent; cpal owns device enumeration.
- `g10.017` is paused on hardware alignment evidence, not implementation.
  Signal owns input capture, monitor-only and capture-plus-monitor sessions,
  and the render-plane live-input monitor path.

The remaining `g10.017` check is an explicit recorded-take alignment test and
consumer workflow evidence. This batch did not infer or perform cross-repo
work.

## Routing

No feature batch became ready. The operator must choose the next Signal
priority explicitly:

- finish `g10.017` hardware evidence
- reopen `g10.023` with a streaming artifact writer/cache contract
- pull a concrete, consumer-backed item from the post-`g10` backlog

`g10.028`, rejected stretch owners, creative routing, product exposure, and
cross-repo work remain paused or closed.

## Scope

Documentation only. No Signal DSP, hardware, render-plane, runtime, public API,
test, fixture, or harness surface changed. The unrelated local binaural/reverb
edits remained untouched.

## Next Task

Operator selects one listed Signal priority. Do not infer an implementation
batch from another bare continuation while this gate remains unresolved.

Later correction: the operator explicitly reopened the PaulXStretch-like
`Dream` goal. `g10.031` Batch 31.20 is now the only ready batch.
