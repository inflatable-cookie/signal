# 001 Shared DSP and Host Boundary

Status: active
Owner: core-product
Updated: 2026-03-08
Related vision: `docs/vision/001-signal-vision.md`
Related architecture: `docs/architecture/system-architecture.md`

## Purpose

Freeze the ownership rule that Signal is the canonical home for reusable DSP,
analysis, and audio-runtime logic, while Finch and Loophole keep only thin
consumer or authority-specific layers around it.

## Contract

1. Reusable DSP, analysis, and graph/runtime algorithms belong in Signal-owned
   crates or runtime modules.
2. Finch may own workflow, UI, sidecar, and library-specific integration logic,
   but not duplicate Signal-owned algorithm implementations.
3. Loophole may own authority, orchestration, and product-assembly concerns, but
   not duplicate Signal-owned DSP or analysis logic.
4. Plugin-format wrappers, hardware adapters, and sandbox bridges may use native
   languages where required, but they must remain thin integration layers.
5. Historical compatibility paths such as `loophole/signal -> ../signal` are
   temporary and must not be treated as the architectural source of truth.

## Acceptance Signals

- New algorithm and crate-shape research lands in `signal/docs/research/`.
- Finch docs refer back to Signal for DSP and analysis authority.
- Signal package planning can proceed without app-local duplicate crate plans.

## Next Task

Freeze the first concrete package names and host-entrypoint names that satisfy
this ownership boundary.
