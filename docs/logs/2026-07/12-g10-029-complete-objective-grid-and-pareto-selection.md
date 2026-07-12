# Complete Objective Grid And Pareto Selection

Date: 2026-07-12
Roadmap: `g10.029`
Batch: `29.6BK` objective checkpoint
Status: complete; concealed development export ready

## Result

Signal executed every frozen complete-system configuration. Each configuration
ran synthetic identity, pitch, event, linked-decision, closure, coverage,
boundary, finiteness, movement, and repeat gates plus all nine frozen
development rows. Total development renders: `972`.

The deterministic local report is
`target/stretch-successor-bk-objective-grid.tsv`, SHA-256
`aef95b0ce18bf790cfd611e4a94e34dd443a7c0e58b4566f8f84ed16f3f89da0`.

## Hard Gates

- configurations: `108`
- passing: `68`
- combined identity/pitch/event failures: `40`
- exact-length failures: `0`
- coverage failures: `0`
- non-finite failures: `0`
- boundary failures: `0`
- event-order failures: `0`
- selected-event movement failures: `0`
- repeat failures: `0`
- linked-decision failures: `0`

All `36` `[256,1024,4096]` configurations fail. The other failures are
`g512-sr-u0-rs-v0`, `g512-sr-u1-rs-v0`, `g512-sr-u2-rs-v0`, and
`g1024-sr-u0-rs-v0`.

## Pareto Result

The five lower-is-better development fields are crest change, zero-crossing
movement, normalized derivative change, endpoint discontinuity, and normalized
second-difference residual. `25` configurations are nondominated.

The three deterministic representatives are:

- `g512-sr-u0-rc-v1`
- `g512-sr-u1-rc-v1`
- `g512-sc-u1-rc-v0`

Objective evidence does not rank these candidates or authorize promotion.

## Boundary

Only the nine development rows were decoded. Holdout reads are exactly zero.
No holdout render or metric exists. Candidate audio and the concealed assignment
key have not yet been exported.

## Next Task

Continue Batch 29.6BK. Export the three selected frontier configurations against
current Signal and Rubber Band R3 across the nine development rows with stable
concealment and shared level matching. Keep the key and holdout closed until
development notes are frozen.
