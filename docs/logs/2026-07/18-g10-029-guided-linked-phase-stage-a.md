# g10.029 Guided Frequency-Partitioned Linked-Phase Stage A

Date: 2026-07-18
Batch: 29.7AK
Status: Stage A passes; Stage B closes at fixed capacity

## Stage A

The report-only `48 kHz` kernel reuses the Signal-owned `16384/8192/512`
painless frame, `4096/2048/1024` supports, and `750 Hz`/`6000 Hz` ownership.
It adds one synchronized two-channel phase-state pass, ordinary recurrence,
conditional greatest-energy peak-trajectory borrowing, peer-owned magnitude
and current analysis-relative phase, and per-channel canonical-dual synthesis.

Peak identity error is `2.914335439641036e-16`. Duplicate equality,
mono/duplicate parity, silent-peer isolation, and swap are exact. Long, middle,
and short owner counts remain `127/448/769`. Reset, attack, ordinary, unlocked,
and locked counts are `673/673/673/673/18844`. Linked/unlinked region counts are
`1540/2673`; high-water is `156/673`. Structural, finite, repeat, and overflow
failures are zero. Evidence hash: `79b0cc2047f563b6`.

## Stage B Stop

The preregistered workspace is not sample-rate invariant. At the frozen `8 kHz`
mechanics gate, the same `16384/512` frame and fixed-Hz boundaries require
`2432` signed and `1217` nonnegative-frequency atoms. The frozen capacities are
`1344/673`. `CapacityExceeded` fires before the `48` objective rows.

The attempted whole-source output representation also scales coefficient count
with padded render duration. Expanding either bound would invalidate Rule 31R.
The Stage B path is removed. No objective, mono, long-development, listening,
or holdout evidence exists.

## Validation

- focused release tests: `3` passed
- Stage A repeat and explicit Stage B capacity rejection pass
- `signal-dsp-stretch` debug suite: `269` tests passed across library, binary,
  integration, and documentation targets
- missing-docs library check passed
- `effigy qa:docs`, `effigy qa:northstar`, `effigy health`, and
  `effigy validate` passed
- workspace `cargo fmt --check` remains blocked by unrelated unformatted
  binaural work; the `signal-dsp-stretch` package format check passes

## Next Task

Run Batch 29.7AL under Rule 31S. Decide whether the synchronized channel kernel
can cross a fixed exact two-slice representation without reopening projection,
overlap ownership, or duration-sized state. Do not implement another renderer.
