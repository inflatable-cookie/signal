# 2026-03-21 - g08.012 preview workflow boundary closure and g08.013 handoff

## Summary

Closed `g08.012` by widening the existing preview boundary so it proves the
runtime-owned preview-browser queue, media audition orchestration, and
transform-scheduling seam without opening a second preview-workflow-only
acceptance lane.

## Work completed

- updated `signal-supervisor-tools` so
  `signal.runtime.preview-transform-boundary` now points at
  `docs/contracts/063-preview-browser-queue-media-audition-and-transform-scheduling-contract.md`
  and explicitly describes the `preview_workflow` receipt family alongside the
  existing preview-transform and preview-device surfaces
- kept the existing `effigy acceptance:preview-transform-boundary` lane as the
  repo-owned proof path instead of creating a second preview-workflow
  acceptance shell
- marked `g08.012` complete, opened `g08.013`, and rolled the next-step
  references through the roadmap and architecture surfaces

## Validation

- `cargo fmt --all`
- `cargo test -p signal-supervisor-tools preview_transform_boundary_text_reports_runtime_and_host_edge_proofs -- --nocapture`
- `cargo test -p signal-supervisor-tools preview_transform_boundary_json_reports_runtime_and_host_edge_proofs -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-preview-transform-boundary --format=json`
- `effigy acceptance:preview-transform-boundary`
- `git diff --check`
- `effigy health`
- `effigy qa:docs`

## Next task

Continue `g08.013` with Batch 13.1 by freezing the first runtime-owned
asset/session transform persistence, retention, and cache placement policy
contract on top of the closed preview-workflow seam.
