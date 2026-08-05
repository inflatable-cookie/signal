# g10.041 Batch 41.1 - The Phase-Reset Hypothesis Is Eliminated

Status: complete
Created: 2026-08-05
Scope: finding `A18`, low-mid pops on transients

## The Hypothesis

`A18` has been carried since `g10.036` with a working hypothesis attached: that
the pops share the phase-restart mechanism behind the seam pulse.

It was plausible on the mechanics. `should_reset_phase_at_transient` sets every
bin's synthesis phase to the analysis phase when spectral flux reaches `0.30`
and the energy ratio reaches `1.20`. Low bins have long periods, so an identical
phase jump produces a far larger waveform step there than at high frequencies —
which is the shape of a low-frequency pop.

## It Is Not What Is Happening

Measured on an `80 Hz` tone plus a percussive attack every `250 ms`, at the
`2048`/`512` offline geometry, worst step in a `375 Hz` low band:

| ratio | transient reset | no reset | source |
| --- | --- | --- | --- |
| `1.5` | `0.02399` | `0.02065` | `0.02470` |
| `2.0` | `0.02449` | `0.02665` | `0.02470` |
| `3.0` | `0.02650` | `0.03209` | `0.02470` |

The reset makes the low-band step *smaller* at ratio `2.0` and `3.0`. Every
stretched value sits at or below the source's own `0.02470`.

The hypothesis is eliminated in both directions: the reset is not adding a
low-band discontinuity, and at higher ratios it is reducing one.

## The First Probe Returned A Confident Null

The first version of this measurement used a `16`-sample click. Both modes
produced identical output to five decimal places across all three ratios.

That looked like a clean negative result and was worthless. A `16`-sample click
inside a `2048`-frame analysis window contributes almost no spectral flux, so
the detector never fired at all — the probe was comparing the transient-reset
path against itself.

The attack was replaced with a `25 ms` exponentially decaying broadband burst,
which is what "the ticks" in the listening material actually were, and the two
modes then diverged. Only that second run is evidence of anything.

This is the same failure as `g10.040`'s first `G3`, which passed the shipped
broken kernel, and `g10.039`'s five structural gates, which passed a renderer
emitting pure silence. A test that cannot trigger the mechanism it tests returns
a confident null, and the only way to notice is to check that it fires.

## The Metric Found Nothing Anywhere

Outlier counts were `0` in every condition, including the unprocessed source.
Worst-step values across all six rendered conditions sit within `30%` of the
source's own.

Two readings are possible and this batch cannot distinguish them: the artifact
is not in the raw vocoder path, or a step in a low band is not what a "low-mid
pop" is. Batch 41.2 must not assume the first without testing the second.

The listeners heard this through the offline *artifact* path — chunking, both
seam smoothers, and the `1.0e-3` normalization gate that zeroes thin-coverage
output. This batch measured the raw vocoder underneath all of it.

## Next Task

Open Batch 41.2, and build the injected-pop fixture before anything else in it.
Batch 41.1's null is only as trustworthy as a metric that reported zero outliers
in the source, and that has not been shown to fire on a pop it should catch.
