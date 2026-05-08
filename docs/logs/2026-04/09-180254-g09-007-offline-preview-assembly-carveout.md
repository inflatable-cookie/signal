# 2026-04-09 - g09.007 offline preview assembly carveout

## Summary

Completed the first strict `g09.007` batch by carving the heavy offline preview
request assembly wall into narrower internal helpers while keeping the public
runtime entrypoint stable.

## Changes

- updated
  `~/Dev/projects/signal/crates/signal-runtime/src/interfaces_offline_contract_family/request_preview.rs`
  to include explicit helper modules for the request-preview family
- added
  `~/Dev/projects/signal/crates/signal-runtime/src/interfaces_offline_contract_family/request_preview/validation.rs`
  for request-level validation
- added
  `~/Dev/projects/signal/crates/signal-runtime/src/interfaces_offline_contract_family/request_preview/stem_targets.rs`
  for stem-target resolution
- added
  `~/Dev/projects/signal/crates/signal-runtime/src/interfaces_offline_contract_family/request_preview/freeze_artifacts.rs`
  for freeze-artifact recall derivation
- reduced
  `~/Dev/projects/signal/crates/signal-runtime/src/interfaces_offline_contract_family/request_preview/request_assembly.rs`
  to the stable orchestration entrypoint over those helpers

## Validation

- `cargo test -p signal-runtime --lib --no-run`
- `effigy health`
- `effigy qa:docs`
- `effigy qa:northstar`

## Outcome

The offline preview assembly seam is now materially smaller and easier to
review by responsibility. The public
`RuntimeOfflineRenderContractPreview::from_runtime_state` entrypoint stayed
stable, and the next meaningful `g09.007` seam is now the broad runtime test
front door in `crates/signal-runtime/src/tests.rs`, not another similarly
heavy internal assembly wall.

## Notes

- `cargo test -p signal-runtime --lib --no-run` still reports the pre-existing
  unused-import warning cluster in `crates/signal-runtime/src/tests.rs`; that
  warning is now the target of the follow-on test-front-door normalization
  batch rather than this carveout.

## Next Task

Continue the active strict `g09` lane from
`docs/roadmaps/g09/batch-cards/005-g09-007-runtime-tests-front-door-normalization.md`.
