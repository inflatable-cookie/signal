# God-File Split: resumable engine + LV2 turtle parser residue

Status: complete
Created: 2026-08-07
Scope: `signal-dsp-stretch` resumable/engine; `signal-plugin-lv2` turtle/parser

## Baseline

After topology/MIDI/broker batch: remaining production highs were
resumable/engine (~509) and turtle/parser (~405).

## What Changed

### `resumable/engine`

→ public API stays in `engine.rs`; pipeline + spectral helpers move to
`pipeline.rs` / `spectral.rs`

### `turtle/parser`

→ `parser/{mod,lexer,statements,literals}` (include!-wired impl blocks so
private Parser methods stay one module)

Move-only.

## After

Production high band cleared except evidence bin / test residue. No non-test
criticals.

## Validation

- stretch: clippy + resumable_gates
- LV2: clippy --tests + full package tests

## Next Task

Stop for review, or start warn-band prod shrinkage / doctor fail-on-god-files
baseline reassessment.
