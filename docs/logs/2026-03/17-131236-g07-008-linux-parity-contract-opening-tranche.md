# 2026-03-17 - g07.008 Linux parity contract opening tranche

## Summary

Completed Batch 8.1 of `g07.008` by freezing the first bounded Linux
cross-adapter plugin parity and sandbox-policy contract on top of the now
closed CLAP, VST3, and LV2 Linux plugin boundaries.

This tranche keeps the Linux plugin story bounded: Signal now has one explicit
portable versus guarded contract for Linux plugin breadth, but it still does
not pretend all three adapters are extension-identical.

## Key changes

- added `039-linux-cross-adapter-plugin-parity-and-sandbox-policy-contract.md`
  to freeze:
  - the Linux-facing portable, guarded, adapter-private, and unsupported bands
    across CLAP, VST3, and LV2
  - the rule that Linux sandbox and placement policy must reuse the existing
    runtime-owned shared-sandbox and continuity contract rather than a
    Linux-only wrapper taxonomy
  - which Linux plugin claims remain deferred, including richer extension depth
    and the later ALSA, JACK, and PipeWire backend parity queue
- advanced the `g07.008` roadmap so Batch 8.2 is now the active implementation
  queue
- rolled the shared contract, roadmap, and architecture references forward so
  the repo-wide next task points at runtime Linux parity receipts instead of
  more contract opening

## Validation

- `git diff --check`
- `effigy health --repo .`
- `effigy validate --repo .`
- `effigy qa:docs --repo .`

## Residual risk

This is contract-only. Runtime-owned Linux lifecycle, render, failure, and
placement receipt alignment across CLAP, VST3, and LV2 still belongs to Batch
8.2, and richer extension depth remains explicitly deferred.

## Next Task

Continue `g07.008` with Batch 8.2 by aligning lifecycle, render, failure, and
placement receipts across Linux adapters so supervisor export and stable
host-edge surfaces stay on one Linux plugin vocabulary.
