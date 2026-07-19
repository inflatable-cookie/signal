# g10.030 Stretch Consolidation

Date: 2026-07-19
Status: complete
Roadmap: `g10.030` Batch 30.1

## Change

The production OfflineHighQuality behavior was frozen. Rejected successor code,
hidden review APIs, candidate-only tests, and experiment report modes were
removed.

Two commits own the code cleanup:

- `43e9a96a` removed the frequency-adaptive research family: `50,397` lines
- `1d1b02f1` removed the remaining rejected renderers and reduced the quality
  harness to the external comparator and blind listening pack: `16,103` lines

Total code deletion: `66,500` lines.

The retained `signal-dsp-stretch` source surface is about `14,835` lines.

## Retained Product Surface

- current `2048/512` OfflineHighQuality renderer
- compression and expansion short-window selectors
- linked stereo, pitch, dynamic ratio, artifact, cache, and RealtimePreview
  contracts
- byte-exact and package regression tests
- Signal-versus-external objective metrics
- five-family, three-ratio level-matched blind listening pack

## Removed Research

- frequency-adaptive and common-grid families
- structural hybrid and adaptive timeline
- fixed-map peak transient treatment
- H/R/P separation and additive rendering
- whole-band phase-gradient rendering
- tail anchors and fades
- stability-adaptive, tracked-peak, magnitude-slew, and compression-anchor modes
- candidate-specific corpus reports and tail listening packs

## Validation

- `cargo test -p signal-dsp-stretch`: passed, `155` tests across retained targets
- `RUSTFLAGS='-D missing-docs' cargo check -p signal-dsp-stretch --lib`: passed
- `cargo fmt -p signal-dsp-stretch --check`: passed
- `git diff --check`: passed
- `effigy health`: passed
- `effigy validate`: passed
- `effigy qa:docs`: passed
- `effigy qa:northstar`: passed
- `effigy doctor`: still reports pre-existing repo findings, but god-file count
  fell from `98` to `57`; the stretch-specific count fell from `50` to `9`

## Decision

`g10.029`, Contract `082`, and Batch 29.7BE are historical. Contract `084` and
`g10.030` now require one complete successor in an isolated branch or worktree.
No candidate code enters `main` before long-form mono and linked-stereo
admission.

## Next Task

Run `g10.030` Batch 30.2. Freeze one end-to-end successor brief before writing
new DSP code.
