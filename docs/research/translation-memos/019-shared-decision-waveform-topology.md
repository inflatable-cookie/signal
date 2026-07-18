# Shared-Decision Waveform Topology

Status: promoted
Memo: `g10.029` Batch 29.7AJ
Owner: dsp
Last updated: 2026-07-18
Related roadmap: `g10.029`

## Decision

Select one clean-room `GuidedFrequencyPartitionedLinkedPhaseVocoder` proof.
The waveform owner is the complete per-scale, all-channel phase update followed
by per-channel inverse synthesis. Material classification supplies guidance;
it never owns or mixes output waveforms.

This is not a revival of the rejected Batch 29.6CH three-STFT prototype. That
prototype demonstrated frequency partitioning around an incomplete phase and
channel kernel. Batch 29.7Y later proved exact frequency-adaptive
reconstruction, then failed because independent per-channel polar transport
preceded the shared material operator. The selected topology makes exclusive
scale ownership, synchronized phase-state selection, conditional linked peak
trajectories, and per-channel synthesis one indivisible kernel.

## Pinned Source Study

Only three complete source families were compared.

| Family | Shared decision | Channel synthesis | Result |
| --- | --- | --- | --- |
| Rubber Band R3 `4.0.0` | Per-channel material guidance; one synchronized all-channel phase update per active scale; greatest-magnitude channel selected per bin; borrowing only inside compatible linked peak regions | Every channel retains an ordinary recurrence. A borrowing peer retains its local analysis-relative offset. Each exclusive scale is inverse-synthesized per channel and scales sum only within that channel | Select the complete topology shape; exclude GPL expression, constants, thresholds, masks, and ranges |
| Signalsmith Stretch `57b93f4e` | Greatest-energy channel selected at each bin after preliminary horizontal prediction | Reference vertical prediction is completed first; each peer is reconstructed from its current complex relation to that reference at peer target energy | Retain as a clean single-grid equivariance control, not the target: no complete material state or frequency-owned scale system |
| Bungee `746833f6` | One channel-summed field selects shared peak regions and one region rotation | The same rotation is applied to every channel coefficient before per-channel inverse synthesis | Retain common rotation as locked-state evidence only: channel summation can cancel and the kernel lacks complete ordinary, unlocked, material, and scale policy |

Rubber Band's defining separation is exact enough to explain the corrected hard
mechanics. Analysis and guidance remain channel-local where material differs,
but schedule, branch order, scale geometry, and state commit are synchronized.
Peak borrowing is deterministic and channel-permutation equivariant. Identical
channels therefore follow identical paths; a silent peer remains silent because
no peer magnitude is synthesized from the owner; swapping channels only swaps
the per-channel results.

Dorran, Lawlor, and Coyle independently establish the multichannel boundary:
independent decisions can damage the image even with identical parameters, so
one correlation decision must control all channels; preserving the original
same-bin phase relation then keeps the channel relationship through synthesis.
Signalsmith and Bungee supply independent source implementations of that
decision/synthesis separation. Rubber Band is the only reviewed family that
also composes it with complete material states and exclusive scale ownership.

## Selected Kernel

The proof must implement all of these as one topology:

1. one bounded global source/output schedule shared by every channel and scale
2. one full-band material-control pass whose outputs are guidance only
3. simultaneous low, middle, and high transforms with nonoverlapping,
   exhaustive frequency ownership and identical geometry across channels
4. one synchronized all-channel phase-state update per scale in the order
   reset or attack, ordinary or unlocked, then compatible tracked-peak lock
5. an ordinary recurrence for every channel before any linked decision
6. deterministic greatest-energy reference selection and conditional
   trajectory borrowing only inside the linked, compatible locked state
7. peer synthesis from peer magnitude and its current analysis-relative phase,
   never from owner magnitude or a post-hoc image projection
8. per-channel inverse/window synthesis; scale outputs sum only inside the same
   channel and each frequency is synthesized exactly once
9. fixed declared capacities, bounded work per frame, and an explicit overflow
   result before any audio is rendered

For `C` channels and fixed scale sizes `N_low`, `N_mid`, and `N_high`, one
synthesis step is bounded by three forward and inverse transforms per channel,
finite classifier and crossover scans, and finite peak/state scans. Transform
work is `O(C * sum(N_s log N_s))`; coefficient, peak, guidance, and overlap
state is `O(C * sum(N_s))`. Source/output rings add only their declared frame
capacities. No track collection, search, or history may grow with render
duration.

Signal must derive scale durations, crossover bounds, classifier policy,
state ranges, and all numeric controls from Signal-owned physical invariants
and frozen development evidence. Rubber Band values are not candidates.

## Why Prior Families Stay Closed

- Batch 29.6CH combined exclusive bands with an incomplete source-studied phase
  translation. Listening rejected its stutter, smear, transient damage, and
  tonal loss. Frequency ownership alone was not the selected kernel.
- `FrequencyAdaptiveMaterialPhase` proved one exact canonical dual, then let
  independent polar channel interpolation create incompatible coefficient
  fields. A later shared operator could not restore waveform relations.
- `RelationOwnedSlicedMaterialTransport` preserved a relation through transport
  but modified redundant fields independently before synthesis.
- shared rotation and Bungee-style region locking cover only the locked state;
  unconditional use already failed Signal's tone and boundary evidence.
- Signalsmith remains a strong mono and single-grid stereo control, but its
  documented high-ratio diffusion and absent full material/scale state make it
  incomplete for the professional target.

## Falsifiable Proof

Batch 29.7AK is one stop-gated implementation batch.

Stage A must prove the representation and channel kernel before material
quality work: exact exhaustive scale ownership, identity reconstruction at or
below `1e-12` peak error in `f64`, finite/crop/coverage/repeat guarantees,
fixed-capacity overflow behavior, and the four Rule 31Q mechanics at `1e-6`.
Every phase-state branch and every scale must be exercised. Any miss closes the
topology without policy tuning.

Only after Stage A passes may Stage B run one preregistered complete policy
through the frozen synthetic, six-row mono, long-development, and corrected
`48`-row stereo evidence. It must pass the unchanged calibrated gate, improve
at least `245/384` local windows, fail at most `13/48` retained local rows, keep
maximum normalized-Gram residual at or below `0.01744693815260`, and introduce
no row-complete mono regression. One miss closes the implementation. There is
no factor sweep, per-row repair, concealed listening, or holdout access.

## Provenance Boundary

The study used clean ignored checkouts only. No external source file enters the
Signal tree.

| Source file | Revision | SHA-256 |
| --- | --- | --- |
| Rubber Band `PhaseAdvance.h` | `1d95888bec3ae0a17c0c4af791810d5a63f6bc35` | `ee9c164d50cba827160480ff6eb2fa2d6c2dce30b7615e753a10e077b4032f73` |
| Rubber Band `Guide.h` | same | `fdcda0ec555c97c59c17dc730f4b4e72d747218ed2a11d115547390be1e44f58` |
| Rubber Band `R3Stretcher.cpp` | same | `99bae620129131cf0a74e85ad6ceaa5310a4b93cd98d2568a1d3a0e09c0efe4c` |
| Signalsmith `signalsmith-stretch.h` | `57b93f4e9206a089a45387eaa39bdc9f310d3308` | `1188667959ac19dd40c0a6abbce694e44705615ec4f0e8db8af0f1cfb4c5dea7` |
| Bungee `Stretcher.cpp` | `746833f68a574d997ec50443e7cfd2d37b026302` | `2ea181936b167fb5a3159ebc93f41a91788ec2bc2840aae04eb772c98a704881` |
| Bungee `Synthesis.cpp` | same | `d2d342775bc20c3ec5fdbd5b1b95473c2eeec0764bfe9df83dfe1762f2044065` |
| Bungee `Partials.cpp` | same | `666dc3adbff816aa0f42d5f050d5ff58cfed7a726642dfe7cfbbda3f7c5792c5` |

## Sources

- [Rubber Band R3 phase advance](https://github.com/breakfastquay/rubberband/blob/v4.0.0/src/finer/PhaseAdvance.h)
- [Rubber Band R3 guide](https://github.com/breakfastquay/rubberband/blob/v4.0.0/src/finer/Guide.h)
- [Rubber Band R3 stretcher](https://github.com/breakfastquay/rubberband/blob/v4.0.0/src/finer/R3Stretcher.cpp)
- [Rubber Band technical notes](https://breakfastquay.com/rubberband/technical.html)
- [Signalsmith Stretch source](https://github.com/Signalsmith-Audio/signalsmith-stretch/blob/57b93f4e9206a089a45387eaa39bdc9f310d3308/signalsmith-stretch.h)
- [Signalsmith Stretch design](https://signalsmith-audio.co.uk/writing/2023/stretch-design/)
- [Bungee source](https://github.com/bungee-audio-stretch/bungee/tree/746833f68a574d997ec50443e7cfd2d37b026302)
- [Dorran, Lawlor, and Coyle, *Multi-channel phase-vocoder processing using a phase-synchronization technique*](https://mural.maynoothuniversity.ie/8793/1/BL-Multi-channel-2005.pdf)

## Next Task

Batch 29.7AK validates the fixed `48 kHz` mechanics kernel, then closes its
whole-source integration. Batch 29.7AM validates memo 020's normalized
two-slice representation at hash `0407f765c7d84375`; Batch 29.7AN then passes
synchronized state mechanics at hash `90c10cd2e66d4faf`. Rule 31V now freezes
the unchanged material policy and complete failure-first evidence matrix.
Batch 29.7AO passes synthetic mechanics but rejects at `46/48` calibrated
stereo failures. Batch 29.7AP then locates the first operator break at the
interior `Unlocked` state commit in every row. Both active layers receive
exactly that residual; inverse and overlap are downstream. Rule 31X retains
the complete topology but replaces independent unlocked channel commits with
one greatest-energy reference rotation applied to every peer's current
coefficient. This is the relationship-preserving ordering shared by the
Signalsmith control and Bungee's common-rotation evidence, without importing
their expression or numeric policy. Batch 29.7AQ proves the isolated law but
rejects the complete topology. Mechanics and synthetic evidence pass at hash
`875b0768ba2066bf`; the single corrected stereo run records `40/48` calibrated
failures, `125/384` improved windows, `44/48` local-row failures, and hash
`88d9c0f68ea2954b`. The correction is locally effective but insufficient.
Memo 021 then corrects the source interpretation: ordinary and unlocked R3
recurrence is channel-local, while cross-channel borrowing is restricted to
compatible locked peak regions. It also closes the extra outer meta-slice as a
quality topology. Batch 29.7AR now restores the direct scale timeline on paper
under Rule 31Z. Run representation-only Batch 29.7AS next. Keep concealed
listening, the holdout, Batch 29.8, and product work closed.
