# g10.042 Batch 42.4 - Pitched Renderer Pack Built

Status: pack built; blocked on listening admission
Created: 2026-08-05
Scope: the A/B that decides whether the chunked renderer and its seam smoother go

## What The Pack Decides

Pitch-shifted multi-chunk artifacts are the only case still served by
`materialize_chunked_offline_stretch_artifact_frames`, and therefore the only
remaining reason `smooth_artifact_chunk_boundaries_interleaved` exists. If the
resumable renderer is no worse, both are deleted.

`~/Downloads/signal-listening-pack-42-pitch`. Two cases at `90s` and `150s`,
crossing `3` and `5` chunk boundaries, `+5` semitones at ratio `1.25`, on a
sustained low chord with a percussive attack every `500 ms`.

Built by an `#[ignore]`d test inside `offline.rs`, which is what lets it reach
both private renderers directly with one shared chunk plan rather than
approximating the comparison from outside the crate.

## Checked Before Delivery

- both renderers honour the planned length exactly
- every decile of both carries audio
- the sides differ by `1.65` peak, so the pack can discriminate
- RMS matches within `0.2%`, so level gives nothing away

The decile check exists because `g10.039` shipped three silent specimens: five
structural gates passed a renderer emitting nothing, since each measured a
relationship between renders rather than content.

The discrimination check exists because `g10.039` revision 2 came back "no
significant difference" partly because it never reproduced the artifact it was
built to judge.

## Two Numbers That Have To Be Read Together

A `1.65` peak difference on material peaking near `1.05` looks like one side is
broken. The levels say otherwise: RMS matches within `0.2%`, so the sides are
equally loud and differ in waveform, which is what two phase-vocoder renders of
the same material look like.

Either check alone misleads. The difference alone suggests breakage; the levels
alone would have passed a pack whose sides were identical and unjudgeable.

## Not Adopted

`resumable_render_supported` still excludes pitch. Pitched artifacts take the
legacy path and both smoothers stay until listening admits the change, per
Contract `084` Rule 5 — the same gate that rejected `g10.039`'s first attempt at
the default path.

## Next Task

Judge the pack. If no case prefers the legacy side, route pitched artifacts
through the resumable renderer and delete the chunked renderer and its smoother.
