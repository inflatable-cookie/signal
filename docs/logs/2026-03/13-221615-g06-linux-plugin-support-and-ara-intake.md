# 2026-03-13 22:16:15 GMT - g06 Linux plugin support and ARA intake

## Summary

Adjusted the new `g06` planning spine so Linux plugin support and ARA are
explicitly planned rather than left implicit behind broader plugin-breadth
language.

## Work completed

- updated `g06` generation framing so the feature-breadth lane now explicitly
  includes Linux-hosted plugin coverage and ARA-capable clip/plugin context
- widened `g06.009` so the VST3 baseline names Linux-hosted plugin coverage as
  a required part of the first real adapter path
- clarified in `g06.010` that AU is intentionally macOS-scoped and should not
  be treated as the Linux answer
- retitled and widened `g06.011` so cross-adapter conformance also carries
  explicit Linux platform coverage where CLAP/VST3 are expected to support it
- retitled and widened `g06.013` so Signal plans a first bounded ARA-capable
  plugin context contract alongside preset/state interchange and portable recall

## Current assessment captured in planning

- internal engine processing is clearly `f32` today (`Sample = f32`), but that
  does not by itself settle all separate questions around file/container bit
  depth, integer PCM interchange, or future `f64` engine policy
- stretch/warp is already a real Signal-owned runtime substrate, but the docs
  still record it as bounded warp/tempo realization rather than the whole
  product-facing time-stretch workflow
- AU and VST3 remain planned breadth rather than implemented current adapter
  reality, so they belong in the active feature lane rather than a distant
  backlog
- no Signal-owned ARA surface is currently documented or implemented enough to
  count as supported, so `g06` now plans the first bounded reusable contract

## Validation

- `effigy qa:docs`
- `git diff --check`

## Next task

Keep `g06.001` first for the other thread, but treat `g06.009`, `g06.011`, and
`g06.013` as the explicit feature-breadth anchors for Linux plugin support and
ARA once the recovery contract is frozen.
