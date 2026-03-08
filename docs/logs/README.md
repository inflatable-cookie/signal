# Logs

Logs capture dated evidence and assessments.

## Segmentation model

- Group logs by month directory: `YYYY-MM/`
- Name each log: `DD-HHMMSS-<slug>.md`

## Cadence rule

- Create logs per completed batch or update cycle.
- Do not create a separate log for every task.

## Lean evidence rule

- Manual validation notes and commands run are the default evidence format.
- Add checker scripts only when recurring pain justifies automation.
- If a checker is added, include an automation adoption note with owner/cadence/sunset trigger.

## Decision policy

- Do not maintain a separate `decisions/` folder by default.
- Capture major decisions as dedicated decision logs in this folder.
- For legacy migrations, rewrite references and remove old `decisions/` files in the same batch (no compatibility shim files).

## Current seed

- `2026-03/08-131500-signal-docs-bootstrap-and-research-migration.md`

## Templates

- `templates/roadmap-contract-delta-template.md`
- `templates/roadmap-gate-log-template.md`
- `templates/decision-log-template.md`
- `templates/automation-adoption-note-template.md` (optional)
- `templates/roadmap-currentness-triage-template.md` (optional)

## Next task

Keep logging at the batch level and add the next Signal log only when a
meaningful docs, architecture, or implementation slice closes.
