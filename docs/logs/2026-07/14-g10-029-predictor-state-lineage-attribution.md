# Predictor State-Lineage Attribution

Date: 2026-07-14
Roadmap: `g10.029`, Batch 29.6CQ
Scope: report-only horizontal versus corrected-state recurrence

## Decision

Vertical-state feedback is not necessary for the frame-rate sideband.

A phase oracle carries prior horizontal state while synthesizing at current
input magnitude. This removes raw horizontal recurrence's startup-magnitude
memory without changing its phase lineage. The oracle is cleaner than prior
corrected-state feedback for every isolated tone and the mixture, but it still
fails the frozen `-60 dB` ceiling.

## Evidence

| Tone | Corrected feedback | Horizontal phase | Improvement | Phase variance | Spur offset | Phase hash |
| ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `110 Hz` | `-26.719393 dB` | `-41.444546 dB` | `14.725153 dB` | `1.703e-7` | `33.554688 Hz` | `c444b98538042212` |
| `164.8138 Hz` | `-37.780438 dB` | `-41.828961 dB` | `4.048523 dB` | `3.853e-9` | `33.466144 Hz` | `af17c2deab11133d` |
| `220 Hz` | `-23.586788 dB` | `-48.535978 dB` | `24.949190 dB` | `7.572e-10` | `33.417969 Hz` | `b606b0323bac0dba` |
| `329.6276 Hz` | `-51.511127 dB` | `-52.739473 dB` | `1.228346 dB` | `5.833e-11` | `33.240881 Hz` | `5f089be7738bf6b0` |

Every isolated phase-oracle spur is within `0.222 Hz` of the
`33.333333 Hz` output frame rate. Mixed leakage improves from `-29.975234` to
`-41.558047 dB`; the phase-oracle mixed hash is `3f0d01c020563c31`.

Hashes and measurements repeat exactly. No candidate equation, geometry,
scheduling, window, vertical normalization, fallback, dependency order, or
overlap choice changes.

## Interpretation

Direct horizontal phase recurrence can produce the modulation without
vertical-state feedback. It is also substantially cleaner than corrected-state
feedback. This does not select another horizontal equation: independent-bin
horizontal transport is an incomplete intermediate field, not the specimen's
complete output.

Testing intermediate spectra against the final-output ceiling has reached its
limit. The next discriminator is the pinned complete upstream engine under the
same synthetic measurement. If upstream passes, Signal still has a translation
divergence. If upstream fails, the current Rule 31G target is not attainable by
this studied topology.

## Closed Lanes

- equation, floor, weight, window, interval, distance, and FFT changes
- corpus, holdout, listening, stereo, and dynamic ratio
- cache and production routing

## Next Task

Run Batch 29.6CR. Measure pinned Signalsmith Stretch revision `57b93f4e` on the
frozen tones and chord under the same final-output gate. Keep the comparator
report-only and out of production dependencies.
