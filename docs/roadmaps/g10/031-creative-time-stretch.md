# 031 - Creative Time-Stretch

Status: active; Batch 31.8 ready
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

- centers `8x` and studies `4x`, `8x`, and `16x`
- presents stable intent controls rather than algorithm parameters
- routes smoothly from coherent slow motion to spectral dream and later cloud
- preserves exact duration, determinism, linked stereo, and bounded memory
- stays separate from `OfflineHighQuality` and RealtimePreview

## Non-Goals

- no transparent successor reopening
- no RealtimePreview or audio-thread work
- no Loophole or Chorus UI implementation
- no external production dependency
- no cyclic renderer in the first candidate
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

Status: ready

Implement the frozen `ContinuousExcitationComplexRelation` brief exactly once
in a disposable worktree. Run the coefficient-level relation proof first, then
the exact failed common-polarity renderer row. Stop and delete on any miss. The
crest row remains closed until every structural control passes.

## Later Batches

Closed until a diffusive owner passes:

- minimal production admission
- coherent/diffusive overlap
- `LayeredCloud` study and candidate
- diffusive/cloud overlap
- dynamic-ratio state continuity
- cache and product-path review
- optional cyclic character
- `100x+` texture/freeze range

## Completion Gate

- [x] one product architecture and governing contract exist
- [x] comparator target character is frozen
- [x] two candidate briefs were rejected and one final complete brief is frozen
- [ ] one isolated diffusive candidate passes structural and synthetic gates
- [ ] long-form mono listening passes at `4x`, `8x`, and `16x`
- [ ] linked-stereo mechanics and independent listening pass
- [ ] one overlap band is audibly continuous
- [ ] only minimal admitted product surface enters `main`

## Next Task

Execute Batch 31.8 only. Create one disposable worktree and implement the final
frozen brief without changing `main`. Run the relation proof and prior
common-polarity row first. Do not open the crest row before structural success,
or open `Cloud`, `Cyclic`, routing, dynamic ratio, cache, or product APIs.
