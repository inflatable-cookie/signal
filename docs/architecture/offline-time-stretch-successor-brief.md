# Offline Time-Stretch Successor Brief

Status: rejected at Batch 30.3 synthetic gate
Owner: dsp
Updated: 2026-07-19
Contract: `084`
Roadmap: `g10.030`, Batch 30.3

## Decision

Build one `SourceAnchoredMultiresolutionPhaseField` renderer. It is a fixed-
ratio, offline-only, native-channel phase vocoder with three simultaneous STFT
scales, one absolute source/output map, one shared material state map, and one
atomic all-channel phase commit.

This is the only Batch 30.3 candidate. It is not a menu and does not authorize
separate detector, window, phase, stereo, crossover, or tail variants.

## Batch 30.3 Outcome

The isolated implementation passed the structural gate and isolated-tone
pitch rows. It failed the first anti-replica row at `0.75x`.

The centered middle-scale detector committed an impulse onset `896` source
samples early: source centre `7424`, refined event `7296`, actual event `8192`.
The fixed `H/2` refinement interval and same-centre short-scale reassignment
could not reach the actual event. The rendered primary landed `128` output
samples late and a second peak landed `257` samples after the projection at
amplitude `0.17113242`; the `-24 dB` ceiling was `0.063095726`.

This is an architecture-level detector/scheduler alignment failure. Contract
`084` rejects the candidate. The implementation, tests, and instrumentation
were deleted with the disposable worktree. This brief remains rejection
evidence, not implementation authority.

The candidate combines three supported architecture lessons:

- Rubber Band: simultaneous frequency-owned scales and material guidance
  control one renderer; they are not independent full-band outputs.
- Signalsmith: horizontal prediction precedes vertical correction, and one
  content-selected channel phase decision preserves each peer's current
  analysis-relative relation.
- Signal's rejected work: full-band layer sums, post-hoc stereo projection,
  independent branch crossfades, raw per-bin material switching, and segmented
  event timelines do not return.

No external implementation, constants, tables, masks, or control flow enter
Signal. The candidate is Signal-owned and uses no production dependency.

## Supported Candidate Domain

- mono or linked stereo; one or two channels
- fixed output/input ratios from `0.5` through `2.0`, inclusive
- finite `f32` input; empty input returns empty output without entering the
  renderer
- identity is a byte-exact copy
- whole-buffer offline output with duration-independent working state apart
  from the required output buffer

Requests outside that domain fail before rendering in the candidate worktree.
They do not silently clamp, switch topology, or widen Batch 30.3. Dynamic
ratio, pitch composition, RealtimePreview, cache routing, and product routing
remain on the frozen production behavior until the fixed-ratio candidate is
promoted and reviewed.

## Transform And Frequency Ownership

All scales use centered periodic square-root Hann analysis and synthesis
windows:

| Scale | FFT/window | Base ownership |
| --- | ---: | --- |
| long | `4096` | low frequencies |
| middle | `2048` | middle frequencies and full-band guidance |
| short | `1024` | high frequencies and one-shot attack reassignment |

The common source analysis lattice is `H=256` samples. Every scale analyzes
the same source centres. No scale runs an independent scheduler.

Base crossover centres are `1/16` and `1/3` of Nyquist and initialize at the
nearest middle-scale bins. For each control tick:

1. form a three-frame temporal median of the joint middle-scale magnitude,
   zero-filling missing earlier guard frames
2. choose the lowest-energy local minimum in `[1/32, 3/32]` of Nyquist for the
   low crossover
3. choose the lowest-energy local minimum in `[1/4, 5/12]` of Nyquist for the
   high crossover
4. retain the previous crossover if no local minimum exists
5. require the new bin for three consecutive ticks before committing it
6. move a committed crossover by at most one middle-scale bin per tick

Ties choose the lower-frequency bin. The lower scale owns bins below a
crossover; the upper scale owns the crossover bin and bins above it. Ownership
is evaluated in normalized frequency, then sampled on each scale. It is
exhaustive and non-overlapping: one normalized frequency has one synthesis
owner.

An `Attack` bin is reassigned to the short scale for its single event-commit
tick, regardless of base band. The same bin is removed from its base owner for
that tick. This is the only time-dependent resolution change. There is no
full-band short render, layer blend, waveform crossfade, or duplicated attack.

Each scale inverse-transforms only its owned coefficients. Scale outputs sum
inside the same channel after per-scale overlap normalization.

## Source/Output Map And Scheduler

For input length `L`, requested ratio `r`, and exact target `T=round(L*r)`, the
effective ratio is `q=T/L`. This `q`, not the caller's unrounded ratio, owns the
render.

Analysis centres are `x_k=kH` for every integer `k` whose long window intersects
the source. Synthesis centres are calculated from the absolute map:

`y_k=round(q*x_k)`

Negative centres are valid guard centres. Source reads outside `[0,L)` are
zero. Output accumulation uses signed guarded coordinates and crops `[0,T)`.
The scheduler processes all centres whose long synthesis window intersects the
crop, then stops. It never pads the requested crop to satisfy coverage.

The phase engine receives the actual source increment `H` and actual synthesis
increment `y_k-y_(k-1)`. It never reconstructs a constant rounded hop. The map
is monotonic; a source event at sample `e` owns output sample `round(q*e)` with
at most `0.5` sample of projection error.

Ownership is explicit:

- the planner owns `L`, `T`, `q`, guarded centre bounds, and scale capacities
- the analyzer owns native-channel spectra at one source centre
- the guide owns crossover, event, and terminal material decisions
- the phase field owns all channel phases and track state for that tick
- the synthesizer owns masks, inverse transforms, overlap rings, normalization,
  and exact crop emission

No later stage may move an event, choose another scale, or repair a channel
relationship.

## Material Guide And Transient State Machine

Guidance is computed from native-channel middle-scale spectra. Joint magnitude
is the square root of summed channel energy, so opposite-polarity content
cannot cancel.

The guide maintains eight preceding ticks of joint magnitude and positive log-
magnitude flux. A band onset occurs when both conditions hold:

- current band flux is greater than `median + 3*MAD` of those eight ticks
- current band energy is greater than the preceding tick

The first eight ticks compare against zero history. Four bands are used:
`[0, low)`, `[low, Nyquist/4)`, `[Nyquist/4, high)`, and
`[high, Nyquist]`, using the committed crossovers. A band cannot re-arm until
its flux falls to or below its rolling median.

For an armed onset, the event sample is the maximum positive rise in summed
channel sample energy inside `[x_k-H/2, x_k+H/2)`. Ties choose the earlier
sample. The event token is shared by every scale and channel.

The attack spectrum applies the analysis-time linear phase ramp for `e-x_k`
and the inverse synthesis-time ramp for `round(q*e)-y_k`. This reassigns the
short-scale attack to the mapped event sample rather than merely resetting the
nearest frame centre. Both ramps are common to all channels.

Each owned atom commits exactly one terminal state in this order:

1. `Silence`: joint magnitude is exactly zero; emit zero and clear active
   synthesis ownership.
2. `Attack`: the atom's positive log-magnitude flux exceeds its frame median
   plus `3*MAD` and its band has an armed event; reassign it to the short scale
   and reset the linked phase at the event tick.
3. `TonalLocked`: the atom belongs to a qualified current peak track; use the
   tracked coherent phase field.
4. `ResidualUnlocked`: use ordinary instantaneous-frequency recurrence.

An event token commits once. Overlapping frames after the event use
`TonalLocked` or `ResidualUnlocked`; they cannot reset again from the same
onset. The detector must re-arm before another `Attack`. This is the replica
policy. There are no unity-ratio islands, duplicated source reads, attack
layers, or post-render crest repair.

## Tonal Tracks And Coherent Phase

Every scale and channel detects strict local magnitude maxima. A peak is
qualified when it is non-zero and has existed for two consecutive ticks.
Tracks are assigned monotonically by smallest predicted-bin distance, with a
maximum move of two bins per tick. Ties choose the lower current bin, then the
lower predecessor bin. Assignment is linear in bin count; no all-pairs search
is allowed.

Every scale updates its analysis and track state even when it does not own
synthesis at that frequency. Crossover movement and one-tick attack
reassignment therefore reveal already-current state; they never copy phase
between scales or initialize a new branch at the ownership boundary.

Each track stores current bin, preceding bin, instantaneous frequency, output
phase, age, dormant age, and active state. A missing peak becomes `Dormant` for
one scale-window span, `N_s/H` ticks. Its phase continues to advance at the
last instantaneous frequency but it emits no magnitude. A peak within two bins
may reactivate that state. A later or farther peak is a new track initialized
from current analysis phase. Track storage is one fixed slot per non-negative
frequency bin; overflow is impossible inside the declared domain.

For `TonalLocked`, build three complex phase predictions for the selected
reference channel:

- ordinary horizontal tracked-peak recurrence
- a low-to-high prediction from the first qualified peak found one, then four,
  bins below
- a high-to-low prediction from the first qualified peak found one, then four,
  bins above

Neighbour predictions transport the neighbour's predicted phase by the
current analysis-phase difference. Each prediction is weighted by its joint
current magnitude. Missing observations contribute zero. The output phase is
the argument of their complex sum. If the sum magnitude is below one quarter
of total prediction weight, use horizontal recurrence. Current peak-relative
analysis offsets then place the remaining region atoms.

The two directional passes read the same preliminary horizontal field and
commit together. Neither pass feeds the other. This prevents traversal order
from becoming hidden phase state. `ResidualUnlocked` bypasses vertical
prediction and keeps its ordinary recurrence. No random high-ratio diffusion
is allowed.

## Linked-Channel Ownership

Stereo is analyzed and synthesized in native left/right channels. There is no
mid/side transform and no independent mono renderer.

Scale ownership, crossovers, event tokens, terminal material state, traversal,
and synthesis centres are shared. At every active atom, select the reference
channel by greatest current magnitude. Equal magnitudes select the
lexicographically greater `(real, imaginary)` analysis coefficient; identical
coefficients are equivalent.

The reference channel computes the selected terminal phase. Every peer keeps
its own magnitude and current same-atom analysis relation:

`phase_peer_out = phase_reference_out + wrap(phase_peer_in-phase_reference_in)`

An exactly silent peer remains zero. The phase commit for all channels is
atomic. No peer magnitude, channel sum, post-hoc image projection, or
channel-local branch decision is allowed. Duplicate, mono-parity, silent-peer,
and channel-swap mechanics are hard gates. Polarity and gain equivariance are
reported diagnostics, not exact professional-renderer invariants.

## Windowing, Boundaries, And Exact Length

- periodic square-root Hann analysis and synthesis at all scales
- real DC and Nyquist bins; explicit conjugate symmetry before inverse FFT
- zero exterior support; no reflection, wrap, source-tail anchor, or hidden
  content extension
- one normalization ring per scale and channel accumulating analysis-window
  times synthesis-window weight
- divide only where the scale normalization exceeds `1e-12`; otherwise emit
  zero for that scale
- sum normalized scale samples once inside each channel
- crop exactly `[0,T)`; no resize fill, tail fade, endpoint envelope, limiter,
  loudness correction, or boundary repair

Any uncovered active crop sample is a structural failure. Identity bypasses
the transform and returns the input byte-exact.

## Memory, Determinism, And Cost

Let `C<=2`, `S=4096+2048+1024`, and
`B=(4096/2+1)+(2048/2+1)+(1024/2+1)`.
Candidate working allocation, excluding the borrowed input and required
`C*T` output samples, may not exceed:

- `2*C*S` complex transform/spectrum values
- `6*C*B` scalar magnitude/phase/history values
- `4*C*B` fixed peak-track records
- `4*B` joint guidance/state values
- `6*C*(4096+2H)` scalar overlap and normalization-ring values
- `16H` scalar planner and event scratch values

All slabs allocate before the first frame. No allocation occurs in the frame
loop. No collection or history grows with duration. A request that exceeds the
declared channel, ratio, length-arithmetic, or slab bounds fails before audio
is rendered.

Each tick performs three forward and three inverse FFTs per channel plus linear
guidance, track, phase, mask, and overlap scans. Work is
`O(C*sum(N_s log N_s))` per tick and `O(C*S)` state. Peak assignment must remain
monotonic and linear.

Execution uses fixed traversal order, explicit tie rules, finite guards, and
no random state. Two runs on the same supported target must be sample-bit
identical. CPU and peak working heap are reported, not promotion proxies.

## Fixed Admission Sequence

Failure at any step stops the sequence.

### 1. Structural Gate

Run mono and stereo at `0.5`, `0.75`, `1.0`, `1.5`, and `2.0` over empty,
one-sample, shorter-than-short-window, exact-window, silence, impulse, boundary-
active, tone, deterministic-noise, and mixed inputs.

Pass requires:

- identity sample-bit equality
- exact `round(L*r)` output length
- finite output and normalization state
- monotonic map and event projection error at most `0.5` sample
- no uncovered active crop samples
- OfflineHighQuality integrity: at most `0.5` frame length drift, `7 dB`
  active-endpoint RMS change, `0` added silence frames, and `6 dB` positive
  peak growth
- sample-bit equality across two runs
- working allocation within the formula above and identical working-slab
  counts and capacities for matched five-second and sixty-second renders
- duplicate stereo equals mono duplication, silent peer remains silent, and
  channel swap swaps output, each within `1e-6`

### 2. Synthetic Quality Gate

Use the retained pitch, event-placement, dense-replica, transient-detail,
tonal-texture, and linked-stereo measurements at `0.75`, `1.5`, and `2.0`.

Pass requires:

- pitch error at most `5` cents on every isolated tone and chord partial
- every declared source event matched once within `256` samples of
  `round(q*event)`
- no unmatched secondary event above `-24 dB` of its source event inside one
  long-window projected guard
- transient crest growth at most `3 dB` and no event worse than the frozen
  baseline at either the baseline's worst event or the candidate's worst event
- no tone, chord, or pad row worse than the frozen baseline for unsupported-
  bin energy, spectral residual, fast spectral movement, or short-time
  envelope movement
- at least half of the `1.5x` and `2.0x` tonal rows strictly improve both
  unsupported-bin energy and fast spectral movement
- all linked-stereo structural mechanics pass and every calibrated image,
  interchannel-phase, delay, and local-relation row is no worse than the
  frozen baseline

Moving a crest, replica, or tonal defect to another row fails. Aggregate wins
cannot hide a row-complete regression.

### 3. Long-Form Mono Blind Gate

Use one at-least-five-second mono source for each retained family: percussion,
bass, vocals, pads/sustains, and full mix. Use ratios `0.75`, `1.5`, and `2.0`:
fifteen source/ratio rows. For every row create two concealed, RMS-matched
pairs with the existing `0.95` peak ceiling:

- candidate versus frozen Signal baseline
- candidate versus the pinned external reference

The current production note checker names historical ratios
`0.75/1.25/1.5`. Contract `084` supersedes `1.25` with `2.0` for candidate
admission because long expansion is the audible gap. Batch 30.2 changes no
harness code; the disposable worktree owns this private pack. Admission may
update the retained checker only if the candidate passes.

Notes must cover transient definition and placement, tonal stability,
grain/ringing, blur/replicas, boundaries, loudness, and preference before the
key opens.

Pass requires:

- no row marked unusable for blur, doubled attack, micro-echo, stutter, tonal
  loss, pre-ringing, crest pop, boundary discontinuity, or arbitrary loudness
- candidate preferred or tied against baseline on all fifteen rows, with a
  preference on at least five and at least one preference at each ratio
- candidate preferred or tied against the external reference on at least ten
  rows, with no family losing at both `1.5x` and `2.0x`

Listening is the authority. Objective results do not override a failed blind
gate.

### 4. Linked-Stereo Admission

After mono passes, render the same families and ratios from linked-stereo
sources. Repeat the structural stereo measurements, then require concealed
listening by an eligible listener independent of the mono operator. The current
operator's one-ear hearing does not satisfy this gate.

Pass requires no image pull, width pumping, centre shift, channel echo, or
one-sided transient damage judged worse than baseline, plus the same
preferred-or-tied rule against baseline on all fifteen rows. If no eligible
listener is available, promotion remains blocked; the gate is not waived.

### 5. Product Review

Only after fixed-ratio mono and linked stereo pass may work review dynamic
ratio, pitch composition, cache identity, artifacts, and product routing.
RealtimePreview and render-plane source fill remain separate and paused.

## Rejection And Cleanup

Any structural, synthetic, mono-listening, or stereo-listening miss rejects the
complete candidate. Record one dominant cause and the stopped gate in the
current-month log. Do not repair a row, sweep a threshold, add a selector, or
retain the implementation as a review mode.

Delete the disposable worktree and branch, including private fixtures, report
modes, renders, and instrumentation. `main` remains on the frozen baseline.
Two complete candidates failing for the same dominant cause force an
architecture reassessment under Contract `084`.

## Minimal Admission Surface

If every fixed-ratio gate passes, merge only:

- one private renderer module behind existing `OfflineHighQuality` fixed-ratio
  mono and linked-stereo calls
- structural and promoted synthetic regression tests
- the minimum comparator/listening ratio update needed to guard promotion
- the deliberate cache engine-version and promotion-receipt change

The candidate does not become a public review path. Temporary diagnostics,
generated audio, candidate names, and worktree-only fixtures are deleted.
The displaced phase vocoder may remain only as an explicit internal fallback
for unsupported ratios and unreviewed dynamic/pitch product paths. The product
review must either promote those paths onto the successor or keep the fallback
explicit; it may not silently mix engines.

## Remaining Risks

- hard exclusive crossovers may color or modulate material despite valley and
  hysteresis control
- one-tick short-scale attack reassignment may still soften low-frequency
  attacks or create a scale-transition crest
- the coherent vertical phase field may trade grain for tonal mutation
- same-atom peer relation may not preserve every local stereo waveform
  relationship after inverse overlap-add
- the fixed `0.5..2.0` candidate domain may prove too narrow for later product
  requirements

These are complete-renderer risks. They are judged through the fixed gates,
not split into advance experiments.

## Next Task

Create one disposable Batch 30.3 branch or worktree from the Batch 30.2 commit.
Implement this renderer exactly, keep all candidate-only surfaces there, and
stop after the structural and synthetic gates decide whether listening audio
may be generated.
