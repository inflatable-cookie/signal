# g10.029 Formant And Boundary Diagnostic

Date: 2026-07-10
Roadmap: `g10.029`
Scope: remaining formant and boundary classification

## Measurement

Added one offline, source-relative diagnostic with separate formant and
boundary evidence:

- a `300 Hz`-smoothed spectral envelope over `80-5000 Hz`
- normalized envelope residual at 24 ratio-projected windows
- absolute broad-envelope centroid shift
- gain-invariant exterior-step crest at the head and tail
- absolute exterior step in dBFS

The exterior step is only the transition from digital silence to the first
sample or from the final sample to digital silence. Interior endpoint
derivatives are excluded because they measure ordinary nearby audio, not a
render boundary. Inactive source/output edges below `1e-6` are excluded from
relative crest evidence, while a newly introduced output step above that floor
still measures.

Synthetic proofs cover identity, whole-render gain, a shifted vowel-like
spectral envelope, introduced head/tail steps, inaudible silent-edge noise, and
pitch-preserving `1.5x` projection. The diagnostic allocates and performs FFT
work; it is not an audio-thread surface.

## Formant Result

The 60-row broad pack contains 12 vocal renders. Signal had lower
source-relative broad-envelope residual than Rubber Band in every vocal row.

| Ratio | Rows | Signal envelope residual | Rubber Band | Independent bins | Signal centroid shift | Rubber Band | Independent bins |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `0.75x` | 4 | `0.027999` | `0.075354` | `0.065000` | `34.582 Hz` | `84.204 Hz` | `64.056 Hz` |
| `1.25x` | 4 | `0.019672` | `0.067244` | `0.064099` | `22.693 Hz` | `77.836 Hz` | `50.631 Hz` |
| `1.5x` | 4 | `0.027364` | `0.070154` | `0.040012` | `30.601 Hz` | `81.648 Hz` | `31.973 Hz` |

This does not establish perceptual superiority or exact vowel-formant
tracking. Ratio-projected fixed windows can include small event-alignment
differences. It does show no objective broad-envelope failure in Signal and no
basis for adding a formant correction to the current no-pitch-shift path.

## Boundary Result

Existing full-render evidence remains intact:

- Signal passed absolute integrity in `60/60` rows
- no Signal row added a silence span
- worst Signal endpoint-energy change remained `5.772470 dB`

The exterior sample reveals a narrower issue. Signal's louder exterior edge was
the tail in `59/60` rows. Signal ended above `-20 dBFS` in `17/60` rows versus
`11/60` Rubber Band rows, and exceeded Rubber Band by more than `3 dB` in
`29/60` rows.

| Ratio | Rows | Mean Signal max edge | Rubber Band | Independent bins | Signal above `-20 dBFS` | Rubber Band | Signal > external by `3 dB` |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `0.75x` | 20 | `-27.564 dBFS` | `-31.205 dBFS` | `-29.871 dBFS` | 5 | 1 | 9 |
| `1.25x` | 20 | `-29.187 dBFS` | `-28.700 dBFS` | `-29.023 dBFS` | 7 | 6 | 11 |
| `1.5x` | 20 | `-27.984 dBFS` | `-30.293 dBFS` | `-28.519 dBFS` | 5 | 4 | 9 |

The worst row is pads/sustains source `000870` at `1.25x`: Signal ends at
`-6.328693 dBFS`, Rubber Band at `-13.555 dBFS`, and independent bins at
`-13.310 dBFS`. The same source reaches `-9.687987 dBFS` in Signal at `0.75x`
versus `-33.382 dBFS` externally.

This is an exterior-tail discontinuity classification. It is not output-length
drift, missing endpoint content, added zero fill, or a general head-boundary
failure. Render-plane declick can mask it in product playback, but an offline
artifact must not rely on a later consumer to make its own boundary safe.

Evidence is target-local at
`target/stretch-corpus-g10-029-formant-boundary-v1.tsv`.

## Decision

Close formant classification with no current failure. Keep formant policy in
the later structural checkpoint for pitch shifting and independently reviewed
stereo/vocal behavior, not as a speculative correction to fixed-ratio stretch.

Classify fixed-ratio offline tail anchoring as the remaining boundary defect.
Do not change production DSP in this diagnostic batch. A candidate must reduce
the actual exterior tail step, retain endpoint content, and avoid transient,
tonal-texture, formant-envelope, and full-render integrity regressions across
the same pack before promotion.

## Next Task

Build and gate one bounded offline tail-anchor candidate against the 60-row
exterior-step failures. Keep production unchanged unless the candidate passes
the combined integrity and quality gate. Independent stereo and row-level
listening completion remain external blockers.
