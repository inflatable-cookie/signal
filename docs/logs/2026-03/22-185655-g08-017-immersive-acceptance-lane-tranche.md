# 2026-03-22 - g08.017 immersive acceptance lane tranche

## Summary

Materialized the first repo-owned immersive render and monitoring acceptance
descriptor and Effigy lane on top of the already-closed spatial boundary.

## Work completed

- widened `signal-supervisor-tools` with
  `signal.runtime.immersive-acceptance-lane` and the
  `--describe-immersive-acceptance-lane` machine-readable surface
- added `effigy acceptance:immersive-acceptance-lane` to compose the
  already-closed spatial boundary proof with the grouped immersive descriptor
- rolled the `g08.017` contract, roadmap, and feature-reference trail forward
  so the next remaining step is the grouped consumer proof closure

## Validation

- `cargo fmt --all`
- `cargo test -p signal-supervisor-tools parse_args_supports_describe_immersive_acceptance_lane_mode -- --nocapture`
- `cargo test -p signal-supervisor-tools immersive_acceptance_lane_json_reports_required_and_deferred_policy -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-immersive-acceptance-lane --format=json`
- `effigy acceptance:immersive-acceptance-lane`
- `git diff --check`
- `effigy health`
- `effigy qa:docs`

## Next Task

Continue `g08.017` with Batch 17.3 by proving the widened immersive render and
monitoring acceptance seam through shared runtime, supervisor, and stable
host-edge surfaces without introducing a renderer-private or workflow-local
acceptance shell.
