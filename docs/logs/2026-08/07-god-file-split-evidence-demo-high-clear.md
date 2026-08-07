# God-File Split: evidence bins, demo script, high residue clear

Status: complete
Created: 2026-08-07
Scope: stretch-corpus-report + fma-stretch-corpus-select bins; plugin capability
browser demo; remaining high test topic files

## Baseline

After dream/benchmark batch: remaining criticals were the demo TS script and
evidence corpus-report bin; highs included FMA select bin and several test
topic files.

## What Changed

### evidence `stretch-corpus-report` → bin dir `{main,args,manifest,external,listening,quality}` (+ existing pack/tracker)
### evidence `fma-stretch-corpus-select` → `{main,args,select,report,tests}`
### demo `run_plugin_capability_browser_demo.ts` → thin runner + `plugin_capability_browser/*`
### render-plane tests `events` / `clips_samples` / `compile_graph` → topic subdirs
### stretch `corpus_benchmark` → topic subdir
### dream `structural` → `structural_owners` + `structural_tests` include! fragments

Move-only.

## After

**No high or critical files remain** on `effigy scan god-files`. Warn band still
has rows (74 hidden by default).

## Validation

- evidence bins: clippy + 7 + 6 tests; --help unchanged
- render-plane lib 145; stretch corpus_benchmark 14
- demo: bun import / fixture-mode run (receipt artifacts not committed)

## Next Task

Warn-band shrinkage or doctor fail-on-god-files baseline reassessment.
