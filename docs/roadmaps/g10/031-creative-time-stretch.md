# 031 - Creative Time-Stretch

Status: paused; fixed-ratio Dream public surface admitted
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

## Batch 31.46 - Bounded Linked STN Capacity Reconciliation

Status: complete; capacity-audited v3 candidate ready

- [x] exhaustively recomputed every geometry and capacity row over supported
  sample rates `8000..192000`
- [x] retained the per-geometry first-residual formula
  `N_t+2(h_s*A_s+N_s)`
- [x] corrected its exhaustive maximum from the impossible cross-geometry
  `59392` row to `53248`
- [x] corrected the conservative short/source packed model from `9.841 MiB` to
  `9.700 MiB`
- [x] confirmed every other capacity maximum and the `89 MiB` category sum
- [x] retained the `12 MiB` short/source ceiling, `7 MiB` unassigned reserve,
  and `96 MiB` terminal actual-allocation gate
- [x] retained every audible formula, source, metric, threshold, assertion,
  gate, and two-pass ownership rule
- [x] froze fresh identity `CapacityAuditedBoundedLinkedStnNoiseMorph` and
  exact Batch 31.47 worktree, branch, module, prefixes, cleanup, and gate order
- [x] changed documentation only; did not recover Batch 31.45 or change DSP,
  tests, harnesses, dependencies, APIs, routes, cache, artifacts, Loophole, or
  Chorus

Decision:

- `53248` is the only maximum produced by the retained formula; using `59392`
  would require a different global-half-width allocation formula with no
  consumer or safety need
- the correction reduces modeled memory and does not weaken source lookahead,
  ring eviction, or the terminal allocation gates
- Batch 31.47 may implement the complete renderer once under fresh identity;
  creative quality and bounded execution remain unproved

## Batch 31.47 - Capacity-Audited Linked STN Candidate

Status: complete; stopped on contradictory construction authority

- [x] started from exact main commit `4fd15e5b`
- [x] created the exact disposable worktree and fresh capacity-audited branch
- [x] kept all candidate work private and changed no production, public,
  dependency, route, cache, artifact, Loophole, or Chorus surface
- [x] independently evaluated the complete supported-rate geometry before
  checkpoint
- [x] found `R_v=59` at `F=8000`, where the frozen brief requires the
  exhaustive bound `R_v<=57`
- [x] traced the counterexample through `N_t=2048`, `N_s=256`, `A_s=64`,
  `round(1800*N_s/F)=58`, and the frozen upward odd midpoint tie
- [x] did not change the formula, rounding law, odd rule, bound, helper, or
  construction assertion
- [x] did not run compile, construction, structural, synthetic, or listening
  admission and did not create a checkpoint
- [x] deleted the worktree, branch, private source, tests, and candidate state;
  no candidate code entered `main`

Decision:

- capacity-audited v3 is internally contradictory and is not implementation
  authority
- this stop is not a renderer-quality result and does not count as a complete
  candidate failure
- Batch 31.48 must reconcile every geometry-derived median maximum docs-only
  before another implementation can be authorized

## Batch 31.48 - Linked STN Geometry-Authority Reconciliation

Status: complete; geometry-audited v4 candidate ready

- [x] exhaustively evaluated every integer sample rate `8000..192000` under
  the frozen positive rounding, upward odd midpoint, power-of-two, and clamp
  rules
- [x] froze positive rational half-rounding to the larger integer so candidate
  code cannot choose a language-default tie rule
- [x] independently reproduced maxima `Q_h=17`, `Q_v=97`, `R_h=19`, and
  `R_v=59`
- [x] recorded first witnesses `F=16534`, `8000`, `17500`, and `8000`
- [x] corrected only the contradicted `R_v<=57` construction bound to
  `R_v<=59`
- [x] froze shared median-selection scratch as
  `max(Q_h,Q_v,R_h,R_v)=97` `f64` scalars
- [x] exhaustively rechecked every dependent capacity maximum, including
  first residual `53248`, component rings `147712`, claim arena `98816`, live
  events `39`, envelope `32772`, and output finalization `139520`
- [x] confirmed the `9.700 MiB` short/source model, `89 MiB` design sum,
  `7 MiB` unassigned reserve, and `96 MiB` actual gate remain unchanged
- [x] retained every transform, mask, map, threshold, source, metric, quality
  assertion, gate, two-pass rule, and cleanup rule
- [x] froze fresh identity `GeometryAuditedBoundedLinkedStnNoiseMorph` and
  exact Batch 31.49 worktree, branch, module, prefixes, checkpoint, cleanup,
  and gate order
- [x] changed documentation only; did not recover Batch 31.47 or change DSP,
  tests, harnesses, dependencies, APIs, routes, cache, artifacts, Loophole, or
  Chorus

Decision:

- `R_v=59` follows directly from the retained formula and tie rule at
  `F=8000`; `57` was an incorrect exhaustive summary
- `Q_v=97` already dominates median scratch, so the correction has no memory
  or cost consequence
- Batch 31.49 may implement the complete renderer once under fresh identity;
  creative quality and bounded execution remain unproved

## Batch 31.49 - Isolated Geometry-Audited Linked STN Candidate

Status: complete; candidate rejected at structural exact-silence gate

- [x] started the exact disposable worktree and branch from authorized `main`
  head
  `feeb76fe255aa56640de8f732a842942aca936d0`
- [x] implemented only the frozen private module and declaration; no public,
  product, report, fixture, cache, artifact, Loophole, or Chorus surface
- [x] passed compile after permitted pre-checkpoint visibility-only assembly
  fixes
- [x] passed construction `1/1` and froze checkpoint `e2ef62f8` with tree
  `85dc0e45`
- [x] ran structural admission once; `S01..S14` and `S16..S18` passed, while
  `S15` failed exact silence for deterministic residual output around `1e-14`
- [x] diagnosed contradictory authority: `ln(power+eps)` creates positive
  power for zero endpoints while the boundary requires bit-exact zero
- [x] kept synthetic and listening closed; no rendered-output digest exists
- [x] deleted the candidate worktree, branch, checkpoint reference, source,
  tests, build state, receipt, and outputs without repair or rerun
- [x] kept production DSP and all unrelated work unchanged

Decision:

- geometry-audited v4 is rejected; no creative renderer is admitted
- the miss is an end-to-end ownership contradiction, not permission for an
  epsilon, threshold, assertion, or candidate-code repair
- Batch 31.50 must reconcile exact-zero residual ownership docs-only before a
  fresh complete identity can be considered

## Batch 31.50 - Linked STN Exact-Silence Ownership Reconciliation

Status: complete; zero-preserving v5 candidate ready

- [x] reconciled residual diagonal interpolation with bit-exact silence across
  mono, native stereo, mid/side factorization, excitation, WOLA,
  mapped-envelope recombination, and final conversion
- [x] froze `zlog`: two exact-zero endpoints return positive zero; every
  one-zero/one-positive or two-positive row retains the v4 formula
- [x] froze zero-power coherence, cross-power, spectra, factorization, and
  final samples as canonical positive zero without a threshold
- [x] extended zero ownership through duplicate, common-negation, anti-phase,
  channel-swap, signed-zero, local-envelope, and exact-crop controls
- [x] strengthened `S12`, `S13`, `S15`, `S16`, and `S18` without changing the
  `18` structural or `10` synthetic owner inventory
- [x] retained every transform, map, positive-power formula, stochastic stream,
  source, threshold, comparator row, listening pack, gate order, and cleanup
  rule
- [x] confirmed no mask, duration state, variable traversal, or allocation was
  added; the `89 MiB` design, `96 MiB` actual, and cost bounds remain unchanged
- [x] froze fresh identity
  `ZeroPreservingGeometryAuditedBoundedLinkedStnNoiseMorph` and exact Batch
  31.51 worktree, branch, module, prefixes, checkpoint, cleanup, and gate order
- [x] changed documentation only; no DSP, tests, harnesses, dependencies, APIs,
  routes, cache, artifacts, Loophole, or Chorus surface entered `main`

Decision:

- the v4 contradiction is closed without thresholding or changing non-zero
  synthesis behavior
- zero-preserving v5 is one complete buildable renderer authority
- creative quality, exact implementation, synthetic character, mono
  listening, and independent stereo remain unproved

## Batch 31.51 - Isolated Zero-Preserving Linked STN Candidate

Status: complete; candidate rejected at structural geometry-vector gate

- [x] started the exact disposable worktree and branch from authorized `main`
  head `570da1604ba21204c1dccfb3aed6d2980ed239ac`
- [x] implemented only the frozen private module and declaration; no public,
  product, report, fixture, cache, artifact, Loophole, or Chorus surface
- [x] passed compile without repair
- [x] passed construction `1/1` and froze checkpoint `95909451` with tree
  `080bea36`
- [x] ran structural admission once; `S01` and `S03..S18` passed, while `S02`
  failed the handwritten 8 kHz `Q_h=5` vector
- [x] confirmed the formula and renderer both produce
  `odd(round(0.240*8000/256))=9`; the evidence vector was wrong
- [x] diagnosed construction coverage: exhaustive maxima passed but the
  per-rate structural vector was not independently cross-checked
- [x] kept synthetic and listening closed; no rendered-output digest exists
- [x] deleted the candidate worktree, branch, checkpoint reference, source,
  tests, build state, receipt, and outputs without repair or rerun
- [x] kept production DSP and all unrelated work unchanged

Decision:

- zero-preserving v5 is rejected; no creative renderer is admitted
- this is executable-evidence failure, not a renderer-geometry result
- Batch 31.52 must audit all exact geometry vectors and bind them into
  construction before fresh authority can be considered

## Batch 31.52 - Linked STN Geometry-Vector Authority Audit

Status: complete; construction-bound v6 candidate ready

- [x] independently evaluated every integer sample rate `8000..192000` with
  separately implemented Ruby and JavaScript exact-integer evaluators
- [x] reproduced all `184001` complete geometry rows, transform transitions,
  positive-round ties, upward odd counts, extent maxima, and first witnesses
- [x] matched binary table SHA-256
  `22d14913f01143007a114fad7a97d44a7e2b07cf5b254b92bc59c7f805e73697`
  and FNV-1a-64 `7ffb5aa02900893e` in both evaluators
- [x] corrected the sole 8 kHz sentinel to
  `(N_t,A_t,N_s,A_s,N_r,A_r,H,Q_h,Q_v,R_h,R_v)=`
  `(2048,256,256,64,1024,256,128,9,97,9,59)`
- [x] independently reproduced every geometry-derived `MEMORY_SPEC` maximum
  and froze its exact first/last sample-rate witnesses
- [x] froze one compile-linked `GEOMETRY_SPEC` as the only literal geometry
  table and prohibited geometry literals in individual gates
- [x] required one independently coded exhaustive oracle and one shared
  `assert_geometry_authority` helper in construction and `S02`
- [x] retained every renderer, map, exact-zero, stereo, memory, quality, gate,
  receipt, and cleanup rule
- [x] froze fresh identity
  `ConstructionBoundZeroPreservingLinkedStnNoiseMorph` and exact Batch 31.53
  worktree, branch, module, prefixes, checkpoint, and cleanup boundaries
- [x] changed documentation only; no DSP, tests, harnesses, dependencies, APIs,
  routes, cache, artifacts, product, Loophole, or Chorus surface entered `main`

Decision:

- the complete geometry and capacity authority is internally consistent
- construction now owns the exact table consumed by structural admission
- Batch 31.53 may implement the complete renderer once under fresh identity;
  creative quality and bounded execution remain unproved

## Batch 31.53 - Isolated Construction-Bound Linked STN Candidate

Status: complete; candidate rejected at structural admission

- [x] started from exact Batch 31.52 `main` head
  `fdad84326d1d2b576f6a73e96499b77be76dcd4e`
- [x] created only worktree `signal-candidate-31-53` on branch
  `candidate/g10-031-construction-bound-zero-preserving-linked-stn-noise-morph`
- [x] implemented only the private
  `creative_construction_bound_zero_preserving_linked_stn_noise_morph` module
  and private `lib.rs` declaration, using existing dependencies
- [x] made `GEOMETRY_SPEC` the sole literal geometry table; construction and
  `S02` shared the complete-domain authority assertion
- [x] passed compile after one permitted pre-checkpoint Rust ownership repair,
  then passed construction `1/1`
- [x] froze checkpoint `366ac24b5cec936209b3e1cbcadafce45eb06bbc`
  and tree `68da7e43784acf8ae1a9d23e77d244153504fd76`
- [x] ran structural exactly once; `16/18` passed
- [x] stopped on `S06` peak-plateau ownership (`[1,3,4]` vs `[1,3]`) and
  `S18` private-surface containment (forbidden `pub fn`)
- [x] kept synthetic, mono, and stereo listening closed
- [x] deleted the worktree, branch, checkpoint reference, private source,
  tests, and `3.4 GiB` of build state without repair or rerun
- [x] admitted no candidate or production surface to `main`

The dominant cause is incomplete construction ownership of structural
semantics. Construction proved geometry but did not prove the frozen
peak-plateau tie law or private-surface token boundary before checkpoint.

## Batch 31.54 - Linked STN Executable-Authority Reassessment

Status: complete; linked-STN closed without promotion

- [x] inspected the canonical v6 authority and Batch 31.53 receipt without
  recovering deleted candidate source
- [x] audited every `S01..S18` owner against honest construction coverage
- [x] found `S18` belongs before checkpoint, while construction-owning `S06`
  or the other runtime rows would duplicate or move structural admission
- [x] rejected a locally corrected v7 as evidence-protocol churn rather than a
  materially different renderer
- [x] closed `LinkedStnNoiseMorph` under Contract `084` Rule 7 after six
  implementation attempts produced no synthetic or listening evidence
- [x] kept the PaulX-like neutral `Dream` product target active and unadmitted
- [x] did not implement DSP, add tests or harnesses, change production, reopen
  routing or product exposure, touch Loophole or Chorus, merge, or push

## Batch 31.55 - Creative Candidate Evidence-Protocol Reassessment

Status: complete; reusable Contract `085` protocol frozen

- [x] classified compile, construction, structural conformance, acoustic
  objective, mono listening, and independent stereo authority separately
- [x] permitted implementation correction only before one frozen acoustic
  checkpoint, with exact source and test identity retained for comparison
- [x] preserved one complete candidate, anti-sweep, stage-stop, cleanup,
  comparator, and listening-promotion rules
- [x] allowed one explicit reconsideration of a conformance-only family under
  fresh implementation and acoustic identity; acoustic failures remain closed
- [x] retained exact checkpoint source and tests through reassessment in one
  local-only evidence ref, never on `main`
- [x] froze reusable Contract `085` Rule 11 and made owner selection the next
  docs-only checkpoint
- [x] did not implement or recover DSP, add harnesses, change production,
  routing, cache, product exposure, Loophole, or Chorus

## Batch 31.56 - Creative Owner Eligibility And Selection

Status: complete; linked STN selected once under Contract `085` Rule 11

- [x] inventoried every closed creative family by highest valid Rule 11 stage
- [x] distinguished conformance-only closure from synthetic, mono, stereo, or
  listening rejection without reinterpreting any historical receipt
- [x] required complete canonical architecture, retained source backing, and a
  plausible path through every current hard and listening gate
- [x] kept acoustically rejected diffusive, cyclic, and renewal families closed
- [x] selected linked STN as the sole conformance-only eligible family for one
  fresh protocol-bound brief
- [x] named separate brief-freeze and implementation batches; implementation
  remains blocked until the brief is complete
- [x] did not recover candidate source, implement DSP, add harnesses, change
  production, routing, cache, product exposure, Loophole, or Chorus

Decision:

- `DiffuseSpectral` reached synthetic crest rejection
- both continuous-excitation owners stopped in structural conformance; one was
  superseded and the final brief contains a contradictory relation proof
- `CyclicGrain` reached synthetic pitch rejection; `SimilarityAlignedCyclic`
  stopped structurally on a frozen search-reachability miss
- renewal identities reached synthetic and mono admission, then stereo
  rejection; a later checkpoint also failed synthetic admission
- linked STN reached structural conformance only across six attempts; no
  synthetic, comparator, or listening gate ran
- the complete linked-STN architecture remains pinned-source-backed and owns
  every current material, stereo, boundary, memory, and gate seam

## Batch 31.57 - Protocol-Bound Linked STN Brief

Status: complete; fresh Rule 11 authority frozen, docs only

- [x] preserve the complete construction-bound v6 renderer, sources, gates,
  thresholds, and listening packs without changing DSP authority
- [x] freeze one fresh family/candidate identity and one canonical brief; do
  not add another memo or parallel architecture file
- [x] bind working implementation, conformance-complete tree, and immutable
  acoustic checkpoint states to Contract `085` Rule 11
- [x] make compile, construction, and complete `S01..S18` structural passage
  jointly precede the acoustic checkpoint
- [x] freeze exact worktree, branch, module, test, receipt, and local evidence-
  ref ownership for the later isolated candidate
- [x] make every synthetic source, seed, helper, metric, threshold, assertion,
  comparator row, listening pack, and stage order self-contained before code
- [x] confirm no missing authority requires a DSP,
  evidence, threshold, comparator, or listening-policy choice
- [x] change documentation only; do not recover source or implement DSP

Decision:

- fresh identity is `ConformanceBoundLinkedStnNoiseMorph`
- isolated identity, files, gate prefixes, conformance ledger, acoustic ref,
  receipt, and cleanup behavior are exact
- conformance may iterate without acoustic output; one clean tree must pass
  compile, construction `1/1`, and structural `18/18` twice before checkpoint
- synthetic, concealed mono, speaker, and independent stereo gates remain
  terminal and run once from that checkpoint
- retained half-cosine source fades and full-active-support `Y07` denominator
  correct old prose transcription without changing source bytes or thresholds
- no candidate code, harness, product surface, Loophole, or Chorus changed

## Batch 31.58 - Isolated Protocol-Bound Linked STN Candidate

Status: stopped pre-acoustic; retained for docs-level reconciliation

- [x] start from fresh source in the exact isolated worktree
- [x] iterate only compile, construction, and structural conformance against
  frozen authority, recording every failed owner and corrective diff
- [ ] freeze one clean conformance-complete acoustic checkpoint and local
  evidence ref before any synthetic render
- [ ] run synthetic, concealed mono, and independent stereo gates once in order
- [ ] reject and clean up on the first acoustic miss, or retain the isolated
  checkpoint and receipt for a separate minimal-admission decision after a
  complete pass
- [x] keep production, routing, cache, product exposure, Loophole, and Chorus
  closed

Decision:

- retained stop commit is `ae618c90827ddd748dc224632920ee32f785cc65`;
  tree is `de551fc6fa458d500239ac603ed26dee1a4458d6`
- focused compile, construction `1/1`, independent full-buffer `S05`, and
  bounded-allocation `S17` passed
- `S09` exposed a missing numerical tie rule: reconstructed impulse derivative
  powers two `f64` encodings apart choose `p+1`, while `Y03` requires `p`
- no formal clean pass, synthetic gate, rendered audio, listening, or acoustic
  ref ran
- the candidate worktree, branch, source, and nine-round conformance ledger
  remain retained; no candidate code entered `main`

## Batch 31.59 - Transient-Anchor Authority Reconciliation

Status: complete but superseded by Batch 31.61; docs only

- [x] record the Batch 31.58 stop and retained executable identity
- [x] freeze one exact derivative-power comparison rule with no tunable choice
- [x] bind `S09` to equality-boundary and observed-impulse vectors
- [x] distinguish refined source anchor from mapped target ledger anchor in
  `Y03`
- [x] extend Contract `085` Rule 11 for a retained pre-acoustic authority stop
- [x] propagate the stopped/resume state through roadmap front doors and log
- [x] keep candidate DSP, acoustic execution, production, routing, Loophole,
  and Chorus unchanged

Decision:

- non-negative finite `f64` derivative-power scores within four adjacent
  encodings are equal; earliest sample owns the tie
- distance `5` or greater selects the larger score; score values are not
  changed, floored, or thresholded
- the retained pre-acoustic implementation may resume under the same
  `ConformanceBoundLinkedStnNoiseMorph` identity only after this docs closeout
  commit is applied there
- all previous partial passes are diagnostic; full compile, construction
  `1/1`, and structural `18/18` conformance must restart twice from one clean
  commit before any acoustic ref

## Batch 31.60 - Resume Protocol-Bound Linked STN Conformance

Status: stopped pre-acoustic; four-ULP authority incomplete

- [x] apply the Batch 31.59 docs closeout commit to the retained candidate
  worktree
- [x] implement only four-ULP transient refinement, target-ledger semantics,
  and direct `S09`/`Y03` structural ownership
- [ ] commit one clean resumed tree and restart the complete compile,
  construction `1/1`, and structural `18/18` sequence twice
- [ ] create the immutable acoustic ref only after both complete passes
- [x] run no synthetic, rendered comparator, or listening work before that ref
- [x] keep production, routing, cache, product exposure, Loophole, and Chorus
  closed

Decision:

- compile passed; `S09` stopped at the `0.65` impulse-train event before `S10`
  or a complete structural round
- reconstructed rise/fall powers were `0x3fdb0a3d4f5c2900` and
  `0x3fdb0a3d4f5c290a`, distance `10`; four ULPs did not preserve authored `p`
- retained stop commit is `4cb82a2ef7731aeaf306d3955766c75c9863aa89`;
  tree is `6083e84604bb95f561fd6b7c25aef55b9a49b12a`
- conformance ledger round `10A` records the correction and stop; no synthetic
  execution, rendered audio, listening, or acoustic ref exists

## Batch 31.61 - Transform-Error Authority Reconciliation

Status: complete; docs only

- [x] record the Batch 31.60 stop and retained executable identity
- [x] reject representation-local ULP counting rather than increment its bound
- [x] freeze one scale-relative error rule covering the complete bounded
  short-transform and derivative-score path
- [x] bind `S09` to zero, equality, `64`/`65` ULP boundary, isolated impulse,
  and `0.65` train vectors
- [x] retain exact mapped target-ledger ownership in `S10` and compiled `Y03`
- [x] propagate the stopped/resume state through active front doors and log
- [x] keep candidate DSP, acoustic execution, production, routing, Loophole,
  and Chorus unchanged

Decision:

- for non-negative finite scores `a,b`, use
  `tau=64*f64::EPSILON*max(a,b)`; later `b` wins only when `b>a` and `b-a>tau`
- `64` is the next-power-of-two budget above four rounding sites across the
  maximum `12` short-transform stages; it owns the whole reconstruction and
  score path rather than one observed bit distance
- there is no absolute floor and scores remain unchanged
- retained pre-acoustic identity may resume only after this docs closeout is
  applied; all partial passes remain diagnostic

## Batch 31.62 - Resume Transform-Bounded Linked STN Conformance

Status: complete; synthetic non-completion, receipt later invalidated

- [x] apply the Batch 31.61 docs closeout commit to the retained worktree
- [x] replace only the ULP comparison and direct `S09`, `S10`, and compiled
  `Y03` ownership
- [x] commit one clean resumed tree and restart complete compile, construction
  `1/1`, and structural `18/18` conformance twice
- [x] create the immutable acoustic ref only after both complete passes
- [x] run no synthetic, rendered comparator, or listening work before that ref
- [x] run the frozen synthetic command once and stop when `Y09` did not
  complete with a passing owner result
- [x] keep production, routing, cache, product exposure, Loophole, and Chorus
  closed

Decision:

- acoustic checkpoint is commit
  `61922465b446dfce8ed086bc5dd61f4a9619a837`, tree
  `fc57cd4c5eeb3c889293de3e8236863ca5513e7c`
- both clean conformance rounds passed compile, construction `1/1`, and
  structural `18/18`
- the local evidence ref is
  `refs/signal-evidence/creative/linked-stn/31-58-acoustic`
- the one-shot synthetic command selected `Y09` first; its initial linked-
  stereo render pair remained compute-bound for about `59` minutes, then
  nextest returned exit `100` without an inner panic or completed-owner row
- required synthetic admission was not `10/10`; no other synthetic owner,
  comparator render, mono listening, or independent stereo listening ran
- the exact failed assertion is unavailable and is not inferred; the recorded
  dominant cause is non-completion of the linked-stereo owner under the frozen
  executable gate shape
- the candidate worktree, branch, build state, and ignored receipt were
  deleted after closeout; the local evidence ref remains for reassessment
- no candidate DSP or product surface entered `main`

## Batch 31.63 - Linked STN Synthetic Execution Reassessment

Status: complete; linked STN closed without acoustic result

- [x] inspect the retained acoustic ref, frozen `Y09` owner, command result,
  and runtime shape without executing candidate DSP
- [x] classify timeout/non-completion separately from an acoustic assertion
  miss; do not invent the suppressed assertion result
- [x] decide whether the frozen gate was executable evidence, whether the
  renderer's computational shape itself failed, or whether authority was
  incomplete
- [x] close linked STN or freeze one evidence-backed complete next owner; no
  local optimization, release-build substitution, split owner, or rerun
- [x] delete the local evidence ref when the evidence question closes
- [x] keep candidate implementation, production, routing, cache, product
  exposure, Loophole, and Chorus closed

Decision:

- the non-completion is not an acoustic assertion failure and does not prove
  release-profile computational infeasibility
- executable `Y09` omitted canonical swapped-input, duplicate-versus-mono,
  descriptor-diagonal, and residual-side-energy hard rows
- construction counted owner IDs and function pointers but did not bind
  canonical assertions or receipt fields to executable owners
- one unoptimized owner combined `45` rows, `90` full stereo renders,
  `80,640,000` output frames, and exact-length band FFTs without incremental
  row persistence or a frozen execution envelope
- no valid synthetic receipt exists; linked STN has no acoustic pass or reject
- this repeats Batch 31.53's incomplete-executable-authority class after Rule
  11 was introduced to prevent it; another evidence-only identity is closed as
  protocol churn
- linked STN closes without promotion; the PaulX-like neutral `Dream` target
  remains active and unadmitted
- the local evidence ref was deleted; no candidate source or product surface
  entered `main`

## Batch 31.64 - Simpler PaulX-Like Owner Study

Status: complete; product-gate reset recommended, not authorized

- [x] re-audit pinned PaulXStretch, CDP, Potenza, retained coherent/component
  dossiers, and accepted Signal listening evidence
- [x] test every remaining family against neutral `Dream`, source backing,
  linked stereo, exact boundaries, bounded deterministic state, and whole-
  renderer complexity
- [x] find no unused fifth family; alternatives own another character or add
  more state than the closed linked-STN renderer
- [x] identify direct PaulX-style magnitude renewal as the smallest owner of
  the accepted sound without mislabeling it as a new DSP family
- [x] define the one evidence-backed product-gate reset required before renewal
  can reopen under Contract `085` Rule 11
- [x] keep the reset conditional and change documentation only; no candidate,
  harness, fixture, public API, route, cache, Loophole, or Chorus work opened

Decision:

- no new implementation family is selected
- a fresh `DirectRenewalDream` brief is the recommended path only if the
  operator explicitly authorizes the product-gate reset
- hard integrity, level, long-form mono, and eligible independent stereo
  promotion remain; exact creative pitch, impulse-region, and sample-algebra
  rows become complete diagnostics rather than automatic vetoes
- no Batch 31.65 implementation is ready; authorization opens one docs-only
  complete-brief batch first

Authority:

- `docs/architecture/offline-creative-direct-renewal-owner-study.md`

## Batch 31.65 - DirectRenewalDream Authority Freeze

Status: complete; one isolated candidate ready, docs only

- [x] record the operator's explicit direct-renewal product-gate reset
- [x] retain hard integrity, level, deterministic, boundary, dropout, and
  linked-channel energy admission
- [x] make exact creative pitch, impulse-region, local-image, and non-zero-
  `space` sample algebra complete diagnostics under listening authority
- [x] freeze one exact long-window direct-renewal transform, sample-centred
  source map, linked-channel rotation law, compensated adjacent-frame blend,
  asymmetric boundary envelope, and exact target crop
- [x] freeze bounded state, deterministic counter addressing, exact structural
  and synthetic sources, retained PaulX identities, executable owners, runner
  envelope, incremental receipts, acoustic ref, gate order, cleanup, and
  minimal admission
- [x] change documentation only; add no DSP, candidate harness, fixture,
  report mode, public API, route, cache, Loophole, or Chorus surface to `main`

Decision:

- `DirectRenewalDream` is one fresh Rule 11 candidate identity, not a repair
  or recovery of any rejected renewal checkpoint
- the fixed-ratio private candidate targets neutral `Dream` at exact `4x`,
  `8x`, and `16x`; listening remains promotion authority
- Batch 31.66 may implement the complete brief once in its named isolated
  worktree and must stop on authority mismatch or failed gate

Authority:

- `docs/architecture/offline-creative-direct-renewal-dream-brief.md`
- `docs/contracts/085-creative-time-stretch-product-and-routing-contract.md`

## Batch 31.66 - DirectRenewalDream Isolated Candidate

Status: complete; fixed-ratio candidate passed

- [x] started from exact Batch 31.65 closeout `a2759b39` in worktree
  `signal-candidate-31-66` on branch
  `candidate/g10-031-direct-renewal-dream`
- [x] implemented one fresh private candidate without recovering rejected
  source or changing production, routing, Loophole, or Chorus
- [x] passed two unchanged clean conformance rounds: compile, construction
  `1/1`, and structural `10/10`
- [x] froze immutable acoustic checkpoint
  `760da32d2c87b2838bda48f32af90ae4ae51f8d9` and matching local evidence ref
- [x] passed `Y01..Y05`: `88/88` synthetic rows and `76/76` renders
- [x] passed concealed long-form mono as `15/15` usable ties with no family
  loss
- [x] retained the material-dependent slower-entry/faster-tail behavior on
  L001 and L006 as a non-terminal listening caveat
- [x] passed `45/45` long-form stereo hard rows, `15/15` `space` trio rows,
  and `1400/1400` finite mapped diagnostics
- [x] recorded worst whole/three-band error `0.138039758 dB` against
  `0.75 dB` and worst trio spread `0.064465954 dB` against `0.50 dB`
- [x] recorded the operator's satisfactory speaker review and explicit waiver
  of eligible independent review for this effect
- [x] kept the candidate checkpoint and evidence isolated; no candidate code
  entered `main`

Decision:

- `DirectRenewalDream` passes fixed-ratio neutral-`Dream` candidate admission
- the stereo decision is an explicit operator-owned Contract `085` Rule 5
  exception for this checkpoint, not an independent-listening result
- the one-ear hearing limitation and largest local image diagnostic remain
  recorded risks
- no public API, route, cache, dynamic ratio, other character, Loophole, or
  Chorus surface is admitted

## Batch 31.67 - Minimal DirectRenewalDream Admission

Status: complete; private fixed-ratio renderer admitted

- [x] started from the Batch 31.66 docs closeout and retained exact checkpoint
  `760da32d2c87b2838bda48f32af90ae4ae51f8d9` as source authority
- [x] admitted the renderer formulas without acoustic change; analysis, plan,
  stereo, and synthesis source remain byte-identical to the checkpoint
- [x] admitted only the private fixed-ratio neutral-`Dream` request and renderer,
  structural/synthetic regression owners, diagnostic receipt schema, and one
  internal creative-engine version
- [x] reproduced construction `1/1`, structural `10/10`, and synthetic
  `88/88` rows with `76/76` renders after integration
- [x] proved retained synthetic hashes, assertions, and diagnostics identical
  row-for-row after excluding checkpoint, stage, and round labels
- [x] kept public controls, routing, cache, artifacts/reports, dynamic ratio,
  other characters, Loophole, and Chorus closed
- [x] retained the isolated worktree and acoustic ref only through final
  admission validation; cleanup follows the secured admission commit
- [x] validated and committed the bounded Signal-only admission without push

Decision:

- `DirectRenewalDream` is now a private, production-compiled, unrouted Signal
  renderer for exact `4x`, `8x`, and `16x`
- internal engine identity is
  `signal-creative-direct-renewal-dream-v1`
- no public or product route is implied by private admission

## Batch 31.68 - Coherent/Dream Lower-Overlap Reassessment

Status: complete; lower overlap remains paused

- [x] re-audit the admitted exact-ratio `Dream` owner against Contract `085`'s
  paused `2x..4x` coherent/diffusive overlap intent
- [x] determine whether the frozen coherent renderer and admitted `Dream`
  owner can share one map, boundary, level, image, and deterministic blend
  without changing either admitted renderer
- [x] treat exact `2x`, interior overlap probes, and exact `4x` as mandatory
  evidence boundaries; do not infer continuous-ratio support from `4x`
- [x] decide whether one complete lower-overlap architecture can be frozen or
  the overlap must remain paused
- [x] change documentation only; do not add adapters, blends, ratio support,
  public controls, routing, cache, Loophole, or Chorus

Decision:

- `OfflineHighQuality` covers arbitrary positive fixed ratios, but
  `DirectRenewalDream` supports only exact `4x`, `8x`, and `16x`; mandatory
  exact `2x` and interior probes have no Dream render
- their frame lattices, schedulers, and exterior envelopes are not shared
- a hard switch, post-resample, second stretch pass, or exact-`4x` blend cannot
  satisfy Contract `085` Rules 1, 2, and 4
- the lower overlap remains paused; neither renderer changes or fails
- reopening requires one separately admitted complete lower creative owner or
  a newly versioned and re-admitted generalized Dream renderer

## Batch 31.69 - LayeredCloud Owner Feasibility Study

Status: complete; docs, research, and architecture only

- [x] audit existing source-backed cloud, spectral, and granular evidence for
  one complete owner of the future `32x..100x` range
- [x] bind exact fixed-ratio evidence at the `16x` boundary, `32x`, and one
  high-range point no lower than `100x`
- [x] require one monotonic source map, exact target length, bounded and
  deterministic voice/state ownership, linked stereo, one normalization law,
  and explicit exterior boundaries
- [x] decide whether one owner can later meet the `16x..32x` upper-overlap
  obligations without treating an arbitrary wet stack as a renderer
- [x] freeze at most one source-backed complete owner brief or close the lane;
  do not create a menu of mechanisms
- [x] change documentation only; keep `DirectRenewalDream`, candidate DSP,
  overlap implementation, routing, controls, cache, dynamic ratio, Loophole,
  and Chorus unchanged

Decision:

- Csound `sndwarpst` supplies the selected complete pointer-led granular
  family: one time pointer, bounded overlapping unit-rate grains, and a shared
  stereo schedule
- SuperCollider `Warp1` confirms the family but keeps randomized grain state
  per output channel, so it is not the linked-stereo authority
- Signal freezes one clean-room `LayeredCloud` renderer covering every fixed
  ratio from `16x` through `100x` with its own map, launch lattice, duration
  counter, normalization, boundaries, memory ceiling, and complete gate order
- the upper overlap remains paused because `DirectRenewalDream` has no
  interior `16x..32x` renders
- no DSP, candidate harness, experiment module, route, control, cache,
  dynamic-ratio, Loophole, or Chorus surface entered `main`

Canonical authority:
[Offline Creative LayeredCloud Renderer Brief](../../architecture/offline-creative-layered-cloud-brief.md).

## Batch 31.70 - Isolated LayeredCloud Candidate

Status: complete; synthetic receipt invalid, no quality result

- [x] start from the Batch 31.69 closeout commit and create worktree
  `signal-candidate-31-70` on branch `candidate/g10-031-layered-cloud`
- [x] implement the frozen private `LayeredCloud` renderer and compile-linked
  Rule 11 authority without changing one DSP, source, seed, assertion,
  threshold, comparator, or listening choice
- [x] pass the complete compile, construction, and `S01..S08` conformance
  sequence twice unchanged before freezing the acoustic checkpoint
- [x] create local ref
  `refs/signal-evidence/creative/layered-cloud/31-70-acoustic`
- [x] run `Y01..Y05` in order; invalidate the apparent green result when the
  post-run audit finds missing frozen `Y05` diagnostics
- [x] keep long-form mono and independent stereo listening closed because the
  complete synthetic receipt did not pass
- [x] keep admitted DSP, overlaps, routing, controls, cache, dynamic ratio,
  Loophole, Chorus, and `main` unchanged

Pre-conformance reconciliation:

- [x] construction stopped before candidate source because the frozen `L=1`
  and `L=H-1` success rows contradict the validity-weight floor
- [x] no candidate DSP or acoustic output was created
- [x] freeze `L>=H` as the minimum non-empty request and reject shorter input
  before output allocation
- [x] replace impossible boundary success rows with `L=2H` and `L=12H`
- [x] correct structural authority from `100` to `101` rows while retaining
  `51` renders
- [x] apply the docs closeout commit to the retained clean worktree and resume
  the complete implementation under otherwise unchanged authority

Decision:

- checkpoint `ee42f50c4c338db4af8a7feaa89bb8b21e8d0860`, tree
  `cfc28c8c6c4095f0c91ae95d0724962656bcec97`, passed two complete compile,
  construction `1/1`, and structural `8/8` rounds; each structural receipt
  records `101/101` rows and `51/51` renders
- `Y01..Y04` completed and the `Y05` executable returned green; total
  synthetic counts were `33/33` rows and `45/45` renders
- post-run audit proved `Y05` measured only whole-buffer balance,
  correlation, and width; it omitted the frozen three-band and mapped-window
  diagnostics and persisted no natural-stereo diagnostic values
- the green synthetic receipt is invalid; it is not a renderer-quality pass
  or rejection, and no comparator or listening pack opened
- the isolated branch, worktree, build state, receipts, and evidence ref remain
  retained only through Batch 31.71's required evidence-integrity decision

## Batch 31.71 - LayeredCloud Evidence-Integrity Reassessment

Status: complete; one fresh docs-first identity justified

- [x] record Batch 31.70 as evidence-invalid without using its output as
  renderer-quality evidence
- [x] audit every frozen acoustic helper, assertion, diagnostic, receipt field,
  row, render, and construction-manifest edge against executable ownership
- [x] decide whether the still-unjudged pointer-led topology warrants one fresh
  audited identity or closes here
- [x] delete the retained worktree, branch, build state, generated output, and
  local evidence ref after the decision is committed
- [x] keep candidate DSP, product code, routing, controls, cache, dynamic ratio,
  Loophole, and Chorus unchanged

Decision:

- the checkpoint omitted canonical spec owners, full construction mapping, the
  tracked runner profile, enforceable row deadlines, multiple structural
  assertions, truthful stereo frame counts, required Y02-Y05 diagnostics, and
  all comparator/listening executable ownership
- this is invalid evidence, not a Cloud quality result; checkpoint output does
  not authorize renderer, source, helper, or threshold changes
- the small source-backed topology has no previous incomplete-evidence identity
  and no valid acoustic decision, so one fresh `AuditedLayeredCloud` authority
  is justified
- the replacement starts docs-first and source-clean; a second evidence-
  integrity failure closes the family without another rebinding

## Batch 31.72 - AuditedLayeredCloud Authority Freeze

Status: complete; docs only

- [x] freeze one complete replacement brief around the unchanged canonical
  pointer-led architecture, not Batch 31.70 source or output
- [x] freeze compile-linked render, source, vector, evidence, memory, run,
  comparator,
  listening, and cleanup specs with exact hashes and owner mappings
- [x] give every structural and synthetic assertion an exact executable owner;
  resolve component-presence and row-timeout mechanics explicitly
- [x] freeze truthful frame/sample receipt semantics, specific assertion and
  diagnostic payloads, tracked nextest profile, incremental persistence, and
  complete comparator/listening decisions
- [x] make one fresh isolated implementation batch ready only if Contract `085`
  Rule 11 is satisfied with no decide-later gap
- [x] keep admitted DSP and product surfaces unchanged

Decision:

- [Offline Creative AuditedLayeredCloud Renderer Brief](../../architecture/offline-creative-audited-layered-cloud-brief.md)
  is the sole replacement authority
- the renderer remains the same source-backed pointer-led system; invalid
  Batch 31.70 audio did not guide any DSP, scalar, source, or threshold change
- one compile-linked manifest owns every row, assertion, diagnostic, receipt,
  comparator capture, concealment, listening decision, and cleanup boundary
- one test process per structural or synthetic row makes `120 s` and `900 s`
  nextest deadlines enforceable; capture rows own `7200 s`
- `Y02` now requires at least `2^-10` of one-sided energy inside each authored
  `+-4 Hz` band and persists the complete pitch record
- `Y03` persists centroids, `Y04` scans cross-block dropout and the final
  remainder, and `Y05` persists whole, three-band, and mapped-window stereo
  diagnostics
- a second evidence-integrity failure closes Cloud; any acoustic miss remains
  terminal for the checkpoint

## Batch 31.73 - Isolated AuditedLayeredCloud Candidate

Status: complete; stopped before structural admission, Cloud closed

- [x] create only worktree `signal-candidate-31-73`, branch
  `candidate/g10-031-audited-layered-cloud`, the frozen private module,
  tracked manifest/profile, and ignored evidence root
- [x] implement the renderer and compile-linked construction surface from
  the Batch 31.72 brief without recovering Batch 31.70 source or output
- [x] compile every owner and pass construction `1/1`
- [x] stop before structural execution when the frozen `S03` strict-support
  equations prove maximum occupancy `21` against required result `22`
- [x] create no commit, checkpoint, acoustic ref, synthetic output,
  comparator output, or listening pack
- [x] classify the contradiction as the second Cloud evidence-integrity
  failure and close the family without another rebinding
- [x] stop for docs reassessment on unanswered authority; stop and
  reject on any post-checkpoint miss
- [x] keep admitted DSP, product surfaces, overlaps, routing, cache, dynamic
  ratio, Loophole, and Chorus unchanged

Decision:

- compile passed and construction returned `1/1`
- strict `2|q|<D` with `D<=20H` admits at most `20` regular launches; one
  distinct terminal admits at most `21`
- the brief's required exhaustive maximum `22` is unreachable and cannot be
  corrected inside an implementation batch
- no acoustic identity exists and no Cloud renderer-quality conclusion follows
- the disposable candidate worktree, branch, source, tests, and build state
  are deleted after this docs closeout commits

## Batch 31.74 - Creative High-Range Reassessment

Status: complete; broader range paused, docs only

- [x] reconcile Cloud closure with admitted `DirectRenewalDream` coverage and
  the operator's accepted creative effect
- [x] decide whether `16x..100x` remains an active product target, narrows to
  admitted coverage, or requires a materially different complete owner study
- [x] do not rebind LayeredCloud, repair Batch 31.72, or infer a replacement
  candidate from unjudged output
- [x] keep admitted DSP, routing, controls, cache, dynamic ratio, Loophole,
  Chorus, and cross-repo surfaces unchanged
- [x] make no implementation batch ready unless one complete, materially
  different architecture and executable authority are frozen docs-first

Decision:

- current executable creative coverage is exact fixed `4x`, `8x`, and `16x`
  neutral `Dream` through private `DirectRenewalDream`
- `16x` is an admitted endpoint, not evidence for a continuous band or any
  ratio above it
- `16x..100x` leaves the active queue and becomes deferred research intent
- Cloud has no acoustic pass or rejection; its second evidence-integrity
  failure closes only the pointer-led `LayeredCloud` family
- the complete-owner audit found no unused fifth family; CDP-like spectral,
  cyclic, coherent, STN, image, and learned paths either own a different
  character, are closed, or lack a complete Signal boundary
- no replacement study or implementation is ready
- automatic routing, both overlaps, dynamic ratio, public controls, cache,
  product exposure, Loophole, and Chorus remain paused or absent
- reopening above `16x` requires explicit operator authority plus one
  materially different, source-backed complete owner frozen docs-first

## Batch 31.75 - Fixed-Ratio Public Surface Freeze

Status: complete; docs only

- [x] audit the admitted private request against the existing `TimeStretcher`,
  tier, cache, and product contracts
- [x] freeze one semantic public API that keeps `DirectRenewalDream` internal
- [x] expose only mono/stereo input, sample rate, exact target frames, fixed
  `Dream`, and admitted `space`
- [x] retain the admitted fixed seed; do not expose seed or reroll
- [x] reject unsupported targets without transparent fallback
- [x] keep the API offline, allocating, whole-buffer, and audio-thread
  unsupported
- [x] freeze public error mapping, constants, UI meaning, implementation file
  scope, and byte-identity gates
- [x] keep current cache identity, tiers, routing, overlaps, dynamic ratio,
  pitch, motion, detail, runtime, Loophole, and Chorus unchanged
- [x] change documentation only

Decision:

- the existing `TimeStretcher` trait is not widened because its arbitrary
  mutable ratio, infallible result, and mono-first shape do not own the
  creative request
- the public product name is `CreativeStretch`; `DirectRenewalDream` remains
  an internal renderer identity
- exact target frames remain authoritative and must resolve to `4x`, `8x`, or
  `16x`
- `space` is the only adjustable creative control; its default is `0.5`
- public seed/reroll remains blocked pending multi-seed character review
- the existing transparent cache schema must not identify creative output
- Batch 31.76 was the only wrapper implementation batch

Authority:

- [Offline Creative Fixed-Ratio Public Surface](../../architecture/offline-creative-fixed-ratio-public-surface.md)

## Batch 31.76 - Minimal CreativeStretch Public Wrapper

Status: complete; fixed-ratio public surface admitted

- [x] add only the frozen public `CreativeStretch` types, constants, wrapper,
  rustdoc, and focused tests
- [x] map the public request onto the admitted private renderer with the exact
  fixed seed and unmodified `space`
- [x] prove byte-identical public/private output for mono and stereo at `4x`,
  `8x`, and `16x`
- [x] map every private failure into the frozen public error
- [x] keep `analysis.rs`, `plan.rs`, `stereo.rs`, and `synthesis.rs`
  byte-identical
- [x] rerun integrated construction `1/1`, structural `10/10`, and synthetic
  `88/88` with `76/76` renders
- [x] add no cache, route, tier, dynamic ratio, pitch, motion, detail, report,
  runtime, Loophole, Chorus, or cross-repo surface

Decision:

- public `CreativeStretch` now exposes exact fixed `Dream` at `4x`, `8x`, and
  `16x`, with `space` as its only adjustable creative control
- public/private output is byte-identical at every admitted ratio and frozen
  space anchor
- all four acoustic-module hashes remain unchanged
- integrated construction, structural, and synthetic admission remains green
- no new listening was required because the admitted seed and renderer output
  did not change
- no cache, artifact, route, tier, dynamic ratio, product integration,
  Loophole, or Chorus work is authorized

## Batch 31.77 - Planning Surface Refocus

Status: complete; docs only

- [x] audit live architecture, inventory, contract, roadmap, and front-door
  currentness after public admission
- [x] correct stale claims that `g10.031` is active, the API is only frozen,
  the renderer remains private-only, or no creative owner is admitted
- [x] retain the exact-ratio API, acoustic DSP, routing, cache, artifacts,
  dynamic ratio, runtime, Loophole, and Chorus unchanged
- [x] replace the implied implementation queue with one operator intent
  checkpoint

Decision:

- `g10.031` is paused with exact public `4x`, `8x`, and `16x` neutral `Dream`
- no current architecture or contract makes another implementation batch ready
- the next direction is a product-priority choice, not a missing technical
  detail
- current authority supports only freezing the lane, a named-consumer
  integration study, or explicitly renewed source-backed high-range research

## Later Batches

Deferred product work requires separate authority:

- cache identity and artifact integration for admitted creative output
- any materially different complete owner above `16x`
- any coherent/Dream or Dream/high-range overlap
- dynamic-ratio state continuity
- runtime or product-workflow integration
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
- [x] capacity reconciliation retained the formula, corrected its exhaustive
  maximum to `53248`, and froze fresh v3 candidate identity
- [x] no candidate code entered `main` during capacity reconciliation
- [x] capacity-audited v3 stopped before checkpoint when exhaustive geometry
  produced `R_v=59` against the frozen `R_v<=57` construction bound
- [x] the disposable candidate surface was deleted without compile or gate
  execution; no candidate code entered `main`
- [x] geometry reconciliation corrected `R_v` to `59`, froze shared median
  scratch at `97`, and retained every dependent memory ceiling
- [x] no candidate code entered `main` during geometry reconciliation
- [x] one geometry-audited v4 candidate passed construction `1/1`, then failed
  structural admission at `17/18` on bit-exact silence
- [x] the rejected v4 checkpoint, implementation, tests, worktree, branch,
  build state, and outputs were deleted without repair or rerun
- [x] no candidate DSP entered `main` after v4 rejection
- [x] exact-silence reassessment froze one complete zero-preserving v5 authority
  without candidate DSP or product exposure
- [x] one zero-preserving v5 candidate passed construction `1/1`, then failed
  structural admission at `17/18` on an incorrect handwritten geometry vector
- [x] the rejected v5 checkpoint, implementation, tests, worktree, branch,
  build state, and outputs were deleted without repair or rerun
- [x] no candidate DSP entered `main` after v5 rejection
- [x] geometry-vector audit independently reproduced all `184001` rows and
  every geometry-derived capacity witness
- [x] one construction-owned `GEOMETRY_SPEC` now supplies construction and
  structural `S02` without a second handwritten table
- [x] no candidate DSP entered `main` during geometry-vector audit
- [x] construction-bound v6 passed construction `1/1`, then failed structural
  admission at `16/18` on peak-plateau ownership and private-surface
  containment
- [x] the rejected v6 checkpoint, implementation, tests, worktree, branch, and
  build state were deleted without repair or rerun
- [x] no candidate DSP entered `main` after v6 rejection
- [x] all `S01..S18` owners were audited against construction coverage
- [x] a locally corrected linked-STN v7 was rejected as gate duplication, not
  fresh architecture
- [x] `LinkedStnNoiseMorph` closed without promotion after six implementation
  attempts and no synthetic or listening evidence
- [x] the PaulX-like neutral `Dream` target remains active and unadmitted
- [x] Contract `085` now separates iterative conformance from immutable
  acoustic candidate identity
- [x] exact checkpoint source and test identity survives through required
  reassessment without entering `main`
- [x] conformance-only closure has one explicit reopening path; acoustic
  failures retain their terminal meaning
- [x] every closed family classified under the new evidence protocol
- [x] linked STN selected as the sole conformance-only eligible owner
- [x] fresh protocol binding separated from later isolated implementation
- [x] fresh `ConformanceBoundLinkedStnNoiseMorph` identity, full conformance
  loop, acoustic checkpoint, evidence corpus, and terminal gate order frozen
- [x] Batch 31.57 changed documentation only; no candidate DSP, harness,
  product surface, Loophole, or Chorus entered `main`
- [x] Batch 31.58 stopped before acoustic identity when reconstructed impulse
  refinement exposed contradictory exact-comparison and event-anchor authority
- [x] retained stop commit, tree, worktree, branch, and conformance ledger make
  the pre-acoustic execution state auditable
- [x] Batch 31.59 froze one four-ULP earliest-owner rule and exact `S09`/`Y03`
  ownership without changing candidate DSP or running acoustic work
- [x] Batch 31.60 proved that ULP-local rule incomplete on the frozen `0.65`
  train event and stopped before complete structural or acoustic execution
- [x] Batch 31.61 replaced ULP counting with one transform-bounded
  scale-relative equality rule and complete boundary vectors
- [x] Contract `085` and every active front door bind the retained Batch 31.62
  pre-acoustic resume and complete conformance restart
- [x] Batch 31.62 passed two complete conformance rounds and froze one exact
  acoustic checkpoint before synthetic execution
- [x] the one-shot synthetic command stopped in `Y09` without the required
  completed-owner result; later acoustic and listening stages remained closed
- [x] rejected candidate source and build state were removed from the isolated
  branch while its local evidence ref was retained for reassessment
- [x] Batch 31.63 proved the checkpoint's `Y09` owner and receipt construction
  incomplete against canonical authority without running candidate DSP
- [x] the synthetic non-completion was classified as invalid evidence, not an
  acoustic rejection or release-profile performance result
- [x] repeated incomplete executable authority closed linked STN without
  promotion; no replacement owner was inferred
- [x] the local evidence ref was deleted after reassessment
- [x] one materially simpler source-backed owner study found no unused fifth
  family and recommended one explicit direct-renewal product-gate reset
- [x] the operator authorized that reset; one complete `DirectRenewalDream`
  renderer and executable evidence brief now satisfies Contract `085` Rule 11
- [x] Batch 31.65 changed documentation only and opened exactly one isolated
  candidate implementation batch
- [x] Batch 31.66 passed two clean conformance rounds, all `88` synthetic
  rows, concealed mono `15/15`, and all `45` stereo hard rows
- [x] the operator accepted the fixed-ratio stereo effect and explicitly
  waived eligible independent review for this checkpoint under scoped
  Contract `085` Rule 5 authority
- [x] one exact passed candidate checkpoint and local evidence ref remain
  isolated for minimal admission
- [x] Batch 31.67 admitted only the private fixed-ratio renderer, request,
  regression owners, diagnostic schemas, and one internal engine version
- [x] the four acoustic implementation files remain byte-identical to
  checkpoint `760da32d`
- [x] integrated construction `1/1`, structural `10/10`, and synthetic
  `88/88` rows with `76/76` renders passed
- [x] no public API, route, cache identity, dynamic ratio, other character,
  Loophole, or Chorus surface entered the admission
- [x] Batch 31.68 proved no unchanged-renderer `2x..4x` overlap can satisfy
  the shared-map and mandatory-probe rules
- [x] the lower overlap remains paused without rejecting or changing either
  admitted renderer
- [x] Batch 31.69 froze one complete source-backed `LayeredCloud` renderer and
  Rule 11 evidence authority without adding candidate DSP to `main`
- [x] Batch 31.70 passed two conformance rounds but produced an invalid green
  synthetic receipt; no comparator, listening, or quality decision opened
- [x] Batch 31.71 audited the complete retained checkpoint without executing
  DSP and found construction, runner, structural, synthetic, receipt, and
  listening ownership incomplete
- [x] one fresh docs-first `AuditedLayeredCloud` identity is justified; the
  failed checkpoint cannot be repaired or rerun and no source/output transfers
- [x] Batch 31.72 froze one complete replacement renderer and executable
  evidence brief with every Batch 31.71 gap closed before implementation
- [x] Batch 31.73 compiled the source-clean candidate and passed construction
  `1/1`, then proved the frozen occupancy result `22` unreachable under the
  same brief's strict support and `D<=20H` rules
- [x] the second Cloud evidence-integrity failure closed the family before
  structural admission, acoustic identity, synthetic output, or listening
- [x] no candidate DSP, harness, fixture, comparator render, listening output,
  production surface, Loophole, or Chorus change entered `main`
- [x] Batch 31.74 narrowed current executable creative coverage to exact
  fixed `4x`, `8x`, and `16x` neutral `Dream`
- [x] `16x..100x` moved from the active queue to deferred research without
  converting Cloud's invalid evidence into an acoustic judgment
- [x] the prior complete-owner audit found no materially different replacement
  ready for another implementation brief
- [x] Batch 31.74 made no implementation, routing, public-control, cache,
  dynamic-ratio, or cross-repo batch ready
- [x] Batch 31.75 froze one minimal semantic public boundary without changing
  code
- [x] the public boundary exposes only exact `4x`/`8x`/`16x` target length,
  fixed `Dream`, and admitted `space`
- [x] the public boundary fixes the admitted seed and leaves seed/reroll,
  motion, detail, pitch, dynamic ratio, routing, cache, and tiers absent
- [x] Batch 31.76 is ready as wrapper-only implementation with byte-identical
  acoustic files and output required
- [x] Batch 31.76 admitted that wrapper with byte-identical acoustic source and
  output, complete public error mapping, and all retained gates green

## Next Task

Operator intent checkpoint: choose whether to freeze this lane and move to
another `g10` priority, name one Signal consumer for docs-first integration
authority, or explicitly reopen source-backed research above `16x`. No
implementation batch is ready before that choice.
