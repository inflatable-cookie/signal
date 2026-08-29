# Stale lower-authority Next Task pointers

Status: open
Found: 2026-08-29
Owner: core-product

## Observation

The active front doors consistently say Signal is baseline-routed with no
ready card and should stop for operator selection. Two lower-authority
surfaces still name completed work:

- `docs/architecture/system-inventory.md` says to execute the completed
  `g11.002` batch card `005`.
- `docs/contracts/contract-index.md` says to execute the completed `g10.034`
  Batch `34.3`.

The superseded `docs/architecture/package-map.md` also carries historical
execution text, but its banner already marks the map as superseded in part.

## Disposition

Keep explicitly open for a bounded docs-currentness repair. Do not treat these
lower-authority pointers as live execution authority. The repair needs to align
their `Next Task` sections with the current roadmap and contract front doors.

## Next check

Core-product or the next docs-currentness batch should update the stale
pointers, then run `effigy qa:docs` and `effigy qa:northstar`.

## Next Task

Resolve the two stale pointers in a docs-authorized currentness pass before
another strict lane or backlog item is promoted.
