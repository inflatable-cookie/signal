# g10.032 Centred Cyclic Impulse Attribution

Date: 2026-07-24
Status: Batch 32.17 complete; Batch 32.18 ready

## Cause

The failed impulse was not lost. At `2x`, neutral cycle:

- `H=2117`
- ideal event centre `88200.5`
- positive ledger groups `86440..86441`, `87498..87499`,
  `88557..88558`, `89615..89616`
- failing window `[88179,88400)`
- mapped source interval `[44090,44199]`
- source RMS `-20.414 dBFS`
- output RMS exact zero

The window falls between commanded replicas. Its `1058.5`-frame spacing
matches retained ReaReaRea evidence.

## Decision

Continuous mapped-window activity owns sustained material. The event ledger
owns sparse impulses. Select fresh
`EventLedgerAuditedCenteredCompressedAnchorCyclic` with unchanged renderer
formulas and corrected sparse-event evidence.

The rejected checkpoint also contains placeholder Y02, Y03, and Y04
diagnostic implementations. Recover none of it. Freeze complete executable
owner known answers before new isolation.

## Next Task

Execute Batch 32.18 only. No implementation or acoustic execution.
