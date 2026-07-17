# g10.029 Linked Phase-Field Family Selection

Date: 2026-07-17
Batch: 29.7S
Status: complete

## Scope

Compare complete linked-stereo phase-field families. Select at most one
clean-room boundary. Do not render.

## Finding

Joint phase-gradient integration is not a fresh candidate. Signal's full
fixed-grid kernel failed mono attack, timing, replica, formant, boundary, and
combined gates. Exact-lattice repair retained the rejection. Published PGHI
also supplies no joint multichannel integration law.

Peak-region phase locking has independent support across the complete system:
Laroche-Dolson for region locking, Dorran-Lawlor-Coyle for stereo ownership,
Röbel for reset, Ottosen-Dörfler for trajectory and representation, MIT
AudioTSM for the basic operator, and MPL Bungee for dynamic multichannel
whole-kernel feasibility.

## Decision

Select one separate `SharedRotationRegionLocked` report-only kernel. A
cancellation-safe joint energy map chooses regions and owner channels. One
tracked peak rotation owns every channel and bin in its region. Reset owns the
complete region when continuity is invalid. No current weighted-predictor
state, late overlay, independent peer recurrence, or Rubber Band expression
enters the kernel.

## Next Task

Run Batch 29.7T as one fixed-grid proof at `0.75x`, `1.5x`, and `2.0x`. Require
exact mechanics, zero calibrated stereo and local-consistency failures, and no
row-complete mono regression. Stop after one candidate.
