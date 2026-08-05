# 041 - A18 Transient Pop Triage

Status: active; Batch 41.1 complete, one hypothesis eliminated
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
phase-restart mechanism behind the seam pulse — is eliminated by Batch 41.1.

## Goals

- [ ] reproduce the pop in a measurement, not a listening pack
- [ ] localise which layer introduces it: raw vocoder, chunked artifact path,
  seam smoothers, or normalization
- [ ] state a mechanism, or state that it is not reproducible and close the
  finding honestly
- [ ] add a permanent guard only if a mechanism is found

## Non-Goals

- no quality tuning before a mechanism exists
- no new listening pack until there is something specific to judge
- no changes to the RealtimePreview lane

## Batches

### Batch 41.1 - Eliminate The Phase-Reset Hypothesis

Status: complete; hypothesis eliminated

- [x] test whether the transient phase reset produces a low-band discontinuity
- [x] compare the shipped `IdentityLockedTransientReset` mode against
  `IdentityLocked` on material that actually trips the detector
- [x] record the result either way

The mechanism was plausible. `should_reset_phase_at_transient` sets every bin's
synthesis phase to the analysis phase when flux is at least `0.30` and the
energy ratio at least `1.20`. Low bins have long periods, so an identical phase
jump is a much larger waveform step there than at high frequencies — exactly the
shape of a low-frequency pop.

It is not what is happening.

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

Status: ready

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

- [ ] the artifact is reproduced in a measurement, or its absence is established
  with a metric proven to fire on an injected instance
- [ ] the layer that introduces it is named, or ruled out layer by layer
- [ ] `A18` is closed with a mechanism or closed as unreproduced

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

Open Batch 41.2. Before anything else in it, build the injected-pop fixture and
prove the metric fires on it — Batch 41.1's null result is only as trustworthy
as the metric that produced it, and that metric reported zero outliers even in
the source.
