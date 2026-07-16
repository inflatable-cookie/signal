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

## Translation Memos

| Memo | Decision | Status |
| --- | --- | --- |
| [Offline Time-Stretch Successor](./translation-memos/001-offline-time-stretch-successor.md) | Historical one-global-map successor sequence | Superseded by memo 002 |
| [Rubber Band Behavioural Forensics](./translation-memos/002-rubber-band-behavioural-forensics.md) | Reopen local timing, transient phase treatment, and simultaneous multi-resolution synthesis from measured comparator behaviour | Promoted |
| [Non-Duplicating Stretch Ownership](./translation-memos/003-non-duplicating-stretch-ownership.md) | Select one time-adaptive painless NSG frame after redundant full-band union rejection | Promoted |
| [Source-Studied Stretch Architecture](./translation-memos/004-source-studied-stretch-architecture.md) | Reject frequency partitioning; retain the coherent fixed-grid weighted predictor as the report-only source-studied baseline | Validated for mono source translation |
| [Weighted Predictor Fidelity](./translation-memos/005-weighted-predictor-fidelity.md) | Correct scheduling, geometry, vertical twists, normalization, fallback, and update ordering as one topology | Promoted |
| [Linked-Stereo Relationship-Preserving Recurrence](./translation-memos/006-linked-stereo-recurrence.md) | Select one per-bin reference recurrence and preserve peer current-input complex relation plus magnitude | Promoted |

## Consumer Guidance

| Consumer | Role | Start here |
| --- | --- | --- |
| Finch | Wrapper, review UX, sidecar/output integration | Source Hub 002 and the relevant value track |
| Loophole | Runtime host and authority integration | Source Hub 002 and architecture/system docs |

## Next Task

Implement Batch 29.7E as the bounded report-only reference-relative recurrence
proof. Exercise exact ties and ownership crossings, then rerun the unchanged
29.7C quality gate before stereo export.
