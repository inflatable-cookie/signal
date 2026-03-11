# DSP And Analysis Feature Reference

Status: active
Owner: core-product
Updated: 2026-03-11
Vision refs: `docs/vision/001-signal-vision.md`
Architecture refs: `docs/architecture/system-architecture.md`, `docs/architecture/package-map.md`

## Purpose

Document the DSP and analysis functionality that is implemented today in the
Rust workspace. This file is intentionally narrower than the research docs and
the package map: it describes shipped code paths, current result surfaces, and
clear scope limits.

## Scope Summary

The implemented DSP and analysis surface currently lives in these crates:

- `signal-primitives`
- `signal-dsp`
- `signal-dsp-spectral`
- `signal-analysis`
- `signal-analysis-rhythm`
- `signal-analysis-tonal`
- `signal-analysis-loudness`
- `signal-analysis-character`

Everything in this document is based on the current crate implementations under
`crates/`, not on roadmap intent.

## Shared Foundations

### `signal-primitives`

Current features:

- `Sample = f32`
- `SampleRate`, `FrameCount`, `ChannelCount`, `Seconds`, `FrequencyHz`, and
  `GainLinear` newtypes
- sample-rate helpers for seconds-to-frames and frames-to-seconds conversion
- `ChannelLayout` with `Mono`, `Stereo`, and arbitrary counted layouts
- `StepSegment` for simple sample-accurate range/value tagging
- `AudioBuffer` with interleaved sample storage
- buffer construction via zeroed allocation or `from_interleaved`
- direct sample slice access with mutable and immutable views
- channel-count introspection
- buffer clearing
- mono mixdown via channel averaging
- `seconds_per_frame()` helper

Current constraints:

- no planar buffer type
- no timestamped transport/range model beyond frame/sample-rate helpers
- analysis code typically converts buffers to mono before processing

### `signal-analysis`

Current features:

- `AnalysisMode::{Offline, Streaming}`
- `Confidence(f32)` with clamped `0.0..=1.0` construction
- `AnalysisStage<Output>` trait with `mode()` and `analyze()`

Current constraints:

- all implemented analysis stages currently report `AnalysisMode::Offline`
- the crate is a shared contract layer, not an algorithm host

## DSP Crates

### `signal-dsp`

Current features:

- `DspKernel` trait with `reset()`, bypass control, and in-place block
  processing
- mix helpers:
  - `Gain`
  - `apply_gain_in_place()`
  - `sum_in_place()`
  - `mix_in_place()`
  - `clear_block()`
- control helpers:
  - `LinearRamp`
  - `ExponentialRamp`
  - `SmoothedValue`
  - `ControlSegment`
  - `ControlSegmentShape`
  - `ControlPlan`
  - `ControlSegmentPlayer`
- stateful kernels:
  - `DelayLine`
  - `OnePoleLowPass`
  - `PeakMeter`
  - `RmsMeter`
  - `EnvelopeFollower`
- block helpers that apply sample-accurate control streams to kernels:
  - `apply_gain_control()`
  - `process_low_pass_with_cutoff_control()`
  - `process_delay_with_feedback_control()`
- deterministic signal fixtures for tests and examples through `SignalFixture`
- denormal flushing helpers

Current constraints:

- no EQ families beyond a one-pole low-pass
- no dynamics processors such as compressors, gates, or limiters
- no resampling, convolution, oscillators, or nonlinear transfer kernels
- block helpers assume the caller manages channel layout and multichannel
  routing externally

### `signal-dsp-spectral`

Current features:

- `StftConfig { window_size, hop_size, compute_phases }`
- `hann_window(size)` helper
- `Stft::analyze_mono()` forward STFT over mono audio
- zero-padded final frame handling
- `Spectrogram` result with:
  - sample rate
  - STFT config
  - per-frame magnitudes
  - per-frame phases
- `Spectrogram::bins()` convenience method
- `Spectrogram::chroma()` pitch-class energy summary over all frames
- `Spectrogram::chroma_with_reference(reference_hz)` for non-440 tuning
  references
- `Spectrogram::spectral_centroid()` for median framewise brightness
- mel-scale helpers:
  - `MelScale`
  - `LogCompression`
  - `MelFilterNorm`
  - `MelFilterbankConfig`
  - `MelSpectrogramConfig`
  - `MelSpectrogram`
  - `Spectrogram::to_mel_spectrogram()`

Implementation details that are part of the current behavior:

- FFT is provided by `rustfft`
- analysis uses a Hann window
- only the non-redundant positive-frequency bins are retained
- phase extraction can be skipped entirely when `compute_phases` is `false`
- chroma uses a `1/frequency` weighting so higher octaves do not dominate the
  pitch-class profile
- chroma can shift pitch-class boundaries around an explicit tuning reference
- chroma skips bins that are too low-frequency for reliable semitone
  resolution and bins above `5 kHz`
- chroma is globally normalized so the 12 bins sum to `1.0` when energy exists
- spectral centroid is reduced by median across frames for transient-robust
  brightness reporting
- mel conversion is an offline projection from the linear spectrogram, not a
  separate transform implementation

Current constraints:

- mono-only analysis surface
- no inverse STFT
- no streaming/incremental STFT API
- no spectral rolloff, flux, contrast, or MFCC surface
- no tuning estimator beyond caller-supplied chroma reference

## Rhythm Analysis

### `signal-analysis-rhythm`

Primary surface:

- `BeatTracker`
- `BeatTrackerConfig`
- `BeatAnalysisResult`

Default configuration:

- STFT window size: `2048`
- STFT hop size: `512`
- tempo search range: `70.0..=180.0 BPM`
- beat tolerance: `0.2` beat periods

### Rhythm Input Model

Current behavior:

- accepts `AudioBuffer` through `AnalysisStage`
- always mixes input to mono before analysis
- runs as offline analysis only

### Implemented Onset Features

The beat tracker does not rely on one onset function. It builds a weighted
multifeature onset envelope from these cues:

- spectral flux
- bandwise spectral flux over 6 spectral bands
- complex-domain spectral difference using phase prediction
- high-frequency content
- energy flux from time-domain RMS windows

Current combination weights:

- spectral flux: `0.28`
- bandwise spectral flux: `0.22`
- complex-domain difference: `0.30`
- high-frequency content: `0.12`
- energy flux: `0.08`

After cue fusion the envelope is:

- sharpened against a local mean baseline
- boosted on rising edges
- normalized to unit peak

### Implemented Tempo Estimation

Current tempo estimation behavior:

- computes autocorrelation-like lag scores over the onset envelope
- scores each lag with weighted support from:
  - the base lag
  - the `2x` lag
  - the `3x` lag
- searches only within the configured BPM range
- extracts local lag maxima as tempo candidates
- suppresses near-duplicate lag candidates within two lag bins
- refines the winning lag with parabolic interpolation
- chooses beat phase by maximizing beat-aligned energy while penalizing strong
  offbeat energy

The returned tempo surface includes:

- a primary BPM estimate
- up to three public tempo candidates
- a confidence score
- a separate tempo ambiguity score

The ambiguity score is not just runner-up proximity. It is strengthened when
the top two hypotheses look related by common meter/subdivision ratios such as
approximately `2x`, `1/2x`, or `3/2x`.

### Implemented Beat Tracking

Current beat tracking behavior:

- projects a beat grid from the chosen lag and phase
- searches locally around each expected beat using the configured tolerance
- tracks both forward and backward from the chosen phase anchor
- de-duplicates beat frames after local refinement
- refines beat positions to sub-frame resolution with parabolic peak
  interpolation
- refines the reported BPM from beat-to-beat intervals after outlier filtering

Returned beat outputs:

- `beat_positions_seconds`
- `onset_envelope`

### Implemented Tempo Diagnostics

`BeatAnalysisResult::tempo_diagnostics` currently includes:

- local interval tempo points measured beat-to-beat
- four-beat window tempo points
- median BPM
- drift span in BPM
- mean absolute deviation in BPM
- a core-window view that excludes the boundary windows when enough windows
  exist
- boundary-bias measurement for edge instability
- trend diagnostics
- beat-grid error diagnostics

Trend diagnostics currently expose:

- `Stable`, `Accelerating`, or `Decelerating`
- fitted start BPM
- fitted end BPM
- total drift BPM
- slope in BPM per beat
- fit mean absolute deviation

Beat-grid error diagnostics currently expose:

- residual per beat against a linear best-fit grid
- anchored drift per beat against a fixed-interval grid
- mean absolute residual
- max absolute residual
- edge mean absolute residual
- core mean absolute residual
- end anchored drift
- mean absolute anchored drift

### Implemented Tempo Interpretation Layer

The tracker has a second layer above raw BPM estimation. It interprets the
diagnostics and recommends how much the caller should trust the result.

Current interpretation support signals:

- core-window consensus
- drift stability
- beat-grid stability
- closeness to an integer BPM
- boundary pressure

Current interpretation outputs:

- trust level:
  - `Stable`
  - `Guarded`
  - `Tentative`
- recommendation:
  - `UseRefined`
  - `UseCoreWindow`
  - `SnapInteger`
  - `Defer`
- reason:
  - `StableRefinedPulse`
  - `StableCoreWindow`
  - `NearIntegerPulse`
  - `UnstableTempo`
- recommended BPM
- optional snapped BPM
- support breakdown
- interpretation profile

This means the crate already implements three distinct tempo behaviors:

- keep the refined beat-grid BPM
- prefer the core window median when edges are unstable
- snap to a nearby integer BPM when the evidence is strong enough

### Implemented Tempo State And Continuity Model

Above tempo interpretation there is a state/recommendation surface for callers
that want continuity semantics rather than just one BPM number.

`BeatAnalysisResult::tempo_state` currently includes:

- action:
  - `Lock`
  - `Monitor`
  - `Defer`
- reason:
  - `StableIntegerTempo`
  - `StableRefinedTempo`
  - `CoreWindowFallback`
  - `TempoDeferred`
- confidence
- a full `TempoContinuityPlan`

The continuity plan currently models:

- continuity action:
  - `Lock`
  - `Retain`
  - `Reacquire`
  - `Clear`
- source:
  - current tempo
  - prior tempo
  - core window
  - cleared state
- severity:
  - `Confirmed`
  - `Guarded`
  - `Fragile`
  - `Cleared`
- history:
  - `Reinforcing`
  - `Preserving`
  - `Degrading`
- arc:
  - `Recovering`
  - `Stalling`
  - `Collapsing`
- arc rationale and support metrics
- explicit arc decision with fallback action and expiry
- trigger classification
- unresolved span tracking
- cause stack
- provenance classification
- refresh strength
- trusted beat count and revalidation interval
- lifecycle transitions for refresh and two decay stages

This is already a substantial continuity surface, not a placeholder. The crate
encodes policy for how a previously chosen tempo should persist, degrade, or be
cleared when the fresh analysis is unstable.

### Implemented Meter Detection

`BeatAnalysisResult::meter` and `BeatAnalysisResult::meter_state` add a separate
meter-analysis path.

Current meter features:

- beat-strength extraction from the onset envelope around each detected beat
- meter-cue extraction from:
  - low-band spectral flux
  - band-profile change
- combined meter cue weighted `0.55` low-band and `0.45` profile change
- whole-track meter hypothesis search
- trailing/segment recovery hypothesis search
- downbeat phase inference
- downbeat timestamp export

Current supported bar-length hypotheses:

- `3/4`
- `4/4`

No other meter families are currently evaluated.

### Implemented Meter Estimate Surface

When a meter is emitted, the estimate currently includes:

- `beats_per_bar`
- confidence
- detection kind:
  - `WholeTrack`
  - `SegmentRecovery`
- trust:
  - `Stable`
  - `Recovering`
  - `Tentative`
- recommendation:
  - `Lock`
  - `Monitor`
  - `Defer`
- support profile
- confidence breakdown
- optional recovery context
- `downbeat_positions_seconds`

The recovery context currently describes:

- recovered beat range
- recovered bar count
- time span of the recovered region
- number of supporting windows

### Implemented Meter Continuity Model

`meter_state` is not just a wrapper around `meter`. It carries a full continuity
recommendation with separate plans for bar length and downbeat phase.

Current meter-state action space:

- `Lock`
- `Hold`
- `Watch`
- `Clear`

Current reason space:

- `StableMeter`
- `RecoveringMeter`
- `TentativeMeter`
- `DestabilizedHold`
- `RecoveryEmerging`
- `MeterCleared`

Each continuity plan currently models:

- action
- source
- severity
- history
- continuity arc
- arc rationale and support
- reason
- confidence
- trigger
- unresolved span in beats and bars
- cause stack
- trusted-beat lifetime
- revalidation interval
- refresh plus two decay transitions

This gives callers explicit state about whether a meter should be locked, held
provisionally, watched for recovery, or cleared.

### Rhythm Validation Coverage

The rhythm crate has unusually deep synthetic-fixture coverage. Current tests
exercise at least these scenarios:

- steady click tracks at multiple tempi
- integer-BPM refinement
- swung patterns
- syncopation without halved-tempo collapse
- subdivision ambiguity
- phase selection against loud offbeats
- `4/4` and `3/4` bar-phase inference
- pickup bars
- weak backbeats
- section transitions
- fill-density variants
- dropout-heavy transitions
- mixed bar-length suppression
- harmonic-rhythm changes
- meter recovery after destabilized windows
- continuity severity, cause, trigger, arc, provenance, and expiry calibration

This matters because much of the rhythm surface is policy-oriented rather than
just numeric. The tests are part of the feature definition.

### Current Rhythm Constraints

Not implemented today:

- streaming rhythm analysis
- stereo-aware rhythm features beyond mono mixdown
- meters outside `3/4` and `4/4`
- explicit swing ratio output
- groove templates or shuffle classification
- beat salience classes beyond the current downbeat export
- section segmentation as a public API

## Tonal Analysis

### `signal-analysis-tonal`

Primary surface:

- `KeyDetector`
- `KeyDetectorConfig`
- `TonalAnalysisResult`

Default configuration:

- STFT window size: `4096`
- STFT hop size: `2048`
- key profile: `Krumhansl`

Current features:

- mono STFT analysis
- spectrogram chroma extraction through `signal-dsp-spectral`
- two profile families:
  - `Krumhansl`
  - `Temperley`
- 24 key correlations:
  - 12 major
  - 12 minor
- best-key selection from the strongest correlation
- confidence based on margin between best and second-best correlation

Returned tonal outputs:

- optional detected key
- confidence
- normalized 12-bin chroma vector
- full 24-bin correlation array

Current constraints:

- offline only
- mono mixdown only
- no tuning estimation
- no chord detection
- no modulation timeline
- no scale-degree or harmonic-function surface

The current tonal feature set is therefore: global chroma plus whole-track key
detection.

## Loudness Analysis

### `signal-analysis-loudness`

Primary surface:

- `LoudnessMeter`
- `LoudnessMeterConfig`
- `LoudnessAnalysisResult`

Default configuration:

- target loudness: `-14.0 LUFS`
- block size: `400 ms`
- hop size: `100 ms`
- short-term window: `3.0 s`

Current features:

- mono loudness analysis
- integrated loudness estimate
- loudness range estimate
- true-peak estimate
- confidence score

### Implemented Loudness Algorithm Behavior

Current loudness behavior includes:

- optional K-weighting stage, but only for `48 kHz`
- two hardcoded biquad stages for the `48 kHz` K-weighting path
- fallback to unweighted samples for all other sample rates
- block mean-square measurement
- absolute gating at `-70 LUFS`
- relative gating at `10 LU` below the absolute-gated mean
- integrated loudness from the surviving gated blocks
- short-term loudness range from the `10th` to `95th` percentile
- true peak estimated by `4x` linear interpolation between adjacent samples

Current confidence behavior:

- confidence is `0.0` for silence or invalid input
- `48 kHz` analysis gets full sample-rate credit
- non-`48 kHz` analysis is down-weighted
- longer material increases confidence through block coverage

Current constraints:

- offline only
- mono mixdown only
- K-weighting coefficients only for `48 kHz`
- non-`48 kHz` paths are approximations, not equivalent-weighted implementations
- true peak is an approximate interpolated estimate, not a band-limited
  oversampled reconstruction
- no momentary loudness output
- no short-term loudness timeline output
- no per-channel or surround loudness handling

## Character Analysis

### `signal-analysis-character`

Primary surface:

- `CharacterAnalyzer`
- `CharacterAnalyzerConfig`
- `CharacterAnalysisResult`

Current configuration surface:

- STFT window and hop size through `StftConfig`
- optional centered analysis-duration cap
- onset-threshold multiplier for spectral-flux peak counting
- convenience presets:
  - `CharacterAnalyzerConfig::low()`
  - `CharacterAnalyzerConfig::medium()`
  - `CharacterAnalyzerConfig::high()`

Current features:

- offline mono character analysis over the full track or a centered excerpt
- spectral centroid as a brightness descriptor
- onset density derived from spectral-flux peak counts
- zero-crossing rate as a noisiness proxy
- RMS energy and peak amplitude
- transient density from sample-slope event counting
- sustain ratio using a fixed silence threshold
- dynamic range as peak-minus-RMS crest headroom
- confidence scoring based on analyzed duration and sample-rate support

Implementation details that are part of the current behavior:

- excerpted analysis is taken from the center of the track rather than the
  beginning
- onset density uses the STFT spectrogram and counts flux peaks above
  `onset_threshold * mean_flux`
- transient density uses direct time-domain slope checks with a minimum event
  spacing
- RMS energy is clamped into `0.0..=1.0`
- dynamic range is reported as a simple amplitude-domain crest difference, not
  a loudness-domain range metric

Current constraints:

- offline only
- mono mixdown only
- no per-frame descriptor timeline output
- no learned embeddings or classification labels
- no multiband timbral descriptors such as rolloff, flatness, or contrast
- no stereo-width or spatial character metrics

## What Is Implemented Versus Planned

Implemented now:

- shared audio/time/channel primitives and simple control-segment helpers
- reusable DSP kernels for gain, smoothing, low-pass filtering, delay, level
  tracking, block mixing, and deterministic signal fixtures
- STFT, spectral magnitudes/phases, tuned chroma, spectral centroid, and mel
  spectrogram projection
- offline rhythm analysis with beat, tempo, diagnostics, interpretation,
  continuity, and limited meter inference
- offline global key detection
- offline loudness summary metrics
- offline audio character descriptors

Planned elsewhere but not implemented in these crates yet:

- streaming analysis implementations
- richer meter families
- tuning/chord/modulation analysis
- learned embedding and classifier analysis surfaces
- richer loudness timelines and standards-complete weighting coverage
- broader DSP families such as resampling, dynamics, convolution, and
  oscillator kernels

## Current Entry Points

Useful entry points for readers who want the implementation after this doc:

- `crates/signal-primitives/src/lib.rs`
- `crates/signal-dsp/src/lib.rs`
- `crates/signal-dsp-spectral/src/lib.rs`
- `crates/signal-analysis/src/lib.rs`
- `crates/signal-analysis-rhythm/src/lib.rs`
- `crates/signal-analysis-tonal/src/lib.rs`
- `crates/signal-analysis-loudness/src/lib.rs`
- `crates/signal-analysis-character/src/lib.rs`
- `crates/signal-analysis-rhythm/examples/offline_rhythm_demo.rs`
- `crates/signal-analysis-rhythm/examples/file_rhythm_probe.rs`
- `crates/signal-analysis-tonal/examples/offline_tonal_demo.rs`
- `crates/signal-analysis-loudness/examples/offline_loudness_demo.rs`

## Next Task

Add deeper API-local docs and examples for the new low-level DSP module types
such as `ControlPlan`, mel spectrogram configuration, and the character
analyzer presets so the current feature reference stays synchronized with the
growing implementation surface.
