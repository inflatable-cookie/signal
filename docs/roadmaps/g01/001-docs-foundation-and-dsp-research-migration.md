# Roadmap g01.001: Docs Foundation and DSP Research Migration

Status: complete
Owner: core-product
Created: 2026-03-08
Vision tags: RES, RT, AUTH
Target envelope: establish Signal as the canonical docs and research home for
shared DSP/analysis work, using a Northstar-aligned docs skeleton and migrated
research artifacts from Finch.

## Problem

Signal had no project-local docs authority after being extracted into its own
repo, while Finch still held the most complete algorithm and crate-shape notes
for work that is no longer Finch-owned.

That split made the shared-DSP direction incoherent:

1. Signal lacked a Northstar docs spine for vision, architecture, contracts,
   roadmaps, and logs.
2. Reusable algorithm research lived in Finch even though Finch is becoming a
   wrapper/integration consumer rather than the owner of core DSP work.
3. The existing research corpus was spread across too many Finch-local entry
   points.

## Goals

- Seed `signal/docs/` from the Northstar bundle.
- Replace generic examples with Signal-specific vision, architecture, and
  contract entry points.
- Migrate reusable DSP and analysis research from Finch into
  `signal/docs/research/`.
- Leave explicit migration breadcrumbs in Finch docs so active research threads
  can find the new authority without losing context.

## Non-Goals

- Finalize the full Signal crate map in this batch.
- Move Finch-specific workflow or UX documentation into Signal.
- Rewrite the legacy C++ implementation in this batch.

## Execution Plan

### 001.1 Docs skeleton

- [x] Copy the Northstar template bundle into `signal/docs/`.
- [x] Replace example artifacts with Signal-specific vision, architecture, and
  contract docs.
- [x] Create the first Signal roadmap milestone and batch log.

### 001.2 Research authority migration

- [x] Move reusable DSP and analysis research into `signal/docs/research/`.
- [x] Keep the Signal research section Northstar-aligned while allowing the
  custom `algorithm-specs/` folder for implementation-facing algorithm detail.
- [x] Rewrite research entry points to speak from Signal ownership rather than
  Finch-local ownership.

### 001.3 Finch breadcrumbs

- [x] Leave migration notes in Finch research docs.
- [x] Make Signal the canonical target for future algorithm/crate-shape updates.

## Acceptance Signals

1. A contributor can start in `signal/docs/README.md` and find the current
   vision, architecture, contract, research index, roadmap, and latest log.
2. Finch research docs clearly point to Signal as the authority for DSP and
   analysis work.
3. Signal research contains the Rust ecosystem hub, Signal architecture hub,
   value tracks, Essentia dossier, and algorithm specs needed for the next
   implementation batch.

## Next Task

`g01.001` is complete. Reopen only if Signal needs another docs-authority or
research-migration pass.
