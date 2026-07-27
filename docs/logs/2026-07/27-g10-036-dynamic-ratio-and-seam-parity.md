# 2026-07-27 g10.036 Batch 36.4 Dynamic Ratio Segments And Seam Parity

Status: objective evidence complete; concealed listening pending operator

Batch 36.4 corrects the two remaining audible defects. Both change output
inside the retained `0.5x..4x` product range, so Contract `084` Rule 5 applies:
the code and objective rows are complete, and admission waits on concealed
listening.

## Segment Coalescing

`coalesce_short_dynamic_ratio_segments` merges adjacent curve spans until every
render segment carries at least `min_dynamic_ratio_segment_frames`. The merged
target frame count is the sum of the counts its constituent spans would have
produced, so total output length and average tempo are preserved exactly and
the merged segment renders at the mean ratio of the spans it covers.

Applied to the mono, linked-stereo, and dynamic-ratio-with-pitch render paths,
and to `dynamic_ratio_output_boundaries` so the seam metric measures the seams
the renderer actually produces.

### Choosing the minimum

Contract `046` froze one window as the floor. Measurement showed one window is
not enough: a single-window segment avoids the interpolation fallback but hands
the phase vocoder one analysis frame. Swept on a `440 Hz` tone through a curve
sampled every `1024` frames at ratio `2.0`, retained `2048/512` geometry:

| minimum | source frames | dominant frequency | pitch error |
| --- | --- | --- | --- |
| `window + 3 hops` | `3584` | `445.0 Hz` | `19.6` cents |
| `window + 8 hops` | `6144` | `440.7 Hz` | `2.8` cents |
| `window + 16 hops` | `10240` | `440.7 Hz` | `2.8` cents |
| `window + 32 hops` | `18432` | `440.0 Hz` | `0` cents |
| `window + 64 hops` | `34816` | `440.0 Hz` | `0` cents |

Eight extra hops is frozen. Beyond it the pitch gain is inaudible and the cost
is real: a longer minimum averages more of the ratio curve into one segment,
so ratio-curve time resolution falls. At the retained geometry the minimum is
`6144` source frames, `128 ms` at 48 kHz. Contract `046` is amended with this
table and the chosen value.

## Seam Parity

The mono dynamic-ratio path now runs the same
`smooth_dynamic_segment_boundaries_interleaved` pass the interleaved path
already ran, at one channel.

## Objective Results

| measurement | before | after |
| --- | --- | --- |
| dense-curve dominant frequency, `440 Hz` source | `220.0 Hz` | `440.7 Hz` |
| dense-curve output length | `96000` | `96000` |
| mono seam click | `-28.940011 dBFS` | `-180.617997 dBFS` |
| stereo seam click | `-180.617997 dBFS` | `-180.617997 dBFS` |
| tempo-ramp dominant frequency | — | `440.5 Hz` |

The mono and stereo seam figures are now identical rather than `151.7 dB`
apart. The stereo path is unchanged, which is the intended result: it was
already correct.

## Owners

All ten owners in `transparent_correctness_owners.rs` are active and passing;
none remain `#[ignore]`d. Two were added or tightened in this batch:

- `dense_ratio_curve_preserves_pitch` tolerance tightened from `2%` to `0.5%`,
  since the measured result is `0.16%`. The loose tolerance would not have
  guarded a regression
- `segment_coalescing_preserves_total_output_length` is new. It asserts the
  summed-target law across four curve shapes — dense uniform, two coarse spans,
  tempo ramp, and a short tail span — in mono and linked stereo

## Scope Boundary Recorded

The coalescing law governs the renderer. `plan_offline_stretch_chunks` still
segments on raw curve points, so a dense curve can still produce sub-window
chunks in the artifact path. That is not fixed here because the durable fix is
a renderer that carries state across a chunk boundary, which is `g10.039`.
Contract `046` records the boundary.

## Concealed Listening Pack

Built at `~/Downloads/signal-listening-pack-36-4`. The pre-correction side was rendered from
a clean git worktree at `e24dadc6`, the commit before any Batch 36.3 or 36.4
code, so both sides come from real renderers rather than a reconstruction.

Source material is a sustained three-note chord with a percussive click every
`250 ms`, four seconds at 48 kHz, so tonal continuity and transient placement
are judged together.

| case | tests | curve |
| --- | --- | --- |
| `C1` | dense-curve pitch preservation | one point every `1024` frames, ratio `2.0` |
| `C2` | mono segment seam | `1.5` for two seconds, then `0.75` |
| `C3` | tempo ramp | eight spans, `1.0` rising to `1.84` |
| `C5` | static ratio in the audible overlap window | fixed `4.0` |

`C4`, the linked-stereo control at the `C2` curve, is byte identical on both
sides and is supplied as a single file rather than a pair.

A and B assignment is randomized per case and recorded in `key.tsv`. The
labelled renders were moved to `~/Downloads/signal-listening-pack-36-4-reveal` so the pack
itself carries no hint. `notes.tsv` holds one row per case.

The temporary render example and the pre-change worktree were deleted after
use; neither entered `main`.

## Validation Run

- `cargo test -p signal-dsp-stretch -p signal-render-plane -p signal-runtime`:
  green. `182` lib tests, `10` owners, `144` render-plane tests, all runtime
  boundary suites
- `cargo clippy -p signal-dsp-stretch --all-targets --all-features`: no new
  warnings; the nine pre-existing ones are unchanged and remain `g10.038` work
- `effigy validate`
- segment-minimum sweep and before/after measurements through temporary probes,
  deleted after use

## Next Task

Operator action: audition `~/Downloads/signal-listening-pack-36-4`, fill `notes.tsv`, then
open `key.tsv`. The correction is admitted only if no case prefers the
pre-correction side. On admission, close Batch 36.4 and execute Batch 36.5:
full corpus comparison and acceptance report, contract and front-door updates,
and the corrected Transparent behavior matrix.
