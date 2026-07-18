# Direct Scale-Timeline Preregistration

Status: promoted
Date: 2026-07-18
Roadmap: `g10.029`, Batch 29.7AR
Contract: `082`, Rule 31Z

## Decision

Preregister one direct three-scale STFT timeline. It restores the topology
selected in memo 019 without the normalized outer slices rejected in memo 021.
No implementation or sound-quality claim exists.

## Physical Geometry

At proof rate `F`, `H=F/100`. Long, middle, and short transforms are `8H`,
`4H`, and `2H`; every scale advances on the same `H` output lattice. Periodic
square-root Hann analysis/synthesis windows include the static factor `2H/N_s`,
so each unmasked scale overlap-adds to unity without a dynamic denominator.

Fixed ownership is `[0,750)`, `[750,6000)`, and `[6000,Nyquist] Hz`. Exact
crossover ties move upward. At `8 kHz`, short is inactive and middle owns
Nyquist. The resulting nonnegative coefficient totals are `191`, `592`, and
`631` at `8`, `44.1`, and `48 kHz`. These match the prior bounded physical
frame while removing its extra transform layer.

## Schedule And Boundaries

For target `T=round(L*r)`, use effective ratio `q=T/L`. Output centre `nH`
maps to source centre `round(nH/q)` from the absolute lattice. This prevents
accumulated source-map drift. Process exactly the synthesis centres whose long
window intersects the target crop, plus nine analysis-only guidance ticks at
each side. Input reads use even reflection. Output is cropped to `[0,T)` with
no fill or tail repair.

The longest half-window plus the future guidance dependency is reported as
`13H`, or `130 ms`, offline lookahead. Discontinuity tokens are unsupported in
this proof. Exact silence remains zero; joint-region recovery resets on the
first supported tick.

## State Ownership

Every channel computes ordinary recurrence before the shared terminal
decision. Reset and attack use current local analysis phase. Unlocked retains
local ordinary recurrence. Locked state first attempts local tracking, then
may borrow a compatible greatest-energy predecessor peak below `6000 Hz`.
Borrowing retains peer magnitude and current peer peak-relative analysis
offset. No ordinary/unlocked common rotation, cross-scale trajectory, or
post-hoc relation projection exists.

## Fixed Capacity

For channels `C<=2`, hop `H`, and owned nonnegative atoms `P`, prepare:

- `12HC` source samples
- `10CP` pending complex coefficients
- `19P` joint guidance magnitudes
- `2CP` prior phase values
- `2CP` peak/region records
- `8HC` output samples
- `C*14H` reusable transform values
- at most `16H` planner scratch values

At `48 kHz` stereo, these are `11520`, `12620`, `11989`, `2524`, `2524`,
`7680`, `13440`, and `7680`. Any excess fails before processing. No term grows
with duration.

## Identity Correction

Three independently windowed, frequency-masked STFT operators do not sum to a
perfect-reconstruction operator in general. Batch 29.7AR does not repeat the
outer-slice mistake by claiming otherwise.

The public unity path remains a bit-exact bypass. Batch 29.7AS must separately
prove each unmasked scale's overlap and full-band reconstruction, then report
the inert masked multi-scale residual on fixed controls. That residual is a
diagnostic for the later objective gate, not identity evidence and not a
parameter-tuning surface.

## Batch 29.6CH Boundary

Reuse only its centred extraction, even reflection, absolute crop, and
same-channel sum as proof concepts. Reject its fixed `128` hop, dynamic
crossovers/valleys, duration-sized buffers, per-scale dynamic normalization,
unconditional peer projection, hard locks, and code types.

## Current State

Batch 29.7AS passes representation mechanics at hash `fdf90f6127749341` and
Batch 29.7AT passes direct state mechanics at hash `430543f8e1dce721`. Batch
29.7AU passes synthetic evidence at hash `00e522a01b817bb6`, then rejects the
stereo gate at `40/48` calibrated failures, `118/384` improved windows,
`36/48` local failures, and hash `af461c9576729c4e`. Tone improves `0/192`
windows. That result opened Batch 29.7AV attribution against compatible
borrowed-peak phase ownership before another candidate. Tuning, retry,
listening, and holdout remained closed.

Batch 29.7AV confirms the collapse at hash `346e329081adf701`. A compatible
borrowed peak loses its full `0.95 rad` inter-channel relation and exits at
zero, while reset/attack remain exact and unlocked/exact-`6000 Hz` local lock
remain channel-local. Rule 31AA freezes one correction: borrowed atoms measure
their current phase offset from the owner peak; local atoms retain their own-
channel peak. Batch 29.7AW is mechanics-only. Objective audio remains closed.

Batch 29.7AW now applies exactly that correction. The borrowed peak retains
its complete `-0.9500000000000002 rad` input relation with zero error; the
focused fixture repeats at hash `425400ebb580b3e1`. Complete direct mechanics
pass `9/9` at corrected state hash `52d6b8b2bb6edff0`, while representation
remains `fdf90f6127749341`. The old `430543f8e1dce721` state hash is the
pre-correction baseline. No corpus audio ran.

## Next Task

Run Batch 29.7AX. Freeze and commit its full unchanged failure-first evidence
order and thresholds before generating corrected candidate audio, then stop at
the first hard miss.
