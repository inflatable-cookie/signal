# 2026-04-10 - g09.015 Interactive Demo Strategy And Gap Inventory

Roadmap: `docs/roadmaps/g09/015-operator-visible-interactive-demo-and-plugin-browser-proof.md`
Card: `docs/specs/batch-cards/042-g09-015-interactive-demo-strategy-and-gap-inventory.md`

Reopened `g09` for a new interactive-demo stream that stays inside the
generation and focuses on operator-visible proof rather than crate-readiness.

## What changed

- promoted `081-operator-visible-interactive-demo-and-low-dependency-ui-contract.md`
- opened `g09.015`
- completed the planning batch `042`
- promoted `043-g09-015-plugin-capability-browser-bootstrap.md` as the first
  implementation card
- explicitly chose a low-dependency UI posture:
  - existing binaries plus operator prompts where possible
  - otherwise static browser-native assets or lightweight terminal surfaces
  - no heavyweight product-style UI framework by default

## Validation

- `effigy tasks`
- `effigy doctor` (reported the existing repo-wide god-file and attention-marker findings)
- `effigy qa:docs`
- `effigy qa:northstar`

## Next Task

Continue the reopened strict `g09` lane from
`docs/specs/batch-cards/043-g09-015-plugin-capability-browser-bootstrap.md`.
