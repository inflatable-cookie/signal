# Signal Research Master Index

Purpose: provide one implementation-facing map from Signal research outputs to
crate planning, algorithm work, validation, and downstream consumers such as
Finch and Loophole.

Stretch translation memos are retained research evidence. Their historical
`Promoted` labels mean promoted into the old proof sequence, not authorized for
production or new implementation. Contract `084` and `g10.030` closed the
successor program without promotion. The non-phase-vocoder feasibility
decision controls any future reopening.

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
| [Creative Stretch Source Triangulation](./specimen-dossiers/creative-stretch-source-triangulation.md) | PaulXStretch, CDP, Potenza, and pinned SiTraNoStar whole-path ownership behind retained creative targets | Reviewed; no unused fifth owner, direct-renewal gate reset recommended |
| [Cyclic Time-Stretch Source Architecture](./specimen-dossiers/cyclic-time-stretch-source-architecture.md) | Akai fixed `CYCLIC` versus adaptive `INTELL`, Potenza slow-anchor grains, SickoCV repeat/jump cycles, Sonic period insertion, and ReaReaRea forensics | Executable forensics complete; behavioral synthesis ready |

Current stretch feasibility decision:
[Offline Time-Stretch Non-Phase-Vocoder Feasibility](../architecture/offline-time-stretch-non-phase-vocoder-feasibility.md).

Current creative-stretch decision:
[Offline Creative Time-Stretch Study](../architecture/offline-creative-time-stretch-study.md).
Its automatic `4x`-`16x` spectral route is paused. Both explicit cyclic
candidates are rejected and deleted: the first failed synthetic pitch, and the
similarity-aligned replacement failed structural search reachability. Final
ownership reassessment found no third materially different, source-backed
whole-renderer path under the evidence available then.

The operator later reopened explicit `Cyclic` as a separate research program.
Original Akai manuals separate fixed `CYCLIC` from adaptive `INTELL`, and
pinned SickoCV adds an unstudied repeat/jump schedule. Batch 32.2 now
distinguishes repeat/jump from compressed-anchor behavior and records
ReaReaRea's separate centred event placement. Both prior Signal candidates
remain rejected and deleted; no renderer brief or DSP is ready. This does not
reopen the transparent successor lane.

Explicit operator research reopening and pinned source triangulation selected
one materially different neutral `Dream` family: `RenewalSpectral`. Later
batches corrected crest calibration, passed complete mono admission, rejected
the first linked-stereo law, and froze native left/right source-relative
ownership. Batch 31.29 passed construction and structural admission, then
failed one `16x` replica row and two `4x` pitch rows. Batch 31.30 found that
candidate seed was not frozen across the otherwise matching mono evidence.
Pinned PaulX uses one renewal path across the retained ratios, so the failed
receipt could not select a range switch. Batch 31.31 then tested the audited
seed: construction and structural admission passed, `Y04` cleared, but `Y02`
failed the `8x` chord pitch row. Two complete checkpoints now fail tonal pitch
across different seeds, material, and ratios. Batch 31.32 found no eligible
complete source-backed replacement with intrinsic tonal coherence. Renewal is
closed under that terminal comparator gate without closing the PaulX-like
product target. The operator then made finite PaulX-relative pitch delta a
mandatory diagnostic rather than a rejection threshold. Batch 31.33 froze one
fresh listening-led source-relative candidate. Batch 31.34 rejected it at
synthetic `Y08`. Batch 31.35 classified the over-broad complete-output dropout
scan as executable evidence-construction failure and froze one fresh
support-audited authority. Candidate DSP and product exposure remain absent
from `main`. Batch 31.36 passed compile, construction, structural, synthetic,
and concealed mono admission, then failed valid exact-source stereo at `16x`:
local mapped-window balance drift reached about `2.00 dB` on bass and
`9.37..9.42 dB` with channel-dominance reversal on the full mix. Batch 31.37
found the native-channel law already preserves exact current-frame complex
relation by common rotation. Independent frame renewal and waveform blending,
not a missing same-frame relation formula, own the local image failure.
Source-backed temporal corrections select already-closed coherent families;
post-hoc gain, covariance, consistency, smoothing, phase, and `space`
variants are unsupported repair paths. Renewal is closed under the current
stereo hard gate. The operator then made mapped local source balance
diagnostic while retaining hard structural and whole/band controls plus
eligible independent listening. Batch 31.38 froze one fresh complete
candidate without reviving deleted code. Batch 31.39 rejected that candidate
at synthetic `Y04` and `Y09`. Batch 31.40 found no retained executable
identity capable of reconciling its `7/9` receipt with Batch 31.36's `9/9`
receipt. Further renewal implementation is closed; the product target remains.

Batch 31.41 then completed the explicitly commissioned different-owner study.
Pinned SiTraNoStar supplies executable classical STN/noise-morphing evidence,
while the related papers own two-stage decomposition, component scheduling,
and `4x`/`8x` listening. The source is mono-only, nondeterministic, full-file,
approximate-length, GPL clean-room evidence, so it is not a production
dependency or an implementation brief. `LinkedStnNoiseMorph` is selected for
one complete Signal brief with linked tonal state, shared transient events,
continuous multichannel residual excitation, exact boundaries, deterministic
bounded state, and the retained long-form listening order.

Linked STN later closed without acoustic evidence after repeated executable-
authority failure. Batch 31.64 found no unused, materially simpler fifth
family. Direct PaulX-style magnitude renewal remains the smallest source-backed
owner of the accepted sound. Batch 31.65 records the operator-authorized
product-gate reset and freezes one complete implementation authority:
[Offline Creative Direct-Renewal Owner Study](../architecture/offline-creative-direct-renewal-owner-study.md).

Batch 31.66 passed that complete candidate and Batch 31.67 admitted its exact
private fixed-ratio surface. Batch 31.68 retained the lower-overlap pause.
Batch 31.69 selected Csound's stereo pointer-led granular family, rejected
channel-local `Warp1` state as stereo authority, and froze one complete
`LayeredCloud` brief for continuous fixed `16x..100x`. The upper overlap stays
paused because admitted Dream has no interior `16x..32x` render. Batch 31.70's
green synthetic receipt was evidence-invalid. Batch 31.71 audited and deleted
that identity. Batch 31.72 froze one source-clean `AuditedLayeredCloud`
replacement with complete executable evidence ownership.

Admitted private renderer authority:
[Offline Creative DirectRenewalDream Renderer Brief](../architecture/offline-creative-direct-renewal-dream-brief.md).

Frozen high-range candidate authority:
[Offline Creative AuditedLayeredCloud Renderer Brief](../architecture/offline-creative-audited-layered-cloud-brief.md).

Closed evidence-invalid brief and audit ledger:
[Offline Creative LayeredCloud Renderer Brief](../architecture/offline-creative-layered-cloud-brief.md).

Rejected comparator-audited neutral `Dream` candidate brief:
[Offline Creative ComparatorAuditedRenewalSpectral Renderer Brief](../architecture/offline-creative-comparator-audited-renewal-spectral-brief.md).

Rejected support-audited neutral `Dream` candidate brief:
[Offline Creative SupportAuditedListeningLedSourceRelativeRenewalSpectral Renderer Brief](../architecture/offline-creative-verified-source-relative-renewal-spectral-brief.md).

Rejected-at-compile neutral `Dream` successor brief:
[Offline Creative CompensatedRenewalSpectral Renderer Brief](../architecture/offline-creative-compensated-renewal-spectral-brief.md).

Rejected neutral `Dream` brief:
[Offline Creative RenewalSpectral Renderer Brief](../architecture/offline-creative-renewal-spectral-brief.md).

Rejected similarity-aligned cyclic brief:
[Offline Creative SimilarityAlignedCyclic Renderer Brief](../architecture/offline-creative-similarity-aligned-cyclic-brief.md).

Rejected cyclic-owner brief:
[Offline Creative CyclicGrain Renderer Brief](../architecture/offline-creative-cyclic-grain-brief.md).

Rejected final-candidate brief:
[Offline Creative ContinuousExcitationComplexRelation Renderer Brief](../architecture/offline-creative-continuous-excitation-complex-relation-brief.md).

Rejected replacement brief:
[Offline Creative ContinuousExcitationSpectral Renderer Brief](../architecture/offline-creative-continuous-excitation-spectral-brief.md).

Rejected first-owner brief:
[Offline Creative DiffuseSpectral Renderer Brief](../architecture/offline-creative-diffuse-spectral-brief.md).

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
| [Shared-Decision Waveform Topology](./translation-memos/019-shared-decision-waveform-topology.md) | Select one clean-room guided frequency-partitioned linked-phase proof; Rule 31X validates a local unlocked-state correction but rejects the topology at the stereo gate | Rejected |
| [Bounded Normalized Sliced Integration](./translation-memos/020-bounded-normalized-sliced-integration.md) | Reject fixed-sample cross-rate slicing; validate one 10 ms-lattice exact sliced frame with fixed memory and persistent channel state | Validated |
| [Direct Scale-Timeline Ownership](./translation-memos/021-direct-scale-timeline-ownership.md) | Reject unlocked over-linking and outer meta-slice projection; restore one direct coefficient timeline per exclusive scale | Promoted |
| [Direct Scale-Timeline Preregistration](./translation-memos/022-direct-scale-timeline-preregistration.md) | Freeze direct physical geometry, absolute schedule, state ownership, boundaries, and fixed capacities; correct the multi-scale identity claim | Promoted |
| [Direct Channel-Local Peak Topology](./translation-memos/023-direct-channel-local-peak-topology.md) | Reject the joint peak map; retain each requesting channel's peak location and borrow only a compatible frequency-aligned trajectory | Promoted for mechanics contract |
| [Direct Material-State Frequency Completion](./translation-memos/024-direct-material-state-frequency-completion.md) | Close further peak-ownership repair; require modal frequency completion between fuzzy evidence and terminal phase state | Promoted for no-audio contract |

## Consumer Guidance

| Consumer | Role | Start here |
| --- | --- | --- |
| Finch | Wrapper, review UX, sidecar/output integration | Source Hub 002 and the relevant value track |
| Loophole | Runtime host and authority integration | Source Hub 002 and architecture/system docs |

## Next Task

Execute `g10.032` Batch 32.3 only. Synthesize the Cyclic receipt and correct
the future behavioral gate. Do not select or implement a Signal renderer.
