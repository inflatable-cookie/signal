# g10.029 Direct Scale-Timeline Preregistration

Date: 2026-07-18
Batch: 29.7AR
Status: complete; Batch 29.7AS ready

## Result

Rule 31Z freezes one implementation-free direct scale timeline:

- proof rates `8/44.1/48 kHz`, one or two channels, fixed ratio `0.25..4.0`
- common `10 ms` output lattice and `80/40/20 ms` direct STFT scales
- fixed upward `750/6000 Hz` crossover ties
- exact nonnegative owned-bin totals `191/592/631`
- absolute output-to-source projection with no accumulated hop drift
- even source reflection, exact target crop, and `130 ms` offline lookahead
- channel-local ordinary/unlocked recurrence and compatible locked-only peer
  borrowing
- duration-independent prepared storage and explicit pre-work failure

At `48 kHz` stereo, maximum source, pending-coefficient, guidance, phase,
region, output, transform, and planner-scratch terms are `11520`, `12620`,
`11989`, `2524`, `2524`, `7680`, `13440`, and `7680` values or records.

## Identity Correction

The batch does not claim that three differently windowed masked STFT operators
form a perfect-reconstruction sum. They do not in general. Unity remains the
bit-exact public bypass. Batch 29.7AS must prove each unmasked scale at
`1e-12`, then report the inert masked sum as a diagnostic. This prevents
another mechanically exact local proof from hiding a false complete-kernel
assumption.

## Old Prototype Boundary

Batch 29.6CH contributes centred extraction, reflection, crop, and
same-channel summation as proof concepts only. Its schedule, dynamic
crossovers, duration-sized storage, dynamic per-scale normalization,
unconditional peer projection, hard locking, and code types remain rejected.

No renderer or audio was created. Guided state, objective rows, tuning,
listening, holdout, dynamic ratio, realtime, routing, cache, production, and
product work remain closed.

## Validation

- `effigy qa:docs`: pass
- `effigy qa:northstar`: pass
- `effigy health`: pass
- `effigy validate`: pass; one unrelated concurrent `signal-runtime` test
  unused-import warning remains outside this batch

## Next Task

Run Batch 29.7AS. Implement direct representation and fixed-storage mechanics
only; keep guided phase state and objective audio closed.
