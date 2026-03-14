# Analysis Corpus

Status: active
Updated: 2026-03-11

## Purpose

This directory is the first shared layout for Signal's regression-sensitive
analysis corpus. It exists so analyzer quality work does not stay trapped in
scattered unit tests, ad hoc examples, or app-local fixture folders.

## Layout

- `synthetic/`
  - tiny generated or inline-safe fixtures used for deterministic acceptance
    and regression checks
- `manifests/`
  - corpus manifests, metric notes, and frozen threshold/drift policies
- `external-small/`
  - reserved for repo-local small reference assets once licensing and size are
    explicitly cleared
- `external-large/`
  - reserved for out-of-repo or fetched corpora that should not be committed
    into the main workspace

## Fixture Taxonomy

The first shared taxonomy should classify fixtures by analysis pressure rather
than by one analyzer crate:

- tonal
- noise
- pulse
- sustained
- loudness
- silence
- rate-policy
- semantic

## Working Rule

- keep synthetic fixtures deterministic and cheap enough for local acceptance
  runs
- store licensing and artifact-size notes before adding real audio assets
- prefer manifests and explicit metadata over one-off filename conventions

## Current Harness Entry Points

- `cargo test -p signal-analysis harness -- --nocapture`
- `cargo test -p signal-analysis-rhythm harness -- --nocapture`
- `cargo test -p signal-analysis-tonal harness -- --nocapture`
- `cargo test -p signal-analysis-character harness -- --nocapture`
- `cargo test -p signal-analysis-loudness harness -- --nocapture`
- `cargo test -p signal-analysis-embed harness -- --nocapture`
- `effigy acceptance:analysis`

## Frozen Policy Manifests

- `manifests/frozen-family-policies-v1.md`
  - first frozen thresholds and drift posture for:
    - rhythm structure and tempo ambiguity
    - tonal key/tuning/local-tracking ambiguity
    - character descriptor packs
    - loudness summaries
    - semantic inference baseline

## Next Task

The first shared corpus and harness posture is complete for `g02`. Expand this
corpus only when a new analyzer family or regression class justifies broader
fixtures or harder gates.
