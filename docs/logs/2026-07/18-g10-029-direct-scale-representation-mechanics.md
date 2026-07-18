# g10.029 Direct Scale Representation Mechanics

Date: 2026-07-18
Batch: 29.7AS
Status: complete; Batch 29.7AT ready

## Result

The private release-only `DirectFrequencyPartitionedScaleTimeline` mechanics
proof passes Rule 31Z at hash `fdf90f6127749341`.

- geometry and owned-bin counts pass at `8/44.1/48 kHz`
- every fixed storage formula and actual allocation length matches
- planner scratch peaks at `3840` inside the `7680` cap
- every exact-capacity request passes and every one-past request returns the
  specified `CapacityExceeded`
- unsupported rate, channel count, ratio, target, and discontinuity requests
  reject before processing
- unity copy is bit-exact
- maximum unmasked scale reconstruction error is `3.3306690738754696e-16`
- maximum square-partition error is `4.440892098500626e-16`
- maximum imaginary residue is `2.5760361165042217e-16`
- maximum conjugacy error is `7.791709393054201e-14`
- crop, coverage, finite, work-count, and repeat checks pass

## Masked Diagnostic

The preregistered non-PR masked sum runs on `22` rate/control rows. Silence is
exact and every bounded-lag timing result is zero. Maxima are:

- peak residual: `0.05633852196771144`
- RMS residual: `0.02153483778719231`
- gain movement: `0.4516143396004201 dB`
- boundary error: `0.055518269305318085`

The maxima occur at `750 Hz` or `6000 Hz`; interior tones are effectively
flat. These values are frozen diagnostics. No crossover, window, or mask was
tuned and no objective or listening audio ran.

## Validation

- focused release Rule 31Z tests: pass, `4/4`
- normal `signal-dsp-stretch` suite: pass, `269/269`
- scoped `rustfmt --check`: pass
- release `-D missing-docs` library check: pass
- `effigy qa:docs`: pass
- `effigy qa:northstar`: pass
- `effigy health`: pass
- `effigy validate`: pass
- full release suite: `300` pass, `27` ignored, `4` existing frozen-hash
  assertions fail only under release optimization; the same four failures repeat
  in isolation and none of their source paths changed in this batch
- package-wide release Clippy remains blocked by `78` existing warnings; no
  warning names the new direct-scale module
- workspace `cargo fmt --check` remains blocked by unrelated concurrent plugin
  edits; the five direct-scale Rust files pass scoped formatting

## Next Task

Run Batch 29.7AT. Integrate the frozen direct state mechanics only. Keep
objective and listening audio closed.
