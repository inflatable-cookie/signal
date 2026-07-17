# Independent Material-State Frequency Frame

Status: promoted
Date: 2026-07-17
Roadmap: `g10.029`, Batch 29.7X
Contract: `082`, Rule 31I

## Question

Decide whether independent published work closes the two seams left by Batch
29.7W: material-guided unlocked phase and simultaneous frequency-owned
resolution. If it does, define one clean-room Signal candidate without
repeating the rejected three-STFT renderer.

## Source Matrix

| Source | Material phase | Frequency resolution | Stereo | Reconstruction | Boundary and licence |
| --- | --- | --- | --- | --- | --- |
| Damskagg and Valimaki, 2017 | fuzzy tonalness, noisiness, and transientness control phase lock, noise diffusion, transient shoulder suppression, and reset | fixed STFT; authors identify multiresolution as future work | not specified | ordinary STFT overlap-add | CC BY paper; published MATLAB availability has no reviewed code licence and does not transfer |
| Robel, 2010 | voiced/unvoiced boundary controls ordinary noise treatment and bounded phase randomization; warns against destroying pitch-synchronous noise modulation | one speech phase-vocoder grid | not specified | not an exact-frame contribution | published paper only; speech evidence, not music-wide constants |
| Bonada, 2000 | stable peaks continue; transient type selects original-phase extent | simultaneous parallel windows; long low band, shorter high bands; crossovers move between nearby continued peaks | common frame tags and preserved interchannel peak-phase differences | complementary filters must sum to an all-pass response; no numerical error bound | institutional paper, all rights reserved; architecture evidence only, no expression or constants |
| Liuni et al., 2013 | none | different stationary or nonstationary Gabor analyses inside complementary frequency bands | not specified | explicit error bound; fast weighted reconstruction is approximate and transformed overlap treatment remains open | post-print is all rights reserved; mathematical boundary only |
| Balazs et al., 2011; Holighaus et al., 2012 | none | one frequency-adaptive nonstationary Gabor frame; resolution changes by frequency without full-band render duplication | channel-neutral transform | painless canonical dual gives perfect reconstruction to numerical precision; sliced form has bounded delay and linear work | open papers; public LTFAT code is GPL and does not transfer |
| Driedger, Muller, and Ewert, 2014 | separate harmonic and percussive processors | long harmonic PV plus short percussive OLA | not specified | additive separated full-band components | rejected topology for this lane |
| Derrien, 2007 | atom versus residual ownership | low-frequency multiscale atoms plus high/residual PV | not specified | additive residual reconstruction | preliminary and not frequency-exclusive |
| Ottosen and Dorfler, 2017 | adaptive peak lock and transient reset | resolution selected over time | not specified | exact nonstationary-frame synthesis | time-selected resolution; does not close this seam |

The two seams close at architecture level. Damskagg and Valimaki provide a
music-tested material phase law whose mean listening score was practically the
same as Elastique at `1.5x` and `2.0x`. Robel independently confirms that
noise-like regions need a different phase law and that uncontrolled diffusion
is destructive. Bonada independently demonstrates simultaneous
frequency-owned windows and linked-stereo phase preservation in a complete
time-stretcher. Frequency-adaptive painless-frame work supplies the exact
reconstruction law Bonada's parallel-window description lacks.

This does not license a Rubber Band reconstruction. It supports a different
Signal-owned composition from papers. Paper copyright permits study, not code
copying. No external source, constants, masks, random sequence, or GPL
expression may enter Signal. The research record is not a patent
freedom-to-operate opinion; commercial clearance remains a product-release
concern.

## Rejected Composition

Do not revive Batch 29.6CH's `1024/2048/4096` implementation. It:

- analyzed three full-band STFTs
- masked them after phase processing
- normalized every masked render by its own overlap operator
- summed those independently normalized renders
- used one frame-wide attack boolean
- linked channels unconditionally by same-bin owner after independent phase
  processing

Its stutter, doubled attacks, softness, clicks, and definition loss reject that
realization. The new evidence does not reverse the rejection.

Liuni's result explains the scale risk. Cheap weighted synthesis across
different Gabor analyses is approximate, and overlap handling after spectral
processing is unresolved. Independent per-scale normalization is not an exact
multi-scale dual.

## Selected Representation

Select one `FrequencyAdaptiveMaterialPhase` report-only family.

The representation is one painless frequency-adaptive nonstationary Gabor
frame, not three full-band renders. It uses a common time lattice and a finite
piecewise frequency layout:

- low atoms use long support
- middle atoms use medium support
- high atoms use short support
- every atom has one scale owner
- transition atoms remain single-owned; no coefficient is synthesized by two
  scale engines
- one canonical dual reconstructs the complete frame

The first proof freezes the existing Signal-owned `4096/2048/1024` support
family and the existing nominal `750 Hz` and `6 kHz` boundaries. Those values
are antecedent geometry, not transferred source constants or claims of
optimality. No valley movement or crossover search is allowed in the first
candidate. Failure does not authorize tuning them.

The representation must first prove identity reconstruction to numerical
precision before phase modification. No per-scale overlap denominator,
post-render crossover, gain repair, or residual full-band path exists.

## Guidance And State Order

One decision-only full-band analysis computes channel-joint magnitudes using
the per-bin maximum channel energy. It never sums complex channels and never
synthesizes audio. Time- and frequency-direction median filters produce fuzzy
tonalness, noisiness, and transientness on that shared map.

For every active synthesis atom:

1. compute ordinary instantaneous-frequency advance from the atom's prior
   analysis phase and actual source/output centre intervals
2. compute the tracked peak-region common rotation retained from Batch 29.7T
3. if the shared transient set owns the atom, suppress its pre/post transient
   shoulders and use current analysis phase at the detected centre
4. otherwise apply common region rotation, then add a bounded deterministic
   phase perturbation whose amplitude is continuous in shared noisiness and
   stretch distance
5. apply the same rotation and perturbation to every linked channel
6. synthesize once through the global canonical dual

Complete states are:

- `Silent`: exact zero; no trajectory or random state
- `TransientShoulder`: transient-owned energy suppression before and after one
  detected centre
- `TransientReset`: current analysis phase at that centre
- `LockedMaterial`: common tracked-region rotation with zero material
  perturbation
- `DiffuseMaterial`: the same common rotation plus nonzero material
  perturbation

`LockedMaterial` and `DiffuseMaterial` are one continuous published law, not a
hard classifier threshold. The deterministic perturbation is keyed by render
identity, frame, scale, and atom. It has fixed finite work and no mutable RNG
ordering. Its value is shared by channels, so it changes neither current-frame
interchannel phase difference nor magnitude relation.

No blanket finite-support reset exists. Reflection supplies bounded source
support, while the shared material map decides transient state at boundaries.
Common rotation remains limited to non-transient material. No late overlay,
mid/side transform, independent channel classifier, same-bin post-repair, or
channel borrowing exists in the first proof.

## Compatibility

| Requirement | Candidate contract |
| --- | --- |
| source boundary | existing whole-sample even reflection and exact crop |
| mono continuity | one state trajectory per atom on one exact output map |
| linked stereo | all discrete guidance, state, scale, rotation, and perturbation decisions shared; channel spectra retained |
| scale reconstruction | one painless frame and canonical dual; identity proof precedes phase proof |
| deterministic bounds | fixed atom count, fixed median spans, precomputed dual, counter-keyed perturbation, no iteration |
| clean-room | papers and prior Signal evidence only; no external code or constants |

The material classifier is guidance, not H/P/R source separation. The scale
layout is representation, not three processors. These distinctions are the
architecture.

## Bounded Proof

Batch 29.7Y may implement exactly one report-only candidate in two stop-gated
stages.

Stage A proves the representation with phase untouched:

- one owner for every coefficient
- finite positive frame bounds
- canonical-dual identity peak error at or below `1e-12` in `f64`
- exact requested crop, no uncovered samples, conjugate closure, silence, hard
  pan, swap, polarity, scaled duplicate, boundary reflection, and repeat hash

Any Stage A failure closes the candidate before a time-stretched render.

Stage B freezes the complete state law, then runs once at `0.75x`, `1.5x`, and
`2.0x`:

- synthetic tone, noise, impulse, tone-plus-noise, and tone-plus-transient rows
  must exercise every state without non-finite output or hidden gain
- preserve the unchanged six-row mono gate and all `48` calibrated stereo rows
- require zero calibrated stereo failures and zero local-consistency failures
- require no row-complete mono regression against coherent Signal or frozen
  29.7T
- compare current Signal, the candidate, and Rubber Band on the unchanged
  objective report; do not export listening audio unless all hard gates pass

Stop after that candidate. No support, boundary, median span, transient
threshold, diffusion curve, seed, crossover, scale count, peak map, or blend
change follows a miss. Failure returns to architecture research or closes this
source-studied lane.

## Decision

Independent evidence closes both 29.7W seams and supports one complete
clean-room proof. Open Batch 29.7Y for the representation and material-phase
candidate above. Keep production, Batch 29.8, listening, dynamic ratio,
realtime, routing, cache, and product work closed.

## Sources

- [Damskagg and Valimaki, Audio Time Stretching Using Fuzzy Classification of Spectral Bins](https://doi.org/10.3390/app7121293)
- [Robel, Shape-Invariant Speech Transformation with the Phase Vocoder](https://www.isca-archive.org/interspeech_2010/robel10_interspeech.html)
- [Bonada, Automatic Technique in Frequency Domain for Near-Lossless Time-Scale Modification of Audio](https://mtg.upf.edu/node/163)
- [Liuni et al., Automatic Adaptation of the Time-Frequency Resolution for Sound Analysis and Re-Synthesis](https://doi.org/10.1109/TASL.2013.2239989)
- [Balazs et al., Theory, Implementation and Applications of Nonstationary Gabor Frames](https://doi.org/10.1016/j.cam.2011.09.011)
- [Holighaus et al., A Framework for Invertible, Real-Time Constant-Q Transforms](https://arxiv.org/abs/1210.0084)
- [Driedger, Muller, and Ewert, Improving Time-Scale Modification of Music Signals Using Harmonic-Percussive Separation](https://doi.org/10.1109/LSP.2013.2294023)
- [Derrien, Time-Scaling of Audio Signals with Multi-Scale Gabor Analysis](https://hal.science/hal-00467531)
- [Ottosen and Dorfler, A Phase Vocoder Based on Nonstationary Gabor Frames](https://arxiv.org/abs/1612.05156)

## Next Task

Run Batch 29.7Y Stage A. Implement only the report-only painless
frequency-adaptive representation and its identity/mechanics proof. Do not add
material phase until Stage A passes.
