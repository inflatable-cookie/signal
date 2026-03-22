# 2026-03-22 - g08.016 linux live acceptance closure and g08.017 handoff

## Summary

Closed `g08.016` by adding the grouped consumer-facing supervisor export proof
for the shared Linux live acceptance lane, then opened `g08.017` for immersive
render and monitoring acceptance depth.

## Work completed

- widened `signal-supervisor-tools` so the Linux live acceptance lane now
  requires one grouped supervisor export proof spanning Linux live ownership,
  JACK coordination, PipeWire/ALSA parity, and clock-topology truth
- widened `effigy acceptance:linux-live-acceptance-lane` to run that grouped
  export proof alongside the existing grouped descriptor and boundary-local
  acceptance tasks
- closed `docs/roadmaps/g08/016-linux-live-backend-acceptance-and-failure-injection-depth.md`
  and marked contract `067` complete
- opened `docs/roadmaps/g08/017-immersive-render-and-monitoring-acceptance-depth.md`
  as the next active queue
- rolled the shared index and feature-reference trail forward

## Validation

- `effigy test --plan`
- `cargo fmt --all`
- `cargo test -p signal-supervisor-tools export_json_carries_cross_family_linux_live_acceptance_evidence -- --nocapture`
- `cargo test -p signal-supervisor-tools linux_live_acceptance_lane_json_reports_required_and_deferred_policy -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-linux-live-acceptance-lane --format=json`
- `effigy acceptance:linux-live-acceptance-lane`
- `git diff --check`
- `effigy health`
- `effigy qa:docs`

## Next Task

Continue `g08.017` with Batch 17.1 by freezing the shared immersive render and
monitoring acceptance contract on top of the closed immersive room-policy,
deployment-monitoring, renderer-export, and spatial consumer seams.
