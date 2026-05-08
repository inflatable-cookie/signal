# 032 - g09.013 DSP Processing Lab Bootstrap

Status: complete
Owner: core-product
Updated: 2026-04-10
Master spec refs: docs/specs/001-g09-lane-first-strict-adoption.md
Roadmap refs: g09.013
Governing refs: docs/contracts/001-working-rules.md, docs/contracts/046-sample-domain-time-stretch-engine-contract.md, docs/contracts/047-warp-marker-transient-anchor-and-tempo-assist-analysis-contract.md, docs/contracts/048-post-warp-render-cache-and-transform-artifact-contract.md, docs/contracts/079-interactive-demo-binary-and-crate-capability-proof-contract.md, docs/specs/001-g09-lane-first-strict-adoption.md, docs/roadmaps/g09/013-dsp-graph-analysis-interactive-demo-suite-and-audit-closeout-proof.md
Auto-start next card: no

## Objective

Promote the first honest DSP demo seam by turning the existing stretch,
marker-analysis, and transform-artifact boundary family into one repo-owned DSP
processing-lab scenario.

## Scope

- stay inside the current `signal-supervisor-tools` DSP descriptor commands and
  the existing `effigy acceptance:stretch-boundary`,
  `effigy acceptance:marker-analysis-boundary`, and
  `effigy acceptance:transform-artifact-boundary` tasks
- build one repo-owned manifest, operator notes file, launch task, and receipt
  under `demos/`
- keep the surface focused on DSP processing meaning and do not widen into an
  editor shell, media browser, or tutorial UI
- promote `signal-dsp`, `signal-dsp-resample`, and `signal-dsp-spectral` to
  live coverage only if the manifest and receipt are actually in place
- treat runtime, host, and supervisor crates as reused proof transport, not new
  ownership claims for this batch

## Steps

1. Freeze the bounded DSP seam from contracts `046`, `047`, `048`, and `079`.
2. Add a DSP processing-lab manifest, operator notes, receipt path, and Effigy
   launch task under `demos/`.
3. Implement a repo-owned wrapper that runs the current stretch,
   marker-analysis, and transform-artifact descriptor commands plus their
   acceptance lanes, then emits one machine-readable receipt.
4. Keep the surface explicit about what it proves: stretch posture,
   marker-analysis posture, and transform-artifact truth from one DSP-focused
   inspection family.
5. Update the roadmap, coverage matrix, and strict currentness surfaces if the
   batch closes cleanly.

## Acceptance Criteria

- one repo-owned launch surface captures the current DSP boundary family without
  flattening it into a generic product demo
- the receipt keeps DSP processing meaning explicit across stretch,
  marker-analysis, and transform-artifact seams
- the batch stays bounded to the DSP processing-lab bootstrap seam
- focused validation passes

## Evidence Required

- batch log for the next `g09.013` tranche
- validation actually run
- explicit note which analysis demo seams remain deferred after this batch

## Outcome

- added the live DSP processing-lab surface under `demos/` with one repo-owned
  manifest, scenario notes file, launch script, Effigy task, and
  machine-readable receipt
- promoted `signal-dsp`, `signal-dsp-resample`, and `signal-dsp-spectral` to
  live demo coverage in the coverage matrix
- repaired stale DSP acceptance wiring so the frozen stretch,
  marker-analysis, and transform-artifact proof family now executes cleanly
  through the shared demo wrapper
- left the next `g09.013` seam unpromoted because analysis feature-inspector
  still wants a clearer single-surface operator posture before another honest
  ready card is claimed

## Stop Conditions

- the work starts redesigning DSP/runtime behavior instead of wrapping the
  current descriptor and acceptance surfaces
- the seam needs fresh planning about analysis operator posture before the DSP
  demo can execute honestly
- the batch starts implementing a product shell, waveform editor, or generalized
  tutorial UI

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether the next
honest `g09.013` seam is analysis feature-inspector bootstrap or a continued
planning pause.
