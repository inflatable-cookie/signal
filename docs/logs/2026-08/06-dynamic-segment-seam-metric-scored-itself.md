# The Seam Metric Was Measuring Its Own Smoother

Status: complete
Created: 2026-08-06
Scope: `signal-dsp-stretch` dynamic-ratio rendering, and the promotion metric
that let its seam survive being measured

## The Question

Whether `smooth_dynamic_segment_boundaries_interleaved` can be removed, left
open with three recorded callers.

## Scope First

Established before touching anything, per the Batch `42.1` precedent:

- No consumer uses the dynamic-ratio API. `finch`, `soundcheck`, `loophole`,
  `soundcheck-library` and `songsprout` mention neither
  `stretch_dynamic_ratio_*` nor `OfflineHighQualityStretcher`.
- The offline artifact renderer never reaches it. Selector paths take
  `stretch_interleaved_stereo` whole-buffer and reject dynamic ratio outright;
  `Default` goes to the resumable renderer. Neither branch touches the
  segmented path.

So the segmented dynamic-ratio path was reached only through the crate's own
public API, its tests, and the benchmark corpus.

## What The Segmented Path Actually Does

All three callers share one shape: render each ratio segment independently,
concatenate, smooth the joins. Independent renders mean the phase vocoder
restarts at every join — the same defect the resumable renderer was built to
remove for chunk seams.

Measured on a sustained `110 Hz` tone across a `1.0 -> 1.6 -> 0.8` curve, peak
first-difference near the seams:

| render | seam |
| --- | --- |
| segments concatenated raw | `-14.18 dBFS` |
| the same, smoothed | `-18.83 dBFS` |
| resumable renderer | `-76.39 dBFS` |

The smoother buys `4.65 dB` against a `57.6 dB` problem.

## The Metric Could Not See Any Of That

`DynamicSegmentSeamClickDbfs` read exactly `|x[seam - 1] - x[seam]|` — the one
pair straddling the boundary.

`smooth_dynamic_segment_boundaries_interleaved` iterates outward from the
boundary with `weight = (fade - offset) / fade`. At `offset == 0` the weight is
`1.0`, so both of those samples are assigned their own midpoint. They are equal
*by construction* whenever the smoother has run, whatever the seam is.

Demonstrated rather than argued. A full-scale `+1 -> -1` step, smoothed with a
**one-frame** fade that leaves the discontinuity entirely intact:

```
worst possible seam: raw 2.000000 (6.0 dBFS) -> smoothed 0.000000 (-240.0 dBFS)
samples either side:  [1.0, 1.0, 1.0, 0.0, 0.0, -1.0, -1.0, -1.0]
```

`-240 dBFS` is the silence sentinel. The metric reported whether the smoother
had run, not whether there was a seam.

And it was load-bearing.
`stretch_quality_priorities_are_regression_only_and_sorted` asserts
`priorities.is_empty()`, so a perfect-by-construction score on the baseline was
pinned in place as evidence of no dynamic-ratio seam regressions. Moving the
public API to the resumable renderer surfaced as the *only* regression in the
corpus: baseline `-240.00`, candidate `-46.51`. The correct render looked like
the regression.

## The Fix To The Metric

Two changes, each addressing a separate blindness:

- **A window.** Peak first-difference within `384` frames either side of a seam,
  which exceeds `DYNAMIC_RATIO_SEAM_SMOOTH_FRAMES` (`256`). A smoother does not
  remove a discontinuity, it spreads it across its fade; a window narrower than
  the fade misses where it went. This is why the earlier `+/-32` frame probe put
  the smoothed render at `0.0174` while the full window puts it at `0.114`.
- **A floor.** Adjacent samples of any non-DC material differ, so an absolute
  threshold cannot separate a seam from a waveform. The floor is the `p99.9`
  first-difference over frames outside every seam window — the render's own
  idea of a large step — and only the excess over it is reported.

A render with no frames outside its seam windows returns NaN. Unmeasurable, not
clean: falling back to all steps would let the seam set its own floor and score
itself perfect, which is the failure being replaced.

## The Smoother Is Not Neutral

`offline_high_quality_dynamic_ratio_smoothing_reduces_segment_seams` applied the
smoother to clean sines at two arbitrary frames and asserted the seam got
smaller. Under the fixed metric it fails, because there was no seam to shrink:

```
continuous material: before -240.00 dBFS after -70.86 dBFS
```

The smoother *introduces* a discontinuity where none existed. It drags `64`
frames either side of the nominated frame toward the midpoint of a pair that was
already continuous. The old metric could not report this either — it reads the
pair the smoother equalises.

That test now asserts the real behaviour and is renamed
`dynamic_segment_seam_smoothing_is_not_neutral_on_continuous_material`.

## The Change

`OfflineHighQualityStretcher`'s three dynamic-ratio methods — mono, interleaved
stereo, and pitch-plus-dynamic stereo — render through `ResumableOfflineStretch`
in one call. The pitch variant no longer resamples per segment; resampling runs
ahead of the stretch over the whole stream, the same order the offline artifact
renderer uses.

`RealtimePreviewStretcher` is untouched. It is a different quality tier, and an
early revision of this change moved it by accident because both structs expose
identically named methods.

**No `SIGNAL_STRETCH_BEHAVIOR_VERSION` bump.** The artifact renderer does not
reach any changed path, so no cached artifact renders differently. Bumping would
have invalidated every cache to no effect.

`stretch_dynamic_ratio_mono` and `stretch_mono` are no longer bit-identical at a
static ratio — same length, worst absolute difference `5.4e-5`. That is a real
consequence of the dynamic API moving to a resumable renderer, and it is now
asserted as a bound rather than as equality.

## Answer To The Removal Question

The smoother cannot be deleted. `RealtimePreviewStretcher` still renders
segmented through both `_with_engine` helpers, and the benchmark's
per-channel-independent control uses the mono one. What is now recorded is that
it buys `4.65 dB` on a real seam and costs `-70.9 dBFS` on material without one.

Whether the preview tier should also render resumably is a quality-tier change
and therefore listening work under Contract `084` Rule 5. Not done, not
prescribed — the last two prescriptions written into a log were both wrong, and
this one has no measurement behind it yet.

## Verification

Both new tests were run against the code they describe before being trusted.
`resumable_dynamic_ratio_has_no_seam_where_segmented_rendering_does` asserts the
segmented renders are visible *and* the resumable one is `40 dB` below them, so
it fails if the measurement stops working in either direction. The
`-40 dBFS` bound on the smoothed ramp was written first at a value that failed
against real output (`-60.20 dBFS`) and corrected, rather than the bound being
fitted to whatever came out.

Workspace: `89` test binaries green, clippy clean at `--all-features`.

## Next Task

None here. Open elsewhere: `g10.040` Batch 40.6, gated on a consumer asking for
a live preview path, and the preview-tier question above if a listening pack is
ever warranted.
