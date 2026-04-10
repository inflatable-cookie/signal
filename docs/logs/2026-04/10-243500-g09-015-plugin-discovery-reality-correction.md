# 2026-04-10 - g09.015 Plugin Discovery Reality Correction

Roadmap: `docs/roadmaps/g09/015-operator-visible-interactive-demo-and-plugin-browser-proof.md`
Card: `docs/specs/batch-cards/042-g09-015-interactive-demo-strategy-and-gap-inventory.md`

Corrected the active `g09.015` lane after adapter inspection showed the first
ready browser card was premature.

## What changed

- kept `043-g09-015-plugin-capability-browser-bootstrap.md` as valid follow-on
  work, but downgraded it to `pending`
- promoted `044-g09-015-real-plugin-discovery-gap-burn-down.md` as the actual
  ready batch
- updated the roadmap so plugin discovery realism now precedes browser work
- made the blocker explicit:
  - CLAP discovery is still harness-backed
  - VST3 and AU discovery still rely on Signal-specific metadata files
  - LV2 still has scaffold-backed direct lookup

## Validation

- `effigy tasks`
- adapter code inspection across CLAP, VST3, AU, and LV2 discovery paths
- `effigy qa:docs`
- `effigy qa:northstar`

## Next Task

Continue the reopened strict `g09` lane from
`docs/specs/batch-cards/044-g09-015-real-plugin-discovery-gap-burn-down.md`.
