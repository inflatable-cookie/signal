# Inflatable Cookie Product Ownership Migration

Status: complete
Created: 2026-07-29
Scope: repository ownership and local GitHub identity

## Change

- Transferred the canonical Signal repository from `infinite-loop-audio` to
  `inflatable-cookie`.
- Updated the local `origin` fetch and push URL to
  `git@github.com:inflatable-cookie/signal.git`.
- Updated Cargo repository metadata and made Inflatable Cookie product
  ownership explicit in the README and canonical vision.
- Preserved the 2025 Infinite Loop Audio copyright notice and added Inflatable
  Cookie ownership for 2026.
- Updated confirmed local consumers in Finch and Keepsake.

## Planning State

- No runtime, architecture, contract, or roadmap boundary changed.
- The active `g10` sequence and its `Next Task` remain unchanged.

## Validation

- New remote lookup, fetch, branch tracking, and push dry-run succeeded.
- `effigy health` passed.
- `effigy validate` passed.
- `effigy qa:docs` passed.
- Final workspace scan found no stale Signal repository URLs.

## Next Task

Resume Signal only through the active `g10` roadmap front door. Do not treat
this ownership migration as execution authority for a new batch.
