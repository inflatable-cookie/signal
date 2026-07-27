# 084 Stretch Candidate Isolation And Promotion Contract

Status: closed without promotion; frozen baseline retained; defect correction
authorized 2026-07-27
Owner: dsp
Created: 2026-07-19
Updated: 2026-07-27
Related contracts: `046`, `048`, `049`
Related roadmap: `g10.030`, `g10.036`

## Purpose

Keep the competitive Signal stretch baseline small and stable while one
complete successor is developed and judged. This contract replaces Contract
082 as the authority for new OfflineHighQuality algorithm work. Contract 082
remains historical evidence for rejected families.

## Frozen Baseline

The retained production candidate is the current `2048/512` identity-locked
phase vocoder with expansion transient resets and the promoted compression and
expansion short-window selectors. Its mono, linked-stereo, pitch, dynamic-ratio,
cache, artifact, and RealtimePreview behavior must not change during successor
research.

The retained evidence surface is:

- byte-exact and structural regression tests
- Signal-versus-external objective rows for timing, transient placement and
  crest, tonal texture, formant boundary, render integrity, CPU, and heap
- level-matched long-form blind listening across percussion, bass, vocals,
  sustains, and full mixes

## Rules

### Rule 1: one complete candidate at a time

New work must describe and exercise one end-to-end renderer. Isolated
coefficient, detector, selector, window, tail, phase, or stereo experiments do
not enter the active roadmap unless they diagnose a failure observed in that
complete renderer.

### Rule 2: candidate work stays outside the production branch

Develop the successor in a disposable branch or worktree. Do not add its
modules, public types, hidden review methods, report modes, roadmap ledgers, or
test fixtures to the production branch before admission. Source study,
notebooks, generated renders, and temporary instrumentation stay outside the
shipping crate.

### Rule 3: architecture must cover the whole audible problem

The candidate brief must jointly own:

- one monotonic source-to-output time map
- transient detection, placement, phase treatment, and replica prevention
- sustained and polyphonic phase coherence without atonal ringing
- simultaneous resolution or equivalent material handling
- boundary continuity and exact target length
- shared stereo decisions and stable inter-channel relationships
- bounded deterministic offline execution

A proposal that leaves one of these to a later independent branch is not ready
for implementation.

### Rule 4: evidence runs in a fixed order

The candidate must pass:

1. identity, length, finiteness, boundary, determinism, and bounded-memory
   controls
2. synthetic pitch, event placement, replica, transient crest, tonal, and
   linked-stereo controls
3. the full long-form mono Signal/external blind pack at compression and long
   expansion ratios
4. independent stereo listening when an eligible listener is available
5. dynamic-ratio and product-path review only after fixed-ratio promotion

Failure at a stage stops later stages. A failed candidate is removed from the
candidate worktree rather than retained as another production review mode.

### Rule 5: listening is the promotion authority

Objective metrics diagnose failures and prevent known regressions. They do not
prove professional sound quality. Fixed-ratio promotion requires the candidate
to beat or tie the frozen Signal baseline consistently and be competitive with
the external reference on long-form material, especially at `1.5x` and `2.0x`.

Known audible priorities are:

- reduce long-stretch grain and subtle atonal ringing
- retain transient sharpness without visible or audible pop spikes
- keep transient timing stable
- avoid blur, doubled attacks, micro-echo, and tonal loss

### Rule 6: admission is a replacement, not an addition

When a candidate passes, merge only the renderer, the minimum diagnostics
needed to guard it, and its production regression tests. Remove displaced
algorithm code and temporary instrumentation in the same admission batch.
Update cache identity and promotion receipts deliberately.

### Rule 7: reassess before repeating a failure class

Two failures with the same dominant audible or structural cause trigger an
architecture reassessment. They do not authorize parameter sweeps or narrower
variants. If the complete candidate cannot expose a plausible corrective seam,
close it and choose a different architecture family.

### Rule 8: clean-room research remains allowed

Public source and papers may be studied for architecture, scheduling, state
ownership, and validation ideas. Signal must implement its own code. External
libraries remain comparators, not production dependencies.

## Completion

The preferred close is a successor passing fixed-ratio mono and linked-stereo
promotion and replacing the frozen baseline without research scaffolding. The
contract may also close without promotion after the required architecture
reassessment finds no different family that plausibly owns every gate. That
closure retains the baseline and requires explicit evidence before reopening.

## Current State

Batch 30.3 rejected `SourceAnchoredMultiresolutionPhaseField` after its centered
detector committed an impulse before event refinement or short-window support
could reach the source event. Batch 30.5 then rejected the frozen event-sealed
replacement before implementation: its required 16-sample energy-rise score
and earlier-sample tie break place every isolated impulse token at `e-15`, but
its structural gate requires `e` exactly.

The disposable Batch 30.5 worktree and branch were deleted without DSP or
harness changes. Rule 7 closes this multiresolution phase-vocoder family. No
current successor candidate exists, and another detector, window, scheduler,
or phase-vocoder variant is unauthorized.

Batch 30.7 completed the authorized non-phase-vocoder feasibility study.
WSOLA, direct subband sinusoidal synthesis, deterministic
sines/transients/noise, and learned waveform synthesis each fail at least one
complete-system boundary. The strongest source-backed topology, pinned SBSMS,
already failed Signal's mono, long-form objective, linked-stereo, and exact
mechanics evidence. No successor brief opens. The frozen production baseline
closes this contract.

## 2026-07-27 Defect Correction Amendment

The frozen-baseline clause exists to stop successor research from drifting the
comparison target. That research closed with this contract. The clause was
never scoped to that purpose in its own text, so it also blocks correcting
defects in the baseline itself.

### Rule 9: defect correction is authorized after successor closure

A measured defect in the retained baseline may be corrected while this contract
is closed, under these conditions:

- the defect is reproduced and recorded before any code changes
- the correction restores a behavior the renderer already promises. It does not
  add capability, tune a detector, change a window or selector threshold, or
  introduce a family
- the correction is classified under the Contract `046` correction classes, and
  an audible correction inside the retained `0.5x..4x` product range carries
  objective rows plus concealed listening before admission, exactly as Rule 5
  requires of a successor
- extension-class corrections prove byte-exact output over the range they do
  not affect

Correction work does not reopen successor research and does not authorize a new
candidate. Rules 1 through 8 continue to govern any renderer that is not a
defect correction.

### Rule 10: re-baselining a byte-exact regression owner

Byte-exact regression hashes pin the retained baseline. A correction that
changes output invalidates them by design.

A hash may be re-frozen only in the batch that changed the behavior, only for
the range the correction affects, and only with the objective rows and
listening evidence that justified the change recorded alongside the new value.
A hash may never be re-frozen to make an unexplained difference pass.

### Authorized correction set

`g10.036` is authorized to correct four measured defects: overlap coverage
above `0.75 * window_size` synthesis hop, dynamic-ratio segments shorter than
one window, missing mono seam treatment, and unbounded output allocation. No
other change to the retained baseline is authorized by this amendment.

## Next Task

No successor task remains. Execute `g10.036` Batch 36.2 under Rule 9. Reopen
successor research only when
`docs/architecture/offline-time-stretch-non-phase-vocoder-feasibility.md`
records new whole-system evidence satisfying its triggers.
