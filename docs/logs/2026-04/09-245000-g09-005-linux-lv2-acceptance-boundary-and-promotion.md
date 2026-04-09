# 2026-04-09 - g09.005 linux lv2 acceptance boundary and promotion

## Summary

Closed `g09.005` by adding a focused Linux-native LV2 acceptance boundary and
promoting the milestone.

## Changes

- added `effigy acceptance:linux-lv2-execution-boundary` in
  `/Users/betterthanclay/Dev/projects/signal/effigy.toml`
- added the supervisor-tools boundary descriptor
  `signal.runtime.linux-lv2-execution-boundary`
- wired the new boundary through:
  - supervisor schema runtime contract constants
  - CLI mode and describe flag parsing
  - runtime-core describe dispatch
  - boundary descriptor assertions and supervisor main tests
- updated `g09.005` roadmap state to `complete`
- promoted `g09.005` and moved the generation next action to `g09.006`

## Validation

- `cargo check -p signal-supervisor-tools`
- `cargo test -p signal-supervisor-tools parse_args_supports_plugin_and_linux_boundary_modes -- --exact --nocapture --test-threads=1`
- `effigy acceptance:linux-lv2-execution-boundary`

## Outcome

Signal now has one repo-owned Linux LV2 acceptance surface that proves real LV2
discovery plus broker-backed bounded execution and one recovery-owned host lane
through stable runtime-owned receipts and a stable server-host export surface.
That closes the milestone’s remaining production-proof gap; the residual LV2
demo and broader UI or atom-schema depth stays deferred to later demo and Linux
breadth milestones.

## Next Task

Start `g09.006` with one meaningful structural-repair batch: audit the largest
remaining shared execution and recovery duplication between `signal-host-local`,
`signal-host-server`, and `signal-runtime`, then land one broad consolidation
pass on the highest-leverage seam before widening further host-facing behavior.
