# 001 - Audit Adoption And Generation Open

Status: active
Owner: core-product
Created: 2026-06-11
Depends on: none
Vision tags: `PLANNING`, `AUDIT`, `HONESTY`

## Problem

The 2026-06-11 deep audit (recorded in
`chorus/research/2026-06-11-signal-deep-audit.md`) found that roughly 10-15%
of Signal serves the actual Loophole product. The dominant failure mode is
code that demonstrates an engine rather than being one: simulated backends,
narration-string surfaces, clone crates, and tests that assert the simulation
against itself. Meanwhile the production audio path (output-stream contract,
cpal backend, render plane) is new, real, and under-protected.

Signal needs one generation that turns the audit into an executable program:
fix the production path first, then demolish the dishonest mass, then
consolidate, and defer rebuilds until a product feature pulls them.

## Goals

- [x] open `g10` as the active generation
- [x] compile one roadmap per remediation lane with explicit dependencies
- [ ] keep the audit document referenced as the evidence base for every cut

## Non-Goals

- [ ] no implementation work in this packet
- [ ] no rebuild commitments (real plugin hosting, engine server, beat
      tracking) — those live in the backlog until pulled

## Execution Plan

### Batch 1.1 - Generation Open

- [x] add `g10` to `generation-index.md` with the audit as the stated reason
- [x] refresh roadmap front doors so `g09` is visibly closed and `g10` active
- [x] author packets 002-009 and the rebuild backlog note

## Acceptance Criteria

- [x] one active generation exists with ordered, dependency-explicit packets
- [x] every major audit finding maps to a packet or a backlog entry

## Risks and Mitigations

- Risk: demolition packets stall mid-way, leaving the workspace half-cut.
- Mitigation: every demolition batch ends with a full-workspace build and
  test gate; packets are sized so one batch is one landable commit.

## Next Task

g10.002 (production-path declick and correctness) — user-audible fixes lead
the generation.
