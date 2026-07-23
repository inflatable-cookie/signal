# Cyclic Time-Stretch Source Architecture

Status: reviewed; centred compressed-anchor behavior selected
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

Batch 32.2 ran as evidence-only work under ignored `target/` and disposable
external build state. It added no Signal DSP, private module, report mode,
fixture, feature, or public API to `main`.

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

## Batch 32.2 Receipt

### Envelope

- sample rate: `44100 Hz`
- ratios: exact `2x`, `4x`, and `8x`
- observation cycles: `5 ms`, `48 ms`, and `90 ms`
- inputs: eleven synthetic sources and five retained musical families
- ReaReaRea: `48` rows, including new musical `2x` and synthetic
  `2x`/`4x`/`8x`
- Potenza slow-anchor: `144` rows
- SickoCV repeat/jump: `144` rows
- Sonic period insertion: `9` specialist rows

The cycle values are short, source-default-region, and long observation
anchors. They are not Signal candidate constants.

The Potenza probe was checked directly against the pinned `TimeStretch.h`.
Its maximum absolute difference over an `8000`-frame verifier was
`5.9604645e-8`.

### Integrity

| Path | Rows | Finite | Exact requested frames | Boundary result |
| --- | ---: | ---: | ---: | --- |
| Potenza slow-anchor | `144` | `144` | `144` | naturally reaches zero; tail support varies with ratio and cycle |
| SickoCV repeat/jump | `144` | `144` | `144` | exact crop can retain a non-zero final sample |
| REAPER ReaReaRea | `48` | `48` | `48` | exact item length with its retained item fade |
| Sonic period insertion | `9` | `9` | `0` | error spans `-1950..+17906` frames |

Sonic remains useful automatic-period evidence. It is not an exact-length
full-mix owner.

### Impulse Grammar

ReaReaRea produced:

| Ratio | Replicas around first/second event | Median spacing | Mean mapped-centre error |
| --- | --- | ---: | --- |
| `2x` | `3 / 3` | `1058.5` frames | `-176.3 / -88.3` frames |
| `4x` | `7 / 6` | `1588` frames | `-529.1 / +529.2` frames |
| `8x` | `14 / 12` | `1852` frames | `-308.7 / +308.8` frames |

The source schedules are measurably different:

- SickoCV emits exactly `ratio` replicas per event. At one fixed cycle its
  output spacing stays fixed: `220`, `2117`, or `3969` frames.
- Potenza emits more overlapping appearances than the integer repeat count,
  increasing approximately with ratio. At the long observation cycle its
  spacing rises from `1191.5` to `1787` to `2084` frames across `2x`, `4x`,
  and `8x`.
- ReaReaRea scales replica count like compressed-anchor overlap, not simple
  whole-cycle repetition. Unlike the raw Potenza probe, it centres the
  replicas around the mapped event.

This distinguishes the two families. It does not yet freeze Signal's map,
centering law, or cycle.

### Tail Support

On `M001` and `M005`, ReaReaRea's inactive tail is stable across material:

| Ratio | ReaReaRea | Potenza `48 ms` | Potenza `90 ms` | SickoCV `90 ms` |
| --- | ---: | ---: | ---: | ---: |
| `2x` | `30.6..30.7 ms` | `10.8 ms` | `29.5 ms` | `0 ms`, non-zero crop |
| `4x` | `130.6..130.7 ms` | `60.5 ms` | `131.5 ms` | `0 ms`, non-zero crop |
| `8x` | `306.9..307.1 ms` | `170.5 ms` | `355.8 ms` | `0 ms`, non-zero crop |

This is further compressed-anchor evidence. It also proves that boundary
placement and tail ownership belong in the complete scheduler, not in a final
cosmetic fade.

### Pitch Diagnostic

On the `110 Hz` tone, ReaReaRea's strongest measured component was:

| Ratio | Dominant component | Input-relative delta |
| --- | ---: | ---: |
| `2x` | `117.500 Hz` | `+114.189 cents` |
| `4x` | `110.833 Hz` | `+13.066 cents` |
| `8x` | `117.917 Hz` | `+120.317 cents` |

The old absolute `15`-cent gate is incompatible with the comparator at `2x`
and `8x`. Pitch remains a mandatory diagnostic, not a character rejection
threshold.

### Linked Stereo

Potenza, SickoCV, and ReaReaRea all preserved duplicate, anti-phase,
thirteen-sample-delay, and `-6 dB` right-channel probes under one shared
schedule. Worst balance error was:

- Potenza: `0.001247 dB`
- SickoCV: `0.001183 dB`
- ReaReaRea: `0.001508 dB`

Shared cycle ownership is sufficient for these linked mechanics. Independent
listening remains the authority for image quality after a complete candidate
exists.

### Reproducibility

Commands:

- ReaReaRea:
  `/Applications/REAPER.app/Contents/MacOS/REAPER -newinst -nosplash
  -renderproject <project.rpp>`
- Sonic: pinned `sonic -q -s <1/ratio> <input.wav> <output.wav>`
- disposable runner:
  `python3 target/cyclic-forensics-32-2/run_forensics.py all`

Hashes:

| Receipt item | SHA-256 |
| --- | --- |
| disposable runner | `31aaee1006b62fcd115c321c2f3a34afb9f31b4fc3ea9a005140947cfae3f704` |
| source manifest | `3209e4aba4828966d1cfe8bb4a7639af03b2b059c19ab1d27409194bbb7bb54d` |
| REAPER project manifest | `84f3722b971296812cf7040f3b4a7134b76588476eab2cf1dff86971f11ee515` |
| full measurement manifest | `33e1137747bc7bb8ffab53b595fc4ebdee31b89a7b6bca3dae90c82bf1c48684` |
| Potenza `144`-row output-hash group | `0330def15ca8a2394a460737f955e95eacc586e4b4d1d239aa0093c0e5b33356` |
| SickoCV `144`-row output-hash group | `238e998a98de84b325a753f318eb8f81e7ae269295bb7a6964c6fc553016da71` |
| ReaReaRea `48`-row output-hash group | `5bb7b55456065d8f3d69c7229abc117eacb9280cf298a779b634598a19663e11` |
| Sonic `9`-row output-hash group | `4d548321086ef8ac3d2616ee09dba728b12a4decf7d5a109460377931d85a889` |

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

Batch 32.2 resolves only the evidence questions:

1. Compressed-anchor and repeat/jump grammars are distinguishable.
2. Fixed-cycle musical sufficiency remains a listening question.
3. Sonic supports specialist automatic-period guidance, not a default
   full-mix owner.
4. Neither raw source join law owns Signal's centred exact boundary scheduler.
5. The old absolute pitch ceiling is invalid for this target.
6. One shared clock preserves the tested stereo relations.

## Behavioral Synthesis

Batch 32.3 selects centred compressed-anchor Cyclic behavior:

- one monotonic ideal map owns event centres and order
- one fixed render-wide user cycle owns metallic-to-tremolo character
- forward native-rate waveform reads and compressed source-anchor progress own
  duration
- ratio-dependent replica clusters are centred around mapped events
- one linked schedule owns every channel and the exact boundary crop

Raw repeat/jump is not the primary target. Automatic cycle selection is
optional later assistance for strong-fundamental material, not `INTELL` and
not part of the first candidate.

The old absolute pitch ceiling is removed from Cyclic admission. Exact length,
finiteness, determinism, bounded state, boundaries, commanded-replica
ownership, and linked mechanics remain hard. Pitch, replica spacing, cadence,
crest, level, and tail support are complete comparator-relative diagnostics.
Concealed musical listening decides character and usefulness.

The canonical decision is
[Offline Creative Cyclic Behavioral Synthesis](../../architecture/offline-creative-cyclic-behavioral-synthesis.md).

Batch 32.4 freezes the selected implementation authority:
[Offline Creative CenteredCompressedAnchorCyclic Renderer Brief](../../architecture/offline-creative-centered-compressed-anchor-cyclic-brief.md).

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

Execute `g10.032` Batch 32.5 only. Implement and conform the frozen isolated
Cyclic candidate, bind its comparator manifest, and stop before acoustic
execution. No candidate DSP enters `main`.
