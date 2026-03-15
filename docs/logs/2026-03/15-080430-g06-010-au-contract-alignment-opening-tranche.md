# 2026-03-15 08:04:30 UTC - g06.010 AU Contract Alignment Opening Tranche

## Summary

Opened `g06.010` by freezing the first AU-specific adapter alignment contract so
later runtime realization can widen real Audio Unit support without reopening
host-local ownership or product-local wrapper authority.

## Work completed

- added `docs/contracts/021-au-adapter-baseline-and-runtime-owned-lifecycle-contract.md`
- mapped AU-specific discovery, lifecycle, property, and macOS-scope detail
  onto the existing backend-neutral Signal-owned plugin and runtime contract
  family
- recorded explicit AU realization gaps before `signal-plugin-au` or host
  integration work begins
- moved the roadmap and reference trail from Batch 10.1 contract freeze to the
  Batch 10.2 runtime-baseline queue

## Validation

- `git diff --check`
- `effigy health --repo .`
- `effigy qa:docs --repo .`

## Deferred scope

- no Rust `signal-plugin-au` adapter crate exists yet
- no runtime-owned AU discovery or lifecycle receipts are implemented yet
- AU parameter-tree depth, preset documents, editor integration, and richer
  MIDI or event-model breadth remain later cross-adapter work

## Next Task

Continue `g06.010` with Batch 10.2 by implementing the first real AU adapter
path with runtime-owned discovery, lifecycle, macOS-scoped scan or load
coverage, and aligned supervisor or stable host-edge export.
