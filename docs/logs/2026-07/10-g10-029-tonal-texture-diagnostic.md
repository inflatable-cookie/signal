# g10.029 Tonal Texture Diagnostic

Date: 2026-07-10
Roadmap: `g10.029`
Scope: long-stretch grain and atonal-ringing follow-up

## Operator Finding

The 15-pair blind pack placed Signal close to Rubber Band overall, but Signal
felt slightly grainier and less musical at longer stretches. The reported
texture included a subtle atonal-ringing impression. Stereo remained
unassessed.

## Measurement

Added an offline, source-relative tonal-texture measurement with four separate
outputs:

- L1-normalized spectral residual at 24 ratio-projected source/output windows
- output energy added in bins below the source support floor
- short-cluster frame-to-frame spectral movement relative to source movement
- short-time RMS movement relative to source movement

Spectral normalization makes the first three measurements insensitive to a
whole-render gain difference. Four clusters of eight contiguous frames keep
modulation evidence local instead of comparing unrelated musical sections.
The measurement allocates and performs FFT work; it is diagnostic-only and is
not an audio-thread surface.

Synthetic proofs establish that identity produces negligible residue, a
pitch-preserving `1.5x` tone remains negligible under ratio projection, an
injected `613 Hz` component against a `440 Hz` source raises residual and
sideband evidence, and a `60 Hz` amplitude modulation raises envelope movement.

## Corpus Result

The release report measured the existing 60-row Signal/Rubber Band broad pack.
This decision uses the 40 expansion rows at `1.25x` and `1.5x`.

| Ratio | Rows | Signal residual minus external | Signal sideband minus external | Signal spectral-movement excess | Excess-positive rows | Signal envelope-movement excess |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `1.25x` | 20 | `-0.073883` | `-0.003856` | `+0.003215` | `19/20` | `-0.147166 dB` |
| `1.5x` | 20 | `-0.064281` | `-0.003535` | `+0.005408` | `19/20` | `-0.178505 dB` |

The spectral-movement excess covered all eight vocal rows and seven of eight
pads/sustains rows. Envelope movement did not show the same corpus-wide
direction, so it does not explain the aggregate listening result.

Static spectrum points the other way: Signal remained closer to the projected
source spectrum and added less unsupported-bin mass than the external render.
The current evidence therefore does not confirm a separate added-ringing
failure. The operator's atonal-ringing description can plausibly be the
perceptual result of unstable spectral texture rather than new stationary
sidebands.

The independent-bin draft sharpens the phase-lock tradeoff:

| Ratio | Signal spectral movement | External | Independent-bin draft |
| --- | ---: | ---: | ---: |
| `1.25x` | `0.017165` | `0.013950` | `0.015292` |
| `1.5x` | `0.032058` | `0.026651` | `0.035094` |

At `1.25x`, broad identity locking is less stable than both comparators. At
`1.5x`, it improves on independent propagation but remains less stable than
Rubber Band. Removing phase locking globally is not supported; the earlier
transient controls already showed that local lock changes exchange crest and
timing failures across the corpus.

Evidence is target-local at
`target/stretch-corpus-g10-029-tonal-texture-v1.tsv`.

## Decision

Classify the long-stretch finding as excess fast spectral movement. Do not
classify it as confirmed added sideband/ringing energy. Do not change the
production kernel from this diagnostic alone.

The eventual structural checkpoint must give tonal regions explicit temporal
coherence and multiresolution ownership while retaining separate transient
ownership. This metric is comparative diagnostic evidence, not a promotion
threshold: fixed-window source projection can still include small event
alignment differences, and objective proxies cannot replace the pending
independent listening review.

## Next Task

Close the remaining formant and boundary classification with bounded evidence
from the existing pack. Keep independent stereo and row-level listening
completion open. Do not start the structural hybrid or product promotion.
