# 2026-03-11 17:09:38 GMT - g01 roadmap closure normalization

## Summary

Normalized the remaining stale roadmap state after closing `g01.009`. The
implementation evidence already showed `g01.001`, `g01.004`, and `g01.005` as
done, but their roadmap files were still marked active and still pointed at
follow-on work that had already happened in later milestones.

This tranche closes those stale markers and records `g01` itself as complete.

## What changed

- marked `docs/roadmaps/g01/001-docs-foundation-and-dsp-research-migration.md`
  complete
- marked `docs/roadmaps/g01/004-trust-edge-package-shell-expansion.md`
  complete and removed the drifted scheduler-oriented next-task block that no
  longer belonged to that milestone
- marked `docs/roadmaps/g01/005-core-dsp-kernel-and-control-signal-baseline.md`
  complete and checked its evidence requirements based on the existing logged
  DSP/kernel tranches
- updated `docs/roadmaps/g01/README.md` so the milestone index and generation
  status now reflect that all `g01` milestones in this roadmap set are complete

## Validation

- `git diff --check`

## Completion

`g01` is complete.
