# 2026-07-27 g10.036 Batch 36.1 Defect Authority And Contract Amendment

Status: complete

Batch 36.1 starts from `main` commit `e24dadc6`. Documentation only. No crate
source changed. One temporary probe test was created, run, and deleted; it left
no tracked file.

## Purpose

Freeze the laws the four measured Transparent defects violate, and resolve the
authority conflict that blocked correcting them: Contract `084` froze the
retained baseline's mono, dynamic-ratio, pitch, cache, artifact, and
RealtimePreview behavior for the duration of successor research, and that
research closed without the clause being scoped to it.

## Overlap law selection

The audit recorded that ratios above `4x` produce zeroed output blocks. Batch
36.1 measured the full curve to choose a bound rather than assume one, and
compared two candidates: the conservative `analysis_hop * ratio <=
window_size / 2`, and `analysis_hop * ratio <= 0.75 * window_size`.

Interior 512-frame RMS blocks, 440 Hz at 48 kHz, `2048` window, `512`
requested hop:

| ratio | hop now | ripple now | zeroed now | hop under law | ripple under law | zeroed under law |
| --- | --- | --- | --- | --- | --- | --- |
| `1.0` | `512` | `0.276 dB` | `0` | `512` | `0.276 dB` | `0` |
| `1.5` | `512` | `0.276 dB` | `0` | `512` | `0.276 dB` | `0` |
| `2.0` | `512` | `0.276 dB` | `0` | `512` | `0.276 dB` | `0` |
| `3.0` | `512` | `0.276 dB` | `0` | `512` | `0.276 dB` | `0` |
| `4.0` | `512` | `1.396 dB` | `0` | `384` | `0.276 dB` | `0` |
| `6.0` | `512` | `237.126 dB` | `183/547` | `256` | `0.276 dB` | `0` |
| `8.0` | `512` | `237.126 dB` | `368/734` | `192` | `0.358 dB` | `0` |

Three-tone broadband source, same geometry:

| ratio | ripple now | zeroed now | ripple under law | zeroed under law |
| --- | --- | --- | --- | --- |
| `2.0` | `0.474 dB` | `0` | `0.474 dB` | `0` |
| `3.0` | `0.446 dB` | `0` | `0.446 dB` | `0` |
| `4.0` | `1.615 dB` | `0` | `0.447 dB` | `0` |
| `8.0` | `231.781 dB` | `368` | `0.477 dB` | `0` |

Two results the audit did not have:

- ratio `4.0` is not clean today. It sits exactly at synthesis hop equal to
  window size and carries a `1.396 dB` periodic amplitude ripple on tone and
  `1.615 dB` on broadband. It is inside the retained product range, so the
  overlap correction is partly an audible correction, not purely an extension
  as the roadmap first assumed.
- `0.75 * window_size` measures identically to full `75%` overlap. The
  conservative `window_size / 2` bound would have forced a finer hop at ratio
  `3.0`, disturbing a byte-exact range for no measured gain.

`0.75 * window_size` is therefore frozen. At the retained `2048/512` geometry
it leaves every ratio through `3.0` byte-identical and changes only
`3.0 < ratio`. Cost at ratio `8.0` is `2.67x` more analysis frames.

## Decisions

- `A1` policy: the API keeps accepting ratios above `4.0` and the renderer
  stops destroying them. A hard `4x` API ceiling was rejected: the chunked
  artifact path and `g10.039` both need honest renderer behavior above the
  published product range, and refusing there would only move the defect into
  the caller.
- correction classes frozen. The audible window is `3.0 < ratio <= 4.0` for
  overlap, plus all of `A2` and `A3`. `A4` is extension. `A17` is a
  test-harness repair and neither class.
- the lane's standing byte-exactness control is `0.5x..3.0x`, corrected down
  from the roadmap's original `0.5x..4x`.

## Contract Amendments

`docs/contracts/046-sample-domain-time-stretch-engine-contract.md` gains a
2026-07-27 Transparent renderer defect correction addendum: the overlap
coverage law with its measurement tables, the dynamic-ratio segment law, the
seam parity law, the output bound, and the correction classes.

The dynamic-ratio segment law coalesces adjacent curve spans until every render
segment is at least one window of source frames. The coalesced target frame
count is the sum of the target counts its constituent spans would have
produced, so total output length is preserved exactly and the segment renders
at the mean ratio of the spans it covers. The interpolation fallback survives
for one case only: a whole input shorter than one window.

The seam parity law requires identical treatment across channel counts but does
not freeze the mechanism. The current midpoint-offset smoother is explicitly
interim; `g10.039` replaces it.

`docs/contracts/084-stretch-candidate-isolation-and-promotion-contract.md`
gains Rule 9, authorizing defect correction after successor closure under four
conditions, and Rule 10, governing when a byte-exact regression hash may be
re-frozen. The authorized correction set is named and closed: overlap coverage,
dynamic-ratio segment length, mono seam parity, and output bound. Nothing else.

## Operator Decision: Output Bound

`TimeStretcher::stretch_mono` returned `Vec` and could not refuse an oversized
render. The operator chose to make the trait fallible rather than add a
parallel checked entry point, so no unbounded path survives in the public API.

- `TimeStretcher` and the whole-buffer entry points beside it return a typed
  render result
- ceiling is `268435456` output samples, one gibibyte of `f32`: roughly `93`
  minutes mono or `46` minutes stereo at 48 kHz in one call
- breaking change to an in-repo-only trait, taken pre-1.0 with no compatibility
  shim; `signal-render-plane` and `signal-runtime` update in the same batch
- renders above the ceiling belong to the chunk plan, which `g10.039` makes
  stateful

## Validation Run

- overlap law probe against the public API, two sources, seven ratios, two
  candidate bounds
- `effigy qa:docs` — link, forbidden, heading, index, and next-action checks
  passed

## Next Task

Execute `g10.036` Batch 36.2: repair the thread-unsafe allocation gate, prove
the repair with two full-suite runs, then add the failing regression owners for
overlap coverage and ripple above ratio `3.0`, dense-curve pitch preservation,
seam parity, and the `268435456`-sample output bound.
