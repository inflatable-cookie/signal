# g10.029 Exact-Lattice Reassessment

Date: 2026-07-10
Status: complete
Roadmap: `g10.029`
Contract: `082`

## Trigger

The fixed-resolution phase-gradient candidate improved tonal and direct Rubber
Band evidence but failed timing, attack, replica, formant, integrity, and
combined gates.

## Mapping Finding

The candidate fixed synthesis hop to `1024` and repeated one rounded analysis
hop. The internal lattice did not realize the requested ratios:

| Requested | Analysis hop | Actual lattice ratio | Five-second endpoint drift |
| ---: | ---: | ---: | ---: |
| `0.75` | `1365` | `0.750183` | `+40.38` frames |
| `1.25` | `819` | `1.250305` | `+67.31` frames |
| `1.5` | `683` | `1.499268` | `-161.42` frames |

Exact final cropping corrects buffer length, not event positions inside the
render. The measured candidate timing regression was `+16.738760` frames mean
and `+137` frames worst-case, so lattice error is a material confound.

## External Evidence

Röbel's general phase-vocoder equations describe analysis centres `C_l` and
transformed synthesis centres `C'_l`, with phase propagation normalized by the
actual adjacent centre differences. This supports an integer nonuniform
analysis schedule without changing the whole-band phase-gradient family.

Röbel's shape-invariant extension is speech-specific and adds sinusoidal/noise
classification, correlation-based phase alignment, spectral-envelope
estimation, and voiced/unvoiced balance policy. His peak-local transient method
matches the mechanism family Signal already rejected in Batch 29.6C. Neither
enters the next proof.

Primary sources:

- [Röbel, Shape-invariant speech transformation with the phase vocoder](https://www.isca-archive.org/interspeech_2010/robel10_interspeech.html)
- [Röbel, A new approach to transient processing in the phase vocoder](https://www.dafx.de/paper-archive/2003/pdfs/dafx32.pdf)
- [Prusa and Holighaus, Phase Vocoder Done Right](https://arxiv.org/abs/2202.07382)

## Decision

Open Batch 29.6H with absolute integer analysis positions
`A_n = round(n * 1024 / ratio)`. Backward and forward time-phase differences
use their actual adjacent intervals before centered averaging. Synthesis hop,
frequency integration, window/FFT, tolerance, heap, padding, normalization,
crop, and identity policy remain frozen.

The proof first requires at most `0.5` frame mapping error at every analysis
centre. It then runs the unchanged 60-row complete mono gate. No new transient,
shape, separation, or local-time mechanism is authorized. Linked stereo and
product work remain closed.
