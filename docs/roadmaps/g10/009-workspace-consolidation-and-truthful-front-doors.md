# 009 - Workspace Consolidation And Truthful Front Doors

Status: planned
Owner: core-product
Created: 2026-06-11
Depends on: g10.004, g10.005, g10.006, g10.007, g10.008
Vision tags: `HYGIENE`, `CI`, `DOCS-TRUTH`

## Problem

The workspace has no CI, no `[workspace.dependencies]` (version strings
already skew across crates), no workspace lints, no toolchain/rustfmt/clippy/
deny configuration. The README's repository layout omits 11 crates —
including signal-render-plane and signal-hardware-output-cpal, the two that
are the production audio path. Test counts are inverted relative to value
(supervisor-tools 149 tests vs render-plane 7, output-cpal 0). After the
demolition lanes land, the front doors must describe what actually remains.

## Goals

- [ ] `[workspace.dependencies]` for shared deps; per-crate versions unified
- [ ] `[workspace.lints]` + rustfmt config; clippy clean or explicitly
      allowed with reasons
- [ ] CI: build + test + fmt + clippy on push (host-device-dependent tests
      skippable)
- [ ] README and system inventory rewritten to the post-g10 crate set, with
      the production audio path documented first
- [ ] CHANGELOG entry summarizing the g10 program
- [ ] test-coverage rebalance: smoke tests where the production path is thin
      (output-cpal), delete suites that died with their subjects
- [ ] edition review (2021 → 2024 decision recorded either way)

## Non-Goals

- [ ] no new features
- [ ] no docs beyond truth-restoration (no speculative architecture prose —
      that pattern is what g10 removed)

## Execution Plan

### Batch 9.1 - Cargo Hygiene

- [ ] workspace deps, lints, fmt, toolchain file; fix skew; clippy pass

### Batch 9.2 - CI

- [ ] pipeline running build/test/fmt/clippy; device-dependent tests gated

### Batch 9.3 - Front Doors

- [ ] README, system inventory, CHANGELOG, roadmap front doors; g10 closure
      record

## Acceptance Criteria

- [ ] fresh clone: one command builds and tests green; CI enforces it
- [ ] every crate in the workspace appears in the README with an honest
      one-line description
- [ ] generation-index updated; g10 closed or explicitly continued

## Risks and Mitigations

- Risk: clippy avalanche on legacy code.
- Mitigation: warn-level baseline first, deny on new code; recorded follow-up.

## Evidence Requirements

- [ ] CI run link/output in the progress log

## Next Task

Generation closeout, then rebuild-on-demand items pull from
`docs/roadmaps/backlog/post-g10-rebuild-on-demand.md` when Loophole schedules
the corresponding product features.
