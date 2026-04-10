# 10-193941 - g09.015 Local Scan Containment Ready Handoff

## Summary

Re-entered planning after the honest local launch-target closeout and promoted
the next bounded `g09.015` seam.

The next honest blocker is not deeper plugin interaction yet. The default
interactive browser entrypoint still often shows no local launch buttons on
real multi-plugin systems because the local scan surface can stall or fail on
broad system roots. That makes local-scan containment the right next batch.

## Decision

Promoted the ready card:

- `docs/specs/batch-cards/048-g09-015-local-scan-containment-and-visibility.md`

Kept the scope narrow:

- contain local scan failures so one problematic installed plugin does not
  erase all local visibility
- keep local launch buttons tied to actual local scan truth
- preserve the bounded proof path as the stable validation surface

Did not promote broader live plugin interaction yet because the operator-facing
default still needs a more reliable local visibility substrate first.

## Validation

- `effigy tasks`
- `effigy qa:docs`
- `effigy qa:northstar`

## Next Task

Continue the active strict `g09` lane from
`docs/specs/batch-cards/048-g09-015-local-scan-containment-and-visibility.md`.
