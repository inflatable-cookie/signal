# g10.029 Phase-Gradient Kernel Proof

Date: 2026-07-10
Status: complete
Roadmap: `g10.029`, Batch 29.6F
Contract: `082`

## Change

Added one report-only mono full phase-gradient kernel. It uses the frozen
`4092`-sample Hann window, `8192` FFT, `1024` synthesis hop, nearest-integer
ratio-derived analysis hop, centered time/frequency phase differences,
trapezoidal propagation, and relative tolerance `1e-6`.

The first synthesis frame copies analyzed phase. Explicit padded predecessor
and future frames make every derivative consumed by later trapezoidal
propagation centered. Integration operates on the nonredundant spectrum with
stable heap tie breaks. Bins below tolerance retain analyzed phase. Mirrored
synthesis bins enforce conjugate symmetry. Normalized overlap-add and an exact
crop retain the sample-domain length contract.

The path is available only through the hidden review method. Product routing,
cache identity, corpus reporting, stereo, dynamic ratio, and RealtimePreview
are unchanged.

## Evidence

All controls passed with zero duplicate assignments, zero missing assignments,
zero uncovered output samples, finite derivatives and output, conjugate error
at or below `1e-6`, and deterministic sample/trace hashes.

At `1.5x`:

| Control | Horizontal | Vertical | Significant bins | Heap high-water |
| --- | ---: | ---: | ---: | ---: |
| steady sine | 17104 | 18310 | 35414 | 4098 / 8194 |
| two tone | 20673 | 27422 | 48095 | 4099 / 8194 |
| linear chirp | 39128 | 64065 | 103193 | 4098 / 8194 |
| impulse | 8198 | 16384 | 24582 | 4098 / 8194 |

Bit-exact identity, silent input, deterministic repeat, and `0.75x`
compression exact-length/coverage controls also passed.

## Decision

Batch 29.6F proves the kernel mechanism. It does not prove sound quality.
Batch 29.6G is open for the unchanged 60-row complete mono corpus gate. No
geometry, tolerance, derivative, or heap-priority tuning is authorized inside
that gate. Linked stereo remains closed.
