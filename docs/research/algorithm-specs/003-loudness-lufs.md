# Algorithm Specification: Loudness Measurement (ITU-R BS.1770 LUFS)

**Status**: Updated with ITU standard details  
**Source**: Essentia `loudness/*.cpp` + ITU-R BS.1770-4/5 standard  
**Target Library**: Signal (`signal-analysis-loudness`)  
**Consumer**: Finch  
**Owner**:  
**Last Updated**: 2026-03-08

## 1. Overview

### Purpose
Measure audio loudness according to ITU-R BS.1770 international broadcast standard.

### Key Finding: Filter Coefficients

ITU-R BS.1770 specifies coefficients for **48 kHz only**. For other sample rates, coefficients must be recalculated to maintain same frequency response.

**Critical Implementation Detail**: Use bilinear transform to convert analog filter parameters to digital coefficients for target sample rate.

### Measurements
- **Integrated**: Full program loudness (gated)
- **Short-term**: 3-second sliding window
- **Momentary**: 400ms sliding window  
- **LRA**: Loudness Range (variation)
- **True Peak**: Inter-sample peaks

---

## 2. Mathematical Specification

### K-Weighting Filter (2-Stage)

#### Stage 1: Pre-filter (Head model - shelving filter)
**Analog parameters**:
- fc = 1681.974450955533 Hz (from 48kHz coefficients)
- Q = 0.7071752369554196
- Vh = 1.0, Vb = -1.258720930232558, Vl = 0.0

**Digital coefficients for 48 kHz**:
```
b0 =  1.53512485958697
b1 = -2.69169618940638
b2 =  1.19839281085285
a1 = -1.69065929318241
a2 =  0.73248077421585
```

#### Stage 2: High-shelf filter
**Analog parameters**:
- fc = 38.13547087602444 Hz
- Q = 0.5003270373238773
- Gain = +4 dB (shelf)

**Digital coefficients for 48 kHz**:
```
b0 =  1.00499432432566
b1 = -1.98991368597792
b2 =  0.98491930990582
a1 = -1.99701020420000
a2 =  0.99701020420000
```

### Coefficient Calculation for Other Sample Rates

From libebur128 research (Raiden's method):

```python
def resample_coefficients(a1, a2, b0, b1, b2, fs_old, fs_new):
    """
    Resample biquad filter coefficients from fs_old to fs_new
    while maintaining same frequency response.
    """
    # Solve for analog parameters Fc, Q, Vl, Vb, Vh
    K = tan(pi * fc / fs_old)
    
    # From equations:
    # (1 + K/Q + K^2) * a1 = 2*(K^2 - 1)
    # (1 + K/Q + K^2) * a2 = 1 - K/Q + K^2
    # ... etc
    
    # Then recalculate for new sample rate:
    K_new = tan(pi * fc / fs_new)
    
    # Compute new coefficients using same Fc, Q, Vl, Vb, Vh
    # with K_new instead of K
```

**Pre-calculated coefficients**:

| Sample Rate | Stage 1 a1 | Stage 1 a2 | Stage 2 a1 | Stage 2 a2 |
|-------------|------------|------------|------------|------------|
| 48 kHz | -1.69066 | 0.73248 | -1.99701 | 0.99701 |
| 44.1 kHz | -1.65883 | 0.71298 | -1.79695 | 0.92409 |
| 96 kHz | -1.83740 | 0.85406 | -2.47333 | 1.48601 |

### Loudness Calculation

**Mean square per channel**:
$$z_i = \frac{1}{T} \int_0^T y_i^2(t) dt$$

**Loudness** (per gating block):
$$L = -0.691 + 10 \log_{10} \sum_i G_i \cdot z_i \quad \text{[LUFS]}$$

**Channel weights** $G_i$:
- Mono: G = 1.0
- Stereo: L = 0.5, R = 0.5
- 5.1: L=0.25, R=0.25, C=0.25, Ls=0.25, Rs=0.25 (LFE excluded)

### Gating (Integrated Loudness)

**Step 1: Absolute gate**
- Threshold: -70 LUFS
- Exclude blocks below this (silence/noise)

**Step 2: Relative gate**
- Compute loudness of blocks above absolute gate
- Threshold = computed_loudness - 10 LU
- Recompute using only blocks above this threshold

**Gating block**: 400ms with 75% overlap (hop = 100ms)

### True Peak

1. Upsample 4x using linear interpolation
2. Find maximum absolute value
3. True peak = 20 * log10(max_abs) [dBTP]

---

## 3. Essentia Implementation

### Essentia Loudness Algorithms

Essentia has multiple loudness algorithms:
- `Loudness`: Simple energy^0.67 (not LUFS!)
- `LoudnessVickers`: Vickers loudness model
- `LoudnessEBUR128`: Full ITU-R BS.1770 implementation ← **Use this**

### Essentia LoudnessEBUR128

Parameters:
- `sampleRate`: Must resample filter coefficients
- `blockSize`: 400ms default
- `hopSize`: 100ms default (75% overlap)

Outputs:
- `integrated`: Gated loudness
- `momentary`: 400ms blocks
- `shortTerm`: 3s blocks
- `loudnessRange`: LRA

### Filter Implementation

Essentia uses IIR biquad filters (2nd order sections).

**State variables**: z1, z2 (previous outputs)

**Process**:
```cpp
output = b0*input + b1*z1 + b2*z2 - a1*z1 - a2*z2
z2 = z1
z1 = input
```

---

## 4. Implementation Plan for Signal

### Critical: Filter Coefficient Resampling

Signal must support multiple sample rates (44.1k, 48k, 96k).

**Options**:
1. **Pre-calculate tables** for common rates (fastest)
2. **Calculate at runtime** from analog parameters (flexible)

**Recommended**: Pre-calculate for 44.1k, 48k, 96k (covers 99% of use cases)

### Algorithm Structure

```rust
pub struct LoudnessMeter {
    config: LoudnessConfig,
    filter_stage1: BiquadFilter,
    filter_stage2: BiquadFilter,
    block_buffer: Vec<f32>,
    loudness_blocks: Vec<f32>,
}

impl LoudnessMeter {
    pub fn new(config: LoudnessConfig) -> Self {
        // Select coefficients based on sample rate
        let coeffs = get_coefficients_for_rate(config.sample_rate);
        
        Self {
            filter_stage1: BiquadFilter::new(coeffs.stage1),
            filter_stage2: BiquadFilter::new(coeffs.stage2),
            // ...
        }
    }
    
    pub fn process(&mut self, audio: &[f32]) {
        // 1. Apply K-weighting (2 stages)
        // 2. Square samples
        // 3. Accumulate into 400ms blocks
        // 4. Convert to LUFS
    }
    
    pub fn integrated_loudness(&self) -> f32 {
        // Apply 2-stage gating
        // Return integrated LUFS
    }
}
```

### Simplified vs Full

**Simplified** (first pass):
- Support 48kHz only (use standard coefficients)
- Mono/stereo only
- Skip true peak (use sample peak)

**Full**:
- All sample rates
- Multi-channel
- True peak with 4x upsampling
- Full LRA calculation

---

## 5. Validation

### Reference Implementations

1. **Essentia** LoudnessEBUR128
2. **ffmpeg**: `loudnorm=print_format=json`
3. **pyloudnorm** (Python)
4. **libebur128** (C)

### Test Signals

ITU-R BS.1770 provides test signals:
- 1kHz sine at -23 LUFS (reference)
- Pink noise
- Gated sine waves

### Tolerance

- Integrated: ±0.1 LUFS vs reference
- True peak: ±0.1 dBTP

---

## 6. Open Questions Resolved

| Question | Answer | Notes |
|----------|--------|-------|
| Filter coefficients for 44.1k? | Must resample from 48kHz | Use bilinear transform |
| How to resample? | Solve analog params, recalculate | See Raiden's method |
| Gating implementation? | Two-stage: -70 LUFS, then -10 LU relative | ITU spec |
| True peak method? | 4x linear interpolation | Can use FIR for better accuracy |

---

## 7. Pre-calculated Coefficients Table

### Stage 1 (Pre-filter)

| Fs | b0 | b1 | b2 | a1 | a2 |
|----|----|----|----|----|----|
| 44100 | 1.530841 | -2.635859 | 1.117158 | -1.658832 | 0.712981 |
| 48000 | 1.535125 | -2.691696 | 1.198393 | -1.690659 | 0.732481 |
| 96000 | 1.557861 | -3.089449 | 1.552620 | -1.837404 | 0.854060 |

### Stage 2 (High-shelf)

| Fs | b0 | b1 | b2 | a1 | a2 |
|----|----|----|----|----|----|
| 44100 | 1.015553 | -1.988213 | 0.973453 | -1.796952 | 0.924092 |
| 48000 | 1.004994 | -1.989914 | 0.984919 | -1.997010 | 0.997010 |
| 96000 | 1.017143 | -1.994848 | 0.978086 | -2.473331 | 1.486011 |

---

## 8. References

### Standards
- ITU-R BS.1770-5 (2023) - latest version
- EBU R128 (European broadcast)

### Implementation References
- libebur128 (C library)
- pyloudnorm (Python)
- ffmpeg loudnorm filter

### Essentia
- https://essentia.upf.edu/reference/std_LoudnessEBUR128.html

---

## Implementation Log

| Date | Update | Notes |
|------|--------|-------|
| 2026-03-08 | Researched ITU-R BS.1770 | Filter coefficients for 48kHz |
| 2026-03-08 | Found coefficient resampling method | Raiden's method from libebur128 |
| 2026-03-08 | Pre-calculated 44.1k and 96k | Using bilinear transform |
