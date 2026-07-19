# Offline Time-Stretch Successor Brief

Status: rejected in Batch 30.5; multiresolution phase-vocoder family closed
Owner: dsp
Updated: 2026-07-19
Contract: `084`
Roadmap: `g10.030`, Batch 30.5

## Decision

Build one `EventSealedMultiresolutionPhaseField` renderer. It is fixed-ratio,
offline-only, native-channel, and Signal-owned. It combines:

- one sample-domain event guide finalized ahead of synthesis
- one event-conforming source lattice and absolute source/output map
- three simultaneous, frequency-exclusive STFT scales
- event-sealed nonstationary windows with one source owner per event sample
- one coherent tonal phase field with dormant track state
- one atomic linked-channel phase commit

This replaces the rejected `SourceAnchoredMultiresolutionPhaseField` brief.
The Batch 30.3 failure remains in
`docs/logs/2026-07/19-g10-030-stretch-candidate-rejection.md` and Git history.
No part of that implementation survives as code or hidden review surface.

The architecture uses public specimens only for broad structure: simultaneous
frequency ownership, preliminary horizontal phase followed by coherent
vertical correction, and native-channel relationship ownership. Signal does
not copy external constants, tables, masks, thresholds, or control flow.

## Batch 30.5 Outcome

The brief is retained as rejected architecture evidence. No renderer was
implemented.

Its frozen refinement rule cannot satisfy its frozen structural gate. For an
isolated impulse at source sample `e`,

`mean(E[n..n+16))-mean(E[n-16..n))`

has the same positive maximum for every `n` from `e-15` through `e`. The
required earlier-sample tie break therefore commits `e-15`. The structural
gate requires the token at `e` exactly. All `256` phases of the `H=256`
lattice fail with the same `-15` offset.

Changing the refinement span, tie break, token sample, or gate would create
the prohibited third detector/window variant. Batch 30.5 stopped before DSP
implementation, deleted its untouched worktree and branch, retained the
production baseline, and closed this multiresolution phase-vocoder successor
family under Contract `084` Rule 7.

## Why The First Candidate Failed

The first guide used centered `2048`-sample spectra to trigger an event, then
searched only `[x_k-H/2,x_k+H/2)`. An impulse entered the centered spectrum
hundreds of samples before it entered that search interval or the same-centre
short window. The one-shot token committed a silent location and disarmed.

The replacement changes three connected owners together:

1. boundary-aligned detector blocks locate the event inside the complete block
   that produced novelty
2. the scheduler inserts that exact event sample as an analysis centre before
   synthesis reaches it
3. every non-anchor analysis window is zero at that event sample, so the
   source attack cannot enter several inverse frames

Changing only a detector threshold, refinement radius, or reset tick is not
authorized.

## Supported Domain

- mono or linked stereo; one or two channels
- fixed output/input ratio from `0.5` through `2.0`, inclusive
- finite `f32` input
- empty input returns empty output
- identity returns the input byte-exact
- whole-buffer offline execution
- borrowed input and required output buffer excluded from working-memory caps

Dynamic ratio, pitch composition, cache routing, RealtimePreview, and product
routing remain on the frozen production behavior until fixed-ratio admission
and product review. Unsupported requests fail before rendering; they do not
clamp or select another candidate topology.

## Event Guide

The event guide runs ahead of synthesis with fixed lookahead. It never derives
event time from a centered synthesis spectrum.

Constants are frozen:

- detector boundary interval `D=64` source samples
- adjacent pre/post block length `A=256`
- detector FFT length `512`
- energy-rise refinement span `16`
- flux history `32` detector boundaries
- candidate merge distance `D`
- log-magnitude floor `2^-80`

At detector boundary `n_j=jD`, each native channel analyzes adjacent blocks
`[n_j-A,n_j)` and `[n_j,n_j+A)`. Evaluate every integer `j` whose post block
intersects `[0,L)`, plus the final `A`-sample guard needed to finalize the last
token. Exterior samples are zero. Both blocks use a periodic square-root Hann
and zero-pad to `512` before transformation.

Per-bin joint magnitude is the square root of summed channel energy. Four fixed
detector bands cover `[0,1/16)`, `[1/16,1/4)`, `[1/4,1/2)`, and
`[1/2,Nyquist]`. For each band, positive log-magnitude flux is the sum of:

`max(0, log(max(M_post,2^-80))-log(max(M_pre,2^-80)))`

A band produces a candidate when flux exceeds `median+3*MAD` of its preceding
`32` detector values and post-block energy exceeds pre-block energy. Missing
history is zero. Ties use the lower frequency band.

The candidate sample is not `n_j`. Search the source-supported part of the
full post block, `[n_j,n_j+A) intersect [0,L)`, for the maximum positive
joint-energy rise:

`mean(E[n..n+16))-mean(E[n-16..n))`

Ties choose the earlier sample. This search interval must contain the future
content that caused the adjacent-block novelty; it may not be clipped to a
synthesis hop.

Candidates within `D` samples form one token. The candidate with greatest
summed excess above its band thresholds owns the token; ties choose the earlier
sample. Band masks union. A token finalizes only after the detector cursor is
at least `A` samples beyond it, so later observations cannot move it. Separate
tokens are at least `D` samples apart. Detector histories continue through
silence; exact joint silence emits no token and clears candidate state.

The guide cursor stays at least `4096/2+A` source samples ahead of the next
synthesis centre. Finalized tokens live in a fixed ring and expire after every
analysis window that can intersect them has been synthesized.

## Source Map And Event-Conforming Scheduler

For input length `L`, requested ratio `r`, and target `T=round(L*r)`, the
effective ratio is `q=T/L`. Only `q` owns mapping.

Start with regular centres `kH`, `H=256`, inside `[0,L)`. Form one sorted source
lattice from:

- regular centres
- boundary anchors `0` and `L-1`
- every finalized event sample `e`

Remove a regular centre strictly nearer than `H/2` to an event anchor. Retain
all event anchors, then sort and deduplicate. Equal boundary and event anchors
are one centre. Event merging guarantees a minimum event/event increment of
`D`; retained regular centres are at least `H/2` from an event. The lattice is
strictly increasing.

Each source centre `x_i` maps independently:

`y_i=round(q*x_i)`

The phase engine receives actual source increment `x_i-x_(i-1)` and actual
synthesis increment `y_i-y_(i-1)`. It never reconstructs a constant hop.
Boundary anchors and event anchors therefore own exact projected samples with
at most `0.5` sample rounding error.

Ownership is fixed:

- detector owns finalized event samples and event bands
- planner owns `L`, `T`, `q`, lattice construction, lookahead, and capacities
- analyzer owns native-channel spectra at one lattice centre
- material guide owns scale crossover and terminal atom state
- phase field owns track continuity and all channel phases
- synthesizer owns exclusive masks, inverse transforms, normalization rings,
  and exact crop emission

No downstream stage may move an event, create another token, repair a crest,
or project stereo after synthesis.

## Event-Sealed Analysis And Synthesis Windows

Three scales share every source and synthesis centre:

| Scale | FFT/window | Base frequency owner |
| --- | ---: | --- |
| long | `4096` | low |
| middle | `2048` | middle and crossover guidance |
| short | `1024` | high and event bands at anchors |

Each scale starts with matching centered periodic square-root Hann shapes.
Boundary and event anchors then create deterministic source- and output-domain
seals. Let `H_y=max(1,round(qH))` and
`seal(d,R)=sin^2((pi/2)*clamp(d/R,0,1))`.

For source anchor `a` and non-anchor source centre `x_i`, multiply the analysis
window by:

- `seal(a-n,H)` for `n<=a`, and zero for `n>a`, when `x_i<a`
- `seal(n-a,H)` for `n>=a`, and zero for `n<a`, when `x_i>a`

For mapped anchor `z=round(q*a)` and synthesis centre `y_i`, apply the same
rule to the synthesis window in output coordinates with radius `H_y`.

Each multiplier is zero on the far side of its anchor, zero at the anchor, and
one at its domain radius on the frame's own side. If several seals intersect a
window, multiply them in increasing anchor order. The analysis frame centred
at `a` and synthesis frame centred at `z` are unsealed relative to that anchor,
but remain sealed against other anchors.

Consequences are structural:

- the anchor frame has weight `1` at its source event sample
- every other analysis frame has weight `0` at that source sample
- every other synthesis frame has weight `0` at its mapped output sample
- the event sample enters one analysis frame, not several overlapping frames
- dense events remain separate anchors and seal each other
- overlap normalization, not a waveform crossfade, restores surrounding
  non-event content

This is the anti-replica owner. No attack layer, duplicated read, unity island,
post-render suppression, crest repair, or tail repair is allowed.

## Frequency Ownership And Material States

Base crossover state comes from the joint middle-scale magnitude at the current
event-conforming centre.

- initialize low and high crossovers at `1/16` and `1/3` of Nyquist
- search `[1/32,3/32]` and `[1/4,5/12]` for the lowest-energy local minimum
- retain the prior bin when no minimum exists
- require the desired bin continuously for at least `3H` source samples
- move a committed crossover at most `ceil(delta_x/H)` middle-scale bins
- break energy ties toward the lower bin

Long owns below low, middle owns `[low,high)`, and short owns high and above.
Ownership is sampled in normalized frequency and remains exhaustive and
non-overlapping.

At an event anchor, every normalized frequency inside a token's event bands is
removed from its base owner and assigned to short for that anchor only. The
same source event cannot occur in another frame because of the window seal.
All scales still analyze and update phase state at every centre, including
unowned frequencies.

Each atom commits one state in this order:

1. `Silence`: exact joint zero; emit zero and clear its active phase state.
2. `AttackAnchor`: current centre owns a token and its band; short-scale phase
   resets from current native-channel analysis at `round(q*e)`.
3. `TonalLocked`: a qualified local peak track owns the atom.
4. `ResidualUnlocked`: ordinary instantaneous-frequency recurrence.

After an anchor, ordinary and tonal recurrence continue from the committed
anchor phase. The token cannot reset again. Another attack requires another
finalized event sample at least `D` samples away.

## Tonal Tracks And Coherent Phase

Every scale and channel keeps its own peak topology. DC and Nyquist use
one-sided maxima and remain explicitly real. Interior peaks use strict local
maxima. A non-zero peak qualifies after two consecutive centres and at least
`H` accumulated source age.

Track assignment is monotonic by smallest predicted-bin distance. Maximum
movement is `ceil(2*delta_x/H)` bins, minimum one. Ties choose the lower current
bin, then the lower predecessor bin. Assignment is linear in bin count.

Each track stores current and preceding bin, instantaneous frequency, output
phase, age in source samples, dormant source age, and active state. A missing
peak remains dormant until source age exceeds that scale's window length. Its
phase advances at its last instantaneous frequency but emits no magnitude. A
peak inside the allowed movement bound may reactivate it; otherwise a new
track starts from current analysis phase. Storage is one fixed slot per
non-negative bin.

Horizontal prediction uses actual source and synthesis increments. For a
qualified reference-channel peak, coherent correction combines three complex
predictions read from the same preliminary horizontal field:

- the peak's horizontal recurrence
- the first qualified peak one, then four bins below
- the first qualified peak one, then four bins above

Neighbour predictions transport current analysis-phase difference and use
joint current magnitude as weight. Missing observations contribute zero. If
the complex sum magnitude is below one quarter of total weight, horizontal
recurrence wins. Otherwise its argument owns the peak. Current peak-relative
analysis offsets place the rest of the peak region. The two directional reads
commit together; traversal order cannot feed itself. Residual atoms never use
vertical correction. No random diffusion is allowed.

## Linked-Channel Ownership

Stereo remains native left/right. There is no mid/side transform, independent
mono pair, channel sum, or post-render image projection.

Detector tokens, seals, lattice centres, crossovers, frequency ownership,
terminal material state, and traversal order are shared. Per-channel spectra,
magnitudes, and peak tracks remain native.

For each active atom, greatest current magnitude selects the reference
channel. Equal magnitudes select lexicographically greater `(real,imaginary)`
analysis coefficient. Identical coefficients are equivalent. The reference
computes terminal phase. Every non-silent peer keeps its own magnitude and
current same-atom relation:

`phase_peer_out=phase_reference_out+wrap(phase_peer_in-phase_reference_in)`

An exactly silent peer emits zero. Exact joint silence clears every channel's
atom phase together. The all-channel commit is atomic. Duplicate, mono-parity,
silent-peer, and channel-swap mechanics remain hard gates at `1e-6`.

## Boundaries, Normalization, And Exact Length

- boundary anchors are always present at `0` and `L-1`
- source support outside `[0,L)` is zero
- every scale uses the matching source/output sealed window pair
- DC and Nyquist are real; negative frequencies use explicit conjugate symmetry
- one output and one normalization ring exist per scale and channel
- normalization accumulates paired analysis-window times synthesis-window
  weight
- divide only above `1e-12`; otherwise emit zero for that scale
- sum normalized scale samples once per channel
- crop signed accumulation exactly to `[0,T)`
- no resize fill, reflection, wrap, hidden extension, fade, limiter, loudness
  correction, endpoint envelope, or boundary repair

Any uncovered active crop sample, non-finite normalization value, or event
sample without exactly one non-zero analysis and mapped synthesis anchor owner
is a structural failure.

## Memory, Determinism, And Cost

All storage allocates before the first detector boundary.

Main transform state keeps the prior candidate bounds for `C<=2`, three scale
supports, per-channel spectra and phase histories, fixed per-bin tracks, and
per-scale/channel overlap rings. Added fixed state is:

- `2*C*512` complex detector spectra
- `4*32` scalar band-flux history values
- `2*(4096+A)` scalar detector/sample-energy ring values
- at most `76` event-token records, derived from
  `ceil((4096+2A)/D)+4`
- at most `96` pending source/synthesis-centre records

No frame, detector, token, or history collection grows with source duration.
No allocation occurs in detector, frame, FFT, track, mask, inverse, overlap,
or flush loops. Capacity overflow, length arithmetic overflow, unsupported
channel count, or unsupported ratio fails before affected audio is emitted.

The regular path performs one detector FFT pair per channel every `D` samples
and three forward/inverse scale pairs per retained centre. Event insertion can
add at most one centre per `D` source samples. Worst-case work is bounded and
linear in duration with FFT factors; typical event density is reported, never
used as an acceptance proxy.

Fixed traversal, explicit tie rules, real-bin rules, finite guards, and no
random state make repeated output sample-bit identical on the same supported
target.

## Fixed Admission Sequence

Failure stops the sequence.

### 1. Structural Gate

Run mono and stereo at `0.5`, `0.75`, `1.0`, `1.5`, and `2.0` over empty,
one-sample, sub-window, exact-window, silence, isolated and dense impulses at
every `H` phase, boundary-active, tone, deterministic-noise, and mixed inputs.

Pass requires:

- identity sample-bit equality
- exact `round(L*r)` output length
- finite output and normalization state
- strictly monotonic source lattice and map error at most `0.5` sample
- every declared impulse token at its source sample; soft-onset refinement
  within `D`
- exactly one non-zero analysis-window owner at every event sample and one
  non-zero synthesis-window owner at its mapped output sample
- no uncovered active crop sample
- OfflineHighQuality integrity: `0.5` frame length, `7 dB` active endpoints,
  `0` added silence, and `6 dB` positive peak-growth limits
- sample-bit equality across two runs
- working slabs within the frozen caps and identical capacities for matched
  five-second and sixty-second renders
- duplicate stereo equals mono duplication, silent peer remains silent, and
  channel swap swaps output, each within `1e-6`

### 2. Synthetic Quality Gate

Use retained pitch, event-placement, dense-replica, transient-detail,
tonal-texture, and linked-stereo rows at `0.75`, `1.5`, and `2.0`.

Pass requires:

- pitch error at most `5` cents on every isolated tone and chord partial
- every declared source event matched once within `256` samples of projection
- no unmatched secondary event above `-24 dB` of its source event inside one
  long-window projected guard
- transient crest growth at most `3 dB` and no event worse than the frozen
  baseline at either renderer's worst event
- no tone, chord, or pad row worse than baseline for unsupported-bin energy,
  spectral residual, fast spectral movement, or short-time envelope movement
- at least half of `1.5x` and `2.0x` tonal rows strictly improve both
  unsupported-bin energy and fast spectral movement
- all stereo mechanics pass; every calibrated image, interchannel-phase,
  delay, and local-relation row is no worse than baseline

Moving a defect to another row fails. Aggregate wins do not hide a row.

### 3. Long-Form Mono Blind Gate

Use one at-least-five-second mono source from percussion, bass, vocals,
pads/sustains, and full mix at `0.75`, `1.5`, and `2.0`: fifteen rows. Create
concealed, RMS-matched pairs under the retained `0.95` peak ceiling for:

- candidate versus frozen Signal baseline
- candidate versus pinned external reference

Freeze notes on transient definition and placement, tonal stability,
grain/ringing, blur/replicas, boundaries, loudness, and preference before the
key opens.

Pass requires no unusable row; candidate preferred or tied against baseline on
all rows with at least five preferences and one per ratio; candidate preferred
or tied against external on at least ten rows with no family losing at both
long-expansion ratios. Listening remains promotion authority.

### 4. Linked-Stereo Admission

After mono passes, repeat the families and ratios in linked stereo. Require
structural evidence and concealed listening by an eligible listener independent
of the mono operator. The current operator's one-ear hearing does not satisfy
this gate. No image pull, width pumping, centre shift, channel echo, or one-
sided transient damage may be worse than baseline. Missing an eligible listener
blocks promotion; it never waives the gate.

### 5. Product Review

Only fixed-ratio mono and stereo promotion may open dynamic ratio, pitch
composition, cache identity, artifact, and product routing review.
RealtimePreview and render-plane source fill remain separate and paused.

## Rejection And Cleanup

Any structural, synthetic, mono-listening, or stereo-listening miss rejects the
whole candidate. Record the dominant cause and stopped gate once. Delete the
disposable worktree and branch, including renderer, fixtures, reports, renders,
and instrumentation. Do not repair a row, sweep detector constants, add a
selector, or retain a review mode.

If this second complete candidate fails event placement or replicas, Contract
`084` Rule 7 closes this multiresolution phase-vocoder family. Reassessment must
choose another renderer family or retain the frozen baseline.

## Minimal Admission Surface

If every fixed-ratio gate passes, merge only:

- one private renderer module behind existing fixed-ratio OfflineHighQuality
  mono and linked-stereo calls
- structural and promoted synthetic regression tests
- the minimum comparator/listening ratio update
- deliberate cache engine-version and promotion-receipt changes

Temporary diagnostics, generated audio, candidate names, and worktree-only
fixtures are deleted. The displaced renderer may remain only as an explicit
fallback for unsupported ratios and unreviewed dynamic/pitch paths.

## Remaining Risks

- event seals may color sustained content or produce modulation around false
  event tokens
- inserted event centres may create variable-lattice sidebands
- hard frequency crossovers may color polyphonic material
- coherent vertical correction may trade grain for tonal mutation
- same-atom peer relation may still lose local stereo relationships after
  inverse overlap-add
- detector FFT cost and worst-case event-centre density may exceed practical
  offline targets despite bounded execution

These are whole-renderer risks. The fixed gates judge them together.

## Next Task

Use the `g10.030` architecture checkpoint to decide whether to close the
stretch program on the competitive frozen baseline or commission one complete
successor from a different renderer family. Do not modify this rejected brief
or start another phase-vocoder variant.
