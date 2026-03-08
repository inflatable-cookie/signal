# Source Hub 002: Signal Library Architecture

Status: Draft  
Topic: Signal shared crate structure and consumer integration  
Owner: Research  
Last updated: 2026-03-08

## Purpose

Document the relationship between **Signal** as the shared audio-systems repo
and its first consumers, especially **Finch** and **Loophole**.

## Overview

### Signal
A general-purpose Rust DSP and audio analysis library from the Loophole project.
- **Repository**: sibling `signal` repo (`/Users/betterthanclay/Dev/projects/signal` locally)
- **License**: MIT/Apache (open source)
- **Scope**: General audio DSP, MIR, synthesis
- **Replaces**: Essentia, Librosa (for Rust ecosystem)

### Finch
A music analysis application for music library workflows.
- **Repository**: `finch/finch`
- **Scope**: Desktop app, UI, workflow, library-specific features
- **Uses**: Signal for all audio analysis

## Package Structure

### Recommended Signal Packages

```
signal/
├── crates/
│   ├── signal-primitives/          # Shared audio/time/channel/buffer primitives
│   ├── signal-io/                  # Decode/encode and offline asset/file surfaces
│   ├── signal-dsp/                 # Generic reusable DSP kernels
│   ├── signal-dsp-spectral/        # FFT/STFT and low-level spectral transforms
│   ├── signal-analysis/            # Shared analysis result and confidence model
│   ├── signal-analysis-rhythm/     # Onset, tempo, beat, meter
│   ├── signal-analysis-tonal/      # Chroma, tuning, key, future harmonic work
│   ├── signal-analysis-loudness/   # LUFS, true peak, LRA
│   ├── signal-analysis-embed/      # Embeddings and future classifier support
│   ├── signal-graph/               # Graph model and execution semantics
│   ├── signal-runtime/             # Embeddable runtime orchestration
│   ├── signal-plugin/              # Common plugin host abstractions
│   ├── signal-plugin-clap/         # CLAP adapter
│   ├── signal-hardware/            # Common device abstractions
│   ├── signal-host-local/          # Local desktop runtime host
│   ├── signal-host-server/         # Headless/remote runtime host
│   └── signal-plugin-sandbox/      # Out-of-process plugin container
├── docs/
├── src/
└── Cargo.toml
```

### Finch Crates

```
finch/
├── app/                      # GPUI desktop application
├── controller/               # Rust controller (uses Signal)
│   ├── src/analysis.rs       # Calls signal-* crates
│   └── Cargo.toml:
│       [dependencies]
│       signal-analysis-rhythm = { path = "../../signal/crates/signal-analysis-rhythm" }
│       signal-analysis-tonal = { path = "../../signal/crates/signal-analysis-tonal" }
│       signal-analysis-loudness = { path = "../../signal/crates/signal-analysis-loudness" }
│
├── shared/                   # IPC types
└── docs/research/            # Signal-owned DSP/analysis authority
```

## Dependency Flow

```
Finch (Application)
    │
    ├──► signal-analysis-rhythm
    │      └──► signal-dsp-spectral
    │
    ├──► signal-analysis-tonal
    │      └──► signal-dsp-spectral
    │
    ├──► signal-analysis-loudness
    │      └──► signal-dsp
    │
    └──► signal-primitives
```

## API Examples

### Beat Tracking (Finch using Signal)

```rust
// In Finch controller: controller/src/analysis.rs

use signal_analysis_rhythm::{BeatTracker, BeatConfig, BeatResult};
use signal_primitives::AudioBuffer;

pub fn analyze_beat(path: &Path) -> Result<BeatAnalysis, Error> {
    // Load audio (using Finch's loader)
    let audio: AudioBuffer = load_audio(path)?;
    
    // Use Signal for analysis
    let mut tracker = BeatTracker::new(BeatConfig {
        sample_rate: audio.sample_rate(),
        ..Default::default()
    });
    
    let result = tracker.analyze(audio.samples())?;
    
    // Convert to Finch's internal format
    Ok(BeatAnalysis {
        bpm: result.bpm,
        beat_positions: result.beat_positions,
        confidence: result.confidence,
        // ... sidecar format
    })
}
```

### Key Detection (Finch using Signal)

```rust
// In Finch controller

use signal_analysis_tonal::{KeyDetector, KeyConfig, KeyResult};

pub fn analyze_key(audio: &AudioBuffer) -> Result<KeyAnalysis, Error> {
    let mut detector = KeyDetector::new(KeyConfig::default());
    let result = detector.analyze(audio)?;
    
    Ok(KeyAnalysis {
        key: result.key.to_string(),  // e.g., "C major"
        confidence: result.confidence,
        chroma: result.chroma.to_vec(),
        // ... sidecar format
    })
}
```

### Loudness (Finch using Signal)

```rust
// In Finch controller

use signal_analysis_loudness::{LoudnessMeter, LoudnessConfig, Platform};

pub fn analyze_loudness(audio: &AudioBuffer) -> Result<LoudnessAnalysis, Error> {
    let result = LoudnessMeter::analyze(audio, &LoudnessConfig::default())?;
    
    Ok(LoudnessAnalysis {
        integrated_lufs: result.integrated,
        true_peak_dbtp: result.true_peak,
        loudness_range_lu: result.lra,
        meets_spotify: result.meets_platform(Platform::Spotify),
        // ... sidecar format
    })
}
```

## Development Workflow

### When adding a new analysis feature:

1. **Research** (in Signal docs/research/)
   - Study Essentia algorithm
   - Create algorithm specification
   - Document Signal API requirements

2. **Implement** (in Signal repository)
   - Create/modify the relevant Signal package
   - Implement algorithm in Rust
   - Add tests, benchmarks
   - Document API

3. **Validate** (compare Signal vs Essentia)
   - Run same audio through both
   - Verify accuracy targets
   - Benchmark performance

4. **Integrate** (in Finch controller)
   - Add Signal crate to Finch dependencies
   - Call Signal API from Finch analysis code
   - Convert to Finch sidecar format
   - Add to UI

## API Design Principles

### Signal (Library)
- **General-purpose**: Not specific to Finch or music libraries
- **Ergonomic**: Clean, idiomatic Rust APIs
- **Performant**: Match or exceed Essentia performance
- **Well-tested**: Comprehensive test coverage
- **Documented**: rustdoc for all public APIs

### Finch (Application)
- **Domain-specific**: Music library workflows
- **UX-focused**: User-facing features
- **Integration**: Convert Signal outputs to Finch formats
- **Workflow**: Batch processing, review, export

## Versioning and Releases

### Signal
- Independent versioning per crate
- Semantic versioning
- Published to crates.io (eventually)
- Stable API guarantees post-1.0

### Finch
- Depends on specific Signal versions
- May use git dependencies during development
- Pins to Signal releases for stability

## Open Questions

1. **Monorepo vs separate repos?**
   - Current plan: Signal as a sibling shared repo, Finch separate
   - Git dependencies for development
   - Crates.io for releases

2. **Signal scope boundaries?**
   - What stays in Signal vs Finch?
   - Rule: General DSP/analysis/runtime surface → Signal
   - Rule: Application/domain-specific workflow → Finch

3. **Contribution flow?**
   - Finch discovers need → specifies Signal API
   - Signal implements → releases
   - Finch integrates → uses

## References

| Resource | Location | Description |
|----------|----------|-------------|
| Signal specs | `docs/research/algorithm-specs/` | Algorithm specifications |
| Signal source | `../signal/` | Implementation |
| Finch integration | `finch/controller/src/analysis.rs` | Usage examples |

---

## Next Task

Promote the broader package naming proposal into the roadmap and use it to
retire the older Finch-shaped crate examples.
