# 041 - A18 Transient Pop Triage

Status: active; fix implemented and measured, blocked on listening admission
Owner: dsp
Created: 2026-08-05
Depends on: `g10.039`
Governing contracts: `docs/contracts/046-sample-domain-time-stretch-engine-contract.md`,
`docs/contracts/084-stretch-candidate-isolation-and-promotion-contract.md`
Vision tags: `DSP`, `STRETCH`, `QUALITY`

## Problem

`A18` is the last untriaged finding from the audit that opened this generation.
It was reported by listening, twice, in the same terms: low-mid pops on the
ticks.

- `g10.036` round: "C3-A mostly clean, but low end pops on the ticks / C3-B
  same, low end pops"
- `g10.039` revision 1: "D2-B clean except for a low-mid end pop on the ticks /
  D3-B same, clean but a little bit of low end pop on the ticks"

Revision 2 of the `g10.039` pack reported no significant difference between
sides, which does not separate "gone from both" from "present in both". The
finding has never had a mechanism, and every observation so far is comparative:
one renderer against another. Nothing has measured the artifact itself.

## Why This Needs A Lane

`A18` is the only original audit finding still open, and it is the only one that
has never been reproduced outside a listening pack. `A19`, `A21` and `A22`
closed this generation once each was measured directly; `A20` was relocated once
its assertion was understood. `A18` has resisted because comparative packs
cannot localise an artifact that may be present on both sides.

The working hypothesis carried since `g10.036` — that the pops share the
phase-restart mechanism behind the seam pulse — is **correct**. Batch 41.1
eliminated it on a metric that could not detect the artifact at any threshold;
Batch 41.2 built a metric proven to fire on an injected instance and confirmed
the mechanism.

## Goals

- [x] reproduce the pop in a measurement, not a listening pack
- [x] localise which layer introduces it — the phase-vocoder transient reset,
  not the chunked artifact path, the seam smoothers, or normalization
- [x] state a mechanism
- [x] add a permanent guard only if a mechanism is found

## Non-Goals

- no quality tuning before a mechanism exists
- no new listening pack until there is something specific to judge
- no changes to the RealtimePreview lane

## Batches

### Batch 41.1 - Eliminate The Phase-Reset Hypothesis

Status: **retracted**; the conclusion was wrong and the metric was blind

- [x] test whether the transient phase reset produces a low-band discontinuity
- [x] compare the shipped `IdentityLockedTransientReset` mode against
  `IdentityLocked` on material that actually trips the detector
- [x] record the result either way

The mechanism was plausible. `should_reset_phase_at_transient` sets every bin's
synthesis phase to the analysis phase when flux is at least `0.30` and the
energy ratio at least `1.20`. Low bins have long periods, so an identical phase
jump is a much larger waveform step there than at high frequencies — exactly the
shape of a low-frequency pop.

It is exactly what is happening. The measurement below said otherwise and was
wrong, because the metric could not see the artifact. Batch 41.2 proved that
directly. What follows is kept as the record of the error.

Measured on `80 Hz` plus a percussive attack every `250 ms`, `2048`/`512`
geometry, worst step in a `375 Hz` low band:

| ratio | transient reset | no reset | source |
| --- | --- | --- | --- |
| `1.5` | `0.02399` | `0.02065` | `0.02470` |
| `2.0` | `0.02449` | `0.02665` | `0.02470` |
| `3.0` | `0.02650` | `0.03209` | `0.02470` |

The reset makes the low-band step *smaller* at ratio `2.0` and `3.0`, not
larger. Every stretched value sits at or below the source's own `0.02470`.

Two process notes worth keeping.

The first material did not work. A `16`-sample click inside a `2048`-frame
analysis window contributes almost no spectral flux, so the detector never
fired and both modes produced byte-identical output — five decimal places
across three ratios. A probe that cannot trigger the mechanism it is testing
returns a confident null. The attack was replaced with a `25 ms` exponentially
decaying broadband burst, which is what "the ticks" in the listening material
actually were, and the two modes then diverged.

The metric found nothing anywhere, including in the source. Outlier counts were
`0` in every condition. Either the artifact is not in the raw vocoder path, or a
step in the low band is not what a "low-mid pop" is. Batch 41.2 must not assume
the first without testing the second.

### Batch 41.2 - Localise The Layer

Status: complete; mechanism found

- [x] validate the metric on material with a deliberately injected pop, so a
  null result means absence rather than blindness
- [x] identify the mechanism
- [ ] ~~render through each layer and compare~~ — unnecessary; the mechanism is
  in the vocoder itself and reproduces without the artifact path

#### The Batch 41.1 Metric Was Blind, Proven

Injecting a `pi`-radian phase flip into the sustained `80 Hz` tone — a complete
polarity inversion, unmistakable as a pop — moved Batch 41.1's worst-step metric
from `0.02379` to `0.02360`. Nothing.

The reason is structural rather than a tuning miss. A pop sits *on* a transient,
and the percussive attack at that instant is a larger low-band step than the pop
is. Worst-step always finds the attack. The metric could not have detected this
artifact at any threshold.

That is why Batch 41.1's null was worthless, and the roadmap's own risk entry
called it before the batch ran: "the metric is blind rather than the artifact
absent".

#### The Metric That Works

A low-mid pop *is* a phase discontinuity in the sustained low component. Measure
that directly: quadrature-demodulate at the carrier, low-pass, and take the
largest phase jump.

| condition | carrier phase jump |
| --- | --- |
| unprocessed source, clean | `0.130 rad` |
| unprocessed source, injected `pi` pop | `3.088 rad` |
| stretched `2.0`, transient reset | `2.752 rad` |
| stretched `2.0`, no reset | `0.142 rad` |

The metric fires on the injected pop, so a null from it means absence. The
shipped path measures `2.752 rad` — within `11%` of a deliberate polarity flip.

#### The Mechanism

`should_reset_phase_at_transient` sets every bin's synthesis phase to the
analysis phase. Across ratios, clean material:

| ratio | transient reset | no reset |
| --- | --- | --- |
| `0.75` | `0.128` | `0.128` |
| `1.00` | `0.130` | `0.130` |
| `1.25` | `0.164` | `0.143` |
| `1.50` | `0.478` | `0.135` |
| `2.00` | `2.752` | `0.142` |
| `3.00` | `0.595` | `0.251` |

`0.75` and `1.00` are identical because the reset only engages at ratio `1.0`
and above. The artifact appears from `1.25`, peaks near `2.0` at `19x` the
noise floor, and the no-reset column stays at the floor throughout.

This corroborates the listening reports rather than merely being consistent with
them: `D2`, where the pop was heard, was static ratio `2.0` — exactly where the
artifact peaks. `D3` ramped to `1.75`, between the `1.5` and `2.0` rows.

#### Guard

`tests/a18_transient_pop_guard.rs`. Two tests: one proves the metric fires on an
injected pop and runs always; one reproduces `A18` and is `#[ignore]`d with the
measured value in its reason, following the `g10.039` `G5` precedent. Un-ignored
it fails at ratio `1.5` with `0.478rad` against a `0.130rad` floor.

### Batch 41.3 - Fix And Admit

Status: candidate implemented and measured; listening admission outstanding

The reset exists for a reason — it stops transients smearing — so removing it
outright trades one artifact for another. The fix must keep the transient
behaviour and stop the low-frequency discontinuity.

- [x] reset phase only above a frequency where a phase jump is not a large
  waveform step, leaving low bins to propagate continuously
- [x] confirm the guard passes un-ignored across the ratio range
- [x] confirm transient smearing does not regress, measured against the corpus
  report's `TransientSmearFrames`
- [ ] listening admission under Contract `084` Rule 5 before adoption

#### The Candidate

`PhasePropagationMode::IdentityLockedTransientResetHighBand { crossover_bin }`
resets transient phase only above a crossover, leaving lower bins to propagate
continuously. Low-frequency content is *sustained through* a transient — a bass
note rings on while the attack happens — so resetting its phase destroys
continuity in something that never restarted. High bins are the transient, and
resetting those is what stops smearing.

The crossover is a fraction of Nyquist rather than a frequency, because the
stretch API is sample-rate agnostic all the way down. Frozen at `0.010`, which
is `240 Hz` at `48 kHz` and `220 Hz` at `44.1 kHz`.

It has no production constructor. Contract `084` Rule 2 keeps a candidate
isolated and Rule 5 makes listening the promotion authority, so the shipped
default is unchanged and nothing in the workspace can reach the new path.

#### The Artifact Is Gone

Carrier phase jump, clean material, ratio `2.0`:

| path | jump |
| --- | --- |
| shipped, reset every bin | `2.752 rad` |
| crossover `48 Hz` | `2.752 rad` |
| crossover `120 Hz` | `0.133 rad` |
| crossover `240 Hz` | `0.133 rad` |
| no reset at all | `0.142 rad` |

The `48 Hz` row is the control that confirms the mechanism rather than merely
fitting it. The probe tone is `80 Hz`, so at a `48 Hz` crossover the tone is
still *above* the line and still gets reset — and the result reproduces shipped
exactly. Protection only appears once the crossover rises above the content it
is meant to protect.

#### It Does Not Trade The Pop For Smearing

Measured with the corpus's own `measure_transient_smear` and production
policies, on its own material. Lower is better:

| ratio | shipped | no reset | `120 Hz` | `240 Hz` | `504 Hz` |
| --- | --- | --- | --- | --- | --- |
| `1.5` | `2.0` | `0.0` | `2.0` | `2.0` | `4.0` |
| `2.0` | `1.0` | `0.0` | `1.0` | `1.0` | `7.0` |
| `3.0` | `0.0` | `8.0` | `0.0` | `0.0` | `9.0` |

The frozen crossover matches shipped smear exactly at every ratio. `504 Hz`
regresses, because it starts protecting content that should be reset — so the
safe window is bounded on both sides, and `240 Hz` sits inside it.

Removing the reset outright is not the fix: it scores `8.0` at ratio `3.0`
against shipped's `0.0`. The reset earns its place; it was only ever applied too
widely.

Both facts are permanent tests in `benchmark.rs`, using the established
measurement rather than a proxy. The first proxy tried here — a `10-90%`
envelope rise time — disagreed with itself across ratios, which is how Batch
41.1 went wrong and was not going to be repeated.

#### What Admission Needs

A concealed pack per Contract `084` Rule 5, comparing shipped against the
candidate on material with sustained low content and transients. Objective
evidence says the artifact is gone at no measured cost, and that is not the same
as sounding better.

## Acceptance Criteria

The listeners heard the artifact through the offline *artifact* path, not the
raw vocoder that Batch 41.1 measured. That path adds chunking, both seam
smoothers, and the `1.0e-3` normalization gate that zeroes thin-coverage output.

- [ ] render identical material through the raw vocoder, the whole-buffer public
  path, and the chunked artifact path, and compare
- [ ] measure the normalization weight minimum across a render; a zeroed sample
  amid signal is a pop by definition
- [ ] check both seam smoothers as sources rather than assuming they only mask
- [ ] validate the metric on material with a deliberately injected pop, so a
  null result means absence rather than blindness

That last item is the lesson of Batch 41.1 and of this generation generally: a
gate or metric that has never been shown to fire proves nothing when it passes.

### Batch 41.3 - Mechanism Or Honest Closure

Status: blocked on Batch 41.2

- [ ] state the mechanism and add a permanent guard that reproduces it
- [ ] or record that it is not reproducible by measurement, and close `A18` as
  unreproduced rather than leaving it open indefinitely

## Acceptance Criteria

- [x] the artifact is reproduced in a measurement, with a metric proven to fire
  on an injected instance
- [x] the layer that introduces it is named — the phase-vocoder transient reset,
  not the artifact path
- [ ] `A18` is closed with a fix admitted by listening

## Risks and Mitigations

- Risk: the metric is blind rather than the artifact absent. Mitigation: Batch
  41.2 validates every metric against an injected pop before trusting a null.
- Risk: chasing a perceptual artifact that objective measurement cannot reach.
  Mitigation: Batch 41.3 permits honest closure as unreproduced, which is a
  valid outcome and better than an open finding nobody can act on.

## Evidence Requirements

- [x] the phase-reset A/B, with the material that actually trips the detector
- [ ] per-layer comparison on identical material
- [ ] a metric shown to fire on an injected pop

## Next Task

Build the listening pack. Everything objective is done and the remaining gate is
Rule 5.

The candidate resets transient phase only above `240 Hz` at `48 kHz`, expressed
as a fraction of Nyquist because the stretch API is sample-rate agnostic. It
takes the carrier phase jump at ratio `2.0` from `2.752 rad` to `0.133 rad`, at
or below the no-reset floor, and matches shipped transient smear exactly at every
ratio on the corpus's own measurement. Removing the reset instead would regress
smear to `8.0` at ratio `3.0`, so the reset stays and is simply applied less
widely.

The pack should use material with sustained low content under transients — the
`A18` reports were bass-register pops on ticks — and compare shipped against the
candidate at ratios `1.5` and `2.0`, where the artifact is largest. Randomise
sides per case, as `g10.039` did.

Objective evidence says the artifact is gone at no measured cost. That is not the
same as sounding better, and Rule 5 exists for the difference.

Nothing adopts the candidate until that pack is judged. The shipped default is
unchanged and no production path can reach the new mode.
