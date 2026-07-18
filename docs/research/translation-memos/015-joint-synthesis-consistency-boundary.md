# Joint-Synthesis Consistency Boundary

Status: promoted
Date: 2026-07-18
Roadmap: `g10.029`, Batch 29.7AB
Contract: `082`, Rule 31K

## Question

Explain why Batch 29.7AA preserves the requested peer/reference relation on
every active coefficient but loses stereo quality after synthesis. Decide
whether another frequency-adaptive renderer is justified.

## Exact Attribution

Let `A` be analysis and `D` the canonical-dual synthesis operator. Identity
proves `D A = I` for coefficients produced by analysis. It does not prove that
an arbitrarily modified coefficient field `C` is itself an analysis field.
That stronger condition is:

`A D C = C`.

Batch 29.7AA proves its relation before `D`, but never proves this consistency
condition. The material and relation operations create a field outside
`range(A)`. Synthesis maps that field to one waveform per channel. Re-analysis
would return the projected field `A D C`, not the requested field `C`.

The first divergence is the inner synthesis sum. For atom contributions
`R_k` in the reference channel, the peer is

`P_k = a_k exp(i rho_k) R_k`.

After synthesis, the channels contain weighted sums over atoms. In general,

`sum_k P_k != a exp(i rho) sum_k R_k`

when `a_k` or `rho_k` varies. Stereo material requires both to vary. Exact
per-atom relations therefore do not define an exact relation for the sum.

One two-atom counterexample is sufficient. Start with reference contributions
`[1, 1]` and peer contributions `[1, i]`. Each atom has an exact relation. A
common material phase of `i` on the second atom produces reference sum `1+i`
and peer sum `0`. The common operator cancels inside each atom but changes the
cross-atom interference and destroys the summed relation.

The four requested seams separate as follows:

- inner band synthesis is the first causal sum and already admits the
  counterexample
- outer slice overlap adds another sum with layer-varying peer/reference
  magnitude ratios; it can expose more movement but is not the first cause
- band-varying relation and magnitude ratio are the direct non-commuting terms;
  forcing them constant would destroy real stereo content
- material phase is common within each atom, but changes cross-atom phase and
  therefore the waveform Gram matrix, correlation, and mid/side balance

This agrees with earlier Signal evidence. Batch 29.7G first observed relation
movement at inverse support synthesis. Batch 29.7H proved analytic and real
overlap linearly equivalent. Batch 29.7I found no omitted coefficient class
whose repair closed the result. Stage B repeats the same boundary in a more
redundant frame.

## Primary Evidence

Dorran, Lawlor, and Coyle preserve magnitude and phase differences between
related same-bin peaks. They do not claim that this local condition preserves
arbitrary whole-record covariance after an inconsistent coefficient edit.

Griffin and Lim frame reconstruction from a modified STFT as estimation: the
modified field need not be the transform of a signal. Le Roux, Ono, and
Sagayama make the boundary explicit. A consistent spectrogram is one in the
range of STFT analysis, and their time-scale method iteratively reduces a
consistency objective over overlapping frames.

Masuyama, Togami, and Komatsu confirm the multichannel consequence: an
estimated multichannel spectrogram can change in amplitude and phase after
inverse transform and re-analysis, so quality must be evaluated on the
reconstructed time-domain signal.

Holighaus et al. prove invertibility and bounded sliced execution for
unmodified sliCQ analysis coefficients. That supports `D A = I`, not closure
of arbitrary nonlinear coefficient edits under `A D`.

Clean-room source evidence is consistent with this distinction. Signalsmith,
Rubber Band, and Bungee combine channel ownership with one complete phase and
synthesis topology. None treats an exact same-atom relation as a sufficient
post-synthesis stereo proof. Rubber Band R3 also assigns each frequency to one
scale instead of synthesizing redundant full-band owners.

## Promoted Invariant

Any future redundant-transform candidate must own a synthesis-consistent joint
field, not only coefficient-local relations:

1. every channel field must satisfy `A D C_c = C_c` within a frozen tolerance
2. linked-channel constraints must be evaluated after the same joint
   projection, not before independent channel synthesis
3. the authoritative stereo gate remains reconstructed waveform IPD,
   correlation, mid/side balance, and normalized Gram residual
4. any iterative projection must have a fixed finite work bound, deterministic
   state, and an explicit non-convergence outcome

Per-atom relation remains useful evidence and a projection constraint. It is
not an acceptance invariant by itself.

## Decision

Close the current frequency-adaptive sliced material direction. Its exact
frame, slicing, relation interpolation, and material mechanics are individually
valid, but the composition has no joint consistency owner. Do not add another
overlap repair, relation variant, scale layout, or material-phase parameter.

The Signal-native stretch program remains open. The next research target is a
joint consistency operator on one coherent coefficient field, or a waveform-
domain topology that avoids independently projected redundant channel fields.
No implementation opens until primary evidence supplies the paired spatial
constraint and a bounded projection order.

## Sources

- [Dorran, Lawlor, and Coyle, Multi-Channel Audio Time-Scale Modification](https://mural.maynoothuniversity.ie/id/eprint/8793/1/BL-Multi-channel-2005.pdf)
- [Griffin and Lim, Signal Estimation from Modified Short-Time Fourier Transform](https://dub.ucsd.edu/CATbox/Reader/GriffinLimMSTFT.pdf)
- [Le Roux, Ono, and Sagayama, Explicit Consistency Constraints for STFT Spectrograms](https://www.isca-archive.org/sapa_2008/roux08b_sapa.html)
- [Masuyama, Togami, and Komatsu, Consistency-Aware Multi-Channel Speech Enhancement](https://arxiv.org/abs/2002.05831)
- [Holighaus et al., A Framework for Invertible, Real-Time Constant-Q Transforms](https://arxiv.org/abs/1210.0084)
- [Signal synthesis-closure attribution](../../logs/2026-07/16-g10-029-stereo-synthesis-closure-attribution.md)
- [Signal analytic-overlap rejection](../../logs/2026-07/16-g10-029-analytic-overlap-rejection.md)

## Next Task

Batch 29.7AD selects one calibrated single-grid state-complete linked phase-
vocoder proof in memo 017. Run Batch 29.7AE before implementing another DSP
candidate.
