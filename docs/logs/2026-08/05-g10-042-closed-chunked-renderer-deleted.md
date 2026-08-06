# g10.042 Closed - Chunked Renderer And Seam Smoother Deleted

Status: complete
Created: 2026-08-05
Scope: pitched adoption and the deletion this lane existed for

## Admitted

Judged 2026-08-05. No case preferred the legacy side.

| case | A | B | reported |
| --- | --- | --- | --- |
| `F1` | resumable | legacy | both similar, no obvious seams; `A` consistent DC, `B` varies a lot |
| `F2` | legacy | resumable | same result, `B` consistent, `A` varies |

The listener identified the legacy renderer in both cases without knowing which
side was which, and the sides were swapped between them.

Measured per one-second window: legacy's DC range and worst step are `0.00150`
in both cases, against `0.00125` and `0.00142` for resumable. Small in absolute
terms — around `0.15%` of full scale, visible at zoom rather than audible — but
consistent in direction and picked out blind twice.

Neither side showed the seam artifact the pack was built to look for. The
admission rests on "no case prefers legacy", which is what Rule 5 asks, not on
the seam being visibly fixed.

## Deleted

`materialize_chunked_offline_stretch_artifact_frames`,
`smooth_artifact_chunk_boundaries_interleaved`,
`materialize_stretch_chunk_payload`, and
`OFFLINE_STRETCH_ARTIFACT_CHUNK_CROSSFADE_FRAMES`. `398` lines out of
`offline.rs` against `19` added.

The `is_single_chunk` branch went with them. With `Default` on the resumable
renderer and both selectors rendering whole-buffer, the three variants of
`OfflineHighQualityPath` leave it unreachable — and removing it closes a latent
version of the defect this lane already fixed once. A single-chunk pitched
artifact used to take a whole-buffer call while a multi-chunk one took the
chunked renderer, so length selected the algorithm under one cache key. What
replaced it is an `unreachable!` that names why, rather than a silent fallback.

`SIGNAL_STRETCH_BEHAVIOR_VERSION` advances to
`signal-stretch-behavior-2026-08-05-pitch-resumable`.

## The Surviving Smoother

`smooth_dynamic_segment_boundaries_interleaved` stays, and its callers are now
recorded rather than assumed:

- `TimeStretcher::stretch_dynamic_ratio_pitch_interleaved_stereo`
- `stretch_to_exact_mono`
- `stretch_dynamic_ratio_linked_stereo_with_engine`

None is the artifact path. It patches dynamic-ratio *segment* joins inside the
whole-buffer stretcher, not chunk joins, so it never left with the chunked
renderer and removing it is a separate question.

## A Rate That Needed Its Condition

The `test` gate failed twice today on `signal-plugin-sandbox` `plugin_hosting`,
which is `A22`'s binary and untouched by this lane. Measured rather than
re-rolled: `2` failures in `20` runs.

That number needed its condition attached, and nearly did not get it. The `2/20`
was measured while a release gate was running in another process. Repeated on an
idle machine, the same binary passed `20` consecutive runs.

So it is load-dependent — the `A20`/`A22` family — not a fixed `10%` rate. Left
unqualified, "fails `10%` of the time" would have sent the next attempt looking
for a probabilistic defect in the retry logic rather than for contention.

Two distinct failure modes appeared: a session failing during spawn, and a render
where wet equalled dry because a missed response bypassed. The second is what
epoch retirement looks like from outside, and `process_with_retries` still cannot
recover from it — the `A19` re-attach fix was not applied there because the twelve
call sites each hold their own handle and the lease is not threaded through them.

Both the rate and its condition are now written at that constant.

## Next Task

Nothing in `g10.042`. Open elsewhere: `g10.040` Batch 40.6, a live preview render
path gated on a consumer asking; and the sandbox retry limit above, which now has
a number attached to it.
