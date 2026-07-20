# g10.031 Verified Source-Relative Brief

Date: 2026-07-20
Batch: 31.28
Status: complete

## Vector Audit

Python integer arithmetic and Ruby integer arithmetic independently evaluated
the frozen wrapping operations. They agreed on:

- little-endian `RNWFRAME`, `RNWBIN00`, `RNWBASE0`, and `RNWTEST0`
- `mix64(0)`, both `mix64(1)` rounds, final `mix64(1)`, and
  `mix64(u64::MAX)`
- frame hash, bin hash, left rotation, outer input, final address, and high-53
  numerator for seed `0x0123456789abcdef`, frame `7`, bin `11`, base stream

The authoritative `mix64(1)` value is `0x5692161d100b05e5`. The rejected
assertion's `0x569216d1009b05e5` is a transcription error. Search found no
second handwritten exact counter assertion in the frozen authority.

## Decision

Freeze `VerifiedSourceRelativeRenewalSpectral` as fresh complete authority.
DSP topology and acoustic gates are unchanged. Candidate tests own exact
counter literals once in `COUNTER_VECTORS`; construction validates that table
before checkpointing. Duplicate handwritten counter literals are forbidden.

Fresh identity:

- worktree: `signal-candidate-31-29`
- branch: `candidate/g10-031-verified-source-relative-renewal`
- module: `creative_verified_source_relative_renewal`
- prefixes: `verified_source_relative_renewal_*`

No candidate DSP, test, harness, fixture, API, route, cache, Loophole, or
Chorus surface entered `main`.

## Next Task

Run Batch 31.29 only. Implement the verified brief once in the named disposable
worktree, complete construction `1/1`, freeze one checkpoint, and run `15`
structural then `9` synthetic gates in order. Stop on the first miss. Do not
push.
