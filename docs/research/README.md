# Research

Purpose: keep Signal's reusable DSP, analysis, crate-shape, and dependency
research in one canonical place instead of scattering it across app-local repos.

## In plain words

Research is the "what did we learn before deciding" layer. It feeds
architecture and contracts, and the master index is the front door. Most of
this section is archival evidence (especially the time-stretch dossiers); the
live entry points are the [master index](./master-index.md), the value tracks,
and the algorithm specs. See the [glossary](../reference/glossary.md) for the
shorthand used here.

## Authority rule

Signal owns the research authority for:

- reusable DSP and analysis algorithms,
- shared crate/package boundaries,
- external library and dependency evaluation for audio work,
- comparative studies that affect both Finch and Loophole consumption.

Finch may keep wrapper/integration notes, but DSP and analysis research should be
updated here first.

## Structure

- `master-index.md`: primary navigation for the active research corpus
- `source-hubs/`: curated source maps and ecosystem surveys
- `value-tracks/`: problem-led syntheses for beat, tonal, loudness, embeddings,
  and future analysis areas
- `specimen-dossiers/`: per-library or per-system studies such as Essentia
- `algorithm-specs/`: implementation-facing algorithm notes promoted out of the
  value tracks and dossier work
- `translation-memos/`: future Signal-facing recommendations where tradeoffs
  need explicit promotion
- `research-to-implementation-playbook.md`: workflow for turning research into
  real implementation work
- `research-to-architecture-crossref.md`: map from research findings into
  architecture and roadmap commitments
- `gaps-found-during-implementation.md`: implementation-discovered research gaps
- `templates/`: reusable research templates
- `discovery-intake.md`: intake and triage rules for low-authority signals

## Operating model

1. Start with a concrete DSP, analysis, or dependency problem.
2. Prefer primary sources such as papers, official docs, source trees, and
   standards.
3. Synthesize findings in value tracks or source hubs before promoting
   implementation-facing algorithm notes.
4. Update architecture or roadmap docs only when the research result is stable
   enough to constrain delivery.

## Next task

Promote the migrated crate-shape and algorithm findings into explicit package
and runtime-host naming decisions for the first Signal implementation batch.
