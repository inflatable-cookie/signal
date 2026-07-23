# g10.031 Fixed-Ratio Public Wrapper

Date: 2026-07-23
Status: complete
Roadmap: `g10.031`, Batch 31.76
Contract: `085`

## Changed

- added public offline `CreativeStretch` request, character, error, constants,
  and renderer entry
- exposed exact fixed `Dream` at `4x`, `8x`, and `16x`
- retained admitted seed and `space` without clamping or fallback
- mapped every private renderer error to the frozen public error
- added focused public/private parity, stereo, determinism, length, finiteness,
  empty, and invalid-request tests

## Evidence

- public/private mono and stereo output is byte-identical at every admitted
  ratio
- `space=0`, `0.5`, and `1` pass unchanged
- acoustic `analysis.rs`, `plan.rs`, `stereo.rs`, and `synthesis.rs` retain
  their frozen hashes
- construction `1/1`, structural `10/10`, and synthetic `88/88` with `76/76`
  renders pass
- no acoustic DSP, cache, route, tier, dynamic ratio, report, fixture, runtime,
  Loophole, Chorus, or cross-repo surface changed

## Decision

The accepted creative effect now has its smallest honest public Signal
boundary. No new listening was required because public output is exactly the
admitted private output.

No Batch 31.77 is ready. A named Signal consumer and separate docs-first
authority are required before creative cache, artifact, routing, or product
integration opens.
