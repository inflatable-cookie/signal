# g10.029 Transform-Family Reassessment

Date: 2026-07-11
Status: direction frozen

## Decision

Return to the passing Batch 29.6I painless nonstationary-Gabor bank. Keep its
filters, compact frequency supports, diagonal frame operator, and pointwise
canonical dual. Replace only unequal per-band coefficient scheduling: every
band uses the largest original coefficient count and therefore one common hop.

This is a dense offline feasibility candidate. It does not reuse the rejected
wavelet bank, alias-block dual, tightener, boundary completion, or phase path.

## Why

Batch 29.6I already proved near-unity frame bounds and reconstruction. Its
unequal time lattices blocked a published phase topology. Dense regridding
removes that scheduling gap without changing filters or mixing their support.
The trade is coefficient cost, which must be measured explicitly.

## Stop Gate

Batch 29.6AG must prove unchanged filters/frame/dual, one common lattice,
identity reconstruction, real-spectrum closure, finite repeatable evidence,
and `1e-12` analysis/dual atom excluded energy inside `16384` frames on a large
probe. Any failure stops for operator review. Passage opens only a new
derivative/topology contract.

## Next Task

Implement Batch 29.6AG dense painless common-lattice feasibility. Do not modify
phase or synthesize stretched audio.
