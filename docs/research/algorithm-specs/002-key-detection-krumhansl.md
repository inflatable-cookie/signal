# Algorithm Specification: Key Detection (Krumhansl-Schmuckler Profile Correlation)

**Status**: Updated with Essentia research  
**Source**: Essentia `tonal/key.cpp` + Krumhansl & Kessler 1982  
**Target Library**: Signal (`signal-analysis-tonal`)  
**Consumer**: Finch  
**Owner**:  
**Last Updated**: 2026-03-08

## 1. Overview

### Purpose
Estimate the musical key (tonic pitch class and mode: major/minor) from audio using HPCP (Harmonic Pitch Class Profile) features and key profile correlation.

### Key Finding from Essentia Research

**Essentia's Key algorithm**:
1. Uses **HPCP** (Harmonic Pitch Class Profile) not simple chroma
2. Supports **14 different key profiles**: diatonic, krumhansl, temperley, weichai, tonictriad, temperley2005, thpcp, shaath, gomez, noland, edmm, edma, bgate (default), braw
3. Default profile: **'bgate'** (not Krumhansl!)
4. Computes **polyphonic profiles** (includes harmonic contributions)
5. Outputs **strength** = correlation value (confidence)

**Important Parameters**:
- `numHarmonics`: 4 (default) - includes harmonic contributions
- `slope`: 0.6 - exponential decay for harmonic contribution
- `pcpThreshold`: 0.2 - bins below this set to 0
- `pcpSize`: 36 (3 bins per semitone for fine resolution)

### Inputs
| Parameter | Type | Description | Essentia Default | Notes |
|-----------|------|-------------|------------------|-------|
| `audio` | `&[f32]` | Mono audio samples | - | Full track |
| `sample_rate` | `usize` | Sample rate | 44100 |  |
| `pcp_size` | `usize` | HPCP bins | 36 | 3 bins per semitone |
| `num_harmonics` | `usize` | Harmonics to include | 4 |  |
| `slope` | `f32` | Harmonic decay | 0.6 |  |
| `profile_type` | `Profile` | Key profile | Bgate | 14 options |

### Outputs
| Output | Type | Description | Notes |
|--------|------|-------------|-------|
| `key` | `Tonic` | Tonic note (C, C#, etc.) |  |
| `mode` | `Mode` | Major or Minor |  |
| `strength` | `f32` | Correlation strength (0-1) | Confidence proxy |
| `hpcp` | `[f32; 36]` | HPCP vector (if needed) | 3 bins per semitone |

---

## 2. Mathematical Specification

### HPCP (Harmonic Pitch Class Profile)

HPCP extends simple chroma by:
1. **Higher resolution**: 36 bins (3 per semitone) vs 12
2. **Harmonic contributions**: Includes energy from harmonics
3. **Tuning correction**: Shifts to nearest tempered bin

**HPCP Calculation**:
```
For each spectral peak (frequency, magnitude):
  1. Convert frequency to pitch class (0-11, fractional)
  2. Map to HPCP bin (0-35, 3 bins per semitone)
  3. Add magnitude to that bin
  4. Add magnitude * slope to harmonics
  5. Add magnitude * slope^2 to 2nd harmonics
  ... up to num_harmonics
```

**Tuning Correction** (`averageDetuningCorrection`):
- Shifts HPCP to nearest tempered bin
- Compensates for A440 vs A442, etc.

### Key Profile Correlation

Essentia supports 14 profiles. Key ones for Signal:

1. **Krumhansl-Kessler** (cognitive, from experiments)
2. **Temperley** (probabilistic model)
3. **Bgate** (default, optimized for pop/rock)

**Correlation**:
```
For each of 24 keys (12 major + 12 minor):
  1. Rotate HPCP so tonic is at index 0
  2. Downsample to 12 bins (sum 3 bins per semitone)
  3. Normalize
  4. Compute correlation with profile
  
Select key with maximum correlation
strength = max_correlation
```

### Key Profile Values

**Krumhansl-Kessler Major** (12 values):
```
[6.35, 2.23, 3.48, 2.33, 4.38, 4.09, 2.52, 5.19, 2.39, 3.66, 2.29, 2.88]
```

**Krumhansl-Kessler Minor**:
```
[6.33, 2.68, 3.52, 5.38, 2.60, 3.53, 2.54, 4.75, 3.98, 2.69, 3.34, 3.17]
```

**Normalization**: Unit sum (sums to ~1.0)

---

## 3. Essentia Implementation Details

### From Essentia Source Analysis

**Algorithm**: `tonal/key.cpp`, `tonal/hpcp.cpp`

**Key Parameters**:
| Parameter | Default | Description |
|-----------|---------|-------------|
| `pcpSize` | 36 | HPCP resolution (multiple of 12) |
| `numHarmonics` | 4 | Harmonics for polyphonic profile |
| `slope` | 0.6 | Harmonic decay factor |
| `profileType` | "bgate" | Key profile selector |
| `pcpThreshold` | 0.2 | Threshold to zero out bins |
| `averageDetuningCorrection` | true | Shift to nearest semitone |

**HPCP Computation** (`hpcp.cpp`):
1. Takes spectral peaks (frequencies, magnitudes)
2. Maps to 36-bin HPCP with harmonic weighting
3. Applies tuning correction
4. Normalizes

**Key Selection** (`key.cpp`):
1. Correlate HPCP with 24 profiles
2. Find maximum
3. Return key + strength

### Critical Differences from Simple Chroma

| Feature | Simple Chroma | Essentia HPCP |
|---------|---------------|---------------|
| Bins | 12 | 36 (configurable) |
| Harmonics | No | Yes (configurable) |
| Tuning correction | No | Yes |
| Thresholding | No | Yes (pcpThreshold) |
| Profiles | 2-3 | 14 |

---

## 4. Implementation Plan for Signal

### Recommended Approach

**Core Algorithm**:
1. Compute HPCP (36 bins, 3 per semitone)
2. Include harmonic contributions (4 harmonics, slope 0.6)
3. Apply tuning correction
4. Correlate with key profiles
5. Return key with highest correlation

**Profiles to Implement** (priority order):
1. **Krumhansl-Kessler** (classic, well-tested)
2. **Temperley** (alternative)
3. **Bgate** (Essentia default, modern)

**Confidence** = correlation strength of best match

### Simplified vs Full Implementation

**Simplified** (start here):
- 12-bin chroma (not 36)
- No harmonic contributions
- Krumhansl profiles only
- Faster, easier to implement

**Full** (for accuracy):
- 36-bin HPCP
- Harmonic contributions
- Multiple profiles
- Tuning correction

### API Design

```rust
pub struct KeyDetector {
    config: KeyConfig,
}

pub struct KeyConfig {
    pub sample_rate: usize,
    pub pcp_size: usize,           // 12 or 36
    pub num_harmonics: usize,      // 0 for simplified, 4 for full
    pub slope: f32,
    pub profile: KeyProfile,
    pub pcp_threshold: f32,
}

pub enum KeyProfile {
    KrumhanslKessler,  // Classic
    Temperley,         // Alternative
    Bgate,             // Essentia default
    // ... others
}

impl KeyDetector {
    pub fn analyze(&self, audio: &[f32]) -> Result<KeyResult, Error> {
        // 1. Compute spectrum
        // 2. Compute HPCP/chroma
        // 3. Correlate with profiles
        // 4. Return best match
    }
}

pub struct KeyResult {
    pub key: Key,           // Tonic + Mode
    pub strength: f32,      // Correlation (0-1)
    pub hpcp: Vec<f32>,     // HPCP vector
    pub correlations: [f32; 24], // All 24 key correlations
}
```

---

## 5. Validation

### Against Essentia

Test with Essentia using:
```python
import essentia.standard as es

key_detector = es.Key(
    pcpSize=36,
    numHarmonics=4,
    slope=0.6,
    profileType='krumhansl'  # Match our implementation
)

key, scale, strength = key_detector(hpcp)
```

**Target Tolerance**:
- Exact match (key + mode): > 80% vs Essentia
- Tonic correct: > 90%
- Strength correlation: > 0.9

### Known Challenges

1. **Relative major/minor**: C major vs A minor (same notes)
   - Solution: Report both, use confidence

2. **Modal music**: Dorian, Mixolydian
   - May detect as relative major
   - Flag as "ambiguous"

3. **Atonal music**: No clear key
   - Low strength indicates uncertainty

---

## 6. Open Questions Resolved

| Question | Answer | Source |
|----------|--------|--------|
| Chroma aggregation | HPCP with harmonic weighting | `hpcp.cpp` |
| Default profile | 'bgate', not Krumhansl | Key algorithm docs |
| Confidence | Correlation strength | `key.cpp` output |
| Tuning correction | Shifts to nearest semitone | `averageDetuningCorrection` param |
| Num harmonics | 4 default, slope 0.6 | Default parameters |

---

## 7. References

### Papers
- Krumhansl & Kessler (1982) - Key profiles
- Temperley (2001) - Probabilistic key detection
- Gómez (2005) - Tuning frequency estimation

### Essentia Docs
- https://essentia.upf.edu/reference/std_Key.html
- https://essentia.upf.edu/reference/std_HPCP.html
- https://essentia.upf.edu/reference/std_KeyExtractor.html

---

## Implementation Log

| Date | Update | Notes |
|------|--------|-------|
| 2026-03-08 | Researched Essentia Key | 14 profile types found |
| 2026-03-08 | Documented HPCP computation | 36 bins, harmonics |
| 2026-03-08 | Default profile is 'bgate' | Not Krumhansl as expected |
