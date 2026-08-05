# g10.039 Batch 39.5 - Closeout

Status: complete
Created: 2026-08-05
Scope: resumable offline stretch render, listening admission, lane closure

## Listening Verdict

Revision 2 of the concealed pack was judged on 2026-08-05: no significant
difference between the sides of any pair. Contract `084` Rule 5 makes listening
the promotion authority, and admission requires only that no case prefers the
shipped side, so the resumable renderer is admitted.

Concealment held. The adopted side was `A` in `D1` and `B` in `D2` and `D3`. A
positional bias would have surfaced as a consistent preference across cases and
did not.

## What The Verdict Does Not Say

This is parity, not a demonstrated improvement, and the distinction matters for
what can be claimed downstream.

The pack was built to discriminate on the `g10.036` seam pulse. That artifact
was not reported on the *shipped* side of this pack either, so the pack did not
reproduce the thing it was meant to separate on. What it establishes is that
carrying state across chunk boundaries costs nothing audible — enough to justify
adopting a structurally better renderer, not enough to say the seam artifact is
solved.

`A18` is likewise unresolved. Revision 1 found low-mid pops on the ticks in the
shipped `D2` and `D3`. Revision 2 reports no difference between sides, which
does not separate "gone from both" from "present in both". `A18` stays open and
needs a probe that measures the transient directly rather than another pack that
compares two renderers against each other.

## Seam Mechanisms Remaining

Both seam smoothers stay. This is not deferral: adoption is partial by design.
The resumable renderer owns the default offline path with no pitch shift, while
selector paths and pitch composition still take the legacy per-chunk path and
still create the boundaries the smoothers patch. Removing them requires adopting
those paths, which is `g10.040` work.

## Evidence

- `effigy release gates`: 6/6 pass — `docs`, `fmt`, `lint`, `lint:no-features`,
  `test`, `validate`. The `test` gate is the full workspace suite.
- `cargo run --release -p signal-dsp-stretch-evidence --bin stretch-corpus-report
  --all-features`: 27 comparisons, 14 `Improved`, 13 `Unchanged`, zero
  regressed.
- `effigy qa:northstar`: passes.

## Lane State

`g10.039` is complete. `g10.040` moves from `planned` to `ready`, and Batch 40.1
opens. It inherits both items this lane did not settle: adopting the remaining
offline paths so the smoothers can go, and a direct transient probe for `A18`.

## Next Task

Tag `v0.1.0`. The release lane was blocked only on this admission.
