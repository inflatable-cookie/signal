# Linked-Stereo Recurrence Research

Date: 2026-07-16
Roadmap: `g10.029`, Batch 29.7D
Scope: cross-channel phase ownership research and contract promotion

## Decision

Replace the failed aggregate-mode/per-channel-recurrence design with one
reference-relative recurrence per frame and bin. Select the greatest current
target energy as reference. Preserve peer magnitude and derive peer
phase from reference output plus the peer/reference current input phase
relation.

## Evidence

- Signalsmith Stretch revision `57b93f4e` implements greatest-energy per-bin
  reference prediction and explicit current-input complex-ratio projection.
- Dorran, Lawlor, and Coyle's 2005 AES multichannel TSM paper updates the
  greater-magnitude same-bin peak first and preserves the original phase
  relationship in the lesser peak.
- Rubber Band R3 revision `e4296ac` can borrow the greatest channel's tracked
  peak trajectory while retaining a current analysis-phase offset. This is GPL
  architecture evidence only.
- Signal's 29.7C attribution proves that aggregate shared-mode selection is not
  the primary defect. Independent per-channel recurrence reproduces every
  linked failure mask.

## Rejections

- shared output increment: preserves prior output relation, not current input
  relation
- threshold tuning: does not address the attributed recurrence defect
- mid/side or sample mixing: changes the ownership seam and obscures crossfeed
- Rubber Band translation: incompatible with the clean-room licence boundary

## Promotion

- translation memo 006 added and promoted
- offline synthesis architecture revised
- contract `082`, Rule 31H revised
- Batch 29.7E compiled as the next implementation proof
- Batch 29.8 remains closed

## Next Task

Implement only reference-relative linked recurrence. Exercise both owners,
exact ties, and an ownership crossing, then rerun the unchanged 29.7C quality
gate before listening.
