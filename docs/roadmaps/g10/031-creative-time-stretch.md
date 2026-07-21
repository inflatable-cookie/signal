# 031 - Creative Time-Stretch

Status: active; Batch 31.45 rejected at construction, reassessment next
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

Status: complete; candidate rejected and deleted

- [x] created `signal-candidate-31-29` on
  `candidate/g10-031-verified-source-relative-renewal`
- [x] implemented only the fresh private six-file
  `creative_verified_source_relative_renewal` family and one private `lib.rs`
  declaration without recovering Batch 31.27 source
- [x] defined the audited counter literals once in `COUNTER_VECTORS`; prohibited
  duplicate handwritten counter values
- [x] completed `effigy test compile` and exactly `1/1` construction owner
- [x] froze checkpoint `d94612dd9f4ca9ba51724c826cac1d9375c27ff8`
  without pushing it
- [x] passed exactly `15/15` structural owners once from that checkpoint
- [x] ran all nine synthetic owners once; seven passed, while `Y04` failed one
  `16x` replica row and `Y02` failed two `4x` pitch rows
- [x] stopped before concealed mono and stereo admission because objective
  admission failed
- [x] deleted the complete candidate without tuning,
  repair, or rerun
- [x] kept public APIs, reports, fixtures, cache, routing, other characters,
  Loophole, and Chorus unchanged

Evidence:

- compile: pass
- construction: `1/1`
- structural: `15/15`
- synthetic: `7/9`; `Y04` and `Y02` failed
- listening: not run
- cleanup: complete; no candidate code entered `main`

## Batch 31.30 - Ratio-Range Ownership Reassessment

Status: complete; evidence authority corrected

- [x] reconciled the paired `4x` pitch and `16x` replica failures with the
  pinned PaulXStretch render path and Batch 31.25's passed mono evidence
- [x] found no source-backed range switch: pinned PaulX uses one renewal path,
  while the Signal briefs left candidate seed unfrozen
- [x] withdrew the fixed-resolution range diagnosis without repairing or
  rerunning the rejected Batch 31.29 checkpoint
- [x] froze `SeedAuditedSourceRelativeRenewalSpectral` as one fresh complete
  authority with exact `ADMISSION_SEED` ownership
- [x] kept candidate DSP, tests, harnesses, fixtures, APIs, routing, cache,
  Loophole, and Chorus unchanged
- [x] kept the PaulX-like product target active

Evidence:

- Batch 31.25 and Batch 31.29 own the same normative mono DSP, sources, and
  metrics but used no common frozen candidate seed
- Batch 31.29's synthetic helpers selected seed `17`; Batch 31.25's passing
  receipt did not record its seed
- pinned PaulX uses one buffer geometry, fractional source accumulator,
  magnitude renewal, and adjacent-frame blend across all retained ratios
- a range-aware replacement is therefore not supported by the current source
  or Signal evidence

## Batch 31.31 - Isolated Seed-Audited Source-Relative Candidate

Status: complete; candidate rejected and deleted

- [x] created `signal-candidate-31-31` on
  `candidate/g10-031-seed-audited-source-relative-renewal`
- [x] implemented only the fresh private six-file
  `creative_seed_audited_source_relative_renewal` family and one private
  `lib.rs` declaration; do not recover Batch 31.27 or Batch 31.29 source
- [x] used the address vector's named seed field as the sole `ADMISSION_SEED`
  for every synthetic and listening candidate render
- [x] completed `effigy test compile` and exactly `1/1` construction owner
- [x] froze checkpoint `790119b7936d5166ffb814f9401ba1398d2d5db9`
  without pushing it
- [x] passed exactly `15/15` structural owners once
- [x] selected all nine synthetic owners once; six passed before `Y02` failed
  the `8x` chord pitch row and the runner cancelled `Y08` and `Y09`
- [x] kept mono and stereo listening closed after objective rejection
- [x] stopped on the first miss and deleted the complete candidate without tuning,
  repair, or rerun
- [x] kept public APIs, reports, fixtures, cache, routing, seed/reroll product
  exposure, other characters, Loophole, and Chorus unchanged

Evidence:

- compile: pass
- construction: `1/1`
- checkpoint: `790119b7936d5166ffb814f9401ba1398d2d5db9`
- structural: `15/15`
- synthetic: nine selected; six passed, `Y02` failed, `Y08` and `Y09`
  cancelled
- failed row: `8x` chord maximum partial error `13.351828347` cents against
  an `11.331375778`-cent ceiling
- `Y04`: pass for both impulse sources at all ratios
- listening: not run
- cleanup: complete; candidate worktree, branch, checkpoint, module, tests,
  build state, and artifacts deleted

## Batch 31.32 - Renewal Tonal-Coherence Architecture Reassessment

Status: complete; renewal family closed without promotion

- [x] reconcile Batch 31.29's `4x` tone failures and Batch 31.31's `8x` chord
  failure as one repeated tonal-coherence class
- [x] study whether pinned source or retained research supports one materially
  different whole-renderer owner with intrinsic tonal coherence
- [x] close renewal after finding no eligible complete source-backed
  replacement; do not close the PaulX-like product target
- [x] reject another seed, transform, phase, hop, window, threshold, or scalar
  variant as renewal repair
- [x] keep candidate DSP, tests, harnesses, fixtures, APIs, routing, cache,
  product exposure, Loophole, and Chorus unchanged

Decision:

- magnitude-only renewal owns tonal pitch statistically, not through persistent
  phase or oscillator state; the `4x` and `8x` failures repeat that missing
  ownership across seeds and material
- Signalsmith's high-ratio randomization is not a complete extreme-stretch
  owner; Bungee, Rubber Band, SBSMS, and component hybrids reopen rejected
  families or retain failed whole-renderer evidence
- TSM-Net exposes pretrained inference, but no released training path, usable
  repository licence, intrinsic linked pitch law, or first-party bounded-state
  route
- renewal is closed; `Dream` remains active comparator-backed product intent
- no successor brief or candidate implementation is authorized

## Batch 31.33 - Listening-Led Renewal Gate Correction

Status: complete; docs and architecture only

- [x] record the operator's explicit Contract `085` boundary change
- [x] retain hard integrity, replica, level, discontinuity, dropout, boundary,
  deterministic-state, and linked-stereo gates as terminal
- [x] change `Y02` from PaulX-error-plus-`2`-cent rejection to a mandatory
  complete diagnostic matrix
- [x] keep missing or non-finite pitch evidence terminal while letting
  concealed listening judge finite tonal deviation
- [x] freeze `ListeningLedSourceRelativeRenewalSpectral` under a fresh
  worktree, branch, module, prefix, and checkpoint identity
- [x] keep both prior checkpoints rejected and deleted
- [x] keep candidate DSP, tests, harnesses, fixtures, APIs, routing, cache,
  product exposure, Loophole, and Chorus unchanged

Decision:

- Batch 31.25's `15/15` concealed mono ties and the operator's solid stereo
  assessment outweigh a finite unheard comparator overrun as creative quality
  evidence
- the source-relative balance defect remains a terminal stereo boundary and
  the native-channel successor law remains the frozen correction
- no DSP or other threshold changed; one fresh implementation is authorized

Authority:

- `docs/architecture/offline-creative-verified-source-relative-renewal-spectral-brief.md`

## Batch 31.34 - Isolated Listening-Led Source-Relative Candidate

Status: complete; rejected at synthetic `Y08`

- [x] create `signal-candidate-31-34` on
  `candidate/g10-031-listening-led-source-relative-renewal`
- [x] implement only the fresh private six-file
  `creative_listening_led_source_relative_renewal` family and one private
  `lib.rs` declaration; do not recover rejected candidate source
- [x] complete `effigy test compile` and exactly `1/1` construction owner
- [x] freeze local checkpoint `f76d5bb7`; it was not pushed
- [x] run exactly `15/15` structural owners once
- [x] run all nine synthetic owners once; `Y02` emitted and passed its complete
  matrix of candidate, PaulX, and delta values without a comparator ceiling
  assertion
- [x] stop before listening after `Y08` found an exact-zero `H` block in the
  impulse row at `4x`, `8x`, and `16x`
- [x] record synthetic `8/9`; listening did not open
- [x] delete the complete candidate without
  tuning, repair, or rerun
- [x] keep public APIs, reports, fixtures, cache, routing, other characters,
  Loophole, and Chorus unchanged

Decision:

- the checkpoint is rejected under its frozen `Y08` assertion
- its executable dropout range used the complete impulse output, while the
  brief reserves complete output for impulse first-difference crest and names
  mapped non-zero support for dropout
- Batch 31.25 passed `Y08` under the otherwise matching mono topology, so the
  receipt cannot yet distinguish renderer loss from over-broad gate assembly
- no candidate code entered `main`

## Batch 31.35 - Impulse Support Evidence Reconciliation

Status: complete; fresh candidate authority frozen

- [x] reconcile the audited brief's complete-output impulse discontinuity
  range with its mapped-non-zero-support dropout rule
- [x] compare the Batch 31.25 passed `Y08` receipt and Batch 31.34 failure
  without recovering either rejected implementation
- [x] freeze one exact definition of impulse mapped non-zero support and show
  how every ratio derives it from the source/output map
- [x] classify Batch 31.34 as renderer-support failure or executable-evidence
  construction failure
- [x] if the renderer failed, close or reassess the complete topology; if the
  gate was over-broad, freeze a new complete candidate identity without DSP
- [x] keep thresholds, sources, renderer formulas, product surfaces, routing,
  Loophole, and Chorus unchanged

Decision:

- `Y08` first-difference crest and dropout own separate ranges
- impulse discontinuity still scans complete output
- dropout scans only complete `H` windows wholly inside mapped authored
  support; the isolated impulse maps to `4`, `8`, or `16` frames and therefore
  has no eligible dropout window
- Batch 31.34 over-broadened the executable dropout range; its receipt is an
  evidence-construction failure, not renderer-support evidence
- checkpoint `f76d5bb7` remains rejected and deleted; no receipt is reinterpreted
- one fresh `SupportAuditedListeningLedSourceRelativeRenewalSpectral` authority
  is frozen with a named support table and unchanged renderer, sources,
  thresholds, seed, admission order, and listening packs

Authority:

- `docs/architecture/offline-creative-verified-source-relative-renewal-spectral-brief.md`
- `docs/contracts/085-creative-time-stretch-product-and-routing-contract.md`

## Batch 31.36 - Isolated Support-Audited Listening-Led Candidate

Status: complete; rejected at source-relative stereo admission

- [x] create `signal-candidate-31-36` on
  `candidate/g10-031-support-audited-listening-led-renewal`
- [x] implement only the fresh private six-file
  `creative_support_audited_listening_led_renewal` family and one private
  `lib.rs` declaration; recover no rejected implementation or test source
- [x] use the named `SYNTHETIC_SUPPORTS` table as the sole dropout-support
  authority and prove it in the construction owner
- [x] complete `effigy test compile` and exactly `1/1` construction owner
- [x] freeze local checkpoint `5d8eaf45` without pushing it
- [x] run exactly `15/15` structural owners once
- [x] run all nine synthetic owners once under `ADMISSION_SEED`; require the
  complete listening-led `Y02` diagnostic and support-audited `Y08`
- [x] open retained concealed mono and stereo admission only after objective
  admission passes
- [x] stop at the first terminal miss; delete the complete candidate on
  rejection without tuning, repair, or rerun
- [x] keep public APIs, reports, fixtures, cache, routing, other characters,
  Loophole, and Chorus unchanged

Decision:

- compile, construction `1/1`, structural `15/15`, and synthetic `9/9` passed
  from the immutable checkpoint
- concealed mono passed as `15/15` ties; minor extra low-end noise and a
  gentler entry/more abrupt ending remain audible risks
- the first duplicated-mono stereo assembly was audited out as non-evidence
- valid same-source stereo admission rejected `16x` bass at about `2.00 dB`
  mapped-window error and `16x` full mix at `9.37..9.42 dB` with local channel-
  dominance reversal
- whole-render and band balance remained close; local image stability is the
  dominant failure
- speaker and independent stereo listening did not open
- worktree, branch, checkpoint reference, source, tests, build state, and
  listening assembly were deleted; no candidate code entered `main`

## Batch 31.37 - Renewal Stereo-Ownership Reassessment

Status: complete; renewal closed, product target retained

- [x] reconcile Batch 31.25's global source-balance inversion and Batch
  31.36's local mapped-window reversal as repeated linked-stereo failure
- [x] identify the exact ownership missing from the renewal topology rather
  than proposing a channel gain, threshold, coefficient, phase, or `space`
  adjustment
- [x] inspect retained complete-source evidence for one materially different
  linked-stereo owner that preserves the passed mono character and hard gates
- [x] either freeze one complete source-backed successor direction or close
  renewal without closing the PaulX-like product target
- [x] keep candidate DSP, tests, harnesses, fixtures, APIs, routing, cache,
  Loophole, and Chorus unchanged

Decision:

- the native-channel law already performs a common current-frame rotation at
  `space=0`; exact coefficient relation does not survive independent frame
  renewal and waveform blending as stable local image
- Bungee, Signalsmith, and Rubber Band add temporal phase or peak state and are
  different coherent families; SBSMS source feasibility already failed
- PaulX's independent per-channel renewal defines target character but not the
  current hard source-relative stereo invariant
- post-hoc gain, covariance, consistency, smoothing, phase, and `space`
  variants are unsupported repair paths
- renewal closes without promotion; PaulX-like `Dream` remains product intent
- no successor brief, candidate worktree, or implementation batch opens
- operator intent must decide whether local source-relative stereo stays
  terminal or becomes diagnostic beneath comparator-relative independent
  listening

## Batch 31.38 - Comparator-Relative Stereo Policy And Fresh Brief

Status: complete; docs and architecture only

- [x] record the operator decision that local mapped-window source-relative
  balance and dominance are creative diagnostics rather than numeric terminal
  gates
- [x] retain hard structural stereo relationships, whole-render and three-band
  balance, `space` consistency, exact length, determinism, and bounded state
- [x] make eligible independent comparator-relative stereo listening terminal
  and keep the operator speaker pass reject-only
- [x] preserve the Batch 31.36 local-image failure as diagnostic evidence
  without reviving or reinterpreting its deleted checkpoint
- [x] freeze one fresh complete `ComparatorAuditedRenewalSpectral` brief with
  exact renderer, gate, isolation, cleanup, and minimal-admission ownership
- [x] keep candidate DSP, tests, harnesses, fixtures, APIs, routing, cache,
  Loophole, and Chorus unchanged

Decision:

- the source-relative local-window invariant did not describe PaulX's
  independent-channel target and is no longer a neutral-`Dream` product veto
- the existing linked native-channel renewal law remains the selected renderer;
  no stereo repair, temporal recurrence, channel gain, or new DSP is added
- mapped-window Signal-source and PaulX-source rows must be complete and finite
  but are judged through eligible independent listening
- whole-render and three-band balance retain their frozen hard thresholds
- low-frequency noise and opposite entry/tail energy weighting remain explicit
  mono and stereo scorecard risks
- the new brief starts from fresh source under a new identity; rejected code
  and checkpoints remain deleted

## Batch 31.39 - Isolated Comparator-Audited Renewal Candidate

Status: complete; candidate rejected and deleted

- [x] created only `signal-candidate-31-39` on
  `candidate/g10-031-comparator-audited-renewal`
- [x] implemented the frozen six-file private renderer and `24`-owner gate table
  from fresh source without recovering a rejected checkpoint
- [x] ran `effigy test compile`, required construction `1/1`, then froze one
  immutable local checkpoint
- [x] passed structural `15/15`, then ran all nine synthetic owners once;
  seven passed and `Y04` plus `Y09` failed
- [ ] after objective admission, run the full concealed mono `15`-row pack and
  explicitly score low-frequency noise plus entry/tail energy
- [ ] assemble valid exact-source stereo and PaulX comparator rows, enforce hard
  structural/whole/band controls, and complete mapped-window diagnostics
- [ ] allow operator speaker pre-screen to reject, then require an eligible
  independent listener for neutral `15`-row and `space`-trio promotion
- [x] stopped on the terminal synthetic miss; deleted the complete candidate without
  repair or rerun; after a complete pass retain only the isolated checkpoint
  and receipt for a separate minimal-admission batch
- [x] kept public APIs, reports, fixtures, cache, routing, other characters,
  Loophole, and Chorus unchanged; do not push

Evidence:

- compile: pass
- construction: exactly `1/1`
- immutable checkpoint: `c0cd943f5a5e8499540d5e759aac7a1586579d0a`
- structural: exactly `15/15`
- synthetic: `7/9`; `Y04` and `Y09` failed
- `Y04`: `16x` impulse produced two active replica regions; secondary was
  `-29.801787859 dB`; `-30 dB` is the activity threshold, so the frozen
  one-region / `None` requirement failed
- `Y09`: linked-stereo swap rows failed at `4x` and `8x`
- mono and stereo listening: not run because objective admission failed
- cleanup: complete; worktree, branch, checkpoint reference, module, tests,
  and build state deleted; no candidate code entered `main`

The rejected receipt conflicts with Batch 31.36, which passed `Y04`, `Y09`,
and the full synthetic gate under the nominally same frozen renderer and seed.
That divergence blocks another implementation. It must be reconciled as
authority or evidence construction before any new candidate is considered.

Stop after the complete pass/rejection receipt and cleanup. Do not merge the
candidate to `main` in Batch 31.39. Minimal admission, product exposure,
multi-seed review, routing, and other characters require later batches.

## Batch 31.40 - Synthetic Receipt Authority Reconciliation

Status: complete; renewed candidate path closed

- [x] traced renderer, seed, counter, source, support, metric, threshold, and
  assertion authority from Batch 31.25 through Batches 31.36 and 31.39
- [x] confirmed the counter vectors, `ADMISSION_SEED`, and
  `SYNTHETIC_SUPPORTS` table are exact shared authority
- [x] confirmed construction `1/1` proves owner inventory and selected tables,
  not helper-body or assertion equivalence
- [x] corrected `Y04`: `-30 dB` selects active envelope windows; it is not a
  secondary-region allowance or comparator ceiling
- [x] found `Y09` never froze one executable source-relative swap assertion
  after exact time-domain swap was explicitly disclaimed at the negative-real
  half-angle branch
- [x] found the comparator brief inherited multi-hop gate prose rather than
  one self-contained executable evidence definition
- [x] kept both deleted checkpoints rejected; neither receipt is promoted into
  proof of the other implementation or of the complete topology
- [x] closed further renewal implementation without closing the PaulX-like
  product target
- [x] changed documentation only; no DSP, test, harness, fixture, route,
  product surface, Loophole, or Chorus work opened

Decision:

- Batch 31.36's `9/9` receipt remains historical evidence for its deleted
  checkpoint; its helper bodies, assertions, per-row values, and source digest
  are unavailable under the required cleanup policy
- Batch 31.39's `7/9` receipt remains the terminal decision for its deleted
  checkpoint; `Y04` records two active regions, while the meaning of its
  `Y09` swap assertion cannot be recovered from canonical authority
- the receipts do not establish that identical executable renderers and gates
  disagreed; they establish that the docs did not preserve enough executable
  identity to compare them after deletion
- restoring authority would require inventing a new self-contained evidence
  specification and running a third renewal candidate, not reconciling the two
  existing receipts
- repeated renewal work already reached mono success, stereo rejection,
  architecture closure, policy reopening, and now unrecoverable evidence
  divergence; another renewal candidate would be churn, not a source-backed
  architectural advance
- renewal remains closed; neutral `Dream` remains a valid product target with
  no admitted or ready Signal renderer

## Batch 31.41 - Complete Creative Owner Study

Status: complete; `LinkedStnNoiseMorph` selected for brief-writing

- [x] execute the operator-authorized search for one materially different,
  source-backed complete creative owner
- [x] audit pinned SiTraNoStar `v2.0.1` / `2edf7b693040b5070116299973abf83dc5ba86e5`
  as a runnable classical STN/noise-morphing path
- [x] reconcile the implementation with the fuzzy STN decomposition, Noise
  Morphing, neural STN, transient-placement, and stereo source evidence
- [x] reject direct source transfer: SiTraNoStar is GPL-3.0, mono-only,
  nondeterministic, full-file, approximate-length, and demonstrated only
  through a `10x` public control range
- [x] select one complete clean-room family in which tonal peaks, transient
  events, and stochastic residuals have separate persistent owners on one map
- [x] require channel-symmetric decomposition, linked tonal state, shared
  event placement, and continuous multichannel residual excitation rather
  than unrelated channel noise
- [x] retain `4x`, `8x`, and `16x` long-form mono plus independent stereo as
  Signal admission authority; do not promote short upstream listening
- [x] keep renewal, cyclic, other characters, routing, product exposure,
  Loophole, and Chorus closed or paused
- [x] change documentation only; no DSP, tests, candidate harnesses, fixtures,
  APIs, routes, or product surfaces entered `main`

Decision:

- `LinkedStnNoiseMorph` is materially different from renewal because phase
  forgetting is confined to the separated residual; tonal phase and transient
  waveform events keep temporal owners
- the family plausibly addresses the low-end noise, tonal instability,
  replica/event-placement risk, opposite entry/tail envelope, and stereo drift
  together
- source evidence is sufficient to freeze one buildable brief, not to claim
  quality or start implementation
- `16x`, long-form music, component leakage, linked residual width, exact
  length, deterministic bounded memory, and computational cost remain open
  terminal risks for the brief and candidate
- Batch 31.42 is docs-only and must freeze every implementation and evidence
  boundary before any isolated candidate becomes ready

## Batch 31.42 - Linked STN Renderer Brief

Status: complete; one isolated candidate ready

- [x] freeze one exact signed-rational source/output map and shared synthesis
  lattice
- [x] freeze sample-rate-normalized long tonal and short transient separation
  with reconstructing channel-symmetric soft masks
- [x] freeze persistent linked tonal peak, bin, dormant, reactivation, and axis
  state
- [x] freeze transient detection, refinement, classification, segmentation,
  unit-rate placement, collision, seam, and one-emission ownership
- [x] freeze continuous counter excitation, residual covariance morphing, and
  linked `space` behavior without channel-local noise
- [x] freeze mapped envelope correction, normalized component synthesis,
  zero exterior, exact crop, and no arbitrary head/tail fade
- [x] freeze a `96 MiB` duration-independent state cap, deterministic traversal,
  allocation boundary, and computational shape
- [x] freeze one self-contained `28`-owner evidence specification, retained
  long-form mono pack, independent stereo gate, receipt identity, rejection,
  cleanup, and minimal admission
- [x] keep candidate DSP, tests, harnesses, fixtures, APIs, routes, cache,
  product exposure, Loophole, and Chorus unchanged

Authority:

- `docs/architecture/offline-creative-linked-stn-noise-morph-brief.md`

Decision:

- `LinkedStnNoiseMorph` is one buildable material-separated renderer, not a
  menu of mechanisms
- persistent tonal phase, one-shot native transients, and residual-only noise
  morphing plausibly address tonal ringing, event replicas, transient softness,
  extra low-end noise, and stereo drift together
- the mapped source envelope and absence of a renderer-owned exterior fade
  directly own the observed entry/tail energy mismatch
- source evidence does not prove `16x`, long-form musical quality, linked
  residual image, or cost; construction, objective gates, mono listening, and
  eligible independent stereo remain terminal
- Batch 31.43 may implement this brief once in its named disposable worktree;
  it may not repair the architecture or enter production

## Batch 31.43 - Linked STN Isolated Candidate

Status: complete; candidate rejected at bounded-state structural gate

- [x] created only `signal-candidate-31-43` on
  `candidate/g10-031-linked-stn-noise-morph` from `c84bd538`
- [x] implemented the private eight-file `LinkedStnNoiseMorph` module and its
  compile-linked `28`-owner evidence specification without changing public or
  production surfaces
- [x] passed `effigy test compile` and construction `1/1`
- [x] froze immutable checkpoint `1c383679` with tree `cf413de5`
- [x] ran all structural owners once: `17/18` passed
- [x] stopped at `S17`; the candidate materialized duration-derived component
  arrays and therefore violated the frozen `96 MiB` duration-independent
  working-state boundary
- [x] did not repair or rerun the checkpoint; synthetic, mono, and stereo
  admission did not open
- [x] retained executable-identity digests in the Batch 31.43 closeout log
- [x] deleted the worktree, branch, checkpoint reference, module, tests, and
  worktree-local build state; no candidate code entered `main`

Decision:

- the miss is architectural conformance, not a metric threshold or parameter
  choice: complete source component buffers replaced the required bounded
  analysis rings
- the `17` passing owners establish no creative quality result because the
  bounded-state gate is terminal and listening remains promotion authority
- a second implementation is not ready; Batch 31.44 must decide whether the
  frozen STN owner graph is realizable with bounded monotonic rings without
  changing its map, decomposition, tonal, event, residual, or evidence
  semantics

## Batch 31.44 - Linked STN Bounded-State Reassessment

Status: complete; bounded v2 candidate ready

- [x] preserved every transform, mask, map, tonal, transient, residual,
  envelope, stereo, boundary, synthetic, and listening rule
- [x] identified first-non-zero residual orientation as the sole non-causal
  dependency
- [x] froze one deterministic full-source decomposition/event orientation
  prepass followed by a clean render-state reset
- [x] froze monotonic producer, consumer, lookahead, finalization, and eviction
  frontiers for every material lane
- [x] froze packed spectral rings, source-component rings, residual covariance,
  event claim arena and live-ledger bound, envelope moments/deque, and bounded
  `f64` output finalization
- [x] proved supported-rate maxima `Q_h<=17`, `R_h<=19`, `Q_v<=97`, and
  `R_v<=57`
- [x] froze exact capacity formulas and maximum rows in compile-linked
  `MEMORY_SPEC`
- [x] froze an `89 MiB` owned-state design ceiling with `7 MiB` unassigned
  below the terminal `96 MiB` actual allocation gate
- [x] froze fresh identity `BoundedLinkedStnNoiseMorph` and exact Batch 31.45
  worktree, branch, module, test prefixes, construction, gate order, cleanup,
  and receipt boundary
- [x] changed documentation only; no candidate, production DSP, test, harness,
  fixture, dependency, API, route, cache, artifact, Loophole, or Chorus surface
  entered `main`

Decision:

- the family remains feasible; the Batch 31.43 miss came from an unbounded
  implementation, not an unavoidable owner dependency
- a single render pass is not feasible under the frozen orientation law;
  buffering descriptors or synthesizing before orientation would violate the
  contract
- the prepass carries only one mono or two stereo signs across passes and does
  not add a map or audible owner
- Batch 31.45 may implement bounded v2 once; quality and promotion remain
  entirely unproved

## Batch 31.45 - Bounded Linked STN Isolated Candidate

Status: complete; candidate rejected at construction and deleted

- [x] created only `signal-candidate-31-45` on
  `candidate/g10-031-bounded-linked-stn-noise-morph` from `8f384c09`
- [x] kept the renderer private and changed no production, public, dependency,
  route, cache, artifact, Loophole, or Chorus surface
- [x] passed `effigy test compile` after permitted compiler-only assembly fixes
- [x] ran the construction owner once; it failed `0/1` before checkpoint
- [x] recorded the exhaustive first-residual maximum `53248` against the frozen
  asserted row `59392`
- [x] independently located the mismatch: current-geometry `R_h=13`, `h_s=6`
  at maximum transform geometry, while global `R_h=19`, `h_s=9` occurs only at
  `F=18000`, `N_t=2048`
- [x] did not change the formula, expected row, helper, or assertion
- [x] did not create a checkpoint or run structural, synthetic, or listening
  admission
- [x] deleted the worktree, branch, private source, tests, and build state; no
  candidate code entered `main`

Decision:

- the stop is an executable-authority contradiction, not a renderer quality
  result
- the `59392` row combines maxima from different supported geometries; the
  frozen per-geometry formula exhaustively reaches `53248`
- no implementation retry is ready; Batch 31.46 must reconcile `MEMORY_SPEC`
  docs-only before deciding whether bounded v2 remains eligible

## Later Batches

Closed or paused without promotion. No implementation is ready. Batch 31.46
docs-only capacity-authority reconciliation is the sole next work. Every later
product batch still requires a separately admitted complete renderer:

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
- [x] one verified source-relative candidate passed construction and structural
  admission, then reached a terminal `7/9` synthetic rejection
- [x] evidence reassessment found its candidate seed was not frozen and the
  paired failures cannot support a range-topology conclusion
- [x] one complete seed-audited source-relative authority is frozen
- [x] one seed-audited source-relative candidate passed construction and
  structural admission, then reached a terminal `Y02` pitch rejection
- [x] repeated tonal-pitch failure triggered architecture reassessment
- [x] architecture reassessment found no eligible complete replacement and
  closed renewal without closing the PaulX-like target
- [x] operator changed the creative pitch gate from comparator rejection to a
  mandatory listening-led diagnostic
- [x] one fresh complete listening-led source-relative authority is frozen
- [x] one fresh listening-led candidate passed construction `1/1` and
  structural `15/15`, then reached a terminal synthetic `Y08` rejection
- [x] the rejected checkpoint, implementation, tests, worktree, branch, and
  build state were deleted without repair or rerun
- [x] the `Y08` impulse-support contradiction was reconciled as executable
  evidence-construction failure
- [x] one fresh support-audited listening-led authority is frozen with an exact
  support table and separate discontinuity/dropout ranges
- [x] one support-audited candidate passed all objective and concealed mono
  gates, then reached terminal source-relative stereo rejection
- [x] the rejected candidate worktree, branch, checkpoint reference, source,
  tests, build state, and listening assembly were deleted without rerun
- [x] repeated linked-stereo failure triggered architecture reassessment
- [x] reassessment found no materially different source-backed complete
  renewal owner and closed the family without closing the product target
- [x] operator changed the creative stereo gate from local numeric rejection
  to mandatory diagnostics plus comparator-relative independent listening
- [x] one fresh complete comparator-audited renewal brief is frozen without
  candidate code or product exposure
- [x] one fresh comparator-audited candidate reaches a terminal complete-system
  decision under the revised product gate
- [x] reconcile the contradictory Batch 31.36 and Batch 31.39 synthetic
  receipts before authorizing any further candidate
- [x] close the renewed candidate path after finding no recoverable executable
  identity and no materially different source-backed renewal owner
- [x] commission a materially different complete-owner study and select
  `LinkedStnNoiseMorph` for one complete clean-room brief
- [x] freeze one complete self-contained `LinkedStnNoiseMorph` renderer brief
  with no candidate-time ownership choices
- [x] one isolated `LinkedStnNoiseMorph` candidate passed construction, then
  reached terminal structural `S17` bounded-state rejection without entering
  `main`
- [x] the rejected checkpoint, implementation, tests, worktree, branch, and
  build state were deleted without repair or rerun
- [x] bounded-state reassessment found one complete two-pass monotonic schedule
  and froze fresh `BoundedLinkedStnNoiseMorph` authority
- [x] no candidate code entered `main` during reassessment
- [x] no private renderer or product surface entered `main`
- [x] one bounded-v2 implementation attempt reached terminal construction
  rejection on contradictory first-residual capacity authority
- [x] the failed worktree, branch, private renderer, tests, and build state were
  deleted before structural admission

## Next Task

Run Batch 31.46 as docs-only evidence-authority reconciliation. Resolve whether
the first-residual capacity is the per-geometry formula maximum `53248` or a
deliberately conservative cross-geometry bound `59392`, then align the formula,
maximum row, construction owner, and memory budget. Do not recover or implement
the deleted candidate, change audible ownership, touch Loophole or Chorus, or
push.
