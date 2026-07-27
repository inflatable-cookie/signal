# 2026-07-27 g10.036 Batches 36.4 and 36.5 Admission And Closeout

Status: complete; `A2` admitted with a recorded limitation

Listening round two rejected the revised segment minimum on the same case.
Measurement then found the mechanism, Contract `084` Rule 7 closed segmentation
tuning, and the operator admitted the correction with its residual documented.

## Listening Round Two

Revision-2 pack compared the `128 ms` and `384 ms` minimums.

| case | verdict |
| --- | --- |
| `C1` dense curve | pulse still present in both, on a different time base |
| `C3` tempo ramp | low-end pops on ticks still present |

The `384 ms` minimum reduced measured envelope modulation `4.7x`, from
`0.545 dB` to `0.115 dB`, and made dense-curve pitch exact. It did not remove
what the operator heard. The reported change of time base was the clue: the
artifact tracks segment length, so it is seam-locked.

## Mechanism

Rendering a **constant** ratio `2.0` through the segmented path, against the
same ratio rendered whole:

| measurement | value |
| --- | --- |
| correlation | `0.034` |
| peak sample difference | `1.1470` |
| difference RMS against signal RMS | `0.2474` against `0.1784` |

Two renders of identical material at an identical ratio are almost
uncorrelated, and the difference is louder than the signal. Each segment
restarts the phase vocoder, so the phase relationship across every join is
arbitrary.

The audible artifact is phase re-initialisation, not amplitude modulation. The
envelope metric that guided the first revision was measuring the wrong thing:
it fell `4.7x` while the audible defect did not. Segment length changes the
rate of the discontinuity and never its existence, so no minimum can fix it.

Contract `084` Rule 7 applies: two failures with the same dominant cause
trigger architecture reassessment, not another parameter sweep. Segmentation
tuning is closed as a mechanism.

## This Predates The Lane

Contract `046` already required that dynamic-ratio output "preserves continuous
algorithm state or uses an explicitly measured transition mechanism rather than
raw segment concatenation". The implementation has never satisfied it. The
audit's `A2` finding described the sub-window interpolation fallback, which is
one symptom; the phase restart is the deeper defect underneath it, and it
affects every dynamic-ratio render, not only dense curves.

`A18`, the low-end pops on transients that round one found in both sides, is
almost certainly the same mechanism at transient positions rather than a
separate defect.

## Operator Decision

Admitted under Contract `084` Rule 5, by explicit decision rather than a clean
listening pass. The correction:

- replaces an octave-wide pitch error with a milder seam artifact
- takes dense-curve pitch from `220.0 Hz` to exactly `440.0 Hz`
- cuts joins on the measured curve from about `46` to `10`

The residual pulse is recorded as a known limitation with its measurement, not
carried as a tuning target. `g10.039` removes it by carrying renderer state
across the join.

`segmented_render_matches_whole_render_at_constant_ratio` is new and
`#[ignore]`d, asserting the `0.99` correlation a transparent segmentation must
reach. It is `g10.039`'s acceptance target and is expected to fail until then.

## Batch 36.5 Closeout Evidence

Corpus comparison report, `stretch-corpus-report`:

| field | value |
| --- | --- |
| comparisons | `27` |
| improved | `14` |
| regressed | `0` |
| unchanged | `13` |
| inconclusive | `0` |
| missing required assets | `5`, all operator-provided licensed material |

Zero regressions across timing drift, loop-boundary click, stereo image,
transient smear, pitch error, vertical coherence, and dynamic segment seam
click. The five missing cases are the licensed listening families, which are
operator-supplied and outside the repository as the source policy requires.

## Corrected Transparent Behavior

| behavior | before `g10.036` | after |
| --- | --- | --- |
| overlap coverage above ratio `4.0` | interior blocks zeroed | full coverage at every accepted ratio |
| ripple at ratio `4.0` | `1.396 dB` | `0.276 dB` |
| ratios `0.5x..3.0x` | — | byte-identical, proven structurally |
| dense ratio curve | `220.0 Hz` from a `440 Hz` source | `440.0 Hz` |
| dynamic-ratio segment minimum | none | `window + 32 hops`, `384 ms` at 48 kHz |
| sub-minimum curve spans | rendered as varispeed | merged, rendered at the mean ratio, length exact |
| mono seam click | `-28.940011 dBFS` | `-180.617997 dBFS`, matching stereo |
| oversized render | allocated `4096000000` samples | refused with `OutputTooLarge` |
| whole-buffer ceiling | none | `268435456` output samples |
| segment join phase | arbitrary, unmeasured | arbitrary, measured and recorded; `g10.039` owns it |

## Candidate Finding A19

`cargo test --workspace` failed once on
`signal-plugin-bridge` `shm::tests::served_request_round_trips_through_the_region`.
It then passed three times in isolation and `20/20` across the whole
`signal-plugin-bridge` lib suite, with this lane's changes present, and it
passes with those changes stashed. The failure appeared only under
full-workspace parallel load.

Nothing in `g10.036` touches `signal-plugin-bridge` or shared memory. This is
recorded as a suspected flaky shared-memory test, the same evidence-integrity
class as `A17`, and it needs a reproduction under parallel load before any
diagnosis. It is not attributed to this lane and not proven flaky yet.

## Validation Run

- `stretch-corpus-report`: `27` comparisons, `0` regressed, `0` inconclusive
- `cargo test --workspace`: green apart from the single `A19` occurrence above
- `cargo clippy --workspace --all-targets --all-features`: no new warnings
- `effigy validate`, `effigy qa:docs`
- phase-correlation and envelope-modulation measurements through temporary
  probes, deleted after use

## Next Task

Execute `g10.037` Batch 37.1: enumerate every input that changes rendered
output against the current cache identity fields, amend Contract `046` for
render geometry and stable key tokens, and decide the schema advance and the
creative-cache position. Documentation only.

`A18` needs triage before it is routed to any batch; the working hypothesis is
that `g10.039` resolves it along with the seam pulse. `A19` needs a
reproduction under parallel workspace load before it is diagnosed.
