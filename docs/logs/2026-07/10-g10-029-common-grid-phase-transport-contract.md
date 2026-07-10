# g10.029 Common-Grid Phase-Transport Contract

Date: 2026-07-10
Status: decision frozen

## Decision

Keep analysis and synthesis on the proven `384`-frame common grid. Output
column `m` projects to exact source coordinate `m/ratio`. Interpolate magnitude
and phase-gradient fields at that coordinate; never interpolate wrapped
complex coefficients.

Channel `k` is analyzed at `n*384+d[k]`. Estimate instantaneous frequency from
horizontal phase differences, then subtract `omega_hat*d[k]` to transport
phase to nominal time `n*384`. Only delay-compensated phases may form vertical
differences or heap neighbors.

Integrate positive-frequency coefficients with one bounded deterministic heap.
Canonical-dual synthesis mirrors the solved spectrum for real output.

## Stop Gate

Batch 29.6K must prove derivative scale and compensation sign on synthetic
tones before general integration. Compression, expansion, chirp, impulse,
noise, mixed, and silence controls then prove mapping, assignment, heap,
symmetry, coverage, exact length, placement, finite values, and repeat hashes.

The 60-row corpus, linked stereo, dynamic ratio, cache identity, and product
routing remain closed.

## Next Task

Implement Batch 29.6K and stop on any derivative, compensation, assignment,
symmetry, coverage, placement, or determinism failure.
