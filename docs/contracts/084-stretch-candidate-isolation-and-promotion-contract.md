# 084 Stretch Candidate Isolation And Promotion Contract

Status: active
Owner: dsp
Created: 2026-07-19
Related contracts: `046`, `048`, `049`
Related roadmap: `g10.030`

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

This contract closes when one successor passes fixed-ratio mono and linked
stereo promotion, replaces the frozen baseline without residual research
scaffolding, and has an explicit follow-on decision for dynamic ratios and
RealtimePreview.

## Current Candidate

Batch 30.3 rejected `SourceAnchoredMultiresolutionPhaseField` after its centered
detector committed an impulse before event refinement or short-window support
could reach the source event. Batch 30.4 replaces that topology with the
event-first scheduler and source/output one-owner window invariant frozen in
`docs/architecture/offline-time-stretch-successor-brief.md`.

If the replacement fails event placement or replicas, Rule 7 closes this
multiresolution phase-vocoder family. It does not authorize another detector or
window variant.

## Next Task

Implement `docs/architecture/offline-time-stretch-successor-brief.md` in one
disposable `g10.030` Batch 30.5 worktree. Stop after structural and synthetic
gates. Keep candidate code and evidence surfaces outside `main` until
admission.
