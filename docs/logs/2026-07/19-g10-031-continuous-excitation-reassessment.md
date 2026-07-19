# g10.031 Continuous-Excitation Reassessment

Date: 2026-07-19
Status: Batch 31.5 complete; replacement brief frozen

## Decision

Retained the `6 dB` active-support crest-growth ceiling and closed the rejected
independent-bin phase-diffusion topology. The failure was loss of cross-bin
waveform ownership, not a bad scalar.

The retained musical comparator matrix supports the ceiling. PaulXStretch,
REAPER `Rrreeeaaa`, and the `ReaReaRea` cyclic control had zero rows above it;
their maxima were `3.88 dB`, `4.37 dB`, and `1.30 dB`. CDP `SPECTSTR` exceeded
it on every row. Signal preserves CDP-like spectral colour, not that defect.

## Frozen Replacement

`ContinuousExcitationSpectral` derives stochastic coefficients from one
bounded, deterministic, output-synchronous waveform. It retains each frame's
full complex excitation, including realized magnitude. A flat-envelope
structural control must reconstruct that waveform through normalized
overlap-add. Source-envelope shaping, coherent `Spectral` contribution,
linked-channel relations, exact length, bounded memory, character laws, gate
order, rejection, and cleanup are frozen in one brief.

No limiter, compressor, crest repair, post normalization, external production
dependency, DSP module, harness mode, fixture, public API, or product route was
added. The rejected candidate remains deleted. `OfflineHighQuality` and the
closed Contract `084` program are unchanged.

## Authority

- `docs/architecture/offline-creative-continuous-excitation-spectral-brief.md`
- `docs/contracts/085-creative-time-stretch-product-and-routing-contract.md`
- `docs/roadmaps/g10/031-creative-time-stretch.md`

## Next Task

Run Batch 31.6 only. Implement the frozen replacement once in a disposable
worktree. Run structural admission and the exact prior failing neutral
`Dream`, `4x`, deterministic uniform-noise crest row first. Stop and delete on
failure. Do not produce long-form audio or open later owners or product routing
until every fixed synthetic gate passes.
