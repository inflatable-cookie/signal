# g10.034 Continuous Cyclic Brief

Date: 2026-07-24
Batch: 34.2
Status: complete

## Decision

Freeze
[ContinuousEventLedgerCyclic](../../architecture/offline-creative-continuous-event-ledger-cyclic-brief.md)
as one private evidence candidate for every integer target `2L<=T<=8L`.

The candidate adds one private domain entry and internal behavior identity.
The admitted map, schedule, interpolation, crossfade, exterior-zero,
linked-channel, memory, cost, and synthesis files remain unchanged.

## Evidence

- immutable parent commit, tree, production files, public files, manifest, and
  lockfile are hash-bound
- exact anchor parity covers mono/stereo `2x`, `4x`, and `8x` at `5`, `48`,
  and `90 ms`
- interior acoustic probes are exact `2.5x`, `5x`, and `7.5x`
- conformance is `334` rows
- synthetic admission is `183` rows and `201` candidate renders
- long-form admission is `60` rows and renders
- retained ReaReaRea comparison uses `63` freshly captured interior rows
- concealed mono, cycle-direction, hard linked-stereo, and eligible
  independent review are fixed
- Contract `085` Rule 11 evidence repair cannot masquerade as renderer
  rejection
- failure cleanup and pass-only minimal admission are exact

The Batch 32.25 stereo waiver does not transfer.

## Scope

Documentation only. No DSP, tests, candidate harness, comparator asset,
listening asset, public API, dependency, routing, cache, artifact, runtime,
UI, Loophole, or Chorus surface changed.

## Validation

- `git diff --check`: pass
- `effigy qa:docs`: pass
- `effigy qa:northstar`: pass
- `effigy health`: pass
- `effigy validate`: pass

`effigy doctor` retains the pre-existing god-file and attention-marker
findings. The Northstar bundle protocol-kernel path is absent from the local
skill installation; canonical repo contracts and front doors remain complete.

## Next

Execute Batch 34.3 only. Create the disposable worktree, implement the frozen
private entry and complete evidence system, pass two conformance rounds, freeze
one acoustic checkpoint, then run the gates in order. Keep public widening and
integration closed.
