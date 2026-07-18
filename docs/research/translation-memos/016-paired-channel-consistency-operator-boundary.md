# Paired-Channel Consistency Operator Boundary

Status: promoted
Date: 2026-07-18
Roadmap: `g10.029`, Batch 29.7AC
Contract: `082`, Rule 31L

## Question

Find one independently supported paired-channel operator that preserves a
synthesis-consistent transform field and arbitrary source stereo through time
stretch. Require a complete constraint order and fixed bounded execution.

## Candidate Operators

Per-channel consistency projection is exact and bounded:

`P_C(C) = A D C`.

It supplies no paired-channel target. Applying it independently can change the
coefficient relation that a previous step imposed. Re-imposing a local relation
afterwards returns the field outside `range(A)` in general.

Multiple Input Spectrogram Inversion supplies a real joint projection for a
different problem. Its separated source estimates have a known mixture
constraint: their sum must reconstruct one observed mixture. Alternation can
therefore project between magnitude, consistency, and a defined additive
mixture set. Stereo left and right are observations, not latent sources whose
sum has a known target waveform. Substituting `L + R`, mid, or another chosen
downmix would discard unconstrained side information and turn a source-
preservation problem into a rendering policy.

Spatial covariance matching also solves a different problem. It constructs
outputs with a target covariance using input components and, when needed,
decorrelated energy. That is appropriate for spatial rendering and upmixing.
It is not a transparent projection of an arbitrary original stereo waveform:
the target covariance must be selected or estimated, local covariance does not
identify phase or waveform uniquely, and injected decorrelated components can
change source structure.

A naive alternating sequence therefore has no supported fourth step:

1. project each channel through `A D`
2. impose a local relation, magnitude, or covariance target
3. repeat a fixed number of times
4. accept the waveform

Step 2 is either the already rejected coefficient-local relation or an
underspecified spatial renderer. The constraint sets are nonconvex, their
intersection is not established, and the reviewed sources provide no finite
iteration count or non-convergence rule for this stereo-preservation problem.
Freezing either would be new algorithm invention, not source translation.

## Primary Evidence

Gunawan and Sen define iterative phase estimation around separated sources and
one observed single-channel mixture. Magron and Virtanen generalize that class
as alternating projections over explicitly defined consistency, mixing, and
magnitude objectives. Both depend on the additive mixing set that arbitrary
stereo preservation does not have.

Masuyama, Togami, and Komatsu establish that multichannel coefficient estimates
can change in amplitude and phase through inverse transform and re-analysis.
They support waveform-domain validation, not a missing spatial projection.

Vilkamo, Backstrom, and Kuntz optimize time-frequency spatial output in the
covariance domain. Their framework uses independent input components and can
add decorrelated energy to reach a target covariance. McCormack, Politis, and
Pulkki likewise use covariance matching to render source spread. These are
spatial synthesis methods, not proofs that a projected stretch preserves an
arbitrary input pair.

Clean-room source studies reach the same architecture boundary. Signalsmith,
Rubber Band, and Bungee own linked channels inside a complete analysis, phase,
and synthesis topology. None exposes an independent post-hoc stereo covariance
projection that can be transferred into the rejected Signal frame.

## Promoted Boundary

Transform-domain post-projection closes for the current program:

- do not alternate `A D` with coefficient-local relation repair
- do not treat a chosen mono downmix as a stereo mixing constraint
- do not use covariance matching or decorrelated energy as transparent stereo
  preservation
- do not freeze an iteration count without a supported feasible set,
  convergence condition, and explicit failure result
- retain reconstructed waveform IPD, correlation, mid/side balance, normalized
  Gram residual, and mono quality as authoritative gates

This does not reject transforms or multichannel phase vocoders. It rejects an
independent repair stage after transform modification. A future transform
family remains admissible only when one complete topology owns channel linkage
through synthesis and produces valid waveforms by construction.

## Decision

No reviewed primary source supplies the required paired-channel projection.
Close transform-domain joint projection. Do not implement the speculative
alternating operator.

The Signal-native stretch program remains active. Re-enter family selection at
the waveform boundary. Compare complete source-synchronous, sinusoidal, and
single-grid transform topologies by how they own one stereo timeline, transient
events, tonal continuity, and bounded synthesis. Select one whole architecture
or close each with explicit evidence before another renderer.

## Sources

- [Gunawan and Sen, Iterative Phase Estimation for the Synthesis of Separated Sources From Single-Channel Mixtures](https://doi.org/10.1109/LSP.2010.2042530)
- [Magron and Virtanen, Spectrogram Inversion for Audio Source Separation via Consistency, Mixing, and Magnitude Constraints](https://arxiv.org/abs/2303.01864)
- [Masuyama, Togami, and Komatsu, Consistency-Aware Multi-Channel Speech Enhancement](https://arxiv.org/abs/2002.05831)
- [Vilkamo, Backstrom, and Kuntz, Optimized Covariance Domain Framework for Time-Frequency Processing of Spatial Audio](https://aes.org/publications/elibrary-page/?id=16831)
- [McCormack, Politis, and Pulkki, Rendering of Source Spread for Arbitrary Playback Setups Based on Spatial Covariance Matching](https://doi.org/10.1109/WASPAA52581.2021.9632724)
- [Signal Joint-Synthesis Consistency Boundary](./015-joint-synthesis-consistency-boundary.md)

## Next Task

Batch 29.7AD selects one calibrated single-grid state-complete linked phase-
vocoder proof in memo 017. Run Batch 29.7AE. Keep Batch 29.8 and product work
closed until the frozen candidate passes objective validation.
