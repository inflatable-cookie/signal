# g10.029 Normalized Material Policy Preregistration

Date: 2026-07-18
Batch: 29.7ANR
Status: passes

## Result

Rule 31V maps the unchanged Rule 31R material policy onto the normalized
physical-time frame without changing its sound policy. Rule 31T representation
hash `0407f765c7d84375` and Rule 31U guided-mechanics hash
`90c10cd2e66d4faf` remain frozen.

The `80/40/20 ms` atom supports become exact `4/2/1`-tick temporal-median
radii on the `10 ms` lattice. Frequency medians use only immediate same-scale
neighbours. Joint magnitude remains the channel maximum. Tonalness,
transientness, noisiness, strict transient centres, lower-frequency/lower-
channel ties, the `6000 Hz` link limit, and the existing `1e-24` energy floor
retain their prior laws.

One decision needs exactly `19` guidance ticks: `4` ticks of temporal material
support, one tick on either side for strict centre detection, then another `4`
ticks of magnitude support. That matches the frozen Rule 31T
`19P(C+3)` halo. Guidance runs once per global tick and crosses slice lifetime
without truncation, reset, duplication, or a duration-sized store.

The state terminology is now explicit without changing output. Ordinary
instantaneous-frequency recurrence is computed for every channel first.
Material guidance then terminates in reset, attack, unlocked ordinary, or
tracked lock. Compatible greatest-energy trajectory borrowing remains a
subcase of lock below `6000 Hz`; otherwise lock stays channel-local. The
scripted ordinary decision remains a mechanics bypass, not a new classifier
threshold.

## Objective Boundary

Batch 29.7AO has one failure-first run:

1. frozen geometry, capacity, identity, structural, bounded-work, overflow,
   repeat, and four hard channel mechanics
2. frozen six-source synthetic set at `0.75`, `1.5`, and `2.0`
3. corrected `48`-row stereo gate
4. only after stereo passage, unchanged six exact-source mono rows plus long-
   development metrics

The final stage retains the unchanged calibrated gate, at least `245/384`
improved windows, at most `13/48` local-row failures, maximum normalized-Gram
residual `0.01744693815260`, and no row-complete mono regression. Rubber Band
cell hash `9574e5e2e53d1a63` is attribution-only. The run stops at the first
miss. No sweep, row repair, retry, listening, or holdout access is allowed.

## Boundary

This batch changed documentation only. It produced no renderer, stretched
audio, objective result, listening artifact, or sound-quality claim.

## Next Task

Run Batch 29.7AO once under Rule 31V. Implement the frozen policy, execute the
evidence stages in order, and stop at the first miss. Keep policy changes,
listening, holdout, Batch 29.8, and product work closed.
