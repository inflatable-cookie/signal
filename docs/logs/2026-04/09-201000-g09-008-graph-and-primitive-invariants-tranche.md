# 2026-04-09 - g09.008 Graph And Primitive Invariants Tranche

## Summary

Completed the first `g09.008` substrate-hardening batch across
`signal-primitives` and `signal-graph`.

## Code Reality

- `signal-primitives`
  - added explicit `AudioBuffer::try_new(...)`
  - added explicit `AudioBuffer::try_from_interleaved(...)`
  - added `AudioBufferConstructionError`
  - invalid zero-channel layouts and lossy interleaved sample counts now reject
    explicitly instead of being silently accepted
  - counted one- and two-channel layouts now normalize to canonical `Mono` and
    `Stereo`
- `signal-graph`
  - unsupported channel adaptation no longer presents as an ordinary successful
    zeroed adaptation path
  - graph execution now records explicit degraded adaptation failures through
    `failed_channel_adaptation_count`
  - focused negative graph coverage now proves the degraded-path boundary

## Validation Run

- `cargo test -p signal-primitives`
- `cargo test -p signal-graph`
- `effigy health`

## Reassessment

The next honest `g09.008` seam is CLAP sandbox protocol hardening, not
shared-memory lifecycle ownership yet. The remaining panic-oriented
prepare/activate/deactivate/reset path in the CLAP harness is narrower and can
be batch-carded cleanly without dragging in the broader ownership and stale-
region design work that the shared-memory batch still needs.

## Next Task

Continue the active strict `g09` lane from
`docs/specs/batch-cards/007-g09-008-clap-sandbox-protocol-hardening.md`.
