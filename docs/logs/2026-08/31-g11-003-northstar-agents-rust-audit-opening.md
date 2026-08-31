# g11.003 Northstar AGENTS And Rust Audit Opening

Status: planning complete; worker dispatch pending
Date: 2026-08-31
Owner: core-product

## Summary

The operator selected Signal for the next Northstar AGENTS and language-quality
audit while the independent Nucleus audit continues. Signal has no live project
orchestrator. This opens one baseline-routed maintenance lane inside `g11`; it
does not open `g12` or choose a product backlog item.

## Resolved State

- root `CLAUDE.md` is the exact `@AGENTS.md` bridge
- the installed Northstar advisory reports 111 non-blank AGENTS lines, about
  1,572 tokens, and flags placement/procedure/freshness leads for human review
- the Rust strict activation, profile, and empty deviations file are present
- the workspace owns 28 crates and declares Rust 1.95; the pinned current
  toolchain is 1.97.1
- `effigy doctor` reports the existing god-file threshold baseline plus stale
  graph and attention-marker warnings; those are evidence, not automatic repair
- no TypeScript package is owned here; the root package file is tooling-only,
  so this lane is AGENTS plus Rust

## Planning Result

- roadmap: `docs/roadmaps/g11/003-northstar-instruction-and-rust-quality-audit.md`
- ready card: `docs/roadmaps/g11/batch-cards/008-g11-003-northstar-agents-rust-audit.md`
- scope: repository-wide instruction review and Rust explicit audit-and-repair
- authority: only recorder-approved `review_required` repairs; unsafe/FFI,
  public/foreign error, realtime, compatibility, dependency, and MSRV decisions
  stop for operator direction

## Validation

- `effigy tasks`
- `effigy doctor` — existing `scan.god-files` error baseline; two warnings
- installed Northstar `northstar/check:agent-instructions` — advisory complete
- `git status --short --branch` — clean `main` at `origin/main` before planning

## Next Task

Commit and push this planning batch, publish the worker handoff, then dispatch
card `008` through Paseo for orchestrator review and merge.
