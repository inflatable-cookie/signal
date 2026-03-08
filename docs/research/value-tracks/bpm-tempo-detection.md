# Track 1: BPM and Tempo Detection

Status: Draft
Track: BPM and tempo detection
Owner: Research
Last updated: 2026-03-08
Primary Finch tags: `SIGNAL`, `CORE`, `RUST`

## 1) Problem statement

How should Finch detect BPM (tempo) accurately across diverse musical genres, production styles, and audio qualities? The challenge includes:
- Handling tempo changes within tracks (rubato, accelerando)
- Accurate detection across genres (electronic, acoustic, classical, jazz)
- Distinguishing between half/double time (90 BPM vs 180 BPM)
- Providing confidence estimates for uncertain detections
- Operating efficiently on local hardware without cloud dependency

## 2) Why this track matters to Finch

BPM is one of the most fundamental pieces of metadata for music library submission. Finch's target users need:
- Accurate tempo for catalog organization and searching
- Trust in the output (confidence metrics)
- Quick analysis for batch workflows
- Consistency across similar tracks

BPM detection is a solved problem for clear cases but remains challenging for:
- Ambient/drone music without clear beats
- Classical music with rubato
- Live recordings with timing variation
- Tracks with intentional tempo modulation

## 3) Cross-tool comparison

| Tool/library | Approach | Strengths | Failure modes | Finch signal |
| --- | --- | --- | --- | --- |
| **Librosa** | Onset detection + dynamic programming beat tracking | Well-documented, easy to use, proven algorithms | Pure Python = slower; struggles with tempo changes | Good baseline algorithm; implementation reference |
| **Essentia** | Multi-feature rhythm extractor (RhythmExtractor2013) | C++ performance; multiple algorithms; streaming mode | More complex API; less Pythonic | Primary engine candidate; good for integration |
| **Mixed In Key** | Proprietary algorithm | Very accurate for electronic music; widely trusted by DJs | Black box; no confidence metric exposed; commercial | Target accuracy benchmark |
| **Serato** | Proprietary beat grid analysis | Tight integration with DJ workflow; handles grid editing | Locked to DJ context; no exportable analysis | UX pattern reference |
| **Beatport/Beatsource** | Proprietary + manual curation | Highly accurate for dance music; multiple BPM values | Limited to platform; curation-heavy | Quality benchmark for dance genres |
| **Aubio** | C library, lightweight | Fast, embeddable, real-time capable | Fewer features; less actively maintained | Possible lightweight alternative |

## 4) Repeated patterns

1. **Multi-stage processing is universal**: all tools use onset/envelope detection → tempo estimation → beat tracking/refinement
2. **Dynamic programming for beat tracking**: Ellis (2007) algorithm widely adopted
3. **Multi-feature fusion improves accuracy**: combining onset strength, spectral flux, and other features (Essentia's approach)
4. **Half/double detection is hard**: all tools struggle with the octave ambiguity problem; some use genre heuristics
5. **Confidence is rarely exposed**: commercial tools hide uncertainty; libraries provide some measure of confidence

## 5) Frontier research signals

### Neural Beat Tracking (2020+)
- **Böck et al. (ISMIR 2020)**: "Deconstruct, analyse, reconstruct" - TCNs (Temporal Convolutional Networks) beat prior RNNs
- **BeatNet (2021)**: Online joint beat, downbeat, and meter tracking using CRNN + particle filtering
- **Heydari & Duan (ICASSP 2021)**: "Don't look back" - online beat tracking with RNN + enhanced particle filtering

### Self-Supervised Approaches
- **Singing beat tracking with self-supervised front-end (ISMIR 2022)**: Linear transformers for singing voice beat tracking
- Phase-aware joint beat and downbeat estimation shows promise for difficult cases

### Production Readiness
- TCN-based approaches achieving state-of-the-art on standard datasets
- Open implementations available (madmom library includes TCN beat tracker)
- Real-time capable on CPU

## 6) Signal/Finch Strategy

### Implementation in Signal, Consumed by Finch

**Target: Signal library provides `signal-analysis-rhythm`, Finch consumes it**

1. **Deep study of Essentia's RhythmExtractor2013**
   - Trace through source: onset detection → tempogram → beat tracking
   - Document exact algorithm parameters and defaults
   - Map to Signal implementation

2. **Rust ecosystem mapping (for Signal)**
   - **FFT**: `rustfft` (pure Rust) or `realfft` for real-input optimization
   - **Onset detection**: Build on top of spectral flux in Rust
   - **Dynamic programming**: Standard Rust implementation
   - **Peak picking**: Custom or adapt existing crates

3. **Implementation plan for `signal-analysis-rhythm` (in Signal library)**
   ```rust
   // In Signal library: signal-analysis-rhythm/src/lib.rs
   pub struct BeatTracker;
   impl BeatTracker {
       pub fn new(config: BeatConfig) -> Self;
       pub fn analyze(&self, audio: &[f32]) -> BeatResult;
   }
   ```

4. **Finch integration**
   ```rust
   // In Finch: controller/src/analysis.rs
   use signal_beat::{BeatTracker, BeatConfig};
   
   let mut tracker = BeatTracker::new(BeatConfig::default());
   let result = tracker.analyze(&audio)?;
   // Convert to Finch sidecar format
   ```

5. **Benchmark targets**
   - Accuracy: Match Essentia on benchmark corpus
   - Performance: Comparable to Essentia (within 2x acceptable)
   - Memory: Lower than Essentia (no Python bindings overhead)

5. **Confidence metrics to expose**
   - Onset strength periodicity confidence
   - Tempo octave certainty from tempogram
   - Overall reliability score for trust gates

6. **Handle the octave problem**
   - Report primary estimate + alternative (half/double)
   - Genre-informed heuristics for ambiguous cases
   - Allow user override in review UI

### Risks to avoid

- **Over-engineering with neural**: Classic algorithms work well for 80%+ of cases; neural adds complexity
- **Hiding uncertainty**: Users need to know when BPM is questionable
- **Genre bias**: Electronic music training can fail on acoustic/orchestral
- **Assuming constant tempo**: Consider reporting tempo range for variable tracks

### Evidence or prototype needed

1. **Essentia source analysis**: Document exact algorithm flow in RhythmExtractor2013
2. **Rust crate survey**: Identify existing Rust audio analysis crates (stretcher, aubio-rs?)
3. **Benchmark setup**: Essentia, Mixed In Key on same corpus → accuracy target for Rust impl
4. **Prototype `signal-analysis-rhythm`**: Minimal beat tracking in Rust
5. **Genre-stratified evaluation**: Test across electronic, acoustic, classical, jazz
6. **Performance comparison**: Rust impl vs Essentia vs librosa

## 7) Source inventory

| Source | Type | Confidence | Notes |
| --- | --- | --- | --- |
| Essentia docs | Official | High | Primary implementation reference |
| Librosa docs | Official | High | Algorithm reference |
| Böck et al. ISMIR 2020 | Paper | High | TCN beat tracking state-of-the-art |
| Ellis 2007 | Paper | High | Classic dynamic programming algorithm |
| ISMIR beat tracking papers | Papers | Medium | Various approaches |

## 8) Decision state

- [ ] `continue research` — need more evidence
- [x] `prototype first` — ready to validate with Essentia baseline
- [ ] `promote to concept work` — pending prototype results

## Next Task

Implement a first `signal-analysis-rhythm` prototype against Essentia reference
tracks and calibrate confidence thresholds before Finch-specific wrapper work
begins.
