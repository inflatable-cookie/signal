# God-File Split: dream tests, benchmark, high test residue

Status: complete
Created: 2026-08-07
Scope: creative_direct_renewal_dream tests; stretch benchmark; offline
stretch_artifact tests; render-plane tests/support; VST3 fixture vtables

## Baseline

After test-criticals batch: remaining criticals included dream tests (1659) and
benchmark (1811), plus high residue in stretch_artifact/support/vtables.

## What Changed

### dream `tests.rs`

Macro shell kept; body → `tests/{prelude,manifest,structural,synthetic_*}`
via `include!` (paths relative to lib.rs call site).

### `benchmark` → `{types,synthetic,measure,compare,report,a18_crossover_smear}`

### offline `tests/stretch_artifact` → `{materialization,capability_gates,identity_chunking}`

### render-plane `tests/support` → `{plan,processors,stream}`

### VST3 `fixture/source/vtables` → `{object,midi,component,processor,controller}`

Move-only. Three dream clippy cleanups (const assert, size_of_val,
is_multiple_of) with no assertion-semantic change.

## After

Test/fixture criticals cleared. Remaining criticals: demo TS script + evidence
corpus-report bin. High residue: a few large test topic files + evidence
select bin.

## Validation

- dream 18, corpus_benchmark 14, benchmark 16, render-plane 145, vst3 26 +
  hosting_fixture 6
- clippy -D warnings on touched crates (incl. --lib --tests for stretch)

## Next Task

Split demos/evidence bins if desired, or further shrink remaining high test
topic files, or stop for doctor baseline reassessment.
