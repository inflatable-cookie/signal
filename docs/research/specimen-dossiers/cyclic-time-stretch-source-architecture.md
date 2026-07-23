# Cyclic Time-Stretch Source Architecture

Status: source survey complete; executable forensics ready
Owner: dsp
Updated: 2026-07-23
Contract: `085`
Roadmap: `g10.032`

## Question

What mechanisms create the useful Akai-style cyclic stretch heard in
ReaReaRea, and which complete source-backed path could support a Signal-owned
renderer from `2x` through `8x`?

This is a clean-room architecture study. GPL source supplies evidence, not
production expression, constants, thresholds, tables, or control flow.
Proprietary systems supply behavior and product vocabulary only.

## Product Target

The target is deliberate cyclic colour:

- repeated waveform detail
- metallic or ring-like motion at short cycles
- slower tremolo or echo-like motion at long cycles
- useful output on percussion, bass, vocal, sustain, and full mix
- continuous fixed expansion above `1x`, with mandatory `2x`, `4x`, and `8x`
- one linked source clock for mono or stereo

It is not transparent stretch and does not replace `OfflineHighQuality`.
It is not neutral `Dream`, spectral diffusion, or a random grain cloud.

The operator already found ReaReaRea useful through about `8x`. `16x` remains
context and a likely rejection boundary.

## Evidence Classes

| Class | What it can establish | What it cannot establish |
| --- | --- | --- |
| original hardware manuals | product modes, controls, intended material, artifact direction | executable schedule, exact window, exact arithmetic |
| source-available implementations | complete state and scheduling families | historical Akai identity or production-ready Signal constants |
| proprietary comparators | audible target and behavioral response | implementation ownership |
| papers and general TSM libraries | mechanism vocabulary and known limits | direct Akai character without matching behavior |

No source below is a production dependency.

## Original Akai Product Evidence

### S950

The S950 manual exposes one `D-TIME` control and mono/poly material choice.
It states:

- shorter `D-TIME` produces more metallic output
- longer `D-TIME` produces a tremolo-like result
- `AUTO-D` attempts to select a suitable value
- mono mode targets a single tone
- poly mode targets vocals, drum loops, and other complex material
- expansion reaches `999%`

This is strong evidence that cycle duration owns the audible modulation rate
and that the historical product separated fixed user intent from optional
material guidance. It does not reveal the exact schedule.

### S1000 and S2000

The later manuals make the algorithm split explicit:

- `CYCLIC` uses one fixed cycle length over the selected source
- `AUTO` may propose that cycle length
- `INTELL` varies decisions according to source content
- `CYCLIC` is recommended for single instruments
- `INTELL` is recommended for speech, vocals, drum loops, and other complex
  material
- `QUALITY` and crossfade width belong to `INTELL`, not fixed `CYCLIC`
- the supported factor is `25%` through `2000%`

This matters for Signal. Fixed-cycle Akai colour and correlation-guided
continuity are two algorithms, not successive repairs to one renderer.

## Modern Behavioral References

### ReaReaRea

REAPER `7.69` remains the primary audible target. Its implementation is
proprietary. The retained corpus has five musical families at `4x`, `8x`, and
`16x`. A matching `2x` row and synthetic mechanism probes are still missing.

ReaReaRea establishes whether Signal reaches the intended cyclic region. It
cannot select a source schedule by itself.

### Akaizer

Akaizer `2.5` exposes:

- `CLASSIC` and `REVISED` algorithms
- time factor from `25%` through `2000%`
- cycle length
- independent transpose

The current release is paid and closed source. It is optional behavioral
context, not a mandatory comparator or architecture source.

### TAL-Sampler

TAL-Sampler independently preserves the Akai split:

- `CYCLIC`: Akai-style stretch; density controls grain size
- `INTELL`: correlation stretch; density controls correlation windows

It is proprietary behavioral evidence. Its product model supports one
important Signal conclusion: a user-facing cycle control is musically real,
while correlation belongs to a different character.

## Source-Available Complete Paths

### Potenza Time Stretch

Pinned revision:
`ddb44a8f949b3f49320932e1d2e997b3a02149bb`.
Licence: GPL-3.0, clean-room evidence only.

Complete path:

1. Maintain one slow ideal source cursor.
2. Run a foreground grain forward at unit playback rate.
3. Before that grain ends, launch a second forward unit-rate grain.
4. Offset the new grain anchor by a ratio-dependent fraction of cycle length.
5. Crossfade the two reads.
6. Alternate grain ownership indefinitely.

The stretch ratio changes anchor advance, not local grain playback rate.
Cycle length controls the repeated waveform support. The implementation has no
material detector, similarity search, offline exact-length owner, or linked
stereo policy.

This was the only direct source studied before Signal's first cyclic
candidate.

### SickoCV `sickoSampler2`

Pinned revision:
`b3504b2c41f22454379823faa980606d35d83f70`.
Licence: GPL-3.0-or-later, clean-room evidence only.

Complete path:

1. Play one source cursor forward at the native playback increment.
2. Count one fixed cycle duration.
3. At a cycle boundary, retain the outgoing cursor as the fade source.
4. Jump the main cursor backward for expansion or forward for compression.
5. Crossfade briefly from the retained cursor into the jumped cursor.
6. Repeat whole cycles according to the integer part of the stretch factor.
7. Use a fractional cycle displacement to realize non-integer factors.

The public module exposes `1%..999%` stretch and `1..99 ms` cycle size.
Stereo reads share the same voice cursor, cycle count, jump, and fade.
The implementation warns that extreme settings may alter pitch or produce
chorus/echo.

This is not Potenza's continuously alternating slow-anchor schedule. It is an
explicit repeated-cycle clock with discontinuous cursor correction. That
schedule was not isolated in either rejected Signal candidate.

### Sonic

Pinned revision:
`b93885dcb70aae50c6f76b0fe4e0868f029a077e`.
Licence: Apache-2.0.

Sonic estimates a pitch period, then inserts or removes complete periods with
overlap for speech-rate change. It supplies:

- one material-derived cycle owner
- cycle insertion and removal rather than a free similarity lag
- temporal error accumulation with bounded correction
- a maintained high-factor speech implementation

It does not supply a full-mix Akai renderer. Its pitch estimate is channel-
local and speech-oriented. It is evidence for a possible mono/strong-
fundamental `AUTO` cycle assistant, not the default linked full-mix path.

### WSOLA and SoundTouch

WSOLA chooses a source segment near an expected position by waveform
similarity. SoundTouch supplies a maintained multichannel implementation of
the same broad family.

This aligns with Akai `INTELL`, not fixed `CYCLIC`. It may reduce joins, but it
also changes which source cycle is repeated. Echo, drift, and one-lag
polyphonic compromise remain known risks.

Signal's rejected `SimilarityAlignedCyclic` was an incomplete realization of
this family. Its structural failure does not reject fixed cyclic, but another
search shortlist or score would be a direct repair and is not authorized.

## Mechanism Decomposition

Every studied cyclic system can be described through six independent owners:

| Owner | Choices found in evidence | Audible consequence |
| --- | --- | --- |
| source-progress clock | slow continuous cursor; repeat/jump cursor; adaptive segment selection | event placement and repetition grammar |
| cycle owner | fixed user length; automatic pitch/loop estimate; material-adaptive search | metallic cadence, tremolo rate, or continuity |
| ratio accumulator | compressed anchor advance; integer repeats plus fractional correction; period insertion | exact duration and cycle distribution |
| local read | forward unit rate; pitch-compensated read | pitch retention and optional transpose |
| join law | two-grain crossfade; short jump crossfade; correlation-aligned overlap | clicks, combing, pitch displacement, and softness |
| channel ownership | caller-local; shared cursor; shared search | stereo stability |

Interpolation, bit depth, bandwidth, and output filtering add sampler colour.
They are separable from the cyclic scheduling effect and must not be baked into
the first renderer without comparator evidence.

## Reinterpretation Of Signal's Prior Candidates

### `CyclicGrain`

The candidate proved that one regular two-grain ideal-map renderer can be
bounded, deterministic, linked, and exact length. It failed only the first
synthetic tone:

- target: `110 Hz`
- measured: `111.328 Hz`
- error: `20.778` cents
- frozen ceiling: `15` cents

No ReaReaRea pitch delta or musical comparison ran. For an intentionally
metallic cyclic effect, this receipt proves a mechanism and gate mismatch; it
does not prove that the output missed the target.

The candidate also translated Potenza's slow-anchor family. It did not test
SickoCV's explicit repeat/jump clock.

### `SimilarityAlignedCyclic`

This candidate moved to the `INTELL` family. Its coarse shortlist could hide
an exact between-grid continuation from refinement, so structural admission
correctly rejected that frozen renderer.

That failure says nothing about the fixed `CYCLIC` target. It also does not
authorize search repair.

### Closure Correction

The old "no third family" conclusion is superseded at research level by:

- original Akai evidence that `CYCLIC` and `INTELL` are separate modes
- the unstudied SickoCV repeat/jump schedule
- the absence of comparator-relative cyclic pitch evidence

This does not revive deleted code or select a new renderer. It makes one deep
forensic study eligible.

## Provisional Product Vocabulary

The minimum honest control surface is:

| Control | Meaning | Status |
| --- | --- | --- |
| duration | exact target duration or ratio | required |
| character | explicit `Cyclic` | required |
| cycle | short metallic motion to long tremolo/echo motion | required candidate |
| cycle source | manual or automatic material guidance | research only |
| transpose | pitch composition separate from duration | later |

Do not force `Dream`'s `motion`, `detail`, or `space` controls onto Cyclic.
`cycle` may eventually map behind a semantic label, but hiding the primary
historical control before listening would erase the effect's useful range.

## Executable Forensics

The next batch is evidence-only. Use ignored `target/` or disposable external
build state. Add no Signal DSP, private module, report mode, fixture, feature,
or public API to `main`.

### Specimens

- pinned Potenza
- pinned SickoCV cycle schedule in a standalone source-faithful probe
- pinned Sonic for strong-fundamental contrast
- REAPER `7.69` ReaReaRea
- optional Akaizer or TAL only if already lawfully available

### Inputs

- isolated impulse and two-impulse spacing
- `110 Hz` and one high tone
- two-tone and triad
- impulse train
- deterministic noise burst and amplitude-stepped noise
- the five retained long-form musical families
- duplicate, anti-phase, delayed, and unequal-level stereo

### Matrix

- exact `2x`, `4x`, and `8x`
- fixed cycle durations spanning short metallic through long tremolo regions
- neutral playback rate and no added transpose
- exact source and level policy shared across systems

Do not sweep candidate constants. This matrix identifies mechanism response.

### Measurements

- exact output length and boundary support
- mapped impulse locations and repeat count
- cycle cadence from envelope and spectral sidebands
- dominant pitch and comparator-relative pitch delta
- transition width and discontinuity
- whole and local RMS distribution
- left/right relation under shared source clocks

Metrics diagnose the schedule. They do not rank musical quality.

## Decision Gate

Executable forensics must answer:

1. Does ReaReaRea follow a compressed-anchor or repeat/jump grammar closely
   enough to distinguish them?
2. Is fixed cycle duration sufficient across the five musical families?
3. Does any automatic cycle estimate improve useful settings without turning
   the character into `INTELL`?
4. Which join law preserves the target colour without uncommanded clicks?
5. Is the prior `15`-cent absolute pitch ceiling incompatible with the
   comparator?
6. Can one shared cycle clock preserve stereo without per-channel repair?

Only then may architecture select one complete renderer. If the evidence
cannot distinguish a schedule, stop at research rather than choose by
preference.

## Sources

- [Akai S950 operator manual](https://manualzilla.com/doc/7440972/akai-s950-operator-s-manual)
- [Akai S1000 version 2.0 operator manual](https://theatrecrafts.com/archive/documents/s1000_v2_0_manual.pdf)
- [Akai S2000 version 1.30 operator manual](https://www.polynominal.com/akai-s2000/akai-s2000-manual.pdf)
- [Akaizer](https://the-akaizer-project.blogspot.com/)
- [TAL-Sampler manual](https://helpdesk-listgo.tal-software.com/downloads/docs/TAL-Sampler-UserManual.pdf)
- [REAPER](https://www.reaper.fm/)
- [Potenza source at the pinned revision](https://github.com/dar-io-p/potenza-time-stretch/tree/ddb44a8f949b3f49320932e1d2e997b3a02149bb)
- [SickoCV source at the pinned revision](https://github.com/sickozell/SickoCV/tree/b3504b2c41f22454379823faa980606d35d83f70)
- [Sonic source at the pinned revision](https://github.com/waywardgeek/sonic/tree/b93885dcb70aae50c6f76b0fe4e0868f029a077e)
- [Verhelst and Roelands, WSOLA](https://doi.org/10.21437/Eurospeech.1993-59)
- [SoundTouch algorithm notes](https://soundtouch.surina.net/README.html)

## Next Task

Execute `g10.032` Batch 32.2 only. Build the ignored source-faithful forensic
matrix and capture missing ReaReaRea `2x` plus synthetic probes. Do not write a
Signal renderer or freeze candidate constants.
