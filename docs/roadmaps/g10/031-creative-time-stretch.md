# 031 - Creative Time-Stretch

Status: active; Batch 31.28 complete, Batch 31.29 ready
Owner: dsp
Created: 2026-07-19
Depends on: g10.030 closure
Governing contract: `085`
Vision tags: `DSP`, `STRETCH`, `CREATIVE`, `QUALITY`

## Problem

Signal has a competitive transparent stretch baseline but no first-party path
for intentional long expansion. At `8x`, transparent event reconstruction is
not the only useful goal: controlled smear, evolving spectral motion, and
cloud-like output are valid product behavior.

Exposing unrelated algorithms and their native controls would push DSP policy
into every consumer. A hard ratio switch would create audible and UI seams.

## Goal

Build one offline creative-stretch product surface that:

- centers `8x`; the current neutral `Dream` lane admits at `4x`, `8x`, and
  `16x`
- presents stable intent controls rather than algorithm parameters
- preserves a future route from coherent slow motion to spectral dream and
  later cloud without claiming unavailable owners
- studies one source-backed neutral `Dream` owner before any new candidate
- preserves exact duration, determinism, linked stereo, and bounded memory
- stays separate from `OfflineHighQuality` and RealtimePreview

## Non-Goals

- no transparent successor reopening
- no RealtimePreview or audio-thread work
- no Loophole or Chorus UI implementation
- no external production dependency
- no spectral-router, non-`Dream` character, or cloud implementation while
  their owners are paused
- no `100x+` texture/freeze implementation in the first lane
- no simultaneous diffusive, cloud, and cyclic experiment queue

## Batch 31.1 - Product And Architecture Study

Status: complete

- [x] reframed the target from `800x` to `800%` / `8x`
- [x] separated creative quality from transparent Contract `084` admission
- [x] studied polyphase spectral, diffusive spectral, layered PV/granular,
  granular texture, cyclic sampler, image-resynthesis, STN, and learned systems
- [x] selected one stable product parameter vocabulary
- [x] froze coherent, diffusive, and cloud range ownership with logarithmic
  overlap bands
- [x] selected `DiffuseSpectral` as the first new candidate family
- [x] froze stereo, determinism, exact-length, memory, cache, and cleanup rules
- [x] changed documentation only

Authority:

- `docs/architecture/offline-creative-time-stretch-study.md`
- `docs/contracts/085-creative-time-stretch-product-and-routing-contract.md`

## Batch 31.2 - Comparator Target Freeze

Status: complete

- [x] capture the accessible primary references at `4x`, `8x`, and `16x`
- [x] use the retained percussion, bass, vocal, pad/sustain, and full-mix sources
- [x] level-match under one documented policy
- [x] record the operator's cross-matrix character, usefulness, and preference
  decision; stereo remains separately unassessed
- [x] resolve transition probes: external comparators cannot validate Signal's
  owner blends, so continuity probes remain mandatory at overlap admission
- [x] freeze a parameter space with distinct anchors rather than averaging
  incompatible winners
- [x] freeze explicit structural and listening rejection thresholds
- [x] keep the complete `DiffuseSpectral` brief after this target freeze

Required accessible references:

- PaulXStretch
- REAPER `Rrreeeaaa`
- CDP `SPECTSTR`

Optional supplementary references; absence does not block this batch:

- Sloom `Wide` and `Narrow`
- SoundHack `++spiralstretch`
- Ableton `Texture`

Secondary character controls:

- REAPER `ReaReaRea`
- Akaizer
- Photosounder or ARSS

Captured evidence:

- ignored workspace: `target/creative-stretch-comparator-31-2/`
- matrix: five retained mono sources by `4x`, `8x`, and `16x`
- primary captures: PaulXStretch 1.6.0 default / FFT `16384`, REAPER 7.69
  `Rrreeeaaa`, and pinned CDP 8.0 `SPECTSTR`
- secondary cyclic control: REAPER 7.69 `ReaReaRea`
- CDP profile: full channel decoherence (`d-ratio=1`) with moderate frequency
  randomization (`d-rand=0.5`); this is comparator evidence, not a Signal
  implementation constant
- CDP source feed: fixed `-18 dB` gain to avoid its legacy synthesis clipper;
  common pack normalization removes the capture offset
- exact-length policy: crop PaulXStretch and CDP synthesis extensions from the
  end; REAPER outputs already match the requested length
- concealment: deterministic A/B/C/D assignment recorded only in
  `listening-key.tsv`
- normalization: one common RMS per source/ratio group, bounded so every peak
  is at most `0.95`
- pack validation: 15 cases, 60 stereo float candidate files, `44.1 kHz`, exact
  target frames, finite samples, maximum inter-candidate RMS span below
  `1e-9`, maximum peak below `0.95`

Sloom's full product is paid and its demo cannot save. SoundHack
`++spiralstretch` is paid. Ableton is unavailable to the operator. They remain
useful later references but are not honest prerequisites for this no-purchase
target freeze.

Operator decision:

- `Dream` defaults to the smooth, musical PaulXStretch region across the
  complete `4x`/`8x`/`16x` matrix
- `Spectral` preserves the CDP-like vocoder/decoherence region as an intentional
  destination, never neutral colour
- `Rough` preserves the interesting but novelty-led `Rrreeeaaa` region as an
  intentional destination, never neutral colour
- `Cyclic` preserves the useful `ReaReaRea` / Akai-style region through about
  `8x` as an explicit later character
- `motion`, `detail`, and `space` refine these characters under one stable UI
  vocabulary; Signal may switch or blend owners internally

Reject a one-sound compromise. Neutral `Dream` must stay smooth and musical;
the other anchors must be recognizable, useful, level-stable, and free of
uncommanded clicks or seams. Commanded cyclic repetition and deliberate
spectral exposure are not classified as faults inside their own characters.

No DSP, candidate harness, fixture, report mode, or public API enters `main`.

## Batch 31.3 - Diffusive Candidate Brief

Status: complete

- [x] freeze one long-window STFT topology and fractional source map
- [x] freeze correlated phase diffusion, instantaneous-frequency carrier, and
  dormant/reactivation state
- [x] freeze magnitude evolution and exact `Dream`/`Spectral`/`Rough` laws
- [x] freeze symmetric linked-stereo analysis, relation, and `space` ownership
- [x] freeze boundaries, rolling normalization, exact length, determinism,
  `32 MiB` working-state cap, and computational shape
- [x] freeze structural, creative synthetic, mono, and independent stereo gates
- [x] freeze isolated file shape, minimal admission, rejection, and deletion
- [x] preserve unsupported `Cloud` and `Cyclic` owner seams
- [x] change documentation only

Authority:

- `docs/architecture/offline-creative-diffuse-spectral-brief.md`

## Batch 31.4 - Isolated Diffusive Candidate

Status: complete; candidate rejected at creative synthetic gate

One complete candidate was implemented on
`candidate/g10-031-diffuse-spectral` in the disposable
`signal-candidate-31-4` worktree.

- [x] implemented only the private six-file `creative_diffuse` family
- [x] preserved one fractional source map, linked carrier and phase field,
  deterministic diffusion, magnitude state, spectral `space`, and rolling OLA
- [x] kept production tiers, public APIs, cache, routing, `Cloud`, and `Cyclic`
  unchanged
- [x] passed the implemented structural controls for exact length, finiteness,
  normalization coverage, exact silence, repeatability, seed effect, bounded
  duration-independent state, invalid requests, and linked-stereo mechanics
- [x] opened creative synthetic admission only after structural controls passed
- [x] passed completed pitch, replica, and non-periodicity rows
- [x] stopped on neutral `Dream` at `4x`: deterministic-noise crest-factor
  growth was `7.08 dB`, above the frozen `6 dB` ceiling
- [x] produced no long-form or stereo-listening audio
- [x] deleted the rejected worktree, branch, module, tests, and generated state

The dominant cause is uncontrolled stochastic crest growth from the diffusive
spectral field. The frozen brief permits neither limiter nor crest-repair
stage. No candidate code entered `main`.

## Batch 31.5 - Diffusive Crest Ownership Reassessment

Status: complete

The rejected topology was reassessed at architecture level under the frozen
measurement law.

- [x] retained the `6 dB` crest-growth ceiling; PaulXStretch and `Rrreeeaaa`
  stay below it across all `15` retained musical rows
- [x] attributed the failure to independent per-bin random phase destroying
  bounded cross-bin waveform relationships
- [x] closed the independent-bin `DiffuseSpectral` mechanism without a scalar,
  window, distribution, or coefficient sweep
- [x] selected one output-synchronous bounded stochastic excitation whose full
  complex transform owns phase and realized magnitude together
- [x] froze the source map, transform, excitation, carrier, magnitude,
  character, linked-channel, boundary, memory, determinism, gate, cleanup, and
  minimal-admission laws in one complete replacement brief
- [x] changed documentation only; no candidate, harness, fixture, report mode,
  public API, or product route entered `main`

Authority:

- `docs/architecture/offline-creative-continuous-excitation-spectral-brief.md`

## Batch 31.6 - Isolated Continuous-Excitation Candidate

Status: complete; candidate rejected at structural gate

One complete candidate was implemented in the disposable
`signal-candidate-31-6` worktree.

- [x] added only the private six-file `creative_excitation` family and one
  private `lib.rs` module declaration
- [x] passed `12/13` structural controls, including full-complex excitation
  reconstruction, exact length, finiteness, silence, determinism, bounded
  state, no processing allocation, duplicate, swap, anti-phase, and `space`
- [x] stopped when common-polarity covariance differed by `0.0013287` against
  the frozen `1e-6` bound
- [x] confirmed channel swap was exact and attributed the remaining miss to
  polar per-bin relation reconstruction, not the shared orientation bit
- [x] did not open the prior crest row, remaining synthetic gates, or listening
- [x] deleted the candidate worktree, branch, module, tests, and build state

No candidate code entered `main`.

## Batch 31.7 - Linked-Relation Ownership Reassessment

Status: complete

The failed linked relation was reassessed at representation level.

- [x] closed polar native-relation reconstruction and wrapped angle subtraction
- [x] selected scaled direct complex products against the exact linked sum
- [x] froze a channel-symmetric unoriented axis for exact cancellation
- [x] joined incidental cancellation to the source-orientation state while
  preserving exact whole-source anti-phase behavior
- [x] froze signed-zero, silence, carrier-reference, DC, Nyquist, dormant,
  character, stereo, boundary, memory, determinism, and cleanup laws
- [x] made relation enumeration and the prior common-polarity failure the first
  two gates before the unopened crest row
- [x] froze one complete final-candidate brief without changing DSP

Authority:

- `docs/architecture/offline-creative-continuous-excitation-complex-relation-brief.md`

## Batch 31.8 - Final Isolated Diffusive-Owner Candidate

Status: complete; candidate rejected at relation proof

One complete candidate was implemented in the disposable
`signal-candidate-31-8` worktree.

- [x] added only the private six-file `creative_excitation_relation` family
  and one private `lib.rs` declaration
- [x] completed compile-only validation before gate admission
- [x] ran coefficient relation proof first
- [x] stopped on exact anti-phase enumeration: actual `-1+0i`, expected
  `+1-0i`
- [x] attributed the miss to a mutually incompatible proof expectation:
  exact anti-phase common polarity is itself channel swap, so componentwise
  negation equals swap and cannot also equal negated swap
- [x] did not correct or rerun the proof under the frozen stop rule
- [x] did not open the prior polarity renderer row, structural gate, crest
  gate, remaining synthetic gates, or listening
- [x] deleted the worktree, branch, private module, tests, and build state

No candidate code entered `main`. The final admitted candidate and the current
diffusive owner are closed.

## Batch 31.9 - Creative Range-Owner Reassessment

Status: complete

Reassessed the `4x` through `16x` ownership map after all admitted diffusive
candidates closed.

- [x] rejected `OfflineHighQuality` as a substitute core owner because it has
  no PaulX-centred evidence at the target ratios and does not span the frozen
  character space
- [x] rejected a smear layer or spectral wet stack because it would reopen the
  closed diffusive family under another boundary
- [x] found no granular cloud, image, STN, sinusoidal, or learned family with
  one source-backed path through core quality, linked stereo, exact length,
  deterministic state, and retained musical targets
- [x] paused `Dream`, `Spectral`, `Rough`, `Cloud`, both overlap bands, and the
  automatic router
- [x] selected a narrower explicit `Cyclic` promise above `1x` through `8x`
- [x] retained ReaReaRea as the behavioral target and pinned public Potenza
  revision `ddb44a8f949b3f49320932e1d2e997b3a02149bb` as GPL architecture
  evidence only
- [x] kept Akaizer as optional paid behavioral context, not source backing
- [x] changed documentation only

Do not repair the Batch 31.8 proof, reopen continuous excitation, start a new
diffusion/window/coefficient/scalar variant, or implement `Cloud`, `Cyclic`,
routing, cache, public APIs, or product integration in this batch.

## Batch 31.10 - Cyclic Owner Brief

Status: complete; docs and architecture only

Froze one complete clean-room Signal-owned `CyclicGrain` renderer for fixed
expansion above `1x` through `8x`:

- [x] froze one sample-centred monotonic map and exact target-length crop
- [x] froze one deterministic lattice with at most two overlapping unit-rate
  source reads and normalized raised-cosine crossfades
- [x] made unit-rate reads own pitch and mapped anchor advance own duration
- [x] froze channel-shared positions, normalization, and seed phase plus a
  bounded mid/side `space` law
- [x] mapped `detail` to logarithmic cycle support and `motion` to launch
  density without exposing grain controls
- [x] capped duration-independent state at `8 MiB` and cost at `O(C*T)`
- [x] froze structural and synthetic gates at identity, `2x`, `4x`, and `8x`
- [x] froze the five-source mono pack at `2x`, `4x`, and `8x`, including a
  required missing `2x` ReaReaRea capture under the retained level policy
- [x] froze exact `16x` request rejection before allocation
- [x] froze independent stereo listening, whole-candidate rejection, cleanup,
  and minimal private admission
- [x] changed documentation only

Use public Potenza source only for clean-room architecture. Do not copy GPL
expression, constants, thresholds, or control flow. Do not implement the
candidate in this batch.

Authority:

- `docs/architecture/offline-creative-cyclic-grain-brief.md`

## Batch 31.11 - Isolated Cyclic Candidate

Status: complete; candidate rejected at creative synthetic gate

One complete candidate was implemented on
`candidate/g10-031-cyclic-grain` in the disposable
`signal-candidate-31-11` worktree.

- [x] implemented only the private six-file `creative_cyclic` family and one
  private `lib.rs` declaration
- [x] preserved the sample-centred map, unit-rate two-grain reads, normalized
  crossfade, linked scheduling, semantic macros, exact length, and bounded
  rolling state
- [x] passed all seven structural tests, including identity, length,
  finiteness, silence, determinism, mapping, scheduled replicas, stereo
  covariance, peak bounds, and duration-independent capacity
- [x] opened creative synthetic admission only after structural controls passed
- [x] stopped on the first neutral row: `110 Hz` at `2x` measured
  `111.328 Hz`, or `20.778` cents against the frozen `15`-cent ceiling
- [x] did not run later synthetic rows, capture the missing `2x` comparator,
  render long-form audio, open the `16x` probe, or begin stereo listening
- [x] deleted the worktree, branch, private module, tests, and build state

The dominant cause is pitch displacement from crossfading source-offset
unit-rate grains. No candidate code entered `main`.

## Batch 31.12 - Cyclic Ownership Reassessment

Status: complete; docs and architecture only

Reassess whether explicit `Cyclic` still has one complete Signal-owned path
through the frozen pitch, integrity, mono, and linked-stereo gates.

- [x] did not tune or reimplement `CyclicGrain`
- [x] did not sweep grain length, hop, window, interpolation, seed, threshold,
  or test tones
- [x] reconciled the earlier transparent WSOLA closure with the narrower
  intentional cyclic target
- [x] selected `SimilarityAlignedCyclic`: one output lattice, one ideal map,
  one bounded correlation-selected source path, and native linked-channel
  synthesis
- [x] required strict monotonic selected anchors and non-accumulating
  ideal-map displacement before a candidate may exist
- [x] rejected pitch-synchronous OLA, fixed repetition, unaligned grains, and
  spectral correction as alternatives
- [x] kept `Dream`, `Spectral`, `Rough`, `Cloud`, automatic routing, cache,
  public APIs, and product integration closed
- [x] changed documentation only

The family is materially different from rejected `CyclicGrain`: waveform
similarity chooses segment placement before synthesis instead of crossfading
fixed source offsets. It is source-backed by the original WSOLA work and a
maintained SoundTouch implementation shape. It remains unproven at Signal's
ratios, polyphonic pack, and linked-stereo gate.

## Batch 31.13 - Similarity-Aligned Cyclic Brief

Status: complete; docs and architecture only

Freeze one complete clean-room `SimilarityAlignedCyclic` renderer before any
candidate implementation.

- [x] froze the ideal map, realized strictly monotonic source path, output launch
  lattice, bounded non-accumulating search domain, score, and deterministic
  ties
- [x] froze silence and low-confidence fallback without adding a second
  timeline
- [x] froze segment, overlap, window, normalization, boundaries, exact length,
  anti-replica behavior, memory bound, determinism, and computational shape
- [x] froze one linked-channel score and native-channel synthesis law
- [x] mapped `motion`, `detail`, `space`, and seed without exposing algorithm
  controls
- [x] retained structural, synthetic, mono, `16x` rejection, and independent
  stereo gate order
- [x] froze whole-candidate rejection, deletion, and minimal private admission
- [x] did not implement DSP, add harness surfaces, capture comparator audio, or
  reopen any other creative owner or product surface

Authority:

- `docs/architecture/offline-creative-similarity-aligned-cyclic-brief.md`

## Batch 31.14 - Isolated Similarity-Aligned Cyclic Candidate

Status: complete; candidate rejected at structural gate

One complete candidate was implemented on
`candidate/g10-031-similarity-aligned-cyclic` in the disposable
`signal-candidate-31-14` worktree.

- [x] added only the private six-file `creative_similarity_cyclic` family and one
  private `lib.rs` declaration
- [x] passed compile-only validation before admission
- [x] ran the complete seven-test structural gate once
- [x] passed six structural cases covering request and identity, length and
  determinism, geometry and strict path, impulse support, linked stereo, and
  peak/capacity bounds
- [x] stopped when the known-offset recovery case selected source frame `6432`
  instead of the exact natural continuation at `6352`
- [x] attributed the miss to coarse-shortlist reachability: an exact match
  between coarse samples is unavailable to the full search when neighbouring
  coarse correlations do not reach the top three
- [x] did not correct or rerun the gate, run the `110 Hz` row, open later
  synthetic or listening gates, or capture comparator audio
- [x] deleted the worktree, branch, private module, tests, and build state

This is a different dominant cause from the Batch 31.11 pitch displacement.
Contracted policy returns explicit `Cyclic` to docs-level ownership
reassessment; it does not authorize direct implementation. No candidate code
entered `main`.

## Batch 31.15 - Cyclic Ownership Reassessment

Status: complete; explicit cyclic closed without promotion

Reassess whether explicit `Cyclic` has one remaining materially different,
source-backed whole-renderer path or must close.

- [x] reconciled the Batch 31.11 pitch failure and Batch 31.14 search-reachability
  failure without repairing either rejected candidate
- [x] distinguished a genuinely different ownership topology from a wider search,
  denser coarse grid, larger shortlist, score variant, or threshold change
- [x] required one source-backed path through pitch, scheduled-replica, exact-length,
  bounded-state, linked-stereo, and retained musical gates
- [x] rejected SOLA/WSOLA variants as direct alignment repairs; rejected
  pitch-/epoch-synchronous OLA because it lacks one full-mix linked period
  owner and retained `8x` evidence
- [x] rejected fixed grains, transient/component hybrids, spectral/sinusoidal
  reopening, and learned synthesis as repaired, closed, or separate programs
- [x] found no third materially different complete path and closed explicit
  `Cyclic`
- [x] kept all implementation, candidate harness, comparator capture, public API,
  cache, routing, other creative owners, Loophole, and Chorus out of scope
- [x] changed documentation only

`Cyclic` remains useful comparator and future intent vocabulary, not an
available character. At the Batch 31.15 close, no new brief or implementation
batch was ready.

## Batch 31.16 - Creative Source Triangulation

Status: complete; docs and research only

Reopen one bounded research batch by explicit operator decision.

- [x] pinned PaulXStretch `v1.6.0`, CDP `CDP8.0`, and the retained Potenza
  revision
- [x] traced each complete source clock, representation, expansion mechanism,
  synthesis path, stereo ownership, and output boundary
- [x] connected PaulXStretch and CDP source behavior to the retained `4x`,
  `8x`, and `16x` comparator pack
- [x] confirmed neutral PaulXStretch uses magnitude-only frame renewal and
  output crossfade, not the recurrence and magnitude evolution frozen in the
  rejected Signal spectral briefs
- [x] retained CDP amplitude/frequency-frame interpolation as a separate later
  `Spectral` owner
- [x] retained Potenza as cyclic architecture evidence while keeping both
  rejected Signal cyclic candidates closed
- [x] selected `RenewalSpectral` as one materially different, source-backed
  neutral `Dream` family
- [x] separated hard integrity gates from comparator-calibrated character
  diagnostics and listening authority
- [x] kept DSP, candidate scaffolding, comparator audio, public APIs, routing,
  cache, Loophole, and Chorus unchanged

Authority:

- `docs/research/specimen-dossiers/creative-stretch-source-triangulation.md`
- `docs/architecture/offline-creative-time-stretch-study.md`
- `docs/contracts/085-creative-time-stretch-product-and-routing-contract.md`

## Batch 31.17 - RenewalSpectral Complete Brief

Status: complete; docs and architecture only

Freeze one complete clean-room neutral `Dream` renderer before any candidate
implementation.

- [x] freeze transform support, output-frame cadence, and one exact source map
- [x] freeze deterministic per-frame phase renewal without coherent carrier,
  continuous excitation, magnitude slew, or transient logic
- [x] freeze linked-channel analysis, excitation, synthesis, and semantic
  `space` ownership without inheriting the rejected relation proof
- [x] freeze frame combination, crest ownership, normalization, exterior
  support, and exact target-length crop
- [x] freeze bounded memory, deterministic state, computational shape, and
  offline-only execution
- [x] calibrate every character metric against PaulXStretch or a named hard
  integrity boundary before freezing its pass/fail condition
- [x] retain `4x`, `8x`, and `16x` five-family concealed mono listening and
  independent stereo review
- [x] freeze whole-candidate rejection, cleanup, and minimal private admission
- [x] stop without DSP, harness, fixture, report mode, comparator capture,
  public API, routing, cache, Loophole, or Chorus changes

Authority:

- `docs/architecture/offline-creative-renewal-spectral-brief.md`
- `docs/contracts/085-creative-time-stretch-product-and-routing-contract.md`

## Batch 31.18 - Isolated RenewalSpectral Candidate

Status: complete; candidate rejected at first crest row

Implement the frozen brief once. Failure stops the sequence.

- [x] created `signal-candidate-31-18` on
  `candidate/g10-031-renewal-spectral`
- [x] added only the private `creative_renewal` family and private `lib.rs`
  declaration
- [x] passed compile-only validation before admission
- [x] ran the fixed structural gate once; all controls passed
- [x] ran the mandated first crest row and stopped at `8.263162 dB` growth
  against the frozen `6 dB` ceiling
- [x] did not open remaining synthetic, concealed mono, or independent stereo
  gates
- [x] admitted no candidate surface
- [x] recorded uncontrolled cross-bin summation after independent phase renewal
  as the dominant cause and deleted the candidate worktree,
  branch, module, tests, build state, and candidate listening assembly
- [x] kept public APIs, report modes, fixtures, cache, routing, other creative
  owners, Loophole, and Chorus unchanged

## Batch 31.19 - RenewalSpectral Crest Ownership Reassessment

Status: complete; target closure superseded by operator correction

- [x] reconcile the Batch 31.4 and Batch 31.18 crest failures without repairing
  either rejected candidate
- [x] determine whether any materially different, source-backed neutral-`Dream`
  whole-renderer path intrinsically owns crest without a limiter, post-gain,
  scalar sweep, or renamed phase/window variant
- [x] freeze one complete replacement direction only if it has a credible path
  through crest, linked stereo, exact length, bounded state, and retained
  musical targets; the batch recorded closure, later superseded by operator
  correction
- [x] keep DSP, candidate harnesses, comparator capture, public APIs, cache,
  routing, other creative owners, Loophole, and Chorus unchanged

Decision:

- both independent-phase candidates failed the same `6 dB` crest boundary:
  `DiffuseSpectral` at `7.08 dB`, then `RenewalSpectral` at `8.263162 dB`
- low-crest multisine and IAAFT methods do not provide a source-mapped,
  nonstationary, linked-stereo stretcher with bounded fixed cost
- STN noise morphing is a component path; Signal's complete bounded-excitation
  translation already failed linked-channel ownership
- no source-backed complete neutral-`Dream` renderer remains ready to freeze
- `Dream` stays as unavailable intent vocabulary; reopening requires explicit
  operator direction plus new complete-system evidence

Correction:

- the operator rejected abandonment of the PaulXStretch-like target
- PaulXStretch's `3.88 dB` maximum came from retained musical rows, while the
  Signal stop row was synthetic uniform noise
- the matching PaulX synthetic suite had not run when the candidate stopped
- `RenewalSpectral` substituted equal-power frame blending and fixed gain for
  the source-backed raised-cosine blend plus position-dependent compensation
- the candidate remains rejected; the neutral-`Dream` family closure does not

## Batch 31.20 - PaulX Reference And Gate Recovery

Status: complete; comparator and architecture only

- [x] render the frozen synthetic noise, harmonic-pad, impulse-train, tone,
  chord, impulse, and silence-gap inventory through pinned PaulXStretch 1.6.0
  at `4x`, `8x`, and `16x`
- [x] measure PaulXStretch and the rejected Signal result under the same active
  support, RMS matching, crop, and crest law
- [x] separate hard integrity limits from reference-relative character
  diagnostics; an unmatched synthetic metric cannot close the target
- [x] trace the whole frame-combination path, including raised-cosine blend,
  position-dependent amplitude-modulation compensation, source accumulation,
  window endpoint convention, exterior support, and exact crop
- [x] derive Signal's compensation law from overlap statistics without copying
  upstream constants, expressions, tables, thresholds, or control flow
- [x] freeze one complete clean-room successor brief covering map, transform,
  phase renewal, frame blend and compensation, linked stereo, exact length,
  bounded state, determinism, gates, listening, rejection, and cleanup
- [x] require long-form concealed mono listening for character authority after
  hard integrity; independent stereo remains a later promotion gate
- [x] keep candidate DSP, public APIs, report modes, fixtures, cache, routing,
  other creative owners, Loophole, and Chorus unchanged

Decision:

- pinned PaulX worst-channel uniform-noise crest growth is `9.932`, `11.899`,
  and `10.432 dB` at `4x`, `8x`, and `16x`
- the old `6 dB` ceiling was not PaulX-calibrated; rejected Signal's
  `8.263162 dB` row was below the matching PaulX `4x` result
- hard integrity remains absolute; creative synthetic rows are now compared
  with their matching reference and listening remains promotion authority
- `CompensatedRenewalSpectral` derives position compensation as
  `1/sqrt(a^2+b^2)` for complementary raised-cosine frame weights
- no candidate or production DSP entered `main`

Authority:

- `docs/architecture/offline-creative-compensated-renewal-spectral-brief.md`

## Batch 31.21 - Isolated Compensated Renewal Candidate

Status: complete; candidate rejected at compile-only validation

- [x] create `signal-candidate-31-21` on
  `candidate/g10-031-compensated-renewal`
- [x] implement only the private six-file
  `creative_compensated_renewal` family and one private `lib.rs` declaration
- [x] stop at compile-only validation after one structural-test accumulator
  lacked a concrete `Option` type; no renderer executed
- [x] leave structural and hard-integrity admission unopened
- [x] leave the full reference-relative synthetic matrix unopened
- [x] leave concealed long-form mono listening unopened
- [x] stop on the first miss; record one dominant cause and delete the complete
  candidate without correction or rerun
- [x] leave independent stereo blocked until an eligible listener is available
- [x] keep public APIs, report modes, fixtures, cache, routing, other creative
  owners, Loophole, and Chorus unchanged

Decision:

- the candidate failed to compile because a structural test declared an
  unconstrained `Option` accumulator
- no DSP row ran, so the compensated-renewal topology remains untested rather
  than acoustically or structurally rejected
- the failed implementation was not corrected or rerun
- the disposable worktree, branch, private module, tests, and build state are
  deleted; no candidate surface entered `main`

## Batch 31.22 - Fresh Compensated-Renewal Candidate Authority

Status: complete; docs and architecture only

- [x] retain the Batch 31.21 compile miss as terminal for that implementation
- [x] preserve the reference recovery and compensation derivation as valid,
  still-untested evidence
- [x] freeze one newly named, complete candidate brief for the same selected
  compensated-renewal topology, including an explicit compile-complete test
  surface and fresh worktree, branch, module, cleanup, and gate authority
- [x] resolve every implementation and validation type before marking the next
  candidate batch ready; leave no `decide later` gap
- [x] keep candidate DSP, harness modes, fixtures, public APIs, cache, routing,
  other creative owners, Loophole, and Chorus unchanged
- [x] stop after the fresh brief is validated, committed, and reported

Decision:

- `VarianceCompensatedRenewalSpectral` is the fresh candidate identity
- the DSP topology, source/output map, long transform, phase renewal,
  raised-cosine blend, variance compensation, stereo law, reference matrix,
  and listening authority remain unchanged
- construction may repair compiler-only plumbing before one clean
  compile-completion receipt; the resulting isolated checkpoint is immutable
  for structural and later gates
- no candidate or production DSP entered `main`

Authority:

- `docs/architecture/offline-creative-variance-compensated-renewal-spectral-brief.md`

## Batch 31.23 - Isolated Variance-Compensated Renewal Candidate

Status: complete; candidate rejected at evidence-integrity audit

- [x] create `signal-candidate-31-23` on
  `candidate/g10-031-variance-compensated-renewal`
- [x] implement only the private six-file
  `creative_variance_compensated_renewal` family and one private `lib.rs`
  declaration
- [x] complete construction and `effigy test compile`; allow only frozen
  compiler-plumbing repairs before the clean receipt
- [x] create and record one local compile-complete checkpoint commit; do not
  push it
- [x] run structural and hard-integrity admission once from that checkpoint
- [x] run the synthetic command only after structural admission; invalidate
  its green result when required assertions are found absent
- [x] assemble concealed long-form mono audio, discover the invalid evidence
  surface before listening, and delete the unopened pack
- [x] stop without test repair, tuning, or rerun
- [x] leave independent stereo blocked until an eligible listener is available
- [x] keep public APIs, report modes, fixtures, cache, routing, other creative
  owners, Loophole, and Chorus unchanged

Evidence:

- compile-complete checkpoint:
  `2548c27947b28a59a265cf1bb60ca2b03455b08a`
- structural execution: seven of seven tests passed
- synthetic execution: returned green but did not measure impulse-train crest,
  separated secondary regions, every autocorrelation lag, or the complete
  discontinuity condition
- dominant cause: incomplete frozen evidence construction
- DSP and listening result: unknown; no long-form row was heard
- cleanup: candidate worktree, branch, checkpoint, module, tests, build state,
  and listening assembly deleted

## Batch 31.24 - Evidence-Integrity Reassessment

Status: complete; fresh audited brief frozen

- [x] reconcile the Batch 31.23 invalid green receipt without treating it as
  candidate-quality evidence
- [x] audit every structural and synthetic Contract `085` condition into one
  explicit measurement, assertion owner, and exact execution stage
- [x] decide whether the still-untested compensated-renewal topology warrants
  one fresh complete candidate identity and brief
- [x] if retained, freeze the full executable gate surface before another
  candidate batch becomes ready; leave no sampled substitute for an exact gate
- [x] keep candidate DSP, long-form audio, public APIs, cache, routing,
  Loophole, and Chorus unchanged
- [x] stop after the docs decision is validated, committed, and reported

Decision:

- retain the topology because Batch 31.23 established no valid DSP or
  listening result and the pinned PaulX path remains source-backed
- use fresh identity `AuditedVarianceCompensatedRenewalSpectral`; do not
  recover deleted candidate source or tests
- freeze `22` compile-linked admission owners: `13` structural and `9`
  synthetic
- correct impulse width to shortest 95%-energy intervals, use the exact
  sample-centred event map, evaluate every autocorrelation lag, make replica
  absence explicit, and define discontinuity as PaulX-relative
  first-difference crest
- count actual working allocation, including FFT plans, and reject any
  allocation after frame processing starts
- no candidate or production DSP entered `main`

Authority:

- `docs/architecture/offline-creative-audited-variance-compensated-renewal-spectral-brief.md`

## Batch 31.25 - Isolated Audited Variance-Compensated Renewal Candidate

Status: complete; candidate rejected at linked-stereo image preservation

- [x] create `signal-candidate-31-25` on
  `candidate/g10-031-audited-variance-renewal`
- [x] implement only the fresh private six-file
  `creative_audited_variance_renewal` family and one private `lib.rs`
  declaration; do not recover deleted candidate source
- [x] complete `effigy test compile` and the one-test construction-manifest
  gate before checkpointing
- [x] create and record one local immutable checkpoint; do not push it
- [x] run exactly `13` structural owners once from that checkpoint
- [x] run exactly `9` full synthetic owners only after structural admission
- [x] assemble concealed long-form mono listening only after valid synthetic
  admission
- [x] stop on the first admission miss; delete the complete candidate without
  tuning, repair, or rerun
- [x] stop before independent stereo promotion review after operator speaker
  listening and objective balance evidence expose a source-image failure
- [x] keep public APIs, reports, fixtures, cache, routing, other characters,
  Loophole, and Chorus unchanged

Evidence:

- compile and construction: pass; exactly `1/1` construction owner
- immutable checkpoint: `97ee70569bc2a9dd574970eefb19799873875946`
- structural admission: exactly `13/13` owners passed
- synthetic admission: exactly `9/9` owners passed
- concealed pack:
  `signal-candidate-31-25/target/creative-stretch-audited-31-25/listening-pack/`
- pack validation: `15` rows, `30` concealed files, exact target lengths,
  finite float audio, maximum RMS span `2.22044604925e-16`, maximum peak
  `0.95`
- unopened key SHA-256:
  `7d6a91fc897327a887121df077b514d1da583c823c90ac1f3fcf7bc861c1c962`
- decoded mono decision: `15/15` ties, no unusable row, no family loss, no
  forbidden character; the only material distinction was exterior fade length
- fade diagnosis: pack assembly added no fade; the candidate's fixed
  `16384`-frame envelope is renderer-owned
- stereo source balance, right-minus-left: `-0.4516 dB`
- candidate `8x` balance at `space=0`, `0.5`, and `1`: `+4.2147 dB`,
  `+3.3660 dB`, and `+1.9453 dB`
- dominant cause: source mid/side phase was discarded and component-wide
  polarity came from first non-zero samples more than `141 dB` below peak
- stopped gate: linked-stereo image preservation; no independent promotion
  review and no production admission

Cleanup removed the disposable worktree, branch, private module, tests, build
state, and candidate listening assembly. No candidate code entered `main`.

## Batch 31.26 - Source-Relative Stereo Renewal Brief

Status: complete; `SourceRelativeRenewalSpectral` frozen

- [x] retain the passed source/output map, long magnitude analysis, phase
  renewal, adjacent-frame blend, variance compensation, boundaries, memory,
  determinism, mono synthetic gates, and mono listening pack
- [x] replace first-sample mid/side orientation with one explicit
  source-relative interchannel relationship representation
- [x] freeze how neutral `space`, increasing `space`, DC, Nyquist, silence,
  dormancy, channel swap, common polarity, anti-phase, and duplicate stereo
  compose under that representation
- [x] freeze source-relative channel-balance, centre, width, low-band image,
  and time-local balance gates before implementation
- [x] preserve one complete candidate, one immutable checkpoint, terminal gate
  order, cleanup, and minimal private admission
- [x] change documentation only; do not recover Batch 31.25 code or add DSP,
  harness, fixture, API, route, cache, Loophole, or Chorus surfaces to `main`

The brief must describe one buildable renderer. A first-sample threshold,
post-render channel gain, scalar blend sweep, or repair of checkpoint
`97ee7056` is not a successor.

Authority:

- `docs/architecture/offline-creative-source-relative-renewal-spectral-brief.md`

## Batch 31.27 - Isolated Source-Relative Renewal Candidate

Status: complete; candidate rejected at structural exact-vector proof

- [x] create `signal-candidate-31-27` on
  `candidate/g10-031-source-relative-renewal`
- [x] implement only the fresh private six-file
  `creative_source_relative_renewal` family and one private `lib.rs`
  declaration; do not recover Batch 31.25 source
- [x] complete `effigy test compile` and exactly `1/1` construction owner
- [x] create and record one local immutable checkpoint; do not push it
- [x] run exactly `15` structural owners once from that checkpoint; `14`
  passed and one frozen exact-vector assertion failed
- [ ] run exactly `9/9` full synthetic owners only after structural admission
- [ ] repeat the concealed `15`-row mono pack only after objective admission
- [ ] capture same-source PaulX stereo references, then run objective stereo,
  operator speaker pre-screen, and eligible independent listening in order
- [x] stop on the first miss and delete the complete candidate without tuning,
  repair, or rerun
- [x] keep public APIs, reports, fixtures, cache, routing, other characters,
  Loophole, and Chorus unchanged

Evidence:

- compile: pass
- construction: exactly `1/1` passed
- immutable checkpoint: `1f05cc33dcc57b5714f02bf71f05a44d4ff98f09`
- structural admission: exactly `15` selected; `14` passed, `S04` failed
- actual normative `mix64(1)` result: `0x5692161d100b05e5`
- frozen assertion: `0x569216d1009b05e5`
- dominant cause: the assertion transposed the middle `1d10` into `d100`
- stopped gate: structural mono-renewal exact-vector proof
- synthetic, mono-listening, and stereo gates: not run
- cleanup: disposable worktree, branch, checkpoint, module, tests, build state,
  and candidate artifacts deleted; no DSP entered `main`

## Batch 31.28 - Exact-Vector Evidence Reassessment

Status: complete; verified fresh authority frozen

- [x] derive the `mix64(1)` vector independently from the normative wrapping
  expression and record one authoritative value
- [x] audit every frozen literal exact-vector assertion against its owning
  formula before another implementation is authorized
- [x] classify the Batch 31.27 result as evidence-construction failure without
  claiming a DSP, synthetic, mono, or stereo outcome
- [x] either freeze one fresh complete source-relative candidate brief under a
  new worktree, branch, module, prefix, and checkpoint identity or close the
  topology
- [x] preserve terminal construction, structural, synthetic, listening,
  cleanup, and minimal-admission order
- [x] change documentation only; do not recover Batch 31.27 source or add DSP,
  tests, harness, fixture, API, route, cache, Loophole, or Chorus surfaces

Evidence:

- independent implementations: Python integer arithmetic and Ruby integer
  arithmetic
- audited tags: `RNWFRAME`, `RNWBIN00`, `RNWBASE0`, `RNWTEST0`
- audited controls: `mix64(0)`, both `mix64(1)` rounds and finalizer,
  `mix64(u64::MAX)`
- audited address: seed `0x0123456789abcdef`, frame `7`, bin `11`, base stream;
  both hashes, rotation, outer input, final address, and high-53 numerator
- exact-literal inventory: the rejected candidate contained only the failed
  handwritten `mix64(1)` vector
- decision: retain the architecture under fresh verified identity

Authority:

- `docs/architecture/offline-creative-verified-source-relative-renewal-spectral-brief.md`

## Batch 31.29 - Isolated Verified Source-Relative Candidate

Status: ready; isolated implementation only

- [ ] create `signal-candidate-31-29` on
  `candidate/g10-031-verified-source-relative-renewal`
- [ ] implement only the fresh private six-file
  `creative_verified_source_relative_renewal` family and one private `lib.rs`
  declaration; do not recover Batch 31.27 source
- [ ] define the audited counter literals once in `COUNTER_VECTORS`; prohibit
  duplicate handwritten counter values
- [ ] complete `effigy test compile` and exactly `1/1` construction owner
- [ ] create and record one immutable local checkpoint; do not push it
- [ ] run exactly `15/15` structural owners once from that checkpoint
- [ ] run exactly `9/9` synthetic owners only after structural admission
- [ ] repeat concealed mono and same-source stereo admission only after all
  objective owners pass
- [ ] stop on the first miss and delete the complete candidate without tuning,
  repair, or rerun
- [ ] keep public APIs, reports, fixtures, cache, routing, other characters,
  Loophole, and Chorus unchanged

## Later Batches

Closed or paused without promotion. Work after Batch 31.28 requires a valid
candidate-admission decision:

- minimal production admission
- coherent/diffusive overlap
- `LayeredCloud` study and candidate
- diffusive/cloud overlap
- dynamic-ratio state continuity
- cache and product-path review
- `100x+` texture/freeze range

## Completion Gate

- [x] one product architecture and governing contract exist
- [x] comparator target character is frozen
- [x] three complete candidate briefs reached recorded rejection decisions
- [x] all admitted diffusive-owner candidates reached a recorded terminal
  decision without entering `main`
- [x] the automatic router was paused before direct owner work continued
- [x] one complete cyclic owner brief is frozen
- [x] one isolated cyclic candidate reached a recorded terminal decision
- [x] one materially different source-backed cyclic family is selected
- [x] one complete similarity-aligned cyclic brief is frozen
- [x] one isolated similarity-aligned cyclic candidate reached a recorded
  terminal decision
- [x] final ownership reassessment found no third complete cyclic path
- [x] explicit `Cyclic` closed without implementation or product exposure
- [x] pinned creative source triangulation selected one materially different
  neutral `Dream` family
- [x] one complete `RenewalSpectral` brief is frozen
- [x] one isolated `RenewalSpectral` candidate reached a terminal decision;
  structural admission passed and the first crest row failed
- [x] matching PaulXStretch synthetic reference and gate recovery complete
- [x] one complete clean-room frame-blend-compensated brief frozen
- [x] one isolated compensated-renewal implementation reached a terminal
  compile-only decision without executing DSP
- [x] one fresh complete variance-compensated-renewal brief frozen
- [x] one isolated variance-compensated-renewal implementation reached a
  terminal evidence-integrity rejection without entering `main`
- [x] one fresh audited variance-compensated-renewal brief frozen with complete
  executable gate ownership
- [x] long-form mono `Dream` listening reaches a promotion decision
- [x] linked-stereo evidence reaches a terminal rejection decision
- [x] one complete source-relative stereo successor brief is frozen
- [x] one isolated source-relative candidate reached a terminal structural
  evidence decision without entering `main`
- [x] one verified fresh source-relative candidate brief is frozen
- [x] no private renderer or product surface entered `main`

## Next Task

Run Batch 31.29 only. Create `signal-candidate-31-29` on
`candidate/g10-031-verified-source-relative-renewal`, implement the verified
brief fresh, complete construction `1/1`, freeze one checkpoint, and run gates
in order. Stop on the first miss; do not push.
