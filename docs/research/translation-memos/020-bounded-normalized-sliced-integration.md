# Bounded Normalized Sliced Integration

Status: validated
Date: 2026-07-18
Roadmap: `g10.029`, Batch 29.7AL
Contract: `082`, Rules 31S and 31T

## Question

Can the passing guided linked-phase mechanics cross an exact sliced frame with
one sample-rate formula, fixed live memory, and one persistent channel state?

## Comparison

| Boundary | Fixed `16384/8192/512` | Normalized `H=F/100` |
| --- | --- | --- |
| physical geometry | changes with rate | exact `10 ms` hop, `320 ms` span, `160 ms` advance |
| atom supports | fixed `4096/2048/1024` samples | exact `80/40/20 ms` as `8H/4H/2H` |
| crossover bins | rate-dependent | exact `240/1920` |
| `8 kHz` atoms | `2432/1217`; exceeds capacity | `380/191` |
| `44.1 kHz` atoms | not frozen as a common proof row | `1182/592` |
| `48 kHz` atoms | `1344/673` | `1260/631` |
| duration storage | sliced identity bounded; attempted whole-source guided path unbounded | fixed sliced source/output slabs and state rings |
| selection | reject as cross-rate integration | select for one Stage A proof |

Counts are signed/nonnegative atoms. The normalized rows stay inside the
existing `1344/673` capacity; no expansion is selected.

## Geometry And Dual

For proof rate `F` in `{8000, 44100, 48000}`:

- `H=F/100`
- `N=32H`, `A=16H`, `K=N/H=32`
- supports are `8H`, `4H`, and `2H`
- centre spacing is `4`, `8`, and `16` bins
- physical crossovers map to bins `240` and `1920`; Nyquist ends ownership
  before the second boundary at `8 kHz`

The rows are:

| `F` | `N/A/H` | supports | signed/nonnegative atoms | taps `T=2N-B` |
| --- | --- | --- | --- | --- |
| `8000` | `2560/1280/80` | `640/320/160` | `380/191` | `4740` |
| `44100` | `14112/7056/441` | `3528/1764/882` | `1182/592` | `27042` |
| `48000` | `15360/7680/480` | `3840/1920/960` | `1260/631` | `29460` |

Outer analysis and synthesis use
`h_F[n]=sin(pi(n+0.5)/N)`. Because `A=N/2`, adjacent squared windows sum to
one. The inner painless canonical dual remains frequency-exclusive. The only
synthesis law is `sum_s h_s D_F(C_F(h_s x))`; no independent overlap or scale
normalizer exists.

## State Crossing

For nonempty output length `L`, slice count is
`S(L)=floor((L-1)/A)+2`. The sliced schedule owns
`Q(L)=16S(L)+16` global common-lattice updates including boundary support.

The synchronized channel kernel runs once per global update. Its predecessor
phase, energy, and region state outlive every source or output slice. The
result is written into both active output layers. Each frequency atom therefore
has one decision owner even where two slices overlap. Slice creation and
retirement cannot reset state or run a second decision. Peer magnitude and
current analysis-relative phase remain layer- and channel-owned; no
peer/reference projection is introduced.

## Fixed Memory

Let `C<=2`, signed atoms `B<=1260`, nonnegative atoms/regions `P<=631`, and
`K=32`. Prepared live storage is:

- `8CBK` `Complex64` coefficient slots: six source slabs plus two output slabs
- `2N+K+max(s_N,s_K)` `Complex64` transform slots
- `2CN` `f64` outer-overlap samples
- `19P(C+3)` `f64` magnitude/material halo slots
- `6CP` `f64` guided current/prior slots and `2P` region records
- static `2N` `f64`, `T` tap records, and `B` band records

The maximum coefficient term is `645120 Complex64` slots. `s_N` and `s_K` are
the prepared FFT plans' reported scratch lengths. No term contains `L`.

Unsupported proof rates, rates not divisible by `100`, exceeded atom,
coefficient, region, slice, or scratch capacity return before processing. No
resize, whole-source store, alternate geometry, or recompute loop is a
fallback.

## Work

Per slice and channel, the structural meter counts:

- two `N`-point transforms
- `2B` `K`-point transforms
- `2T` tap visits
- `2BK` coefficient visits
- `4N` sample/window visits
- `N/2+1` conjugate-closure visits

Guided state work is reported separately as `Q(L)` bounded updates over at
most `P` atoms and `P` regions. This avoids pretending a mixed-radix FFT has a
power-of-two butterfly count.

## Decision

Select the normalized sliced representation for one identity and mechanics
proof. This is an execution correction, not support tuning or a sound-quality
claim. The old fixed frame remains valid evidence at `48 kHz` but is not the
cross-rate integration.

Do not add material policy, stretch audio, objective rows, listening, holdout,
dynamic ratio, realtime, routing, cache, or product work in the first proof.

Batch 29.7AM validates the selection at evidence hash `0407f765c7d84375`.
Peak combined identity error is `4.440892098500626e-16`; outer partition error
is `6.661338147750939e-16`; conjugacy is exact. Geometry, ownership, fixed
memory/work, boundary-token, finite, repeat, and overflow checks all pass.
Active-layer high-water is two and maximum coefficient storage remains
`645120 Complex64` slots. No guided or sound-quality claim follows.

Batch 29.7AN validates synchronized guided-state mechanics on that frame at
evidence hash `90c10cd2e66d4faf`. All state and boundary contexts pass; channel
mechanics are exact; local magnitude and analysis-relative phase remain within
`4.45e-16`. This validates the mechanics integration, not material policy or
sound quality.

## Next Task

Run Batch 29.7ANR. Freeze or reject the unchanged Rule 31R material policy and
complete objective evidence matrix on this frozen representation before any
quality renderer opens.
