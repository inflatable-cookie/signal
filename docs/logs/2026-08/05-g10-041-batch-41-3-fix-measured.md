# g10.041 Batch 41.3 - A18 Fix Implemented And Measured

Status: complete except listening admission
Created: 2026-08-05
Scope: the `A18` fix candidate, isolated and unadopted

## The Fix

`PhasePropagationMode::IdentityLockedTransientResetHighBand { crossover_bin }`
resets transient phase only above a crossover. Lower bins propagate continuously
through the transient.

The reasoning is about what the content is doing, not about thresholds.
Low-frequency content is *sustained through* a transient — a bass note rings on
while the attack happens — so resetting its phase destroys continuity in
something that never restarted. High-frequency content *is* the transient, and
resetting it is what stops smearing. The shipped code resets everything, which
is right for the second and wrong for the first.

The crossover is a fraction of Nyquist rather than a frequency, because the
stretch API carries no sample rate at any level. Frozen at `0.010`: `240 Hz` at
`48 kHz`, `220 Hz` at `44.1 kHz`.

No production constructor. Contract `084` Rule 2 keeps a candidate isolated and
Rule 5 makes listening the promotion authority, so the shipped default is
untouched and nothing in the workspace can reach the new mode.

## The Artifact Is Gone, And The Control Proves The Mechanism

Carrier phase jump, clean material, ratio `2.0`:

| path | jump |
| --- | --- |
| shipped, reset every bin | `2.752 rad` |
| crossover `48 Hz` | `2.752 rad` |
| crossover `120 Hz` | `0.133 rad` |
| crossover `240 Hz` | `0.133 rad` |
| no reset at all | `0.142 rad` |

The `48 Hz` row is the useful one. The probe tone is `80 Hz`, so at a `48 Hz`
crossover the tone sits *above* the line and still gets reset — and the result
reproduces shipped exactly, to three decimals. Protection appears only once the
crossover rises above the content it is meant to protect.

That is a control, not a coincidence. A fix that worked at every crossover value
would have suggested the mechanism was something else.

## It Does Not Trade The Pop For Smearing

Measured with the corpus's own `measure_transient_smear` and production
policies, on its own material. Lower is better:

| ratio | shipped | no reset | `120 Hz` | `240 Hz` | `504 Hz` |
| --- | --- | --- | --- | --- | --- |
| `1.5` | `2.0` | `0.0` | `2.0` | `2.0` | `4.0` |
| `2.0` | `1.0` | `0.0` | `1.0` | `1.0` | `7.0` |
| `3.0` | `0.0` | `8.0` | `0.0` | `0.0` | `9.0` |

The frozen crossover matches shipped smear exactly at every ratio. `504 Hz`
regresses, because it protects content that should be reset, so the safe window
is bounded on both sides and `240 Hz` sits inside it.

Removing the reset outright is not the fix: `8.0` at ratio `3.0` against
shipped's `0.0`. The reset earns its place. It was only ever applied too widely.

## The Proxy That Was Not Trusted

The first smear measurement here was a `10-90%` envelope rise time. It reported
the no-reset path as *sharper* than shipped at ratios `1.5` and `2.0` and
*smearier* at `3.0` — a direction that flips with ratio, which no real transient
behaviour does.

It was discarded rather than tuned, and the established corpus measurement used
instead. Batch 41.1 reached a confident wrong conclusion on an unvalidated
metric; doing it twice in the same lane would have been careless.

Both facts are now permanent tests in `benchmark.rs`: the candidate does not
regress smear, and dropping the reset entirely does.

## What Admission Needs

A concealed pack under Rule 5, on material with sustained low content beneath
transients, at ratios `1.5` and `2.0` where the artifact is largest, with sides
randomised per case as `g10.039` did.

Objective evidence says the artifact is gone at no measured cost. That is not
the same as sounding better, and Rule 5 exists for the difference.

## Next Task

Build the listening pack.
