# Algorithm Specification: Beat Tracking (Böck Multi-Feature Approach)

**Status**: Updated with Essentia research  
**Source**: Essentia `rhythmextractor2013.cpp` + Böck et al. ISMIR 2014  
**Target Library**: Signal (`signal-analysis-rhythm`)  
**Consumer**: Finch  
**Owner**:  
**Last Updated**: 2026-03-08

## 1. Overview

### Purpose
Estimate the tempo (BPM) and beat positions in musical audio using onset detection, tempogram computation, and dynamic programming beat tracking.

### Key Finding from Essentia Research

**RhythmExtractor2013** uses a **multi-feature approach** combining several onset detection functions:
1. **HFC** (High Frequency Content) - emphasizes percussive onsets
2. **Complex-domain spectral difference** - detects magnitude + phase changes
3. **Spectral flux** (Mel-frequency bands) - energy changes in perceptual bands
4. **Beat emphasis function** - sub-band analysis weighted by beat strength
5. **Information gain** - histogrammed spectrum differences

These are combined and fed to **TempoTapDegara** for beat tracking.

### Inputs
| Parameter | Type | Description | Essentia Default | Notes |
|-----------|------|-------------|------------------|-------|
| `audio` | `&[f32]` | Mono audio samples | - | Must be mono, f32 |
| `sample_rate` | `usize` | Sample rate in Hz | 44100 | Critical: Essentia expects 44100 |
| `frame_size` | `usize` | FFT window size | 2048 | ~46ms at 44.1kHz |
| `hop_size` | `usize` | Frame hop size | 512 | ~11.6ms at 44.1kHz |
| `min_tempo` | `f32` | Minimum detectable BPM | 40.0 |  |
| `max_tempo` | `f32` | Maximum detectable BPM | 208.0 | 2x default max |

### Outputs
| Output | Type | Description | Essentia Format | Finch Format |
|--------|------|-------------|-----------------|--------------|
| `bpm` | `f32` | Estimated tempo in BPM | Single f32 | Same |
| `beats` | `Vec<f32>` | Beat positions in seconds | Vector of f32 | Same |
| `ticks` | `Vec<f32>` | Beat tick positions | Vector | Optional |
| `confidence` | `f32` | Confidence score (0.0-5.32) | 0.0-5.32 | Normalize to 0.0-1.0 |
| `estimates` | `Vec<f32>` | BPM estimates distribution | Vector | Debug info |

**Critical Note on Confidence**: Essentia's confidence ranges from 0 to ~5.32 (from TempoTapMaxAgreement). The documentation says to **ignore confidence when using 'degara' method** - it's only valid for 'multifeature'.

---

## 2. Mathematical Specification

### Core Algorithm (Multi-Stage)

```
Input: audio samples x[n], sample_rate fs

Stage 1: Multi-Feature Onset Detection
--------------------------------------
For each frame m:
  1. Compute magnitude spectrum: X[m, k] = |FFT(frame_m)|
  2. Compute multiple onset detection functions:
     
     HFC (High Frequency Content):
     HFC[m] = sum(k * |X[m,k]|) weighted by frequency
     
     Complex Domain:
     CSD[m] = sum(|X[m,k]| - |X[m-1,k]| * cos(phase_diff) + 
                  |X[m-1,k]| * sin(phase_diff))
     
     Spectral Flux (half-wave rectified):
     SF[m] = sum(max(0, |X[m,k]| - |X[m-1,k]|))
     
     Mel-Frequency Flux:
     Melflux[m] = sum(max(0, Mel[m] - Mel[m-1]))
     
     Energy Flux (RMS difference):
     EF[m] = max(0, RMS[m] - RMS[m-1])
  
  3. Combine features (specific weights from Essentia):
     onset_function[m] = weighted_combination(HFC, CSD, SF, Melflux, EF)

Stage 2: Tempogram Computation
------------------------------
1. Compute autocorrelation of onset detection function
2. Apply tempo preference curve (Rayleigh weighting)
3. Focus on tempo range [min_period, max_period] in frames

Stage 3: Tempo Estimation
-------------------------
1. Find peaks in tempogram
2. Apply octave cancellation (penalize half/double)
3. Select best tempo candidate: BPM = 60 * fs / (hop_size * period)

Stage 4: Beat Tracking (TempoTapDegara)
---------------------------------------
1. Use Viterbi algorithm for optimal beat sequence
2. Probabilistic framework integrating tempo and phase
3. Returns beat positions

Output: BPM, beat_positions[], confidence
```

### Onset Detection Functions (from Essentia OnsetDetection)

Essentia implements these onset detection methods:

| Method | Description | Use Case |
|--------|-------------|----------|
| `hfc` | High Frequency Content | Percussive onsets |
| `complex` | Complex-domain difference | General purpose |
| `complex_phase` | Phase-only variant | Tonal sounds |
| `flux` | Spectral Flux | Energy changes |
| `melflux` | Mel-band flux | Perceptual changes |
| `rms` | RMS energy difference | Overall energy |

### Key Equations

**HFC (High Frequency Content)**:
$$HFC[m] = \sum_{k=0}^{N/2} k \cdot |X[m,k]|$$

**Complex Domain Spectral Difference**:
$$CSD[m] = \sum_{k} |X[m,k] - |X[m-1,k]| \cdot e^{j(\phi[m,k] - \phi[m-1,k])}|$$

**Spectral Flux** (half-wave rectified):
$$SF[m] = \sum_{k=0}^{N/2} H(|X[m,k]| - |X[m-1,k]|)$$
where $H(x) = \max(0, x)$

**Tempogram (Autocorrelation with Rayleigh weighting)**:
$$T[\tau] = w(\tau) \cdot \sum_{m} O[m] \cdot O[m + \tau]$$
where $w(\tau)$ is the Rayleigh weighting function centered on preferred tempo.

---

## 3. Essentia Implementation Analysis

### Source File Locations

```
essentia/src/algorithms/rhythm/
  ├── rhythmextractor2013.cpp      # Main algorithm
  ├── beattrackermultifeature.cpp  # Multi-feature wrapper
  ├── tempotapdegara.cpp           # Beat tracking core
  ├── tempotapmaxagreement.cpp     # Confidence calculation
  ├── onsetdetection.cpp           # Individual onset functions
  ├── tempogram.cpp                # Tempogram computation
  └── ...
```

### Critical Implementation Details from Essentia

1. **Multi-Feature Combination** (`beattrackermultifeature.cpp`):
   - Uses 5 different onset detection functions
   - Frame sizes vary by function (2048/1024)
   - Some functions are upsampled 2x for alignment

2. **TempoTapDegara** (from documentation):
   - Uses probabilistic framework
   - Viterbi algorithm for beat sequence
   - Integrates tempo and phase observations

3. **Confidence Calculation** (`tempotapmaxagreement.cpp`):
   - Range: 0 to ~5.32
   - Based on mutual agreement between different beat trackers
   - Higher = more agreement = more confident

### Algorithm Flow (Updated)

1. **Input**: Audio at 44100 Hz (Essentia requirement)

2. **Onset Detection** (5 parallel computations):
   - Complex spectral difference (frameSize=2048, hopSize=1024, upsampled 2x)
   - Energy flux/RMS (same settings)
   - Mel-frequency flux (same settings)
   - Beat emphasis function (frameSize=2048, hopSize=512)
   - Information gain (frameSize=2048, hopSize=512)

3. **Combination**: Sum or weighted combination of onset functions

4. **Tempogram**: Autocorrelation + Rayleigh weighting

5. **TempoTapDegara**: Beat position estimation

6. **Confidence**: From TempoTapMaxAgreement (multifeature method only)

---

## 4. Implementation Plan for Signal

### Open Questions Resolved

**Q: What are the exact onset feature weights?**
A: Essentia uses a **committee approach** - runs multiple trackers and selects via maximum agreement, not fixed weights. Each onset function feeds into TempoTapDegara separately, then results are combined.

**Q: How is confidence calculated?**
A: Confidence = mutual agreement score between different onset function results. Range 0-5.32. Only valid for 'multifeature' method.

### Recommended Signal Implementation

Simplified approach (matching Essentia's quality without full complexity):

1. **Compute 3-4 key onset functions**:
   - Complex domain (best general performance)
   - Spectral flux (simple, effective)
   - HFC (for percussion)
   - (Optional) Mel-flux

2. **Sum with equal weights** or simple combination

3. **Tempogram + peak picking** for tempo

4. **Dynamic programming beat tracking** (Böck method)

5. **Confidence from tempogram peak strength** + beat alignment quality

### Target API

```rust
pub struct BeatTracker {
    config: BeatConfig,
    onset_detector: MultiFeatureOnsetDetector,
}

impl BeatTracker {
    pub fn analyze(&mut self, audio: &[f32]) -> Result<BeatResult, Error> {
        // 1. Compute multiple onset functions
        let onset = self.onset_detector.compute(audio)?;
        
        // 2. Tempogram
        let tempogram = compute_tempogram(&onset, self.config.sample_rate)?;
        
        // 3. Tempo estimation
        let (bpm, tempo_confidence) = estimate_tempo(&tempogram)?;
        
        // 4. Beat tracking
        let beats = track_beats(&onset, bpm)?;
        
        // 5. Overall confidence
        let confidence = combine_confidences(tempo_confidence, beat_alignment);
        
        Ok(BeatResult { bpm, beats, confidence })
    }
}
```

---

## 5. Validation

### Against Essentia

Target tolerances:
- BPM ±0.5: > 90%
- Beat positions ±20ms: > 85%

### Test Corpus

Include diverse genres:
- Electronic/EDM (clear beats)
- Rock (live drums)
- Jazz (swing, complex rhythm)
- Classical (tempo variations)
- Ambient (minimal rhythm - test low confidence)

---

## 6. References

### Key Papers
- Böck, Krebs, Schedl. "Multi-Feature Beat Tracking." ISMIR 2014.
- Degara et al. "Reliability-informed beat tracking." 2012.

### Essentia Documentation
- https://essentia.upf.edu/reference/std_RhythmExtractor2013.html
- https://essentia.upf.edu/reference/std_OnsetDetection.html
- https://essentia.upf.edu/reference/std_BeatTrackerMultiFeature.html

---

## Implementation Log

| Date | Update | Notes |
|------|--------|-------|
| 2026-03-08 | Researched Essentia implementation | Multi-feature approach confirmed |
| 2026-03-08 | Documented onset detection methods | 6 methods in Essentia |
| 2026-03-08 | Clarified confidence calculation | 0-5.32 range, mutual agreement |
