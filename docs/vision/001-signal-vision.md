# 001 Signal Vision

Status: active
Owner: core-product
Product owner: Inflatable Cookie
Purpose: define the long-horizon role of Signal as the shared audio-systems
stack for Loophole, Finch, and future apps.

## Long-Term Outcome

Deliver one reusable audio stack where DSP, analysis, graph execution, and
runtime coordination are written once and consumed across multiple products
without duplicating core signal-processing work.

## Strategic Constraints

- Keep reusable DSP, analysis, and graph logic inside Signal-owned crates.
- Keep plugin SDK glue, hardware shims, and other trust-edge integrations thin
  and replaceable.
- Preserve real-time safety in engine/runtime paths.
- Treat untrusted plugin code as the default out-of-process isolation target.
- Keep research, architecture, and roadmap authority inside Signal for
  DSP/analysis topics that Finch and Loophole both depend on.

## Target Envelopes

- Reuse envelope: Finch and Loophole should consume the same algorithm crates
  for core analysis and DSP.
- Isolation envelope: first-party modules should not pay unnecessary IPC costs
  solely for historical process-topology reasons.
- Compatibility envelope: native shims remain acceptable where plugin formats
  or device APIs force them.
- Delivery envelope: migration should happen incrementally, with the existing
  C++ runtime remaining a temporary compatibility island until Rust-owned
  components replace it.

## Alignment Signals

- New DSP and analysis work lands in Signal, not in app-local repos.
- Finch docs point back to Signal for algorithm and crate-shape authority.
- Loophole architecture treats Signal as a sibling shared repo rather than a
  child workspace folder.
- Runtime-host boundaries map to trust edges, not to first-party ownership
  boundaries.

## Next Task

State the vision baseline as current: refresh this file only when Signal
needs a materially new long-horizon constraint or milestone map. Do not fold
delivery sequencing into vision docs.
