# Offline Creative Cyclic Behavioral Synthesis

Status: behavioral owner selected; complete renderer brief frozen
Owner: dsp
Updated: 2026-07-23
Contract: `085`
Roadmap: `g10.032`
Research:
`docs/research/specimen-dossiers/cyclic-time-stretch-source-architecture.md`

## Decision

Select one waveform-domain **centred compressed-anchor Cyclic** behavior for a
future complete renderer brief.

It owns:

- one strictly monotonic ideal source/output map
- one fixed render-wide user cycle
- forward native-rate local waveform reads
- source-anchor progress compressed relative to output progress
- ratio-dependent overlapping appearances around mapped source events
- one centred event ledger rather than a one-sided free-running cursor
- one shared schedule for every linked channel
- an exact-length boundary scheduler

Simple whole-cycle repeat/jump is rejected as the primary Signal target.
SickoCV proves that family is buildable, but its exact integer repeat count and
fixed output spacing do not match ReaReaRea's measured replica scaling.
Similarity search remains `INTELL`, not fixed `Cyclic`.

This decision selects behavior, not implementation. It freezes no cycle range,
default, launch interval, overlap count, window, interpolation, crossfade
width, boundary taper, threshold, or coefficient. Batch 32.4 must freeze those
once as one complete renderer.

## Behavioral Ownership

### Timeline

Target frames remain authoritative. One monotonic ideal map owns event order
and each source event's target centre.

The renderer may repeat waveform support around that centre. It may not create
a second event timeline, accumulate cursor drift, move later events according
to earlier waveform content, or reverse source-anchor order.

### Cycle And Ratio

`cycle` is one fixed render-wide material-support control. Shorter values move
toward metallic or ring-like motion. Longer values move toward tremolo or
echo-like motion.

Fixed cycle is sufficient to define the `Cyclic` product character. Musical
sufficiency across the retained source families remains a candidate listening
gate, not an unresolved product-mechanism choice.

Continuous expansion above `1x` through `8x` is the intended range. The ratio
accumulator changes compressed source-anchor progress and replica
distribution. It does not turn the renderer into an integer cycle repeater.
Mandatory admission points remain exact `2x`, `4x`, and `8x`. Exact `16x`
remains a typed pre-render rejection probe.

### Local Read And Joins

Every active local read moves forward at native rate. Cyclic colour comes from
the source offset and overlap between those reads, not resampling, phase
vocoder propagation, feedback, spectral correction, or random grain
placement.

One bounded overlap/crossfade law must join the reads. The complete brief owns
its exact geometry and proves complete target coverage. Neither Potenza's raw
one-sided schedule nor SickoCV's terminal crop is Signal boundary authority.

### Events And Replicas

Intentional replicas are commanded only by the frozen schedule. The future
brief must construct one auditable ledger containing every read that can cross
each synthetic event.

Pass behavior:

- replica clusters remain centred around the ideal mapped event
- replica population and spacing change coherently with ratio
- every energetic appearance belongs to the ledger
- source-event order remains monotonic

Failure behavior:

- a lobe outside the ledger
- a separate repair echo
- an event centre moved by accumulated cursor drift
- a lost source event
- a doubled attack outside the commanded cluster grammar

Comparator replica count and spacing are diagnostics. Signal need not
sample-match ReaReaRea.

### Linked Channels

Cycle, ratio state, anchors, event ledger, read positions, weights, boundary
decisions, and normalization are shared across linked channels. Each channel
contributes only its native samples.

Hard mechanics cover duplicate, swap, common polarity, anti-phase, delay, and
unequal-level relations. Channel-local cycle detection, per-channel anchors,
independent random motion, and post-hoc balance repair are forbidden.

## Product Controls

Minimum Cyclic UI vocabulary:

| Control | Meaning |
| --- | --- |
| `duration` | exact target duration or output/input ratio |
| `character=Cyclic` | explicit Akai-style cyclic intent |
| `cycle` | short metallic motion through long tremolo/echo motion |

`motion`, `detail`, and `space` are not Cyclic aliases. They remain absent
unless later product evidence gives them stable, non-duplicating meanings.
Transpose remains separate future pitch composition.

Automatic cycle selection is optional specialist assistance, not the default
renderer and not `INTELL`. A later `Auto` control may propose one fixed cycle
for mono or strong-fundamental material. It may not vary the cycle during a
render, use channel-local estimates, or block manual control. Sonic supports
that narrow possibility but supplies neither exact-length nor linked full-mix
ownership. Auto is outside the first candidate.

## Corrected Admission Model

Run gates in this order from one immutable Contract `085` Rule 11 acoustic
checkpoint.

### 1. Hard Structural And Integrity Gate

Pass requires:

- exact target length at every supported request
- finite output and exact-zero silence
- deterministic repeat from the same complete request
- duration-independent bounded working state and deterministic offline cost
- one strictly monotonic ideal map and source-anchor path
- forward native-rate local reads only
- complete normalized coverage of the target crop
- explicit exterior-boundary continuity, including the transition to and from
  exterior zero
- no non-zero terminal crop caused by unfinished overlap state
- every synthetic event appearance owned by the commanded ledger
- linked-channel shared decisions and preserved hard stereo relations
- typed rejection of compression, invalid requests, and exact `16x` before
  candidate audio allocation

A brief must freeze executable numeric bounds for discontinuity, level, and
stereo mechanics. Such bounds are hard only when they express an integrity
invariant or are calibrated against the matching retained comparator row.

### 2. Complete Synthetic Diagnostics

The immutable receipt must record, without using arbitrary character
thresholds:

- dominant tone and chord components plus input- and comparator-relative pitch
- event-centre error, replica count, spacing, and cluster extent
- cadence and spectral sidebands
- crest, active-support level, gap/dropout, and exterior discontinuity
- tail support and final inactive region
- linked whole-render and local balance

Missing or non-finite diagnostics reject the receipt. Finite pitch, replica,
cadence, crest, and tail differences do not reject Cyclic by themselves.
They locate the behavior for listening.

Hard synthetic failures remain: non-finite output, an uncommanded event,
unowned dropout, arbitrary gain step, invalid exterior transition, broken
linked relation, or a semantic inversion of `cycle`.

### 3. Concealed Long-Form Mono

Use the retained percussion, bass, vocal, sustain/pad, and full-mix sources at
exact `2x`, `4x`, and `8x`. Exact `4x` and `8x` are primary. Compare the
candidate with the matching ReaReaRea rows under the retained level policy.
Review one neutral cycle first, then one short and one long direction frozen
by the complete brief.

Pass requires:

- recognizable Cyclic character at every supported ratio
- musically useful output on a majority of all rows and a majority of the ten
  primary `4x`/`8x` rows
- no unusable row and no source family unusable at all three ratios
- short cycle audibly moves toward metallic/ring-like motion
- long cycle audibly moves toward tremolo/echo-like motion
- no uncontrolled click, dropout, arbitrary level change, source-obscuring
  buzz, or doubled attack outside the intended cyclic grammar

Signal need not mimic every comparator artifact. Operator listening is the
promotion authority.

### 4. Linked-Stereo Listening

Only after mono passes, review stable-centre, wide-sustain, transient, delayed,
unequal-level, and anti-phase material at `2x`, `4x`, and `8x`.

Pass requires no centre pull, one-sided texture, width pumping, detached echo,
channel-local cycle, or unusable row. Contract `085`'s default independent
listener rule remains in force. The scoped Dream waiver does not apply to
Cyclic.

## Prior Candidate Status

`CyclicGrain` and `SimilarityAlignedCyclic` remain rejected and deleted.

The new forensics do not erase `CyclicGrain`'s acoustic receipt. They prove
that its only acoustic stop used a product gate the target itself fails.
Contract `085` Rule 11 permits a fresh complete authority after an explicit
evidence-backed gate change addresses that failure class.

Batch 32.4 froze
[CenteredCompressedAnchorCyclic](./offline-creative-centered-compressed-anchor-cyclic-brief.md).
Its checkpoint became evidence-invalid. Batch 32.8 freezes the fresh
[AuditedCenteredCompressedAnchorCyclic](./offline-creative-audited-centered-compressed-anchor-cyclic-brief.md)
authority with the same renderer and complete one-shot evidence ownership. It
does not recover deleted source, tune the old implementation, sweep grain
constants, inherit Dream macros, or reuse the old absolute pitch ceiling.

## Remaining Risk

- no Signal renderer has yet proved that the centred schedule sounds useful
  across all five musical families
- the exact cycle range and default remain uncalibrated
- the exact overlap and boundary law may trade clicks against softened attacks
- short cycles may create excessive pitch displacement or source-obscuring
  buzz
- long cycles may become echo, stutter, or arbitrary envelope pumping
- speaker listening has not replaced Cyclic's independent stereo gate

These are brief and candidate gates. They are not reasons to reopen another
mechanism survey.

## Next Task

Execute `g10.032` Batch 32.9 only. Create the fresh isolated audited identity,
bind its comparator manifest, implement the unchanged frozen renderer and
one-shot evidence owners, and complete two structural conformance rounds. Do
not run acoustic owners before the checkpoint.
