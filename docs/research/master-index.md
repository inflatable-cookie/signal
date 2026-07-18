# Signal Research Master Index

Purpose: provide one implementation-facing map from Signal research outputs to
crate planning, algorithm work, validation, and downstream consumers such as
Finch and Loophole.

## Start Here

1. Read the relevant source hub for ecosystem and dependency context.
2. Read the matching value track for the problem-led synthesis.
3. Use the algorithm spec when implementation detail is already concrete enough
   to shape crate APIs or validation targets.
4. Check architecture and roadmap docs before freezing a new package or host
   boundary.

## Value Tracks

| Track | Problem Area | Status | Intended Signal Surface |
| --- | --- | --- | --- |
| [Track 1: BPM and Tempo Detection](./value-tracks/bpm-tempo-detection.md) | Beat tracking, tempo estimation | Draft | `signal-analysis-rhythm` |
| [Track 2: Key and Tonal Analysis](./value-tracks/key-tonal-analysis.md) | Key detection, chroma, tonal confidence | Draft | `signal-analysis-tonal` |
| Track 3: Loudness and Dynamics | LUFS, true peak, dynamics metrics | Planned | `signal-analysis-loudness` |
| [Track 4: Genre Classification](./value-tracks/genre-classification.md) | Embeddings and semantic classification | Draft | `signal-analysis-embed` |

## Source Hubs

| Hub | Topic | Status |
| --- | --- | --- |
| [001: Rust Audio Ecosystem](./source-hubs/001-rust-audio-ecosystem.md) | Rust dependency and crate survey | Draft |
| [002: Signal Library Architecture](./source-hubs/002-signal-library-architecture.md) | Signal crate map and Finch/Loophole consumption model | Draft |

## Algorithm Specs

| Spec | Algorithm | Intended Signal Surface | Status |
| --- | --- | --- | --- |
| [001: Beat Tracking](./algorithm-specs/001-beat-tracking-boeck.md) | Böck-style multi-feature beat tracking | `signal-analysis-rhythm` | Draft |
| [002: Key Detection](./algorithm-specs/002-key-detection-krumhansl.md) | Chroma plus profile correlation | `signal-analysis-tonal` | Draft |
| [003: Loudness](./algorithm-specs/003-loudness-lufs.md) | ITU-R BS.1770 LUFS | `signal-analysis-loudness` | Draft |

## Specimen Dossiers

| Specimen | Studied for | Status |
| --- | --- | --- |
| [Essentia](./specimen-dossiers/essentia.md) | Reference algorithms, quality targets, migration cues | In progress |
| [Signalsmith Stretch](./specimen-dossiers/signalsmith-stretch.md) | Single-grid weighted phase-prediction control and Signal fidelity gap | Reviewed |
| [Rubber Band Source Architecture](./specimen-dossiers/rubber-band-source-architecture.md) | R2/R3 scheduling, scale ownership, guidance, and phase topology | Reviewed |
| [Bungee Source Architecture](./specimen-dossiers/bungee-source-architecture.md) | Whole-kernel common-region rotation and dynamic multichannel feasibility | Reviewed |
| [SBSMS Source Architecture](./specimen-dossiers/sbsms-source-architecture.md) | Linked subband partial tracking, paired stereo trajectories, and direct oscillator synthesis | Source feasibility rejected |

## Translation Memos

| Memo | Decision | Status |
| --- | --- | --- |
| [Offline Time-Stretch Successor](./translation-memos/001-offline-time-stretch-successor.md) | Historical one-global-map successor sequence | Superseded by memo 002 |
| [Rubber Band Behavioural Forensics](./translation-memos/002-rubber-band-behavioural-forensics.md) | Reopen local timing, transient phase treatment, and simultaneous multi-resolution synthesis from measured comparator behaviour | Promoted |
| [Non-Duplicating Stretch Ownership](./translation-memos/003-non-duplicating-stretch-ownership.md) | Select one time-adaptive painless NSG frame after redundant full-band union rejection | Promoted |
| [Source-Studied Stretch Architecture](./translation-memos/004-source-studied-stretch-architecture.md) | Reject frequency partitioning; retain the coherent fixed-grid weighted predictor as the report-only source-studied baseline | Validated for mono source translation |
| [Weighted Predictor Fidelity](./translation-memos/005-weighted-predictor-fidelity.md) | Correct scheduling, geometry, vertical twists, normalization, fallback, and update ordering as one topology | Promoted |
| [Linked-Stereo Relationship-Preserving Recurrence](./translation-memos/006-linked-stereo-recurrence.md) | Select one per-bin reference recurrence and preserve peer current-input complex relation plus magnitude | Promoted |
| [Rubber Band Linked-Stereo Mechanism](./translation-memos/007-rubber-band-linked-stereo-mechanism.md) | Move conditional, frequency-bounded channel ownership from same-bin projection to tracked peak regions | Promoted |
| [Linked-Stereo State And Trajectory Policy](./translation-memos/008-linked-stereo-state-and-trajectory-policy.md) | Keep reference-relative recurrence as the stereo default and make tracked peak ownership a frequency-aligned overlay | Superseded for current kernel by memo 010 |
| [Peak Owner And Phase-Field Order](./translation-memos/009-peak-owner-and-phase-field-order.md) | Reject late tracked overlays; require one complete peak-owned eligible-region operation with peer relation preserved inside it | Valid ordering law; current-kernel realization rejected |
| [Linked-Stereo Current-Kernel Operator Decision](./translation-memos/010-linked-stereo-current-kernel-operator-decision.md) | Close tracked peaks inside the coherent weighted predictor; require complete kernel-family selection before another renderer | Promoted |
| [Linked Phase-Field Kernel Family Selection](./translation-memos/011-linked-phase-field-kernel-family-selection.md) | Close PGHI for this lane; select one separate shared-rotation region-locked phase-vocoder proof | Promoted |
| [Material-State Phase Architecture Boundary](./translation-memos/012-material-state-phase-architecture-boundary.md) | Close shared rotation as a complete kernel; require independent support for the missing material and scale seams | Promoted |
| [Independent Material-State Frequency Frame](./translation-memos/013-independent-material-state-frequency-frame.md) | Close both seams from independent papers; select one painless frequency-adaptive material-phase proof | Promoted |
| [Relation-Owned Sliced Material Transport](./translation-memos/014-relation-owned-sliced-material-transport.md) | Attribute independent polar interpolation; select explicit peer/reference relation transport and a fixed sliced frame | Promoted |
| [Joint-Synthesis Consistency Boundary](./translation-memos/015-joint-synthesis-consistency-boundary.md) | Attribute post-coefficient loss to inconsistent redundant fields; require joint post-projection ownership | Promoted |
| [Paired-Channel Consistency Operator Boundary](./translation-memos/016-paired-channel-consistency-operator-boundary.md) | Close transform-domain post-projection; require one complete waveform-owning topology | Promoted |
| [Whole-Family Waveform-Ownership Decision](./translation-memos/017-whole-family-waveform-ownership-decision.md) | Close the failed single-grid proof; require waveform-domain linked-stereo ownership | Promoted |
| [Waveform-Domain Linked-Stereo Re-entry](./translation-memos/018-waveform-domain-linked-stereo-re-entry.md) | Close linked subband sinusoidal source feasibility; replace invalid local and exact-mechanics vetoes with a professional-comparator boundary | Validated |
| [Shared-Decision Waveform Topology](./translation-memos/019-shared-decision-waveform-topology.md) | Select one clean-room guided frequency-partitioned linked-phase proof with synchronized channel state and per-channel synthesis | Promoted |
| [Bounded Normalized Sliced Integration](./translation-memos/020-bounded-normalized-sliced-integration.md) | Reject fixed-sample cross-rate slicing; validate one 10 ms-lattice exact sliced frame with fixed memory and persistent channel state | Validated |

## Consumer Guidance

| Consumer | Role | Start here |
| --- | --- | --- |
| Finch | Wrapper, review UX, sidecar/output integration | Source Hub 002 and the relevant value track |
| Loophole | Runtime host and authority integration | Source Hub 002 and architecture/system docs |

## Next Task

Run `g10.029` Batch 29.7AO once under Rule 31V. Implement the frozen normalized
material policy and execute its failure-first objective matrix without a
sweep or row repair. Keep listening, holdout, product surfaces, and Batch 29.8
closed.
