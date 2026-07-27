# 2026-07-27 g10.036 Batch 36.3 Overlap Law And Output Bound

Status: complete

Batch 36.3 is the first batch to change the retained Transparent renderer. It
implements the Contract `046` overlap coverage law and the output bound, under
Contract `084` Rules 9 and 10.

## Overlap Law

`overlap_safe_analysis_hop` lands in `phase_vocoder.rs` and is applied inside
`run_phase_vocoder`, the single choke point every offline engine call passes
through. Mono, linked stereo, pitch composition, and dynamic-ratio segment
renders inherit it without duplicated logic.

The law returns the configured hop unchanged whenever
`analysis_hop * ratio <= 0.75 * window_size`, and otherwise
`floor(0.75 * window_size / ratio)`, clamped to `1..=analysis_hop`. It never
enlarges a caller's hop.

Result at the retained `2048/512` geometry:

| ratio | hop before | hop after | ripple before | ripple after | zeroed before | zeroed after |
| --- | --- | --- | --- | --- | --- | --- |
| `2.0` | `512` | `512` | `0.276 dB` | `0.276 dB` | `0` | `0` |
| `3.0` | `512` | `512` | `0.276 dB` | `0.276 dB` | `0` | `0` |
| `4.0` | `512` | `384` | `1.396 dB` | `0.276 dB` | `0` | `0` |
| `5.0` | `512` | `307` | — | — | `90/…` | `0` |
| `6.0` | `512` | `256` | `237.126 dB` | `0.276 dB` | `183/547` | `0` |
| `8.0` | `512` | `192` | `237.126 dB` | `0.358 dB` | `368/734` | `0` |

## Output Bound

`TimeStretcher::stretch_mono` and the eleven whole-buffer entry points beside
it return `Result<Vec<Sample>, StretchRenderError>`. `73` call sites across six
files updated; `signal-render-plane` and `signal-runtime` build against the new
signatures in this batch, as the operator decision required.

`MAX_OFFLINE_STRETCH_OUTPUT_SAMPLES` is `268435456` and counts every channel,
not frames. Ratio `1.0e6` over `4096` input frames now returns
`OutputTooLarge { requested_samples: 4096000000, maximum_samples: 268435456 }`
instead of allocating `4096000000` samples over roughly a minute.

Internal delegations inside the crate propagate the error rather than
unwrapping. The one place that cannot propagate is the expansion selector's
draft comparison render: it stays on the current path when the draft render
cannot be produced, because switching on missing evidence would be worse than
not switching. That render is the same size as one that already succeeded, so
the branch is unreachable in practice.

## Method Correction: Byte-Exactness Proof

The first attempt froze output hashes for ratios `0.5..3.0` as an integration
test. They passed under `--release` and failed under the default profile. f32
render output is optimization-profile dependent, so an absolute output hash is
only valid in the profile that captured it. A profile-dependent assertion in a
shared suite is a trap, so the hash test was removed.

The claim is now proven structurally in `phase_vocoder::tests`:

- `overlap_safe_analysis_hop_is_a_no_op_through_ratio_three` asserts the law
  returns the configured `512` hop for every ratio through `3.0` at the
  retained geometry. The rendering path is therefore unchanged by construction
  rather than by observation
- `overlap_safe_analysis_hop_bounds_the_synthesis_hop` asserts each adapted hop
  and that every one keeps synthesis hop inside the bound
- `overlap_safe_analysis_hop_stays_within_caller_bounds` covers the clamp,
  non-finite ratio, and zero ratio paths
- the pre-existing `phase_vocoder_bit_exact_baseline` at ratio `1.5` still
  passes with its original hash `0x8255b18311f778f9`

Release-profile hashes captured before and after, as Contract `084` Rule 10
evidence. Ratios through `3.0` are identical; only `4.0` and above moved:

| ratio | mono before | mono after | stereo before | stereo after |
| --- | --- | --- | --- | --- |
| `0.5` | `0xdce5d9954045670d` | identical | `0x03c96dadebe4325c` | identical |
| `0.75` | `0x9c6cb46cdc2d4daa` | identical | `0xa1fc4f0fad525e2a` | identical |
| `1.25` | `0xa7bd73eb6ebc5adb` | identical | `0x25f97bf0ae8db5d2` | identical |
| `1.5` | `0x9a09a840aa6cd8d2` | identical | `0x4ee821fc49823f7b` | identical |
| `2.0` | `0xcd8a6315902c52af` | identical | `0xa1d9a0fb721bb897` | identical |
| `2.5` | `0xcaf4c3bba062b56d` | identical | `0x9a499af8216f58be` | identical |
| `3.0` | `0xead24515a648ae83` | identical | `0xcf279977c20cc666` | identical |
| `4.0` | `0xd3e45b0e45a3d728` | `0x16322f772f1ec017` | `0x94dfeede7146ce2d` | `0xdba1fa57b58acaf3` |
| `6.0` | `0x9b0c8257654a48d8` | `0xbbcc28475460355e` | `0xaf7195637006766d` | `0xb88dbc7e3d98cac9` |

The `4.0` and `6.0` changes are justified by the ripple and coverage tables
above: `1.396 dB` to `0.276 dB` at `4.0`, and `183` zeroed interior blocks to
none at `6.0`.

## Render Cost

One second of 48 kHz mono, release profile:

| ratio | after |
| --- | --- |
| `2.0` | `5.08 ms` |
| `3.0` | `5.01 ms` |
| `4.0` | `7.98 ms` |
| `6.0` | `11.86 ms` |
| `8.0` | `19.44 ms` |

Cost is unchanged where the law is a no-op and rises only where the hop
tightens, which is only where the renderer previously produced ripple or
silence.

## Owner State

| owner | state |
| --- | --- |
| `overlap_coverage_has_no_zeroed_interior_block` | active, passing |
| `overlap_ripple_stays_within_ceiling` | active, passing |
| `oversized_output_request_is_refused` | active, passing |
| `render_inside_the_output_bound_is_served` | active, passing |
| `output_bound_counts_every_channel` | active, passing |
| `overlap_law_leaves_low_ratios_byte_exact` | active, passing |
| `dense_ratio_curve_preserves_output_length` | active, passing |
| `dense_ratio_curve_preserves_pitch` | ignored, Batch 36.4 |
| `dynamic_ratio_seam_click_matches_across_channel_counts` | ignored, Batch 36.4 |

## Validation Run

- `cargo test -p signal-dsp-stretch -p signal-render-plane -p signal-runtime`:
  all green. `182` lib tests, up from `179`; `7` owners passing with `2`
  ignored; `144` render-plane tests; all runtime boundary suites
- `cargo clippy --workspace --all-targets --all-features`: no new warnings.
  The pre-existing set is unchanged and remains a `g10.038` input
- `effigy validate`
- render-cost and hash captures through temporary probes, deleted after use

## Next Task

Execute `g10.036` Batch 36.4: coalesce dynamic-ratio segments so no admitted
curve produces a sub-window segment, give the mono dynamic-ratio path the same
seam treatment as linked stereo, activate the two remaining owners, and run
concealed listening on dynamic-ratio material before admission. Both
corrections change audible output inside the retained product range, so
Contract `084` Rule 5 evidence applies.
