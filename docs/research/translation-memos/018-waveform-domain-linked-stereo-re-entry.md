# Waveform-Domain Linked-Stereo Re-entry

Status: validated; selected candidate closed
Date: 2026-07-18
Roadmap: `g10.029`, Batch 29.7AG
Contract: `082`, Rules 31N-31O

## Question

Find at most one complete source-backed topology whose linked-channel relation
is owned at waveform synthesis rather than asserted on a redundant transform
field. The same renderer must cover polyphonic tone, transients, mono quality,
linked stereo, and fixed bounded execution.

## Candidate Decision

| Candidate | Relation owner | Complete required topology | Decision |
| --- | --- | --- | --- |
| Bonada-style multiresolution stereo sinusoidal TSM | matched spectral peaks and channel-relative phase | still uses frame synthesis and overlap | reject for this proof |
| Dorran-Lawlor-Coyle multichannel phase vocoder | collective channel analysis and synchronized peak relations | strongest reported hybrid range is local to unity; still inverse-STFT synthesis | retain evidence, reject universal `0.75x`-`2.0x` proof |
| Elastique Pro V3 | documented linked stereo analysis and processing | commercial documentation does not expose a testable synthesis invariant | comparator only |
| learned waveform TSM | learned temporal compression or synthesis | no reviewed deterministic finite-state, fixed-bound transparent renderer | reject for this lane |
| SBSMS-style linked subband sinusoidal model | paired partial tracks synthesized directly | one recursive subband tracker and oscillator bank | select for source feasibility |

The selection is one architecture candidate, not a quality claim. No Signal
renderer is authorized by this memo.

## Required Invariant

For each compatible matched partial `q`:

1. both channels refer to one output sample clock and one oscillator trajectory
2. each channel supplies its current partial magnitude
3. paired synthesis phase retains the current analysis-relative phase relation
4. direct oscillator samples are produced before any component sum
5. all subbands sum on the same output timeline

This removes the failed `A D C = C` boundary. The relation is not stored in a
coefficient field that later passes through a full inverse frame, support crop,
or overlap normalization. Each matched component already has one valid output
waveform before summation.

The invariant is component-local. It does not guarantee exact aggregate Gram,
IPD, or correlation after unmatched components and modeling error are summed.
Those reconstructed waveform metrics remain rejection gates.

## Complete Topology Ownership

`LinkedSubbandSinusoidalModel` owns the entire first proof:

- tone and polyphony: partial extraction, identity, continuation, crossing,
  split, and merge across octave subbands
- discontinuities and noise-like material: bounded track births, deaths,
  jumps, and short high-frequency subbands inside the same renderer
- stereo: explicit compatible-track pairing and shared trajectory state
- time: one exact source-to-output schedule and final sample count
- synthesis: direct oscillators, followed only by the subband sum
- boundaries: explicit analysis priming, track starts and ends, output tail,
  and deterministic finalization
- boundedness: predeclared active-track and event capacities, fixed work per
  input/output quantum, duration-independent memory, and explicit overflow

No additive H/P/R renderer, independent transient stretcher, post-hoc image
repair, inverse-frame crop, or parameter sweep may enter this proof.

## Why Source Validation Comes First

The topology changes the causal synthesis boundary, but source inspection
cannot establish professional sound quality. A sinusoidal model can still lose
identity detail, smear attacks, roughen noise, mishandle partial crossings, or
produce boundary events. Writing a clean-room renderer before measuring the
source topology would restart expensive implementation churn.

Batch 29.7AH therefore uses pinned SBSMS only as an external comparator and
architecture control. It must freeze the existing development material before
running the specimen and capture:

- identity/model residual
- tones, chords, crossings, and long decays
- isolated and dense transients
- noise and mixed material
- exact length, start, end, and tail behavior
- mono objective position against current Signal and the existing comparator
- linked-stereo IPD, correlation, mid/side, Gram, pan, swap, and polarity
- repeatability, runtime scaling, active-track counts, and memory behavior

The existing concealed holdout remains unread. Listening remains closed.

## Feasibility Decision

Authorize a clean-room Signal proof only if the exact source topology:

- materially avoids the 29.7AF stereo and support failures
- reaches the declared development objective envelope often enough to justify
  implementation
- shows no broad identity, transient, noise, or boundary defect
- admits explicit finite active state and work bounds

Otherwise close `LinkedSubbandSinusoidalModel`. Do not tune SBSMS, combine it
with another renderer, use it as a dependency or fallback, or read the holdout.

## Validation Result

Batch 29.7AH closes `LinkedSubbandSinusoidalModel`. Pinned SBSMS repeats and
passes the aggregate stereo gate at `0/48` failures, but has six local-
consistency failures and material duplicate, mono-parity, swap, and polarity
errors. Seven mono rows fail hard integrity, two identity rows regress on every
quality field against coherent Signal, and the six long development rows
contain `21` metrics worse than both Signal and Rubber Band. Evidence hash:
`79b5f7c14692b8f5`.

The result separates causal topology from quality. Direct partial oscillators
remove inverse-frame and support-crop loss, but track modeling and component
summation still alter the aggregate waveform. No clean-room renderer opens.

## Clean-Room Boundary

SBSMS is GPL-2.0. Batch 29.7AH may build and execute the pinned source outside
Signal source surfaces. Signal may retain behavioral measurements, public
architecture facts, independently stated invariants, and test cases. Source
expression, file structure, constants, interpolation equations, thresholds,
tables, and derived code do not transfer.

This is technical provenance control, not a patent or legal opinion.

## Sources

- [SBSMS project architecture](https://sbsms.sourceforge.net/)
- [SBSMS `2.3.0` source](https://github.com/claytonotey/libsbsms/tree/e99cd7e6c6367e476577be34d2fdbe2023904d7e)
- [Dorran, Lawlor, and Coyle, A Multichannel Approach to Time-Scale Modification of Audio](https://mural.maynoothuniversity.ie/8793/1/BL-Multi-channel-2005.pdf)
- [Bonada, Audio Time-Scale Modification in the Context of Professional Postproduction](https://mtg.upf.edu/node/2240)
- [Elastique Pro V3 SDK documentation](https://licensing.zplane.de/uploads/SDK/ELASTIQUE-PRO/V3/manual/elastique_pro_v3_sdk_documentation.pdf)
- [SBSMS source dossier](../specimen-dossiers/sbsms-source-architecture.md)
- [Joint-synthesis consistency boundary](./015-joint-synthesis-consistency-boundary.md)
- [Paired-channel consistency boundary](./016-paired-channel-consistency-operator-boundary.md)

## Next Task

Run Batch 29.7AI. Test pinned Rubber Band R3 against the same local-consistency
and exact-mechanics rules that reject SBSMS and Signal candidates. Decide
whether the gate distinguishes the professional target before another
topology. Keep the holdout and product surfaces closed.
