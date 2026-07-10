# g10.029 Hybrid Kernel Seam

Date: 2026-07-10
Status: Batch 29.5 complete; mono hybrid ready

## Changed

- grouped the existing phase-vocoder core into explicit analysis, propagation,
  and synthesis state without changing DSP order or math
- locked the current transient-reset output with deterministic sample-bit hash
  `0x8255b18311f778f9`
- added report-only `StretchHybridTrace`, frame-owner, and transition records
- added the frozen short-window classifier with transient guards, tonal hold,
  identity bypass, compression scope, and boundary guards
- added deterministic low-energy transition scheduling bounded to one projected
  short hop and outside transient interiors
- exposed `OfflineHighQualityStretcher::hybrid_trace_review_mono`; it renders
  only the current path needed for placement and never mixes candidate audio

Trace implementation, transition scheduling, and tests live in separate modules
so the batch does not increase the Doctor god-file baseline.

## Proof

- current phase-vocoder sample bits remain exact after state extraction
- stable expansion enters `Tonal` only after four qualified frames
- compression never enters `Tonal`
- a sudden attack receives the frozen pre/post transient guard
- identity stays entirely `Mixed` with no transitions
- transition search stays inside the projected one-hop bound and avoids
  transient interiors
- building the trace leaves repeated current-path output unchanged

No transient, mixed, or tonal branch audio is combined. Production routing,
cache identity, product receipts, pitch/dynamic paths, and RealtimePreview are
unchanged.

## Validation

- `cargo fmt --all --check`
- `cargo test -p signal-dsp-stretch hybrid_trace`
- `cargo test -p signal-dsp-stretch phase_vocoder`
- `cargo test -p signal-dsp-stretch`
- `RUSTFLAGS='-D missing-docs' cargo check -p signal-dsp-stretch --lib`
- Doctor returned to the existing `48` god-file and `5` attention-marker
  baseline after the trace tests and transition scheduler were split out

## Next Task

Start Batch 29.6. Render the short independent-bin transient branch, current
mixed branch, and long identity-lock/reset tonal branch continuously for one
fixed-ratio mono input. Apply only the frozen owner and transition schedule,
including correlation and normalization rejection. Keep the result report-only
until it passes the local crest, timing, tonal, static-spectrum, and full-render
gates.
