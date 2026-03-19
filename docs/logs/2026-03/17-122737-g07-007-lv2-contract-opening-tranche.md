# 2026-03-17 - g07.007 LV2 contract opening tranche

## Summary

Completed Batch 7.1 of `g07.007` by freezing the first LV2 adapter alignment
boundary on top of the existing backend-neutral plugin, continuity, and
cross-adapter Linux breadth contracts.

This tranche keeps the work honest: LV2 is now an explicit shared runtime goal,
but the repo still does not claim real LV2 adapter realization yet.

## Key changes

- added `038-lv2-adapter-baseline-and-linux-native-plugin-lifecycle-contract.md`
  to freeze:
  - how LV2 discovery, bundle or manifest traversal, URI identity, and
    Linux-native support must collapse into shared runtime-owned discovery
    meaning
  - how LV2 lifecycle and continuity must reuse the existing backend-neutral
    plugin contract family instead of creating a Linux-only wrapper taxonomy
  - which gaps remain explicit before runtime realization widens
- advanced the `g07.007` roadmap so Batch 7.2 is now the active implementation
  queue
- rolled the shared contract, roadmap, and architecture references forward so
  the repo-wide next task points at real LV2 adapter work instead of more
  contract opening

## Validation

- `git diff --check`
- `effigy health --repo .`
- `effigy validate --repo .`
- `effigy qa:docs --repo .`

## Residual risk

This is contract-only. There is still no shared LV2 backend identity in
`signal-plugin`, no Rust LV2 adapter crate, and no runtime-owned Linux-native
LV2 discovery or lifecycle receipt family yet.

## Next Task

Continue `g07.007` with Batch 7.2 by implementing the first real LV2 adapter
path with runtime-owned discovery, lifecycle, Linux-native scan or load
coverage, and aligned supervisor or stable host-edge export.
