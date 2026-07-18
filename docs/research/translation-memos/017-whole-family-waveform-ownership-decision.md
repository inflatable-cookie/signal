# Whole-Family Waveform-Ownership Decision

Status: promoted
Date: 2026-07-18
Roadmap: `g10.029`, Batch 29.7AD
Contract: `082`, Rule 31M

## Question

Choose at most one complete Signal-native family after transform-domain
post-projection closed. Compare source-synchronous overlap-add, explicit
sinusoidal models, and single-grid transform synthesis. The selected family
must own polyphonic tone, transients, stereo, and bounded execution together.

## Family Decision

| Family | Tone ownership | Transient ownership | Stereo ownership | Signal evidence | Decision |
| --- | --- | --- | --- | --- | --- |
| WSOLA and pitch-synchronous overlap-add | one dominant waveform lag | copied grains; detector variants alter the local map | one shared lag is possible but no reviewed arbitrary-stereo law | prior adaptive timing and replica failures | close as universal engine |
| sines+transients+noise | explicit partial tracks | detected and separately modeled transient component | no reviewed joint partial, transient, and residual TSM law | additive component TSM failed broadly | research reserve only |
| single-grid state-complete phase vocoder | ordinary instantaneous-frequency advance plus peak-region lock | reset state inside the same phase machine | shared decisions and common region rotation inside synthesis | coherent mono is competitive; 29.7T reached `1/48` stereo failures | select one calibrated proof |

### Source-synchronous overlap-add

WSOLA is deterministic, bounded, and capable on speech and simpler musical
signals. Its correlation search preserves the most prominent local waveform
period. That is also its universal-engine limit: polyphonic mixtures contain
simultaneous periods, while one lag preserves only the dominant one. Published
reviews identify warble, transient doubling during expansion, and transient
skipping during compression. Transient-safe variants hold local stretch at
unity and compensate elsewhere, reopening the event-timing mechanism that
Signal already rejected.

WSOLA remains a useful control for an explicitly percussive or monophonic mode.
It is not the next universal pro-grade engine.

### Sinusoidal and sines+transients+noise models

Verma and Meng define a complete musical model: tracked sinusoids preserve
pitch, separately modeled transients preserve edges, and a stochastic residual
remains noise-like. Jang and Park extend sinusoidal TSM with multiresolution
analysis and dynamic segmentation for polyphonic audio. These methods directly
address the audible brief.

They do not close Signal's current boundary. The complete output is still the
sum of independently modified tonal, transient, and noise components. Signal's
H/R/P proof reconstructed the source before modification but then failed
timing, replicas, timbre, formants, and boundaries after separate component
TSM. The reviewed sinusoidal sources supply no paired-channel law that jointly
tracks partial phase, transient waveform, stochastic covariance, and their
recombination through stereo synthesis. Selecting that family would reopen the
same unowned seam with a more complicated separator.

Retain explicit partial modeling as a later specialist or offline research
direction. Do not select it for the next universal proof.

### Single-grid state-complete phase vocoder

This is the only family supported at every required boundary:

- phase-vocoder instantaneous-frequency advance owns every bin on one timeline
- identity phase locking owns vertical coherence and reduces phasiness and
  transient smear
- reset and unlocked states remain inside the same synthesis machine rather
  than becoming additive components or post-hoc repairs
- linked-channel decisions occur before inverse synthesis; peers share the
  selected region operation
- fixed frame geometry gives finite memory and work per output frame

External evaluation does not prove Signal quality, but it supports the family:
Roberts and Paliwal found identity-phase-locked PV best overall and best on
music in their objective study, while Elastique led solo instrument and voice.
Rubber Band documents a block phase-vocoder architecture with transient resets,
vertical coherence treatment, and extensive tuning across material and ratio.
Signalsmith independently places multichannel relation ownership inside its
synthesis iteration.

Signal evidence is stronger than analogy. The coherent single-grid kernel is
competitive with Rubber Band on the long mono pack. The separate 29.7T
complete-region kernel reduced calibrated stereo failures from `20/48` to
`1/48` and preserved mono parity. Its remaining 11 tone-local misses show that
universal region locking is incomplete; they do not reject the whole
state-complete family.

## Selected Architecture

Select `StateCompleteLinkedPhaseVocoder` for one report-only calibrated proof.
It is a new family boundary, not promotion of 29.7T and not another repair on
the coherent weighted predictor.

The topology is fixed:

1. one periodic-Kaiser, modified-half-bin STFT grid and one absolute source-to-
   output schedule
2. ordinary instantaneous-frequency advance computed for every active bin
3. one cancellation-safe joint energy and decision field across channels
4. mutually exclusive `Reset`, `Locked`, `Unlocked`, and `Silent` ownership
5. predecessor-compatible peak regions use one common current-analysis-relative
   rotation across linked channels
6. reset regions use current analysis phase; unlocked regions retain ordinary
   advance; silence remains exact
7. one inverse transform and normalized overlap synthesis per channel; no
   second phase owner, coefficient repair, component sum, or image correction

The first proof stays single-grid and fixed-ratio. Frequency partitioning,
adaptive windows, source separation, waveform splicing, random diffusion,
pitch shift, dynamic ratio, realtime, and product routing remain closed.

## Calibration Boundary

The prior one-shot rule treated every numeric state policy as an architectural
hypothesis. That produced repeated rejection without learning how the selected
states interact. Rubber Band's author explicitly identifies tuning across input
material and parameters as a material part of the implementation. Signal will
therefore calibrate one frozen architecture without tuning on the final
validation set.

Only six policy controls may vary:

1. peak prominence
2. predecessor frequency tolerance
3. transient energy-rise threshold
4. reset support
5. unlock coherence threshold
6. linked-history compatibility tolerance

Batch 29.7AE must freeze physical bounds, quantization, ordering, and a maximum
of 64 deterministic development candidates before the first render. Transform
geometry, windows, phase equations, state inventory, channel owner, synthesis,
metrics, and development rows do not vary. A staged search may use short
development rows first and may advance at most four candidates to the complete
development matrix.

After development, exactly one candidate is frozen. The existing six-row
family-balanced holdout remains unread until Batch 29.8 concealed listening.
Holdout failure permits no policy change. Comparator audio may define metric
envelopes but is never a waveform target.

## Acceptance

Development selection and concealed holdout keep separate roles.

- mechanics: exact length, coverage, finite output, silence, identity, pan,
  swap, polarity, owner change, trajectory break, and repeat hash
- stereo development: zero calibrated failures and zero local-consistency
  failures on the existing `48` rows
- mono development: no row-complete regression against coherent Signal or
  frozen 29.7T on the existing synthetic and six-row long-form objective gate
- holdout: remains Batch 29.8; require at least `4/6` concealed preference over
  coherent Signal, no repeatable broad defect, and retained hard mechanics
- stereo listening: requires an independent suitable listener because the
  operator cannot assess stereo image; objective stereo gates remain mandatory

The objective measure itself is not promotion evidence. Roberts and Paliwal
report strong but imperfect correlation with subjective sessions. Signal uses
objective metrics to reject broken candidates, then requires listening.

## Clean-Room Boundary

Papers, official architecture descriptions, pinned source behavior, and Signal
measurements may define states, ordering, failure cases, and validation.
Rubber Band GPL expression, constants, thresholds, ranges, masks, and lookup
tables do not transfer. Signal's numeric policies come only from declared
physical bounds and its own development corpus. Paper review remains distinct
from patent freedom-to-operate review.

## Sources

- [Roelands and Verhelst, Waveform Similarity Based Overlap-Add](https://www.isca-archive.org/eurospeech_1993/roelands93_eurospeech.html)
- [Driedger and Muller, A Review of Time-Scale Modification of Music Signals](https://www.mdpi.com/2076-3417/6/2/57)
- [Verma and Meng, Time Scale Modification Using a Sines+Transients+Noise Signal Model](https://dafx.de/paper-archive/details/pmF6ZgSLbsx9SOayuwZq5g)
- [Jang and Park, Multiresolution Sinusoidal Model with Dynamic Segmentation for Timescale Modification of Polyphonic Audio Signals](https://doi.org/10.1109/TSA.2004.841048)
- [Roberts and Paliwal, An Objective Measure of Quality for Time-Scale Modification of Audio](https://arxiv.org/abs/2006.06153)
- [Rubber Band technical notes](https://breakfastquay.com/rubberband/technical.html)
- [Signalsmith Stretch design](https://signalsmith-audio.co.uk/writing/2023/stretch-design/)
- [Signal linked phase-field family evidence](./011-linked-phase-field-kernel-family-selection.md)
- [Signal material-state boundary](./012-material-state-phase-architecture-boundary.md)

## Next Task

Batch 29.7AE closes without a frozen candidate. Candidate `0` retains the
29.7T boundary at `1/48` calibrated and `11/48` local failures; three
state-changing finalists retain the calibrated miss and worsen local results.
Run Batch 29.7AF. Trace that persistent off-bin `2.0x` tone and the eleven local
misses to one coefficient, inverse-frame, or overlap operation. Do not change
policy values or open the untouched holdout.
