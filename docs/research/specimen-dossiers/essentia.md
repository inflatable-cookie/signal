# Essentia Dossier

Status: Draft
Product: Essentia (MTG, Universitat Pompeu Fabra)
Type: Open-source C++ audio analysis library with Python bindings
Owner: Research
Last updated: 2026-03-08

## Overview

Essentia is a comprehensive C++ library for audio analysis and music information retrieval, developed by the Music Technology Group (MTG) at Universitat Pompeu Fabra in Barcelona. It is one of the most complete open-source MIR libraries available, providing algorithms for:
- Low-level spectral features
- Rhythm analysis (BPM, beat tracking)
- Tonal analysis (key, chords)
- High-level semantic classification (genre, mood, instrumentation)

Finch is considering Essentia as the primary engine for audio analysis due to its C++ implementation (performance), comprehensive algorithm coverage, and permissive licensing.

## Core Capabilities

| Feature | Implementation | Notes |
| --- | --- | --- |
| BPM detection | RhythmExtractor2013, MultiFeature | Multiple algorithms available |
| Beat tracking | BeatTrackerMultiFeature, BeatTrackerDegara | Real-time capable |
| Key detection | KeyExtractor with profile options | Krumhansl, Temperley, etc. |
| Chord estimation | ChordsDetection | From chroma features |
| Loudness | Loudness, LoudnessVickers | Multiple standards |
| MFCCs | MFCC | Standard implementation |
| Spectral features | SpectralCentroid, Rolloff, Flux, etc. | Comprehensive set |
| Genre classification | MusicNN, TensorFlow models | Pre-trained models available |
| Mood classification | Pre-trained classifiers | From MSD embeddings |
| Instrumentation | Pre-trained classifiers | MusicCNN-based |
| Embeddings | MusicCNN feature extraction | 50-dim embeddings |
| Streaming mode | Yes | Efficient for large files |
| Python bindings | Yes | Full API coverage |
| License | AGPL (commercial license available) | Must consider for Finch |

## Architecture

### Standard Mode
Imperative programming style:
```cpp
audio = MonoLoader(filename="input.wav")()
rhythm_extractor = RhythmExtractor2013(method="multifeature")
bpm, beats, confidence, _, _ = rhythm_extractor(audio)
```

### Streaming Mode
Connected algorithm network with automatic scheduling:
```cpp
// Algorithms connected in a network
loader->frameCutter->window->spectrum->mfcc->pool
```

### Key Characteristics
- **FFTW3** for FFT computation
- **Eigen** for matrix operations  
- **TensorFlow** backend for neural models
- **YAML/JSON** for algorithm configuration

## Strengths

1. **Comprehensive coverage**: Implements more MIR algorithms than any other open-source library; covers MPEG-7 audio descriptors
2. **C++ performance**: Significantly faster than pure Python alternatives (Librosa) for batch processing
3. **Pre-trained models**: MusicNN and other models trained on Million Song Dataset ready to use
4. **Dual API**: Standard (imperative) and streaming modes for different use cases
5. **Active maintenance**: Regular releases; MTG actively uses it in research
6. **Well-documented algorithms**: Academic papers cited for most algorithms

## Chronic Failures / Limitations

1. **License complexity**: AGPL v3 with commercial licensing option; Finch must evaluate compliance
2. **Documentation gaps**: API docs good, but integration examples sparse
3. **Python packaging**: Installation can be complex (conda helps; pip is limited)
4. **Model freshness**: Pre-trained models from 2018-2020; newer architectures not integrated
5. **Limited codec support**: Depends on FFmpeg for decoding; some formats may have issues
6. **No built-in confidence calibration**: Raw confidence scores may not be well-calibrated

## Version History

| Version | Date | Notable changes |
| --- | --- | --- |
| 2.1-beta | 2024 | Python 3.11 support, various fixes |
| 2.0 | 2022 | TensorFlow 2.x support, new models |
| 2.0-beta | 2020 | Major streaming improvements |
| 2.1 | 2023-2024 | Bug fixes, maintenance |

## Finch Relevance

### Status: Algorithm Reference for Rust Implementation

**Essentia is a Rosetta Stone — study it, then build in Rust.**

The AGPL license prevents direct use, but Essentia's source code is an invaluable reference for understanding how to implement audio analysis algorithms in production. Finch uses Essentia to:

1. **Understand algorithm implementations** — Read the C++ source, map to Rust
2. **Establish quality benchmarks** — Essentia output = target for Rust crates
3. **Find the papers** — Essentia's algorithms cite the research papers
4. **Validate Rust implementations** — Does `signal-analysis-rhythm` match
   Essentia's BPM?

### Alignment with Finch Rust Crates

| Finch Crate | Essentia Reference | Implementation Notes |
| --- | --- | --- |
| `signal-analysis-rhythm` | `rhythm/rhythmextractor2013.cpp` | Böck algorithm in Rust |
| `signal-analysis-tonal` | `tonal/key.cpp` | Chroma + profile correlation |
| `signal-analysis-loudness` | `loudness/*.cpp` | LUFS standard implementation |
| `signal-dsp-spectral` | `spectral/*.cpp` | Spectrogram, MFCC, chroma |
| `signal-analysis-embed` | `ml/musicnn` | CNN architecture study (train separately) |

### License Blocker

**Essentia is AGPL v3** — Cannot use in Finch product.
- **Solution**: Study algorithms, reimplement in Rust from scratch
- **Clean room**: Read Essentia for understanding, write Rust independently
- **Benchmark**: Use Essentia output to validate Rust implementations

### What Finch Should Extract from Essentia

For each algorithm Finch needs, study Essentia's implementation:

**1. Algorithm Flow (read source → document → implement in Rust)**
- `rhythm/rhythmextractor2013.cpp`: Trace audio → onset → tempogram → beats → BPM
- `tonal/key.cpp`: Trace audio → FFT → chroma → profile correlation → key
- `loudness/loudness.cpp`: Trace audio → filter → mean square → LUFS

**2. Key Parameters (extract constants and defaults)**
- Window sizes, hop sizes, FFT sizes
- Filter coefficients, threshold values
- Profile templates (Krumhansl, Temperley vectors)

**3. Performance Targets (benchmark against Essentia)**
- Analysis time per minute of audio
- Memory usage during analysis
- Target: Rust implementation within 2x of Essentia C++

**4. Output Formats (match for compatibility)**
- BPM as float with confidence
- Key as tonic + mode with correlation score
- JSON structure for sidecar compatibility

### What Finch Should NOT Do

1. **Copy-paste code**: AGPL violation, and Rust idioms differ from C++
2. **Use Essentia's class hierarchies**: C++ OOP doesn't map to Rust traits directly
3. **Depend on Essentia types**: Define Finch's own AudioBuffer, etc.
4. **Use pre-trained models**: MusicNN weights are AGPL; train independent models

## Sources

| Source | Type | Confidence | Notes |
| --- | --- | --- | --- |
| essentia.upf.edu | Official docs | High | Primary reference |
| GitHub: MTG/essentia | Source code | High | Implementation details |
| ISMIR papers on Essentia | Academic | High | Algorithm validation |
| Moffat et al. comparison | Benchmark | High | Comparison with Librosa |

## Comparison with Alternatives

| Aspect | Essentia | Librosa | Aubio |
| --- | --- | --- | --- |
| Language | C++ / Python | Python | C / Python |
| Speed | Fast | Slower | Fast |
| Completeness | High | Medium | Low |
| Pre-trained ML | Yes | No | No |
| License | AGPL | ISC | GPL |
| Documentation | Good | Excellent | Fair |
| Community | Academic | Broad | Niche |

## Next Task

Evaluate license compatibility with Finch's distribution model. If compatible, prototype Essentia integration for BPM and key detection tracks.
