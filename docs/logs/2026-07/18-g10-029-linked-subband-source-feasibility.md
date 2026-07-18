# Linked-Subband Source Feasibility

Date: 2026-07-18
Roadmap: `g10.029`, Batch 29.7AH
Contract: `082`, Rule 31O
Specimen: SBSMS `2.3.0` at
`e99cd7e6c6367e476577be34d2fdbe2023904d7e`

## Decision

Close `LinkedSubbandSinusoidalModel` before clean-room Signal implementation.
The exact source topology improves aggregate stereo metrics but does not own
local waveform consistency, exact linked mechanics, or the required mono
quality envelope.

Do not tune SBSMS, copy GPL source, combine it with Signal, or use it as a
dependency or fallback.

## Frozen Evidence

- `48` existing stereo development rows
- seven one-second mono material families at `0.75x`, `1.0x`, `1.5x`, and
  `2.0x`: tone, chord, partial crossing, long decay, isolated transient, dense
  transients, and noise
- six existing five-second exact-source mono development rows
- duplicate, mono parity, hard pan, swap, polarity, and gain mechanics at all
  three fixed-ratio controls
- no concealed holdout, listening, dynamic ratio, realtime, routing, cache,
  production, or product material

All inputs were frozen before the first specimen render. The two complete runs
are bit-repeatable. Evidence hash: `79b5f7c14692b8f5`.

## Result

| Boundary | Result |
| --- | --- |
| Aggregate stereo gate | `0/48` failures |
| Local waveform consistency | `6/48` failures |
| Duplicate-channel error | `1.184940338135e-4` |
| Stereo-duplicate versus mono error | `1.286610960960e-3` |
| Silent-peer peak | `1.606090194173e-7` |
| Channel-swap error | `4.650082439184e-3` |
| Polarity error | `1.956894993782e-5` |
| Gain error | `0` |
| Mono hard failures | `7` |
| Row-complete regressions against coherent Signal | `2` |
| Development metrics worse than both Signal and Rubber Band | `21` |

The seven hard failures are chord `2.0x`, partial crossing at `0.75x` and
`2.0x`, and noise at every ratio including identity. Tone and long decay at
identity regress on every measured quality field against coherent Signal. All
six five-second development rows contain metrics worse than both coherent
Signal and Rubber Band.

Direct oscillator synthesis removes Signal's inverse-frame and support-crop
losses, but does not preserve the aggregate waveform after track modeling and
the subband sum. The source is therefore useful causal evidence, not a viable
quality foundation.

## Execution Evidence

One pass contains `103` external renders over `3,311,016` input frames and
`5,002,981` output frames. Runtime counters report:

- `1,695,405` track visits
- `165,438` births and `164,545` deaths before caller-owned output termination
- maximum `66` track visits in one renderer time group
- maximum `10,728` track visits in one `4096`-sample output read
- peak RSS about `23.5 MB`; exact process RSS varies between repeated runs

The streaming read API does not emit the public frame callback; the report
records zero frame callbacks rather than inferring an internal frame bound.
The reference implementation uses dynamic track containers and supplies no
fixed capacity or overflow result.

## Reproduction

The source checkout and build remain under `target/`. Build the pinned library
unchanged, compile
`crates/signal-dsp-stretch/tools/sbsms_specimen_adapter.cpp` against it, then
run:

```text
cargo test -p signal-dsp-stretch --release \
  source_studied_linked_subband_sinusoidal_source_feasibility \
  -- --ignored --nocapture
```

Generated evidence lives under `target/stretch-sbsms-source-feasibility/`.
The adapter contains only Signal-owned raw-PCM plumbing and public API calls.

## Next Task

Batch 29.7AI must run pinned Rubber Band R3 through the same local-consistency
and exact-mechanics rules. Determine whether the rejected rules distinguish
the professional target or merely exceed it. Do not implement another
renderer or access the holdout.
