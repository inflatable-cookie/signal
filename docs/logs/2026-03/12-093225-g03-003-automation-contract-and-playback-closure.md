# g03.003 - Automation Contract And Playback Closure

Date: 2026-03-12
Owner: core-product
Roadmap: `docs/roadmaps/g03/003-automation-engine-and-high-resolution-parameter-playback.md`

## Summary

Closed `g03.003` by turning runtime automation projection into a typed
execution contract and proving deterministic multi-block playback through the
existing graph parameter-batch seam.

Implemented in this tranche:

- `RuntimeAutomationLaneProjection` now carries an explicit target
  (`node_id` plus `parameter_id`), interpolation family (`Hold` or `Linear`),
  and per-lane playback resolution (`ramp_step_samples`,
  `max_sub_blocks`).
- `RuntimeAutomationState` now derives transport-aware graph parameter batches
  from automation projections, including block-start values, in-block point
  events, and sampled linear ramp points for cross-block playback.
- `RuntimeAutomationSnapshot`, compact observation text, multiline reports,
  and JSON export now expose projected lane and segment counts, hold/linear
  lane counts, and last-batch playback-policy metadata.
- `signal-graph` now has an explicit dense-event proof that bounded
  `SplitAtEvents` strategies coalesce parameter changes to the configured
  sub-block budget instead of expanding without limit.

## Evidence

Focused proofs landed in:

- `crates/signal-graph/src/lib.rs`
  - `dense_parameter_batches_are_coalesced_by_max_sub_block_budget`
- `crates/signal-runtime/src/runtime.rs`
  - `automation_projection_requires_explicit_targets_and_positive_linear_resolution`
  - `runtime_automation_projection_drives_within_block_parameter_events`
  - `runtime_linear_automation_projection_drives_multi_block_gain_playback`
  - `runtime_hold_automation_projection_drives_plugin_backed_threshold_fixture`

Those checks prove:

- automation targets are validated as explicit Signal-owned engine paths rather
  than free-form host strings
- hold and linear playback stay deterministic across block boundaries
- plugin-backed and stage-backed targets both realize automation through the
  same runtime-owned batch path
- dense automation stays bounded by declared sub-block policy
- runtime observation/export surfaces carry enough playback-policy metadata to
  debug automation timing without host-local inference

## Deferred Scope

Still deferred on purpose:

- no curve families beyond `Hold` and `Linear` yet
- no higher-level mixer aliases, grouped targets, or product-local automation
  naming layers beyond explicit node/parameter addressing
- no richer arbitration policy for overlapping lanes targeting the same
  parameter; the runtime currently relies on deterministic merged batch order
  rather than lane priority semantics

## Validation

Passed:

- `cargo fmt --all`
- `cargo test -p signal-graph`
- `cargo test -p signal-runtime`
- `effigy health --repo .`
- `effigy test --repo .`
- `effigy validate --repo .`
- `git diff --check`

## Next Task

Execute `g03.004` by defining reusable tempo-map ownership, warp modes, and
realized playback state surfaces before proving degraded and not-ready warp
reporting through `signal-runtime`.
