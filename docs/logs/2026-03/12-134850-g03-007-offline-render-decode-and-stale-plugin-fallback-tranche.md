# g03.007 - Offline Render Decode And Stale Plugin Fallback Tranche

Date: 2026-03-12
Status: completed tranche
Roadmap: `docs/roadmaps/g03/007-offline-render-freeze-and-stem-export-pipeline.md`

## Summary

Completed the remaining Batch 7.3 parity work in `signal-runtime`. Offline
render now decodes broader cached media formats through a runtime-owned decode
path, and plugin-backed offline execution no longer depends on whatever the
last live plugin render happened to be. Fresh latest-block plugin overrides are
still accepted when they exist, but stale overrides now fall back to the
Signal-owned plugin stage model instead of freezing offline output on old
host-local state.

## Shipped

- added runtime-owned cached media decode beyond WAV by probing and decoding
  non-WAV cache assets during offline render preparation
- kept decoded sample-rate ownership in runtime so offline render can continue
  to align clip treatment, graph execution, and later export-rate conversion
- narrowed cached plugin render override use to fresh latest-block captures
  from the currently bound ready sandbox
- changed stale plugin override behavior so offline render reuses the
  Signal-owned plugin stage model rather than pinning output to stale live
  plugin buffers
- added focused runtime proofs covering:
  - offline render from non-WAV cached media
  - offline render from the plugin stage model with no cached live override
  - offline render fallback after a cached live plugin override becomes stale

## Deferred

- offline render still does not own a dedicated runtime-to-host plugin sandbox
  execution pass for plugin cases that exceed Signal-owned stage modeling
- artifact receipts/report receipts are still separate surfaces rather than a
  stronger manifest bundle for downstream packaging or resume flows
- multichannel decode beyond the current mono/stereo runtime adaptation is
  still outside this tranche

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime`
- `git diff --check`
- `effigy health --repo .`
- `effigy test --repo .`
- `effigy validate --repo .`

## Next Task

Continue `g03.007` with Batch 7.4 by promoting the current artifact/report
receipts into a stronger manifest/report bundle and by defining the explicit
runtime-to-host offline plugin execution boundary for cases that exceed the
Signal-owned stage model before opening `g03.008`.
