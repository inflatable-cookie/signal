# DSP Processing Lab Bootstrap

Status: active
Updated: 2026-04-10
Manifest: `demos/manifests/dsp-processing-lab.demo.json`

## Purpose

Provide one repo-owned DSP inspection surface that wraps the already-frozen
stretch, marker-analysis, and transform-artifact boundary family.

## Operator Checks

- confirm the rendered companion view exists at
  `demos/receipts/dsp-processing-lab.view.html`
- confirm the receipt reports the expected machine-readable boundary ids:
  - `signal.runtime.stretch-boundary`
  - `signal.runtime.marker-analysis-boundary`
  - `signal.runtime.transform-artifact-boundary`
- confirm the receipt records the matching acceptance tasks as passed:
  - `effigy acceptance:stretch-boundary`
  - `effigy acceptance:marker-analysis-boundary`
  - `effigy acceptance:transform-artifact-boundary`
- confirm the receipt stays DSP-focused and does not claim editor-shell,
  waveform-browser, or tutorial UI breadth
- confirm the rendered companion makes stretch, marker-analysis, and
  transform-artifact posture visually inspectable without replacing the receipt
- confirm the coverage matrix only promotes `signal-dsp`,
  `signal-dsp-resample`, and `signal-dsp-spectral` because this manifest and
  receipt exist

## Deferred Scope

- analysis feature-inspector remains a separate `g09.013` planning decision
- this surface does not replace corpus, benchmark, or acceptance automation
- this surface does not claim media-browser, editor, or product-shell workflow

## Next Task

Continue the active strict `g09` lane from
`docs/specs/batch-cards/054-g09-015-dsp-processing-operator-view.md`.
