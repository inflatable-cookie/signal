# 031 - Creative Time-Stretch

Status: active; Batch 31.14 isolated similarity-aligned candidate ready
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

- centers `8x`; the current cyclic lane studies `2x`, `4x`, and `8x` with
  `16x` as a rejection boundary
- presents stable intent controls rather than algorithm parameters
- preserves a future route from coherent slow motion to spectral dream and
  later cloud without claiming unavailable owners
- first admits only an explicit cyclic character through `8x`
- preserves exact duration, determinism, linked stereo, and bounded memory
- stays separate from `OfflineHighQuality` and RealtimePreview

## Non-Goals

- no transparent successor reopening
- no RealtimePreview or audio-thread work
- no Loophole or Chorus UI implementation
- no external production dependency
- no spectral-router or cloud implementation while their owners are paused
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

Status: ready; disposable worktree only

Create sibling worktree `/Users/tom/Dev/projects/signal-candidate-31-14` on
branch `candidate/g10-031-similarity-aligned-cyclic` from the Batch 31.13 docs
commit. Implement the frozen brief once.

- add only the private six-file `creative_similarity_cyclic` family and one
  private `lib.rs` declaration
- run compile-only validation before admission
- run the complete structural gate
- run retained neutral `110 Hz` at `2x` first after structural admission
- stop on the first failure without correction or rerun
- open remaining synthetic rows, comparator capture, mono listening, `16x`
  rejection, and independent stereo only in the frozen order
- delete the entire worktree, branch, candidate code, tests, build state, and
  generated audio after any failure
- admit nothing to `main` unless every gate passes

## Later Batches

Closed pending the isolated similarity-aligned decision and separate reopening
evidence:

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
- [x] the range promise is narrowed honestly to explicit cyclic expansion
  through `8x`
- [x] one complete cyclic owner brief is frozen
- [x] one isolated cyclic candidate reached a recorded terminal decision
- [x] one materially different source-backed cyclic family is selected
- [x] one complete similarity-aligned cyclic brief is frozen
- [ ] one isolated cyclic candidate passes structural and synthetic gates
- [ ] long-form mono cyclic listening passes at `2x`, `4x`, and `8x`
- [ ] linked-stereo mechanics and independent listening pass
- [ ] `16x` remains unsupported unless separate evidence passes
- [ ] only the minimal cyclic product surface enters `main`

## Next Task

Execute Batch 31.14 only. Create
`/Users/tom/Dev/projects/signal-candidate-31-14` on
`candidate/g10-031-similarity-aligned-cyclic` from the Batch 31.13 docs commit,
implement the frozen `SimilarityAlignedCyclic` brief once, and stop at the
first failed gate. Keep core spectral characters, Cloud, automatic routing,
dynamic ratio, cache, and public APIs closed.
