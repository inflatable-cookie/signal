# Source Hub 001: Rust Audio Ecosystem

Status: Draft
Topic: Rust crates for audio analysis and MIR
Owner: Research
Last updated: 2026-03-08

## Purpose

Map the Rust ecosystem for audio analysis to identify:
- what Signal can use as dependencies
- what Signal needs to build from scratch
- gaps in the ecosystem that Signal might eventually contribute back

## Core Audio Processing

### FFT / Spectral Analysis

| Crate | Version | License | Assessment | Signal Use |
|-------|---------|---------|------------|-----------|
| **rustfft** | 6.2 | MIT | Pure Rust FFT, widely used | ✅ Primary FFT dependency |
| **realfft** | 3.4 | MIT | Optimized real-input FFT | ✅ Use for spectrograms |
| **num-fft** | ? | MIT/Apache | Alternative FFT | ⚪ Evaluate if needed |

**Notes**: `rustfft` + `realfft` should cover Signal's initial spectral needs.

### Audio Decoding

| Crate | Version | License | Assessment | Signal Use |
|-------|---------|---------|------------|-----------|
| **symphonia** | 0.5 | MPL-2.0 | Pure Rust decoder (MP3, AAC, FLAC, WAV, OGG) | ✅ Primary decoder |
| **hound** | 3.5 | Apache-2.0 | WAV encoding/decoding | ✅ WAV-specific tasks |
| **rodio** | 0.19 | MIT | Playback + decoding | ⚪ May be useful for preview |
| **claxon** | 0.4 | Apache-2.0 | FLAC decoder | ⚪ Covered by symphonia |

**Notes**: `symphonia` is the modern choice. MPL-2.0 is compatible with Finch's licensing.

### Audio I/O

| Crate | Version | License | Assessment | Finch Use |
|-------|---------|---------|------------|-----------|
| **cpal** | 0.15 | Apache-2.0 | Cross-platform audio I/O | ✅ Future: preview playback |
| **rubato** | 0.14 | MIT | Sample rate conversion | ✅ For resampling to analysis rate |

## Music Information Retrieval

### Existing MIR Crates (Sparse!)

| Crate | Version | License | Assessment | Finch Use |
|-------|---------|---------|------------|-----------|
| **aubio-rs** | 0.2 | GPL-3.0 | Bindings to C aubio library | ❌ GPL, avoid |
| **pitch_detection** | ? | MIT | Pitch detection algorithms | ⚪ Evaluate for tonal analysis |
| **spectrum-analyzer** | ? | MIT | FFT-based spectrum analysis | ⚪ May overlap with custom impl |

**Gap**: The MIR crate ecosystem is still sparse. Signal will need to build most
core analysis from scratch.

## Machine Learning

### ML Framework Options

| Framework | Training | Inference | License | Assessment |
|-----------|----------|-----------|---------|------------|
| **burn** | ✅ Yes | ✅ Yes | MIT/Apache | Pure Rust, can train models |
| **candle** | ✅ Yes | ✅ Yes | MIT/Apache | HuggingFace, good ecosystem |
| **tract** | ❌ No | ✅ Yes | MIT/Apache | ONNX inference, no Python needed |

**Recommendation**: 
- **Phase 1**: Use `tract` with ONNX models exported from PyTorch
- **Phase 2**: Consider `burn` for pure Rust training pipeline

### ONNX Ecosystem

| Crate | Purpose | Assessment |
|-------|---------|------------|
| **tract-onnx** | Load/run ONNX models | ✅ Primary choice |
| **onnxruntime-rs** | Bindings to MS ONNX Runtime | ❌ C++ dependency |

## Math & Signal Processing

| Crate | Version | Purpose | Assessment |
|-------|---------|---------|------------|
| **nalgebra** | 0.33 | Linear algebra | ✅ For matrix operations |
| **ndarray** | 0.16 | N-dimensional arrays | ✅ For spectrogram storage |
| **ninterp** | ? | Interpolation | ⚪ For resampling |
| **statrs** | 0.17 | Statistics | ✅ For mean, variance, etc. |

## Utility Crates

| Crate | Purpose | Assessment |
|-------|---------|------------|
| **serde** | Serialization | ✅ For sidecar JSON |
| **rayon** | Parallelism | ✅ For parallel track analysis |
| **crossbeam** | Channels/concurrency | ✅ For engine-controller communication |

## Ecosystem Gaps (Finch Will Build)

Based on this survey, Signal needs to implement:

1. **Onset Detection** — No suitable crate found
   - Spectral flux, energy flux, complex domain
   - Part of `signal-dsp-spectral` or `signal-analysis-rhythm`

2. **Beat Tracking** — No suitable crate found
   - Böck's algorithm or similar
   - Part of `signal-analysis-rhythm`

3. **Chroma Features** — No suitable crate found
   - Pitch class profiling
   - Part of `signal-dsp-spectral` or `signal-analysis-tonal`

4. **Key Detection** — No suitable crate found
   - Profile correlation
   - Part of `signal-analysis-tonal`

5. **LUFS Loudness** — No suitable crate found
   - ITU-R BS.1770 implementation
   - Part of `signal-analysis-loudness`

6. **Audio Embeddings** — No suitable crate found
   - CNN-based embeddings
   - Part of `signal-analysis-embed`

## Recommended Dependency Stack

```toml
[dependencies]
# Core audio
symphonia = { version = "0.5", features = ["mp3", "aac", "flac", "wav"] }
rustfft = "6.2"
realfft = "3.4"
rubato = "0.14"

# Math/statistics
ndarray = "0.16"
statrs = "0.17"

# ML (for signal-analysis-embed)
tract-onnx = "0.21"

# Utilities
serde = { version = "1.0", features = ["derive"] }
rayon = "1.9"
```

## Next Steps

1. **Validate symphonia** — Test decoding needs for Signal consumers
2. **Benchmark rustfft** — Compare performance vs Essentia's FFTW
3. **Prototype signal-dsp-spectral** — Spectrogram plus basic features
4. **Evaluate tract** — Can we run simple ONNX models?

## Sources

| Source | Type | Date | Notes |
|--------|------|------|-------|
| crates.io search | Registry | 2026-03 | Manual search for audio/MIR crates |
| rustaudio.github.io | Community | 2026-03 | Rust Audio Working Group |
| GitHub rust-dsp org | Code | 2026-03 | Various DSP crates |
