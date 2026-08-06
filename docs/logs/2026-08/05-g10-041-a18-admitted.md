# g10.041 - A18 Admitted And Adopted

Status: complete
Created: 2026-08-05
Scope: listening verdict, adoption, and one finding that was not ours

## The Verdict

Judged 2026-08-05. No case preferred the shipped side, so the candidate is
admitted under Contract `084` Rule 5.

| case | A | B | reported |
| --- | --- | --- | --- |
| `E1` | candidate | shipped | candidate "very slightly garbled" bass, otherwise clean; shipped sounded the same |
| `E2` | shipped | candidate | candidate "bass tone more centred around zero" |
| `E3` | candidate | shipped | shipped "has bass pop on each tick, makes the bass tone throb when tick plays" |

`E3` is the result that matters. The listener identified the pop on the shipped
side, at ratio `2.0`, without knowing which side was which — and ratio `2.0` is
exactly where the measurement put the peak at `2.752 rad` against a `0.142 rad`
floor. The description, "makes the bass tone throb when the tick plays", is a
phase discontinuity in a sustained low component described from the other end.

That is independent confirmation rather than agreement. The measurement predicted
which case and which side would carry it, before anyone listened.

## The Zero-Offset Wander Was The Fixture

The listener also reported something the measurement had not: the waveform centre
"jumps about a lot" across the render, on both sides, visible in the editor.

Measured per `100 ms` window:

| file | max abs DC | DC range |
| --- | --- | --- |
| `E1` source | `0.01974` | `0.03926` |
| `E1` candidate | `0.02020` | `0.03993` |
| `E1` shipped | `0.02057` | `0.03894` |
| `E2` source | `0.01974` | `0.03926` |
| `E2` candidate | `0.01705` | `0.03182` |

It is already in the source at the same magnitude, so the stretcher is not
introducing it. The `E2` candidate actually reduces it.

The cause is the pack material. The percussive attack is a `30 ms` burst with a
`5 ms` exponential decay, so each burst's mean is dominated by its first few
noise samples and lands non-zero — a DC step at every tick, baked in before any
stretching.

Recording it as a fixture defect rather than a finding. A listener spent
attention on an artifact the test signal contributed, and a future pack should
high-pass or zero-mean its noise bursts.

## Adoption

`transient_reset_phase_vocoder` now uses
`IdentityLockedTransientResetHighBand` with
`TRANSIENT_RESET_CROSSOVER_FRACTION = 0.010` — `240 Hz` at `48 kHz`. The
all-bin variant is retained as the control the `A18` evidence compares against.

`SIGNAL_STRETCH_BEHAVIOR_VERSION` advances to
`signal-stretch-behavior-2026-08-05-a18-crossover`, so cached artifacts rendered
by the old path invalidate correctly.

`a18_offline_stretch_does_not_break_low_carrier_phase` is un-ignored and passes.
The `#[ignore]` carried the measured value in its reason while the defect was
open, per the `g10.039` `G5` precedent; it now guards the fix.

## Findings State

`A18` closed with a mechanism, a fix, and listening admission. Every finding
from the audit that opened this generation is now closed or relocated:

- `A18` closed, fixed, admitted
- `A19` closed, use-after-unmap with a mechanism
- `A20` relocated to the soak lane
- `A21` closed as unmeasurable by its harness
- `A22` closed, caused by soak-test contention
- `A23` closed, process-global env race

## Next Task

None in `g10.041`. Open elsewhere: adopting the remaining offline paths so both
seam smoothers can be removed, and `g10.040` Batch 40.6 gated on a consumer
asking for a live preview path.
