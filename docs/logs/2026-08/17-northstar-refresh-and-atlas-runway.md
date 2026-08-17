# Northstar Refresh and Atlas Runway

Status: decision log
Date: 2026-08-17
Owner: core-product

## Summary

Ran Northstar project refresh and Atlas long-horizon planning after the updated
Northstar agent contract. Signal is baseline-routed with no active strict lane
and no ready batch. The `g10` stretch audit is complete.

## Refresh facet states

| Facet | State | Notes |
| --- | --- | --- |
| Instruction surface | repaired | `AGENTS.md` aligned to Northstar contract; `CLAUDE.md` and `PAPERCUTS.md` added |
| Docs spine | repaired | stale Next Task pointers fixed; strategic runway linked from front doors |
| Architecture and authority | current | architecture README and contract index match live crate posture |
| Planning completeness | current | no ready batch; operator gate explicit |
| Currentness and closeout | repaired | roadmap README, g10 README, generation index updated |
| Validation and distribution | current | `effigy qa:docs` and `effigy qa:northstar` green after repairs |
| Distribution | not-applicable | consumer repo checks only |

## Repairs made

- restructured `AGENTS.md` to Northstar always-loaded boundaries + docs authority
- added `CLAUDE.md` bridge (`@AGENTS.md`)
- seeded `PAPERCUTS.md`
- updated `docs/contracts/001-working-rules.md` for post-`g09` baseline posture
- updated `docs/architecture/product-guardrails.md` Next Task
- indexed `docs/roadmaps/backlog/post-g10-rebuild-on-demand.md`
- fixed stale Next Task pointers in roadmap front doors
- added `docs/roadmaps/strategic-runway.md`
- corrected plugin-hosting docs after operator review: Contract `072`, backlog,
  architecture front doors, root README, and strategic runway now state that
  CLAP/VST3/AU/LV2 hosting is shipped and the open seam is integration depth

## Atlas outcome

Horizon model recorded in `docs/roadmaps/strategic-runway.md`:

- **Horizon A:** close `g10`, operator selects next Signal-only lane
- **Horizon B:** product-pulled integration depth (host-assembly wiring default)
- **Horizon C:** analysis/substrate breadth on demand
- **Horizon D:** ecosystem consolidation and C++ island replacement

Recommended default first `g11` tranche: production host-assembly wiring.

## Correction (same day)

Follow-up operator review found the initial Atlas recommendation was wrong:
Signal already hosts CLAP, VST3, AU, and LV2 through `signal-plugin-bridge`.
The stale post-demolition backlog and architecture front doors caused that
mistake. Docs were corrected in the same refresh pass.

## Open operator decisions

1. Which Signal-only target opens the next lane?
2. Roll `g10` to `g11` now or keep `g10` open for one closeout card?
3. Confirm production host-assembly wiring as the preferred first `g11` tranche?

## Validation

- `effigy qa:docs`
- `effigy qa:northstar`
- Northstar `check:agent-instructions` advisory audit

## Next Task

Operator answers the three open decisions in
`docs/roadmaps/strategic-runway.md`, then route to roadmap compilation for the
selected `g11` milestone family.
