# Track 2: Key and Tonal Analysis

Status: Draft
Track: Key and tonal analysis
Owner: Research
Last updated: 2026-03-08
Primary Finch tags: `SIGNAL`, `CORE`, `RUST`

## 1) Problem statement

How should Finch detect the musical key (tonal center) of audio tracks? Key detection involves:
- Identifying the tonic note (C, C#, D, etc.)
- Determining mode (major vs minor)
- Handling ambiguous or modal cases
- Managing tracks with key changes
- Providing confidence for library submission decisions

## 2) Why this track matters to Finch

Key is the second most important piece of metadata (after BPM) for music library organization. Finch users need:
- Reliable key detection for catalog compatibility
- Understanding of confidence for edge cases
- Handling of modal music beyond major/minor
- Graceful handling of key changes or ambiguous tonality

## 3) Cross-tool comparison

| Tool/library | Approach | Strengths | Failure modes | Finch signal |
| --- | --- | --- | --- | --- |
| **Librosa** | Chroma feature extraction + Krumhansl-Schmuckler key profile correlation | Simple, fast, well-documented | Struggles with modal music; octave errors | Good baseline; chroma is solid foundation |
| **Essentia** | KeyExtractor with multiple profile options (Krumhansl, Temperley, etc.) | C++ performance; configurable; multiple algorithms | Complexity of options | Strong candidate; profile flexibility valuable |
| **Mixed In Key** | Proprietary (chroma-based with ML enhancement) | Very accurate for electronic/pop; industry standard | Black box; occasional mode errors | Accuracy benchmark |
| **KeyFinder (legacy)** | Open-source C++ with Qt GUI | Open implementation to study | Discontinued; doesn't build easily | Historical reference only |
| **Apple Music/Spotify** | Proprietary | Scale-tested on massive catalogs | Not accessible for comparison | Implied quality level |

## 4) Repeated patterns

1. **Chroma features are universal**: 12-dimensional pitch class profile is the standard representation
2. **Key profiles encode expectations**: Krumhansl-Schmuckler profiles (from music psychology) widely used
3. **Temporal averaging is critical**: Single-frame chroma too noisy; need aggregation across track
4. **Mode detection harder than tonic**: Major/minor distinction more error-prone than tonic identification
5. **Modal music is systematically misclassified**: Dorian/Mixolydian often detected as relative major/minor

## 5) Frontier research signals

### Deep Learning Approaches
- **Convolutional key detection**: CNNs on chromagrams achieving higher accuracy
- **Multi-task learning**: Joint key and chord detection improves both
- **Transformer attention**: Self-attention over time for handling key changes

### Key Change Detection
- **Structural segmentation + key**: Detecting key changes at section boundaries
- **Sliding window approaches**: Tracking key evolution over time
- **HMM-based models**: Probabilistic key progression modeling

### Extended Tonality
- **Beyond major/minor**: Detection of Dorian, Phrygian, Mixolydian modes
- **Non-Western scales**: Recognition of non-12-tone and non-Western tonal systems
- **Atonality detection**: Explicit detection of lack of tonal center

## 6) Signal/Finch Strategy

### Implementation in Signal, Consumed by Finch

**Target: Signal library provides `signal-analysis-tonal`, Finch consumes it**

1. **Deep study of Essentia's KeyExtractor**
   - Trace chroma feature extraction pipeline
   - Document profile templates (Krumhansl, Temperley) as Rust constants
   - Map correlation algorithm to Signal implementation

2. **Rust ecosystem mapping (for Signal)**
   - **Chroma features**: Build on `rustfft` → pitch class aggregation
   - **Tuning estimation**: Implement from papers (A440 vs other)
   - **Profile matching**: Cosine similarity or correlation in Rust
   - **Key profiles**: Static arrays or configurable JSON

3. **Implementation plan for `signal-analysis-tonal` (in Signal library)**
   ```rust
   // In Signal library: signal-analysis-tonal/src/lib.rs
   pub struct KeyDetector;
   impl KeyDetector {
       pub fn detect(&self, chroma: &[f32; 12]) -> KeyResult;
       pub fn with_profile(profile: KeyProfile) -> Self;
   }
   ```

4. **Finch integration**
   ```rust
   // In Finch: controller/src/analysis.rs
   use signal_tonal::{KeyDetector, KeyDetectorConfig};
   
   let mut detector = KeyDetector::new(KeyDetectorConfig::default());
   let result = detector.analyze(&audio)?;
   // Convert to Finch sidecar format
   ```

4. **Benchmark targets**
   - Accuracy: Match Essentia KeyExtractor
   - Profile comparison: Krumhansl vs Temperley validation
   - Confidence calibration: Correlation strength → accuracy

5. **Multiple profile evaluation**
   - Krumhansl-Schmuckler (cognitive basis)
   - Temperley (probabilistic model)
   - Custom profiles for specific genres if needed
   - Consensus/disagreement provides confidence signal

6. **Confidence from correlation strength**
   - High correlation = confident
   - Low correlation = flag for review
   - Multiple close candidates = ambiguous

7. **Handle modal music explicitly**
   - Report mode confidence separately
   - Consider "modal" as third option beyond major/minor
   - Flag likely Dorian/Mixolydian for human review

### Risks to avoid

- **Overconfidence on ambiguous tracks**: Many electronic tracks lack strong tonality
- **Western bias**: Default profiles assume Western tonal expectations
- **Ignoring confidence signals**: Low correlation should trigger review, not guess
- **Binary major/minor forced choice**: Mode uncertainty should be expressable

### Evidence or prototype needed

1. **Essentia source analysis**: Document exact KeyExtractor algorithm flow
2. **Rust crate survey**: Existing chroma/key implementations in Rust ecosystem
3. **Profile comparison**: Krumhansl vs Temperley vs others on representative corpus
4. **Prototype `signal-analysis-tonal`**: Minimal key detection in Rust
5. **Genre evaluation**: Classical (clear keys) vs electronic (ambiguous) vs jazz (modal)
6. **Confidence calibration**: Can correlation strength predict accuracy?
7. **Benchmark**: Essentia and Mixed In Key → targets for Rust impl

## 7) Source inventory

| Source | Type | Confidence | Notes |
| --- | --- | --- | --- |
| Krumhansl & Kessler 1982 | Paper | High | Original key profile research |
| Essentia KeyExtractor docs | Official | High | Implementation reference |
| ISMIR key detection papers | Papers | Medium | Various approaches |
| Temperley 2001 | Paper | High | Alternative profile system |

## 8) Decision state

- [ ] `continue research` — need more evidence
- [x] `prototype first` — ready to validate
- [ ] `promote to concept work` — pending prototype

## Next Task

Implement a first `signal-analysis-tonal` prototype with multiple profiles,
then evaluate confidence calibration before Finch-specific integration work.
