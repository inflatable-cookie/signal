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

## A Rate Worth Recording

The `test` gate failed twice today on `signal-plugin-sandbox` `plugin_hosting`,
which is `A22`'s binary and untouched by this lane. Measured rather than
re-rolled: `2` failures in `20` runs, `10%`, across
`fixture_plugin_processes_a_chain_insert_through_the_real_engine_offline_render`
and `real_child_instrument_accepts_zero_input_and_generates_audio_from_note_events`.

Serialising the child-spawning tests earlier today cut the rate without removing
it, which is consistent with the documented limit rather than with contention:
`process_with_retries` cannot recover once the processor retires its epoch after
three consecutive misses. The re-attach fix that closed `A19` was not applied
there because the twelve call sites each hold their own handle and the lease is
not threaded through them.

The measured rate is now written at that constant. A `10%` flake on a release
gate is a gate that can be retried until green, which is the thing this
generation keeps saying it will not accept.

## Next Task

Nothing in `g10.042`. Open elsewhere: `g10.040` Batch 40.6, a live preview render
path gated on a consumer asking; and the sandbox retry limit above, which now has
a number attached to it.
