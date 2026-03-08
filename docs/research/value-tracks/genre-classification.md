# Track 4: Genre Classification

Status: Draft
Track: Genre and style classification
Owner: Research
Last updated: 2026-03-08
Primary Finch tags: `ML`, `SEMANTIC`, `RUST`

## 1) Problem statement

How should Finch approach genre and style classification? This is fundamentally different from signal analysis (BPM/key) because:
- Genre is culturally defined, not acoustically defined
- Multi-label (tracks can belong to multiple genres)
- Taxonomy varies by library/use case
- ML models needed; pure signal analysis insufficient
- Confidence and explainability are critical

## 2) Why this track matters to Finch

Finch's vision specifically mentions "grounded genre and style positioning" for library submissions. Users need:
- Genre suggestions for catalog organization
- Style descriptors that match library taxonomies
- Confidence to know when human judgment is needed
- Grounding in audio features (not LLM hallucination)

Key tension: Finch wants cautious semantic classification that stays grounded in preserved analysis evidence.

## 3) Cross-tool comparison

| Tool/library | Approach | Strengths | Failure modes | Finch signal |
| --- | --- | --- | --- | --- |
| **Essentia (MusicNN)** | Pre-trained CNN on Million Song Dataset | 50-tag multi-label; ready to use | Fixed taxonomy; may not match user's library needs | Good baseline; taxonomy may need mapping |
| **Essentia (transfer learning)** | Fine-tune MusicNN or use embeddings | Custom taxonomy possible | Requires labeled training data | Path to custom classification |
| **Librosa + sklearn** | Feature extraction + classic ML | Full control; interpretable | Requires ML expertise; feature engineering needed | Flexible but labor-intensive |
| **Spotify/Last.fm APIs** | Commercial ML models | Large-scale trained; social validation | API dependency; not local; rate limits | Not suitable for Finch's local-first constraint |
| **Landr/ eMastered** | Proprietary ML | Production-tested on uploaded tracks | Black box; service-based | UX reference; not implementation reference |
| **Sonic Annotator/Vamp** | Plugin architecture for analysis | Multiple algorithm options | Fragmented ecosystem; plugin dependencies | Possible extension point |

## 4) Repeated patterns

1. **Deep learning dominates**: CNNs on spectrograms (MusicNN, VGG-ish) are state-of-the-art
2. **Multi-label is standard**: Tracks rarely fit single genre; tag-based approaches work better
3. **Embeddings are reusable**: Pre-trained model embeddings transfer well to custom taxonomies
4. **Taxonomy is the hard part**: Audio features are solved; mapping to useful categories is bespoke
5. **Confidence calibration matters**: Model probabilities often poorly calibrated; need explicit calibration

## 5) Frontier research signals

### Transformer Models for Music
- **MusicBERT, MERT**: Transformer-based music understanding
- **Contrastive learning**: CLAP-style audio-text alignment for zero-shot classification
- **Large music models**: Similar to LLMs but for audio (MusicLM, MusicGen implications)

### Explainable Classification
- **Attention visualization**: Which audio segments contribute to genre decision?
- **Prototype networks**: Classify by similarity to learned genre prototypes
- **Concept-based models**: Explicit intermediate concepts ("has drums", "has guitar")

### Taxonomy Issues
- **Open taxonomy**: Dynamic genre vocabularies
- **Hierarchical classification**: Exploiting genre hierarchies
- **Cross-dataset learning**: Handling different taxonomies across training sources

## 6) Signal/Finch Strategy

### Implementation in Signal, Consumed by Finch

**Target: Signal library provides `signal-analysis-embed`, Finch consumes it**

1. **Study MusicNN architecture for Rust implementation**
   - CNN front-end architecture (similar to VGG-ish)
   - Mel-spectrogram input → CNN layers → dense embeddings
   - Document architecture for Rust ML framework implementation

2. **Rust ML ecosystem research (for Signal)**
   - **Burn**: Pure Rust deep learning framework (most promising)
   - **Candle**: HuggingFace's Rust ML framework
   - **ONNX Runtime Rust**: Bindings for running ONNX models
   - **tract**: ONNX inference in pure Rust (no C++ deps)

3. **Implementation plan for `signal-analysis-embed` (in Signal library)**
   ```rust
   // In Signal library: signal-analysis-embed/src/lib.rs
   pub struct AudioEmbedder;
   impl AudioEmbedder {
       pub fn from_pretrained(path: &Path) -> Result<Self>;
       pub fn embed(&self, audio: &[f32]) -> Embedding;
   }
   ```

4. **Finch integration**
   ```rust
   // In Finch: controller/src/analysis.rs
   use signal_embed::{AudioEmbedder, EmbeddingConfig};
   
   let embedder = AudioEmbedder::from_pretrained(model_path)?;
   let embedding = embedder.embed(&audio)?;
   // Use embedding for genre/mood classification in Finch
   ```

4. **Training data strategy**
   - MagnaTagATune (open, multi-label tags)
   - FMA (Free Music Archive) dataset
   - Custom dataset from user-contributed analysis
   - **Not**: Million Song Dataset (licensing restrictions)

5. **Model deployment options**
   - Train in Python (PyTorch), export to ONNX, run with `tract`
   - Train directly in Rust with `burn` (future)
   - Start simple: Small CNN, quantize for speed

6. **Confidence-first design**
   - Never present genre as "truth"
   - Show confidence score + nearest neighbors
   - Flag low-confidence for explicit human review

7. **Style descriptors over rigid genres**
   - Instead of "Rock", describe: "electric guitar, driving drums, aggressive vocals"
   - Multi-label tag prediction more useful than single genre
   - Grounded in detectable audio features

8. **Defer production description to Track 5**
   - Genre is culturally contested
   - Production traits are more objectively describable
   - Focus Finch on describable attributes

### Risks to avoid

- **Over-committing to a taxonomy**: User libraries have different category systems
- **False confidence**: Presenting ML predictions as facts
- **Training data bias**: Commercial datasets skew toward popular music
- **LLM for genre**: Language models invent genres without audio grounding

### Evidence or prototype needed

1. **Rust ML framework evaluation**: Compare burn, candle, tract for audio use case
2. **Architecture study**: Document MusicNN for Rust reimplementation
3. **Dataset research**: MagnaTagATune, FMA availability and licensing
4. **Prototype with tract**: ONNX model inference in Rust proof-of-concept
5. **Baseline evaluation**: MusicNN via Essentia (quality reference only)
6. **Taxonomy flexibility test**: Can embeddings map to different genre systems?

## 7) Source inventory

| Source | Type | Confidence | Notes |
| --- | --- | --- | --- |
| Essentia MusicNN docs | Official | High | Primary implementation path |
| Pons et al. MusicNN paper | Paper | High | Architecture reference |
| Million Song Dataset | Dataset | High | Training data context |
| ISMIR genre classification papers | Papers | Medium | Various approaches |
| MagnaTagATune dataset | Dataset | High | Multi-label tag dataset |

## 8) Decision state

- [x] `continue research` — need to validate embedding approach
- [ ] `prototype first` — pending embedding evaluation
- [ ] `promote to concept work` — not yet

## Next Task

Prototype a `signal-analysis-embed` inference path, evaluate embedding quality
on a diverse corpus, and test taxonomy-mapping plus confidence calibration.
