# 004 - Hosting Domain Demolition

Status: planned
Owner: core-product
Created: 2026-06-11
Depends on: g10.001
Vision tags: `DEMOLITION`, `HOSTING`, `HONESTY`

## Problem

`signal-host-server` (~15k LoC) is not a server: no socket, no serve loop —
an in-process copy of `signal-host-local` plus LV2, with a 30-file test suite
mirrored by rename. `signal-supervisor-tools` (~18.6k LoC) supervises no
process: ~9.8k LoC is hardcoded architecture prose rendered by `--describe-*`
CLI flags, validated by tests that assert the prose against itself. Neither
is consumed by the product. Both make every audit, grep, and refactor more
expensive and the workspace's claims dishonest.

The salvageable kernel of the hosting domain — `signal-ipc`'s shared-memory
broker and the real broker child-process spawn plumbing — stays.

## Goals

- [ ] delete `signal-host-server` (crate, tests, workspace member, effigy
      references)
- [ ] delete `signal-supervisor-tools`; relocate any prose worth keeping into
      `docs/architecture` as markdown before deletion
- [ ] keep `signal-ipc` shared memory and the broker spawn path intact and
      green
- [ ] prune effigy tasks / acceptance manifests that ran the deleted binaries

## Non-Goals

- [ ] no changes to `signal-host-local` yet (g10.005 collapses it with the
      runtime re-scope)
- [ ] no replacement engine-server work (backlog until a product feature
      needs out-of-process hosting)

## Execution Plan

### Batch 4.1 - Supervisor Tools

- [ ] extract still-true rationale prose into `docs/architecture` (one page,
      not per-boundary files)
- [ ] delete the crate + its test-support tree + effigy boundary tasks
- [ ] workspace build + full test gate

### Batch 4.2 - Host Server

- [ ] confirm zero consumers outside its own tests (audit found none)
- [ ] delete the crate + mirrored test suite + demo binary
- [ ] workspace build + full test gate

## Acceptance Criteria

- [ ] workspace members drop by two; `cargo build --workspace` and all
      remaining tests green
- [ ] no effigy task references the deleted crates
- [ ] README/system inventory updated (final truth pass happens in g10.009)

## Risks and Mitigations

- Risk: hidden consumer via effigy tasks or fixtures.
- Mitigation: grep workspace + pulse + aura before each cut; acceptance
  manifests updated in the same commit.

## Evidence Requirements

- [ ] LoC delta and test-count delta recorded per batch in the progress log

## Next Task

g10.005 (runtime re-scope) — the largest cut, on the crate these two wrapped.
