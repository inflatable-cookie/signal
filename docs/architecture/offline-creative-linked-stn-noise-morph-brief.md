# Offline Creative LinkedStnNoiseMorph Renderer Brief

Status: construction-bound v6 ready for one isolated implementation
Owner: dsp
Updated: 2026-07-21
Contract: `085`
Roadmap: `g10.031`, Batches 31.42 through 31.53

## Decision

Build one Signal-owned
`ConstructionBoundZeroPreservingLinkedStnNoiseMorph` candidate
for neutral `Dream` at fixed creative expansion from `4x` through `16x`.

This is one material-separated renderer, not three optional effects. A
reconstructing two-stage analysis assigns tonal, transient, and residual
material on one source lattice. Persistent oscillators stretch tonal material.
Native waveform events move once. Continuous deterministic excitation morphs
only the residual. One linked-channel law, envelope, normalization system, and
exact crop own the final output.

The architecture is clean-room. Public SiTraNoStar and the STN papers inform
material ownership and validation only. Their GPL expression, constants,
thresholds, masks, tables, and control flow do not transfer. No external
library or model enters production.

Bounded v2 supersedes only the Batch 31.42 execution schedule and candidate
identity. Every transform, mask, map, tonal, transient, residual, envelope,
stereo, boundary, synthetic, and listening rule below is unchanged. The v2
schedule adds the missing causal prepass, explicit live frontiers, fixed ring
capacities, and category memory budgets. It does not repair or revive the
deleted Batch 31.43 checkpoint.

Capacity-audited v3 retains that complete renderer and schedule. It corrects
one impossible cross-geometry maximum in `MEMORY_SPEC` and assigns fresh
candidate identity after the deleted Batch 31.45 attempt. No audible formula,
threshold, source, metric, assertion, or gate changes.

Geometry-audited v4 retains the same renderer, schedule, capacities, and
quality gates. It corrects the exhaustive short vertical-median maximum from
`57` to `59`, freezes a shared `97`-scalar median-selection scratch bound, and
assigns fresh identity after the deleted Batch 31.47 assembly. No transform,
mask, map, threshold, source, metric, quality assertion, or audible owner
changes.

Zero-preserving v5 retains the same renderer, geometry, schedule, capacities,
and acoustic gates. It resolves the Batch 31.49 exact-silence contradiction by
making exact zero an explicit residual descriptor and synthesis state. No
positive residual interpolation, threshold, transform, mask, map, source,
quality limit, stochastic stream, stereo law, memory ceiling, or cost class
changes.

Construction-bound v6 retains the complete v5 renderer. It replaces the
duplicated handwritten structural geometry evidence with one compile-linked
`GEOMETRY_SPEC`, one separately coded exhaustive oracle, and one shared
authority assertion executed by construction before checkpoint and by `S02`
after checkpoint. No geometry formula, renderer behavior, memory ceiling,
source, quality limit, or gate order changes.

## Supported Request

The private `CandidateRequest<'a>` contains exactly:

- finite mono or interleaved stereo `input`
- `channels` equal to `1` or `2`
- integer `sample_rate` from `8000` through `192000`
- exact `target_frames`
- explicit `seed`
- finite `space` in `[0,1]`

Let source frames be `L`, target frames be `T`, and sample rate be `F`.
Require `4L<=T<=16L` with checked arithmetic. Reject values above `2^53-1`,
partial stereo frames, non-finite input, unsupported rates or channels,
overflow, range misses, and a non-empty request with `T=0` before output
allocation. Also reject when `T*channels*sizeof(f32)>isize::MAX` or exact
`Vec` reservation fails. Empty input with `T=0` returns empty.

The sole entry is:

`render(CandidateRequest<'_>) -> Result<Vec<f32>, CandidateError>`.

The closed error enum owns request, size, allocation-bound, and non-finite
processing failure. Character, motion, detail, pitch, reverse, dynamic ratio,
cache, artifacts, reports, realtime execution, and public exposure are absent.
`space` affects only eligible residual stereo width. Mono ignores it exactly.

## Geometry And Exact Map

Define `nearest_pow2(v)` by absolute integer distance; ties choose the larger
power. Every positive `round(a/b)` below uses checked integer arithmetic,
chooses the nearest integer, and chooses the larger integer at an exact half.
Freeze:

- tonal length `N_t=clamp(nearest_pow2(round(F/6)),2048,32768)`
- tonal analysis hop `A_t=N_t/8`
- short separation length `N_s=N_t/8`
- short analysis hop `A_s=N_s/4`
- residual-morph length `N_r=N_t/2`
- residual analysis hop `A_r=N_r/4`
- shared synthesis hop `H=N_t/16`

Integer division is exact for every supported geometry. At `44.1` and
`48 kHz`, `N_t=8192`, `N_s=1024`, `N_r=4096`, and `H=512`.

### Construction-owned geometry table

V6 freezes one compile-linked `GEOMETRY_SPEC` in `tests.rs`. Its row order is
`F,N_t,A_t,N_s,A_s,N_r,A_r,H,Q_h,Q_v,R_h,R_v`. These are its sole literal
sentinel rows:

| `F` | `N_t` | `A_t` | `N_s` | `A_s` | `N_r` | `A_r` | `H` | `Q_h` | `Q_v` | `R_h` | `R_v` |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 8000 | 2048 | 256 | 256 | 64 | 1024 | 256 | 128 | 9 | 97 | 9 | 59 |
| 8001 | 2048 | 256 | 256 | 64 | 1024 | 256 | 128 | 9 | 97 | 9 | 59 |
| 8500 | 2048 | 256 | 256 | 64 | 1024 | 256 | 128 | 9 | 91 | 9 | 55 |
| 9213 | 2048 | 256 | 256 | 64 | 1024 | 256 | 128 | 9 | 83 | 9 | 51 |
| 12288 | 2048 | 256 | 256 | 64 | 1024 | 256 | 128 | 13 | 63 | 13 | 39 |
| 16534 | 2048 | 256 | 256 | 64 | 1024 | 256 | 128 | 17 | 47 | 17 | 29 |
| 17500 | 2048 | 256 | 256 | 64 | 1024 | 256 | 128 | 17 | 45 | 19 | 27 |
| 18428 | 2048 | 256 | 256 | 64 | 1024 | 256 | 128 | 17 | 43 | 19 | 25 |
| 18429 | 4096 | 512 | 512 | 128 | 2048 | 512 | 256 | 9 | 83 | 9 | 51 |
| 36860 | 4096 | 512 | 512 | 128 | 2048 | 512 | 256 | 17 | 43 | 19 | 25 |
| 36861 | 8192 | 1024 | 1024 | 256 | 4096 | 1024 | 512 | 9 | 83 | 9 | 51 |
| 44100 | 8192 | 1024 | 1024 | 256 | 4096 | 1024 | 512 | 11 | 71 | 11 | 43 |
| 48000 | 8192 | 1024 | 1024 | 256 | 4096 | 1024 | 512 | 11 | 65 | 13 | 39 |
| 73724 | 8192 | 1024 | 1024 | 256 | 4096 | 1024 | 512 | 17 | 43 | 19 | 25 |
| 73725 | 16384 | 2048 | 2048 | 512 | 8192 | 2048 | 1024 | 9 | 83 | 9 | 51 |
| 98304 | 16384 | 2048 | 2048 | 512 | 8192 | 2048 | 1024 | 13 | 63 | 13 | 39 |
| 147452 | 16384 | 2048 | 2048 | 512 | 8192 | 2048 | 1024 | 17 | 43 | 19 | 25 |
| 147453 | 32768 | 4096 | 4096 | 1024 | 16384 | 4096 | 2048 | 9 | 83 | 9 | 51 |
| 179200 | 32768 | 4096 | 4096 | 1024 | 16384 | 4096 | 2048 | 11 | 69 | 11 | 41 |
| 184000 | 32768 | 4096 | 4096 | 1024 | 16384 | 4096 | 2048 | 11 | 67 | 13 | 41 |
| 192000 | 32768 | 4096 | 4096 | 1024 | 16384 | 4096 | 2048 | 11 | 65 | 13 | 39 |

The exact transform transitions are `18428 -> 18429`, `36860 -> 36861`,
`73724 -> 73725`, and `147452 -> 147453`. The lower row retains the old
transform; the upper row selects the next power of two.

The oracle uses only checked integer quotient/remainder arithmetic. It does
not call the renderer's rounding, odd, clamp, power-of-two, or geometry
helpers. It enumerates candidate powers directly and compares distances. For
every integer `F=8000..192000`, construction requires the renderer row and
oracle row to be identical.

`GEOMETRY_SPEC` also owns these exhaustive receipts:

- `184001` rows
- each row serialized as the twelve unsigned `u32` fields above in
  little-endian order, with `F` ascending
- `8832048` serialized bytes
- FNV-1a-64 `7ffb5aa02900893e`, checked in construction without a dependency
- SHA-256 `22d14913f01143007a114fad7a97d44a7e2b07cf5b254b92bc59c7f805e73697`,
  retained as the independent Batch 31.52 audit receipt

Exact tie ownership is also executable authority:

- `round(F/6)` chooses upward for every `F mod 6=3`: `30667` rates, first
  `8001`, last `191997`
- `nearest_pow2` has five six-rate midpoint sets:
  `9213..9218 -> 2048`, `18429..18434 -> 4096`,
  `36861..36866 -> 8192`, `73725..73730 -> 16384`, and
  `147453..147458 -> 32768`
- exact rational half sets are
  `Q_h={8000,11200,14400,17600,22400,28800,35200,44800,57600,70400,89600,115200,140800,179200}`,
  `Q_v=R_v={12288,24576,49152,98304}`, and
  `R_h={8500,9500,10500,11500,12500,13500,14500,15500,16500,17500,19000,21000,23000,25000,27000,29000,31000,33000,35000,38000,42000,46000,50000,54000,58000,62000,66000,70000,76000,84000,92000,100000,108000,116000,124000,132000,140000,152000,168000,184000}`
- the upward odd midpoint applies when the positive rounded value is even;
  exhaustive counts for `Q_h,Q_v,R_h,R_v` are respectively
  `82131,92567,98469,90925`

The exhaustive maxima and first witnesses remain `Q_h=17` at `16534`,
`Q_v=97` at `8000`, `R_h=19` at `17500`, and `R_v=59` at `8000`.
Construction compares the complete derived tie sets, counts, transitions,
maxima, first witnesses, sentinels, and domain fingerprint to
`GEOMETRY_SPEC`. A mismatch blocks checkpoint creation.

Every transform uses the centered periodic square-root Hann:

`w_N[n]=sqrt(0.5-0.5*cos(2*pi*(n+0.5)/N))`, `0<=n<N`.

Analysis frames have integer centres `k*A`. Source reads outside `[0,L)` are
exact zero. Frequency neighbourhoods reflect at DC and Nyquist. Time
neighbourhoods retain exterior zero frames. Forward transforms are unscaled;
inverse transforms use `1/N`.

The sole output-to-source sample-centre map is:

`x(y)=((y+0.5)*L/T)-0.5`.

Evaluate it as a checked signed `i128` rational with numerator
`(2y+1)*L-T` and denominator `2T`. Never accumulate a floating cursor.
Tonal frames, residual frames, transient anchors, envelope samples, evidence
windows, and exact crop all use this map.

Tonal synthesis centres are every integer `y=jH` whose `N_t` support
intersects `[0,T)`. Residual centres use the same `jH` lattice and their
`N_r` support. Process centres in ascending order. Analysis state is produced
only through the greatest source coordinate needed by the current synthesis
centre plus frozen lookahead, then evicted after every consumer passes it.

## Reconstructing STN Analysis

Analysis decisions use channel aggregate power
`P=(sum_c |X_c|^2)/channels`. Masks are scalar, channel-symmetric, and applied
to each native complex coefficient. Let `eps=1e-24` and
`smooth(u)=v*v*(3-2*v)`, where `v=clamp(u,0,1)`.

### Long tonal split

On the `N_t/A_t` source lattice, compute:

- horizontal median `Q_h` across
  `odd(round(0.240*F/A_t))`, clamped to `5..31` frames
- vertical median `Q_v` across
  `odd(round(375*N_t/F))`, clamped to `5..255` bins
- `rho=Q_h/(Q_h+Q_v+eps)`
- tonal mask `M_t=smooth((rho-0.55)/0.30)`

`odd(v)` chooses the nearest positive odd integer; a midpoint chooses the
larger odd integer. The masked coefficients are `X_t=M_t*X`; first residual
coefficients are `X_r1=(1-M_t)*X`.

Matched inverse WOLA reconstructs a tonal source stream and a first residual
stream. Each sample divides by the accumulated `w_N^2` denominator when it is
greater than `1e-12`, otherwise it is exact zero.

### Short transient split

Analyze the reconstructed first residual on the `N_s/A_s` lattice. Call its
native complex coefficients `R_s`. Compute:

- horizontal median `R_h` across
  `odd(round(0.064*F/A_s))`, clamped to `5..31` frames
- vertical median `R_v` across
  `odd(round(1800*N_s/F))`, clamped to `5..127` bins
- `tau=R_v/(R_h+R_v+eps)`
- transient mask `M_s=smooth((tau-0.58)/0.28)`

`X_transient=M_s*R_s` and `X_noise=(1-M_s)*R_s`. Matched WOLA reconstructs
native-channel transient and residual streams.

For finite input, source-domain reconstruction must satisfy
`input=tonal+transient+residual` before event claiming. Maximum absolute error
must be at most `1e-6*max(1,input_peak)` and RMS error at most
`1e-7*max(1,input_rms)`. Masks must remain finite and in `[0,1]`.

These constants are Signal-owned, sample-rate-normalized choices frozen before
candidate implementation. There is no hard mask, binary classification,
channel-local decomposition, or candidate-time threshold selection.

## Tonal Owner

The tonal lane reuses `X_t` on the long source lattice. For source frame `k`,
channel `c`, and bin `b`, define phase `phi[k,c,b]`. Instantaneous angular
frequency is:

`omega=2*pi*b/N_t + wrap(phi[k+1]-phi[k]-2*pi*b*A_t/N_t)/A_t`,

where `wrap` returns `(-pi,pi]` and the negative-real tie is `+pi`.
At `x(jH)`, interpolate log magnitude and `omega` linearly between the two
bracketing source frames. A zero magnitude uses `eps` for interpolation and
returns exact zero only when both endpoints are zero.

Peak candidates are aggregate-magnitude bins `1..N_t/2-1` that:

- are not below `1e-4` of the frame maximum
- are at least both immediate neighbours
- win a plateau at its lowest bin

Current peaks are visited by descending magnitude, then lower bin. Match each
to the unmatched live track with smallest frequency distance inside
`max(2*F/N_t,0.03*f_peak)` Hz; ties choose the lower track ID. Unmatched peaks
create monotonically numbered tracks. Unmatched tracks become dormant.

Each live track owns one output phase accumulator and last angular frequency.
An active or dormant track advances by `omega*H` at every output centre.
Dormant tracks may reactivate without phase reset while mapped source distance
from their last observation is at most `6*A_t`; after that distance they are
retired. Reactivation uses the predicted accumulator, not current analysis
phase. A new track starts at the selected linked analysis-axis phase.

Peak regions end halfway between adjacent peak bins; integer ties belong to
the lower peak. Every non-peak bin in a region uses identity phase locking:
the track accumulator plus its wrapped analysis phase offset from the region
peak. A frame with no eligible peak uses persistent bin oscillators with the
same instantaneous-frequency propagation. DC is linearly interpolated real
content. Nyquist is real with sign from the selected source frame.

### Linked tonal phase

For stereo coefficient pair `(X_L,X_R)`, define `U=X_L+X_R` and `V=X_L-X_R`.
Use axis `Z=U` when `|U|>=|V|`, otherwise `Z=V`; exact silence has no axis.
The choice is channel-symmetric. Swapping channels preserves `U` and negates
`V`, so its gauge change cancels in the native-channel ratios.

Track and bin phase propagate on `Z`. Each channel retains its interpolated
magnitude and wrapped native relation `arg(X_c)-arg(Z)` from the nearer source
frame; a fractional midpoint chooses the earlier frame. Analysis-axis changes
crossfade the old and new unit complex relations over exactly two synthesis
centres, normalized to unit magnitude; exact antipodal ties retain the old
axis for the first centre. Tonal `space` is always zero: the renderer never
widens pitched partials.

Stochastic renewal, phase diffusion, per-channel tracks, dormant phase reset,
and post-render tonal correction are forbidden.

## Transient Owner

Transient detection runs on the reconstructed native transient stream. For
short frame `m`, let `e[m]` be channel-mean windowed energy and:

`d[m]=max(0,ln(e[m]+eps)-ln(e[m-1]+eps))`.

Within `m-24..m+24`, excluding `m`, compute median `mu` and median absolute
deviation `mad`. A candidate must:

- be the earliest maximum in `m-2..m+2`
- satisfy `d[m]>=mu+4*max(mad,1e-6)`
- satisfy `e[m]>=1e-5*max(e[m-24..m+24])`

Exterior frames are zero. Commit only after all `24` future frames exist.
Refine inside `m*A_s-N_s/2..m*A_s+N_s/2` to the sample maximizing
`sum_c (t_c[n]-t_c[n-1])^2`; ties choose the earliest sample.

Around the refined sample, inspect `+/-2N_s`. Find the earliest shortest
interval containing `90%` of its channel-summed transient energy. Classify the
event `Impulse` when that interval is no longer than `N_s/2`, otherwise
`Attack`.

For `Impulse`, search left at most `N_s` and right at most `2N_s`; for
`Attack`, search left at most `2N_s` and right at most `4N_s`. A side ends at
the nearest run of `64` samples whose channel-summed power is at most
`1e-4` of the event-neighbourhood peak. If no run exists, use the cap. Adjacent
segments are clipped at the floor midpoint between refined source anchors.

Each retained segment gets sine-squared edge weights of length
`min(N_s/8,floor(segment_length/4))`. The claimed transient is the weighted
native segment. All unclaimed transient energy, including removed edge weight,
is added to the residual stream before residual analysis. Source-domain
`tonal+claimed events+augmented residual` therefore preserves the same
reconstruction tolerance.

For source event sample `p`, the sole target anchor is:

`q=floor((2p+1)*T/(2L))`,

using checked `u128`. Emit its native samples once at unit rate:
`output[q+(n-p)]+=claim[n]`. Crop out-of-range samples. At `T/L>=4`, source
midpoint clipping makes target event supports disjoint. If checked arithmetic
still finds an overlap, split at the earlier floor midpoint of target anchors;
the earlier event owns the midpoint sample. No event gain, repetition,
stretch, spectral phase, reset copy, wrap, or reflected tail is allowed.

An event ledger keyed by monotonically increasing source event ID permits one
commit and one emission only. Tile or ring boundaries cannot re-detect,
re-segment, or re-emit an event.

Transient phase treatment is the native time-domain waveform. There is no
spectral reset, phase reassignment, or transient copy in either bed lane.

## Residual Noise-Morph Owner

Analyze the augmented residual stream on the `N_r/A_r` source lattice. For
each source frame and bin, estimate the native-channel covariance from the
uniform `5`-frame by `3`-bin neighbourhood. Time exterior is zero; frequency
reflects. Divide by the exact number of samples.

Mono retains scalar power. Stereo retains Hermitian:

`C=[[a,c],[conj(c),b]]`, where `c=E[X_L*conj(X_R)]`.

Project numerical error to positive semidefinite form by clamping `a,b` to
zero and `|c|` to `sqrt(a*b)`. After projection, canonicalize every numeric
zero to positive zero. At a source descriptor, coherence is exact zero with
phase zero when `a*b=0`; otherwise it is `c/sqrt(a*b)`.

For non-negative endpoints `p0,p1` and interpolation weight `u` in `[0,1]`,
define the sole zero-preserving log interpolation:

`zlog(p0,p1,u)=+0` when `p0=0` and `p1=0`; otherwise
`exp((1-u)*ln(p0+eps)+u*ln(p1+eps))`.

Use `zlog` independently for `a` and `b` at `x(jH)`. This is an exact-zero
branch, not a power threshold. One-zero/one-positive and two-positive rows are
byte-for-byte the v4 formula. Interpolate canonical coherence magnitude
linearly and phase on the shortest wrapped arc; a `pi` tie is positive. If
either interpolated diagonal is zero, set `c` to exact complex positive zero
without evaluating phase. Otherwise reconstruct `c` from the interpolated
diagonals, coherence, and phase.

For frequency weight `h`, use zero through `250 Hz`, one from `1500 Hz`, and
smoothstep between. When `Re(c)>0` and coherence `g>0`, replace its magnitude
by `sqrt(a*b)*g^(1+2*space*h)`. Preserve its phase. When `Re(c)<=0`, preserve
`c`. This keeps duplicate and anti-phase material unchanged, never changes
channel power, never reduces side energy, and widens only partially coherent
positive-correlation residual above the low band.

### Continuous deterministic excitation

Freeze `ADMISSION_SEED=0x0123456789abcdef`. Define wrapping `mix64(z)`:

1. `z=(z xor (z>>30))*0xBF58476D1CE4E5B9`
2. `z=(z xor (z>>27))*0x94D049BB133111EB`
3. return `z xor (z>>31)`

For absolute signed output sample `n`, zero-pad when it lies outside the
representable output lattice. Otherwise stream `s` uses:

`r_s[n]=mix64(seed xor mix64(n xor TAG_s))`,

with little-endian tags `STNNOIS0` and `STNNOIS1`. The real excitation sample
is `+1` when bit `63` is set and `-1` otherwise. Window each continuous stream
on the `N_r/H` output lattice, transform it, and normalize every active
positive-frequency coefficient to unit magnitude. A zero coefficient becomes
`1+0i`. DC and Nyquist use its real sign; negative bins are conjugate mirrors.

Exact construction vectors are:

| Input | `mix64` |
| --- | --- |
| `0x0000000000000000` | `0x0000000000000000` |
| `0x0000000000000001` | `0x5692161d100b05e5` |
| `0x0123456789abcdef` | `0xb2c058e4ebb5112c` |
| `0xffffffffffffffff` | `0xb4d055fcf2cbbd7b` |

| Stream | `n` | Counter | Sign |
| --- | ---: | --- | ---: |
| `STNNOIS0` | `0` | `0x075918a4031b66c5` | `-1` |
| `STNNOIS0` | `1` | `0x06735507091ccdd2` | `-1` |
| `STNNOIS0` | `48000` | `0x9aef6e781e8c05f6` | `+1` |
| `STNNOIS0` | `96000` | `0x9a5b0da228689115` | `+1` |
| `STNNOIS1` | `0` | `0xd0db092e952c2515` | `+1` |
| `STNNOIS1` | `1` | `0xce8dd5a236bb7044` | `+1` |
| `STNNOIS1` | `48000` | `0xa9cb60461663733f` | `+1` |
| `STNNOIS1` | `96000` | `0xd0b1541e5ee1c6e1` | `+1` |

Mono emits exact complex positive zero when `a=0`; it does not multiply an
excitation coefficient by zero. Otherwise it multiplies `sqrt(a)*U_0` by the
sign of the first exactly non-zero augmented-residual sample.

Stereo factors in the orthonormal mid/side basis. Convert the post-`space`
left/right covariance to mid power `a_m`, side power `a_s`, and
`d=E[M*conj(S)]`. Let `o_m` and `o_s` be the signs of the first exactly
non-zero time-domain augmented-residual mid and side samples; an exactly silent
component uses `+1`.

- if `a_m>0`, emit `Y_M=o_m*sqrt(a_m)*U_0`; set
  `alpha=conj(d)/(o_m*sqrt(a_m))`,
  `beta=sqrt(max(0,a_s-|alpha|^2))`, and
  `Y_S=alpha*U_0+o_s*beta*U_1`
- if `a_m=0`, PSD projection makes `d=0`; emit `Y_M=0` and
  `Y_S=o_s*sqrt(a_s)*U_1`
- decode `Y_L=(Y_M+Y_S)/sqrt(2)` and
  `Y_R=(Y_M-Y_S)/sqrt(2)`

If `a_m=a_s=0`, set `Y_M`, `Y_S`, `Y_L`, and `Y_R` directly to exact complex
positive zero. Any exact-zero `beta` is also canonical positive zero. Do not
evaluate a stochastic multiply for an inactive component. The same transform
and fixed traversal still execute; zero ownership does not create a
data-dependent fast path.

Common negation flips both orientations. Channel swap preserves `o_m`, flips
`o_s`, and negates `d`. The same basis streams therefore produce exact common
negation and swapped output, including the anti-phase rank-one case.

This is one shared two-stream excitation basis shaped by the source covariance,
not unrelated per-channel noise. Descriptor covariance, channel diagonals,
swap, duplicate, common polarity, anti-phase, and `space` direction are exact
structural owners. Rendered long-window balance remains a hard evidence gate.

## Envelope, Recombination, And Boundaries

From original channel-mean source power, compute a square-root-Hann weighted
RMS envelope `e[n]` of length `N_r`. Let `m[n]` be the maximum `e` within
`+/-N_r/2`, including zero exterior. Define `g[n]=0` when `m[n]<=1e-12`,
otherwise `g[n]=clamp(e[n]/m[n],0,1)`. Linearly interpolate `g` at the exact
map for every output sample.

Tonal inverse frames use `N_t/H` normalized WOLA. Residual inverse frames use
`N_r/H` normalized WOLA. Each lane divides by its own accumulated `w^2`
denominator above `1e-12`. Apply mapped `g` once to the normalized
tonal-plus-residual bed, then add one-shot native transient segments. When
mapped `g` is exactly zero, contribute exact positive zero from the bed rather
than multiplying. Initialize accumulators to positive zero. Before the sole
`f64`-to-`f32` conversion, encode an exact numeric zero as positive `0.0f32`.
This is representation canonicalization, not a non-zero threshold or repair.

Accumulate in fixed-order `f64`, verify finiteness, and convert once to `f32`.
Emit exactly `[0,T)`. There is no renderer-owned head or tail fade, padding in
the returned buffer, wrap, reflect, resize-fill, limiter, compressor, clipper,
automatic gain, channel gain, DC blocker, or post-render repair. Silence is
bit-exact zero. A non-finite intermediate or output peak above `8.0` is a hard
processing error, not a reason to clamp.

This mapped envelope suppresses bed pre-echo around source energy changes. It
does not invent an exterior fade. Entry and tail energy remain source-mapped
and are judged explicitly against PaulXStretch in long-form listening.

## Bounded State, Determinism, And Cost

### Causal pass split

Residual orientation depends on the first exactly non-zero augmented-residual
mid and side samples. Those samples do not exist until long separation, short
separation, event claiming, and residual reassignment complete. Synthesizing
before discovering them changes the frozen stereo law; retaining every
descriptor until end of input violates bounded state.

Bounded v2 therefore uses exactly two source passes:

1. The orientation prepass runs long and short separation, event detection,
   claiming, and residual reassignment in source order. It records only the
   mono residual sign or the two stereo signs `o_m,o_s`. It consumes the full
   source including the fixed zero flush, then discards all analysis state.
2. The render pass resets every counter, frame, event ID, median, WOLA, and
   descriptor state. It reproduces the same source components, computes tonal
   and residual descriptors, synthesizes in ascending output order, and uses
   the two prepass signs. The prepass never supplies component samples,
   spectra, events, masks, envelopes, or output data to this pass.

Both passes use the same sole map, decomposition, event state machine, tie
rules, and traversal. The prepass adds fixed work, not a second audible owner.
Silence completes the pass with `+1` orientation exactly as already frozen.

### Live frontiers and eviction

Let `h_t=(Q_h-1)/2` and `h_s=(R_h-1)/2`. Exhaustive integer evaluation over
every supported sample rate gives `Q_h<=17`, `R_h<=19`, `Q_v<=97`, and
`R_v<=59`; therefore `h_t<=8` and `h_s<=9`.

The render pass owns monotonic frontiers for long analysis, first-residual
samples, short analysis, event decisions, augmented residual, residual
covariance, envelope, synthesis centres, and finalized output. For the next
output finalization interval it first determines the greatest source
coordinate required by:

- every tonal centre whose `N_t` support intersects the interval
- every residual centre whose `N_r` support intersects the interval
- both source descriptor frames bracketing each mapped centre
- both envelope samples bracketing each mapped output sample
- every event whose capped native segment can intersect the interval

It then advances source stages only through that coordinate plus their fixed
median, detector, segmentation, covariance, and window lookahead. This is an
inverse bound on the existing signed-rational map and event-anchor formula,
not a second timeline or floating cursor.

Eviction is exact:

| State | Last consumer |
| --- | --- |
| native long spectrum and aggregate power | its final horizontal-median centre |
| resolved tonal frame | the later mapped bracket and sequential track matcher |
| first-residual sample | the last overlapping short analysis window |
| transient and residual source sample | event right-cap decision, residual frame, and streaming reconstruction check |
| residual covariance frame | the later mapped residual bracket |
| envelope sample or maximum-deque node | the later mapped envelope bracket |
| claimed event sample and descriptor | its cropped target support is finalized |
| tonal or residual output accumulator | every overlapping synthesis centre has executed |

Event candidates are separated by at least three short frames through the
earliest-maximum rule. Claimed segments are source-ordered and midpoint
clipped. The ledger is therefore a fixed live queue plus monotonically
increasing `last_committed_id` and `last_emitted_id`, never a set of all past
events. An output sample finalizes only after all bed centres are accumulated,
its envelope bracket exists, and the event-decision frontier proves that no
anchor through `y+2N_s` can place a capped native segment on that sample.

### Fixed storage

Spectral rings retain packed bins `0..N/2` in `f64` complex form. FFT work
buffers may be full complex arrays but are reused by size and channel. Source
components and output accumulators are `f64`. The sole duration-derived
mutable allocation is the required returned interleaved `Vec<f32>`; each
output sample is converted into it once after finalization. A full-duration
`f64` output, denominator, component, spectrum, descriptor, event ledger, or
envelope is forbidden.

Compile-linked `MEMORY_SPEC` owns these capacities:

| State | Capacity | Maximum |
| --- | --- | ---: |
| native long frames | `Q_h+3` | `20` frames |
| resolved tonal frames | fixed | `4` frames |
| native short frames | `R_h+3` | `22` frames |
| shared median-selection scratch | `max(Q_h,Q_v,R_h,R_v)` | `97` `f64` scalars |
| first-residual samples | `N_t+2(h_s*A_s+N_s)` | `53248` |
| transient and augmented-residual samples, each | `2N_r+16N_s+48A_s+256` | `147712` |
| residual spectrum/covariance frames | fixed | `7` frames |
| claimed-event sample arena | `12N_s+48A_s+512` | `98816` |
| live event descriptors | `ceil(claim_capacity/(3A_s))+4` | `39` events |
| envelope and maximum-deque samples | `2N_r+4` | `32772` |
| output finalization samples | `2N_t+2N_r+8N_s+4H+256` | `139520` |
| peak tracks | `N_t/2-1` | `16383` tracks |
| persistent bin states | `N_t/2+1` | `16385` bins |

V6 adds exact first and last maximum witnesses to `MEMORY_SPEC`:

| State | Maximum | First `F` | Last `F` |
| --- | ---: | ---: | ---: |
| native long frames | 20 | 16534 | 147452 |
| native short frames | 22 | 17500 | 147452 |
| shared median scratch | 97 | 8000 | 8041 |
| first-residual samples | 53248 | 184000 | 192000 |
| each transient/residual ring | 147712 | 147453 | 192000 |
| claimed-event arena | 98816 | 147453 | 192000 |
| live events | 39 | 8000 | 18428 |
| envelope state | 32772 | 147453 | 192000 |
| output finalization | 139520 | 147453 | 192000 |
| peak tracks | 16383 | 147453 | 192000 |
| persistent bin states | 16385 | 147453 | 192000 |

Construction derives these rows from the independent geometry oracle and
separately coded capacity formulas. A maximum or witness mismatch blocks the
checkpoint. The capacities themselves do not change.

Conservative maximum-capacity packed-`f64` models occupy `17.502 MiB`
for long frames, `9.700 MiB` for short/source WOLA state, `4.001 MiB` for
residual covariance and excitation, `1.508 MiB` for claimed event samples,
`1.001 MiB` for envelope state, and `8.516 MiB` for output finalization.
The `R_v` correction adds no allocation: `Q_v=97` already owns the shared
median scratch maximum. Every ring capacity and packed model remains
unchanged.
Category ceilings are:

| Category | Ceiling |
| --- | ---: |
| long spectra, medians, and resolved tonal frames | `22 MiB` |
| short spectra and source WOLA rings | `12 MiB` |
| residual spectra, covariance, and excitation | `9 MiB` |
| events, ledger, and claim arena | `4 MiB` |
| tonal tracks, oscillators, and axis state | `5 MiB` |
| envelope moments and deques | `2 MiB` |
| output accumulation and finalization | `12 MiB` |
| FFT plans, work buffers, windows, and allocator slack | `20 MiB` |
| traversal and miscellaneous fixed state | `3 MiB` |
| **owned-state design ceiling** | **`89 MiB`** |

The remaining `7 MiB` is not assignable. The counting allocator still owns
the terminal actual peak of `96 MiB`, including capacity, plans, scratch, and
allocator overhead after subtracting only returned-output capacity.

The square-root-Hann RMS envelope uses three fixed `f64` sliding moments for
the Hann-weighted power sum, rebased by direct fixed-order evaluation every
`N_r` source samples. A monotonic deque owns the centred `+/-N_r/2` maximum.
This evaluates the frozen envelope with bounded memory and amortized `O(1)`
work per source sample; no duration-derived envelope table exists.

Allocate returned output, all arenas, FFT plans, scratch, windows, medians,
tracks, oscillators, deques, and conversion state before the orientation
prepass. No allocation is permitted in either pass.

The prepass repeats long/short separation and event ownership once. The render
pass adds residual analysis and all output synthesis. Cost is:

`O(2*(L/A_s)*N_s log N_s + 2*(L/A_t)*N_t log N_t +
(L/A_r)*N_r log N_r + (T/H)*(N_t log N_t+N_r log N_r))`.

Traversal is single-threaded, reductions have fixed order, and all ties are
explicit. The same complete request on the supported platform contract must
be byte-identical.

Offline only. No audio-thread source fill, execution, synchronization, I/O,
or allocation is authorized.

## Construction-Bound V6 Candidate Isolation And Construction

Use exactly:

- worktree: `signal-candidate-31-53`
- branch:
  `candidate/g10-031-construction-bound-zero-preserving-linked-stn-noise-morph`
- module:
  `crates/signal-dsp-stretch/src/creative_construction_bound_zero_preserving_linked_stn_noise_morph/`
- files: `mod.rs`, `plan.rs`, `decomposition.rs`, `tonal.rs`, `transient.rs`,
  `noise.rs`, `synthesis.rs`, `tests.rs`

The isolated `lib.rs` may declare the module privately. Use existing crate
dependencies only. No public API, production tier, feature, report, binary,
fixture, cache, artifact schema, route, Loophole, or Chorus change is allowed.
Generated evidence stays ignored under `target/`.

Test prefixes are only:

- `construction_bound_zero_preserving_linked_stn_noise_morph_construction_`
- `construction_bound_zero_preserving_linked_stn_noise_morph_structural_`
- `construction_bound_zero_preserving_linked_stn_noise_morph_synthetic_`

`tests.rs` owns one compile-linked `GATE_OWNERS` table with exactly `28`
unique IDs and function pointers: `18` structural and `10` synthetic. One
compile-linked `EVIDENCE_SPEC` owns every source sample, support, estimator,
table value, threshold, seed, ratio, and assertion below. Helpers may not
select implicit values. One compile-linked `MEMORY_SPEC` owns every frontier,
capacity, maximum, category ceiling, duration vector, and allocation
assertion above. One compile-linked `GEOMETRY_SPEC` owns the sole literal
sentinel table, transform transitions, tie sets and counts, extent maxima and
first witnesses, row count, byte count, and FNV-1a domain fingerprint above.
No gate may carry another literal geometry row.

Construction order:

1. `effigy test compile`
2. run the construction prefix and require exactly `1/1`
3. create one immutable local checkpoint and record its hash
4. freeze source, tests, helpers, assertions, manifest, and checkpoint

The construction owner verifies file inventory, gate inventory, formulas,
tags, non-geometry vectors, source tables, support tables, sole seed, two-pass
state reset, every `MEMORY_SPEC` formula and witness, and the `89 MiB` design
sum. It runs one shared `assert_geometry_authority` helper that compares the
renderer and independently coded oracle at every supported rate, then checks
every `GEOMETRY_SPEC` sentinel, transition, tie set, tie count, maximum, first
witness, row count, byte count, and FNV-1a fingerprint. `S02` must call the
same helper; it may add map vectors but may not restate geometry literals.
Construction also verifies `zlog` truth vectors at `u=0,0.5,1`, positive-zero
bit patterns, canonical zero coherence, and non-zero preservation for the
smallest represented positive `f64` endpoint.
Before the checkpoint, repairs may address compiler, type, visibility,
ownership, or manifest assembly only when they change no DSP formula, literal,
source, metric, threshold, helper result, or assertion. Any later miss is
terminal.

The closeout receipt must record checkpoint hash, candidate Git tree ID,
SHA-256 of every candidate file, `EVIDENCE_SPEC`, `MEMORY_SPEC`, `Cargo.lock`,
`rustc -vV`, platform, per-owner outcome, every numeric row, and SHA-256 of
every rendered synthetic output. Receipts from different checkpoints are comparable only
when all those executable-identity fields match. Generated receipt files do
not enter `main`; the closeout log retains their digests and result summary.

## Structural Gate

Run all owners once after construction. Require exactly `18/18`.

| ID | Owner and pass condition |
| --- | --- |
| `S01` | request matrix rejects every invalid case before output allocation; valid empty returns empty |
| `S02` | the construction-owned `assert_geometry_authority` passes unchanged; signed rational map vectors match independent integer vectors at every boundary and `4x`/`8x`/`16x` ratio; no second geometry table exists |
| `S03` | every analysis and synthesis lattice has complete normalized coverage; constant WOLA reconstructs within `1e-7` |
| `S04` | masks are finite and reconstruct mono/stereo source within the frozen peak and RMS tolerances |
| `S05` | streaming analysis equals an independent full-buffer oracle within `1e-6`; ring wrap does not change event or descriptor ownership |
| `S06` | tonal peak matching, identity regions, bin fallback, frequency propagation, and track IDs match handwritten vectors |
| `S07` | disappearance, dormancy, reactivation, expiry, new-track phase, and two-centre axis transition match exact state traces |
| `S08` | tonal duplicate, common polarity, anti-phase, and channel swap commute samplewise within `1e-6` |
| `S09` | transient novelty, threshold, refinement, class, segment, edge claim, and residual reassignment match impulse and attack vectors |
| `S10` | each source event has one ledger commit, one mapped anchor, one emission, disjoint support, and no boundary duplicate |
| `S11` | residual counter and tag vectors match exactly; repeats match bytes and changed seed changes non-silent residual output |
| `S12` | zero-preserving diagonal interpolation, canonical coherence, covariance projection, and factorization match exact zero/one-zero/positive handwritten vectors; target matrices reproduce within `1e-10`; diagonal powers never change with `space` |
| `S13` | residual duplicate, common polarity, anti-phase, and swap commute within `1e-6`, including exact-zero descriptors and spectra; basis streams are shared, never channel-local |
| `S14` | `space` preserves `0..250 Hz`, leaves tonal/events unchanged, and makes aggregate residual side energy non-decreasing within `1e-9` |
| `S15` | mapped envelope, lane denominators, recombination, exact crop, and no exterior fade match edge-source vectors; all-zero mono/stereo and mapped exact-zero bed regions contain only positive-zero `f32` bit patterns |
| `S16` | one-frame, shorter-than-window, odd/even, impulse-at-edge, sustained-to-edge, positive/signed-zero, and all-zero inputs remain finite and exact length; signed-zero silence returns positive zero |
| `S17` | `MEMORY_SPEC` maxima and category sum match independent formulas; max-geometry stereo plan construction at `F=192000`, `L=N_t,4N_t,16N_t`, and every `4x/8x/16x` row reports identical working capacity, at most `89 MiB` designed and `96 MiB` actual; complete stereo renders over the same duration/ratio matrix at `F=8000` report zero allocation in either pass |
| `S18` | source scan and call graph contain no capacity derived from `L` or `T` except returned `Vec<f32>`, full-duration component or `f64` output, all-history event set, random device, limiter, clipper, channel gain, non-zero silence threshold, denoiser, external DSP, second map, renewal tonal phase, public route, or hidden report path |

`S08` and `S13` are samplewise relationship invariants. `S12` is a descriptor
invariant, not a claim that one stochastic output frame has exact source
magnitude. Long-window channel balance belongs to `Y09` and listening.

## Synthetic Sources

All sources use `F=48000`, `L=96000`, and exact `T=4L`, `8L`, or `16L`.
Unless noted, authored support is `[24000,72000)` with linear fades over
`24000..26047` and `69952..71999`.

- low tone: `0.5*sin(2*pi*110*n/F)`
- mid tone: `0.5*sin(2*pi*440*n/F)`
- chord: amplitude `0.1` at `110`, `164.813778`, `220`, `277.182631`, and
  `329.627557 Hz`
- harmonic pad: `sum(k=1..8) (0.35/k)*sin(2*pi*110*k*n/F)`
- impulse: value `1` at `48000`, support `[48000,48001)`
- impulse train: values `1,-0.8,0.65,-0.5` at
  `19200,38937,58103,77797`, support `[19200,77798)`
- silence gap: harmonic pad with exact zero on `[42000,54000)`
- uniform noise: `0.5*(2*((mix64(n xor TEST)>>11)/2^53)-1)`
- Rademacher noise: `+0.5` for the high bit of `mix64(n xor TEST)`, else
  `-0.5`
- amplitude-modulated noise: Rademacher sign times
  `0.5+0.375*sin(2*pi*1.7*n/F)`

`TEST` is little-endian `RNWTEST0`. Noise uses the same authored support and
fades. Stereo controls are:

- duplicate mid tone
- common polarity: duplicate and exact common negation
- anti-phase: mid tone and exact negative
- delayed pad: right is a zero-padded `37`-sample delay
- mixed: left is chord plus `0.2` uniform noise; right is the delayed chord
  minus the same `0.2` noise

Every candidate render uses `ADMISSION_SEED` and `space=0.5` unless the owner
names another `space` row.

## Synthetic Gate

Run every row in an owner before its one final assertion. Require exactly
`10/10`. Comparator numbers below are pinned PaulXStretch `1.6.0`, default
processing, FFT `16384`, exact-cropped.

### `Y01` finite level and crest

Measure crest growth in dB from authored source support to mapped output
support. Require each candidate row no greater than matching PaulX plus
`2 dB`. Reference values at `4x/8x/16x`:

| Source | `4x` | `8x` | `16x` |
| --- | ---: | ---: | ---: |
| uniform | `9.931823324` | `11.898668127` | `10.432303004` |
| Rademacher | `14.809189700` | `15.525287997` | `15.710575458` |
| amplitude-modulated | `13.019591155` | `12.553111650` | `14.090964151` |
| harmonic pad | `6.348431050` | `6.745264841` | `6.313515214` |

All outputs must be finite, non-clipped, and have peak at most `8.0` before
listening normalization.

### `Y02` sustained pitch diagnostic

Use the central half of mapped authored support, periodic Hann, zero padding
to the next power at least eight times the measured length, search expected
frequency `+/-4 Hz`, select maximum magnitude, then use three-bin log-parabolic
interpolation. Record finite candidate error, PaulX error, and delta for every
tone and chord frequency. Missing or non-finite rejects; no numeric delta does.

| Source | `4x` | `8x` | `16x` |
| --- | ---: | ---: | ---: |
| `110 Hz` | `7.410431632` | `8.816034233` | `7.666572548` |
| `440 Hz` | `2.461974128` | `5.373507937` | `4.838384698` |
| chord maximum | `7.976134703` | `9.331375778` | `13.456683592` |

### `Y03` event placement and crest

For the isolated impulse and every impulse-train event:

- event ledger anchor must equal `floor((2p+1)T/(2L))` exactly
- full-output impulse energy centroid error must be no greater than the PaulX
  error plus `10%` of PaulX `95%` energy width
- shortest inclusive `95%` energy width must be no greater than `1.5` times
  PaulX width
- isolated-impulse absolute peak must be at most `1.25`; maximum absolute
  first difference must be at most `1.5`
- every transient-lane event peak must be no greater than its claimed source
  peak plus `1e-6`

PaulX widths are `79469`, `155953`, `309239`; centroid errors are
`49188.649257221`, `114695.538853499`, `246065.455355601` at
`4x/8x/16x`.

### `Y04` replica prevention

Use RMS windows of `480` frames at hop `240`. Active means at least `-30 dB`
relative to the global window peak. A new region begins when the current
active start is at least `2400` frames after the prior active start. The
primary region contains the global peak. Require exactly one region and an
explicit `None` secondary for isolated impulse and impulse train at every
ratio. `-30 dB` is an activity threshold, not a secondary allowance.

### `Y05` residual non-periodicity

On uniform noise, subtract the mean and measure normalized autocorrelation at
every exact lag `960..48000`. Maximum absolute correlation must be no greater
than PaulX plus `0.05`: `0.017218163`, `0.017727693`, `0.017090511`.

### `Y06` block-energy stability

Within mapped support, use RMS windows of `2400` frames at hop `1200` and
measure coefficient of variation. Candidate must be no greater than PaulX
plus `0.05`:

| Source | `4x` | `8x` | `16x` |
| --- | ---: | ---: | ---: |
| uniform | `0.387747959` | `0.460013282` | `0.492971808` |
| mid tone | `0.617268653` | `0.679139581` | `0.708639858` |

### `Y07` silence-gap ownership

Map the exact source gap through the sole rational map. Gap RMS relative to
the two equal-duration adjacent regions must be no greater than PaulX plus
`3 dB`. PaulX values are `2.565308664`, `2.752688694`, and `3.061967868 dB`.

### `Y08` discontinuity, dropout, and boundaries

Require exact length, finite samples, and no sample clamp for every source.
For dropout, map source support `[a,b)` to
`[floor(aT/L),ceil(bT/L))` with checked `u128`, clipped to `[0,T]`. Examine
only complete fixed `16384`-sample windows wholly inside that hull. A shorter
hull passes vacuously. No eligible window may be exact zero except the authored
interior silence gap. First differences are scanned across complete output;
isolated impulse uses the hard `Y03` bound. Entry and tail quarter energy are
recorded for every row and must be finite; their creative decision is
listening-owned.

### `Y09` linked stereo

Render every stereo control at every ratio and `space=0`, `0.5`, `1`.
Require exact length, finiteness, byte repeat, duplicate/mono within `1e-6`,
common polarity and anti-phase within `1e-6`, and swap-commuted output within
`1e-6` after swapping it back. Require descriptor diagonal preservation within
`1e-10` and non-decreasing residual side energy within `1e-9`.

For each render, candidate-source right-minus-left balance error must be at
most `0.75 dB` over the whole render and bands `0..250`, `250..1500`, and
`1500..Nyquist`. Balance spread across the three `space` renders must be at
most `0.50 dB`. No whole or band channel-dominance reversal is allowed when
source balance magnitude is at least `0.50 dB`.

### `Y10` material ownership

For every mono source and ratio, record finite tonal, claimed-transient,
unclaimed-transient, residual, mapped-envelope, and final-output RMS and peak.
Require source-domain component reconstruction within `S04`, one ledger event
for the isolated impulse, four ordered events for the train, zero event commits
for the steady tones, and exact-zero residual output for exact-zero residual
descriptors. Lane dominance is diagnostic, not a candidate-selected pass
threshold.

## Mono Listening Gate

Open listening only after construction and all objective owners pass. Use the
retained percussion, bass, vocals, pads/sustains, and full-mix families at
`4x`, `8x`, and `16x`. Compare against PaulXStretch `1.6.0` default / FFT
`16384`. Candidate uses `ADMISSION_SEED` and `space=0.5`.

Exact-crop every file. Within each source/ratio row, apply one common RMS
target across source, candidate, and PaulX, reduced only enough to keep every
peak at or below `0.95`. Conceal A/B identity.

Pass requires:

- no unusable candidate row
- candidate preferred or tied on at least `12/15`
- no source family loses every ratio
- no uncontrolled vocoder tone, periodic flutter, cyclic repetition, doubled
  attack, micro-echo, stutter, static freeze, arbitrary loudness, or click

The scorecard must explicitly record smoothness, musical usefulness, tonal
ringing, transient softness or spike, low-frequency noise/haze, event
placement, entry energy, and tail energy. Objective success cannot waive this
gate.

## Independent Stereo Admission

Use these exact stereo originals:

- `0000-drums_percussion-000002.wav`
- `0004-bass-000236.wav`
- `0008-vocals-000010.wav`
- `0012-pads_sustains-000423.wav`
- `0016-full_mix-000144.wav`

Render all five at `4x`, `8x`, and `16x`, and at `space=0`, `0.5`, and `1`.
Capture PaulXStretch `1.6.0` default / FFT `16384` from the same originals.
Exact-crop every file. Neutral A/B uses `space=0.5`; source, candidate, and
PaulX share one row RMS target under peak `0.95`.

Re-run the `Y09` whole and three-band hard controls. Mapped diagnostics use
`4`-second output windows with `2`-second hops and the sole source map. Omit a
window only when both mapped source channels are below `-60 dBFS` RMS; omit a
final clipped window shorter than `2` seconds. Record complete
candidate-source and PaulX-source balance error and dominance. Missing or
non-finite evidence rejects; finite local error and reversal remain listening
diagnostics.

The operator may reject during speaker pre-screen. Promotion then requires an
eligible independent stereo listener; the operator's one-ear hearing cannot
satisfy the pass. Concealed neutral review covers all `15` candidate/PaulX
rows. Pass requires no unusable candidate row, preferred or tied on at least
`12/15`, and no family losing every ratio.

The listener scores centre stability, width, pumping, one-sided texture,
channel echo, low-frequency image/noise, entry energy, tail energy, and musical
usefulness. Every `space=0/0.5/1` trio must move in the preserve-to-widen
direction without an image jump, unrelated channel motion, low-frequency
pull, or unusable setting. Unavailable or ambiguous independent review blocks
promotion; it is not a pass.

## Rejection, Cleanup, And Minimal Admission

Any construction, structural, synthetic, mono-listening, speaker, or
independent-stereo miss rejects the complete candidate. Record the stopped
gate, every completed row, and one dominant cause. Do not tune, repair, rerun,
or reinterpret the checkpoint.

Delete the worktree, branch, checkpoint reference, private module, tests,
build state, generated receipt, synthetic outputs, and listening assembly.
Retain only the canonical closeout and executable-identity digests. Two
complete candidates failing for the same dominant cause require architecture
reassessment, not a parameter sweep.

Only a complete pass may authorize a later minimal-admission batch. That batch
may admit the private renderer, fixed-ratio neutral-`Dream` request, one
internal creative engine version, and the minimum structural/synthetic
regressions needed to guard it. It must delete candidate-only diagnostics and
assembly surfaces.

Do not admit public character, motion, detail, `space`, seed/reroll, cache,
artifact, report, automatic routing, range blends, pitch, dynamic ratio,
other characters, RealtimePreview, Loophole, or Chorus. Product seed variation
requires a later multi-seed character review.

## Batch 31.43 Candidate Outcome

The one admitted implementation passed compile and construction `1/1`, then
ran all structural owners once. Seventeen passed. `S17` failed because the
candidate materialized full-duration tonal, transient, residual, and spectral
component arrays. Those capacities derive from `L`, contrary to this brief's
bounded monotonic rings and `96 MiB` duration-independent working-state rule.

Checkpoint `1c383679` and tree `cf413de5` were not repaired or rerun.
Synthetic and listening admission did not open. The disposable worktree,
branch, checkpoint reference, source, tests, and worktree-local build state
were deleted. That checkpoint remains historical evidence only.

## Batch 31.44 Bounded-State Reassessment

The complete owner graph is feasible under the frozen memory contract. Every
median, WOLA, detector, segment, covariance, envelope, track, event, and output
consumer has finite geometry-derived lookahead and a monotonic last consumer.
The only non-causal scalar is residual orientation. The mandatory full-source
orientation prepass resolves it without retaining component data or changing
the audible law.

Bounded v2 is fresh authority. It retains all renderer and evidence formulas,
adds the exact two-pass schedule and storage proof above, and uses a new
candidate identity. It is not a repair, retry, or reconstruction of checkpoint
`1c383679`. Batch 31.45 may implement it once in the named disposable worktree.

## Batch 31.45 Candidate Outcome

The isolated implementation completed `effigy test compile`, then construction
stopped at `0/1` before any checkpoint. Exhaustive evaluation of the frozen
first-residual capacity formula produced `53248`, while the frozen construction
row requires `59392`.

The two values cannot both follow the current brief. At `F=192000`,
`N_t=32768`, `N_s=4096`, `A_s=1024`, `R_h=13`, and `h_s=6`, so
`N_t+2(h_s*A_s+N_s)=53248`. The global `R_h=19` maximum occurs separately at
`F=18000`, `N_t=2048`; combining its `h_s=9` with maximum transform geometry
produces the stated `59392` row but is not exhaustive evaluation of one
supported geometry.

No assertion, formula, or capacity was repaired. No immutable checkpoint or
candidate tree exists. Structural, synthetic, and listening admission did not
open. The worktree, branch, private module, tests, and build state were deleted;
no candidate code entered `main`.

## Batch 31.46 Capacity-Authority Reconciliation

Exhaustive integer evaluation over every supported sample rate confirms the
existing per-geometry formula is correct and reaches `53248` first-residual
samples at `F=192000`. The formula, source lookahead, eviction owner, and
`12 MiB` short/source category ceiling remain unchanged.

The conservative short/source packed model becomes `9.700 MiB`. Category
ceilings still total `89 MiB`, leaving the same unassigned `7 MiB` below the
terminal `96 MiB` actual-allocation gate. Every other exhaustive capacity row
matches the frozen construction result.

Capacity-audited v3 supersedes the failed candidate identity and the erroneous
`59392` row only. It does not recover Batch 31.45 source or evidence. Batch
31.47 may implement the complete brief once under the fresh isolation surface
above.

## Batch 31.47 Candidate Outcome

Fresh implementation assembly stopped before compile, construction execution,
or checkpoint. Independent exhaustive evaluation found a second frozen
geometry contradiction:

- at `F=8000`, `N_t=2048`, `N_s=256`, and `A_s=64`
- `round(1800*N_s/F)=round(57.6)=58`
- the frozen nearest-odd midpoint rule chooses the larger odd integer, so
  `R_v=59`
- the same brief requires construction to prove the exhaustive maximum
  `R_v<=57`

Both statements cannot hold. Changing the formula, odd rule, bound, or
construction assertion was outside Batch 31.47's permitted pre-checkpoint
repair class. No construction receipt, checkpoint, candidate tree, structural
result, synthetic result, or quality result exists. The disposable worktree,
branch, private source, and tests were deleted. No candidate code entered
`main`.

This is a frozen-authority failure, not evidence against the linked-STN
renderer. Capacity-audited v3 is no longer implementation-ready. A docs-only
geometry reconciliation must audit every derived median extent before any
fresh candidate identity is considered.

## Batch 31.48 Geometry-Authority Reconciliation

Two independent exact-integer evaluations over every `F=8000..192000`
produce:

| Extent | Exhaustive maximum | First witness |
| --- | ---: | ---: |
| `Q_h` | `17` | `F=16534` |
| `Q_v` | `97` | `F=8000` |
| `R_h` | `19` | `F=17500` |
| `R_v` | `59` | `F=8000` |

At the `R_v` witness, `N_t=2048`, `N_s=256`, and `A_s=64`. Positive
nearest-integer rounding gives `58`; the frozen nearest-odd midpoint rule then
chooses `59`. The corrected bound changes no mask formula or selected
neighbourhood.

Every dependent storage formula was re-evaluated in current geometry. The
maxima remain:

| State | Maximum |
| --- | ---: |
| native long frames | `20` |
| native short frames | `22` |
| shared median-selection scratch | `97` `f64` scalars |
| first-residual samples | `53248` |
| each transient or augmented-residual ring | `147712` samples |
| claimed-event arena | `98816` samples |
| live events | `39` |
| envelope state | `32772` samples |
| output finalization | `139520` samples |
| peak tracks | `16383` |
| persistent bin states | `16385` |

`Q_v=97` already dominates the shared median scratch. Correcting `R_v` from
`57` to `59` therefore changes no ring, packed-memory model, category ceiling,
or cost class. The short/source model remains `9.700 MiB`; category ceilings
remain `89 MiB`; `7 MiB` remains unassigned below the `96 MiB` actual gate.

Geometry-audited v4 supersedes only the contradicted v3 identity and bound. It
does not recover Batch 31.47 source or evidence. One fresh implementation is
ready under the v4 isolation surface above.

## Batch 31.49 Candidate Outcome

The isolated implementation started from `feeb76fe255aa56640de8f732a842942aca936d0`.
Compile passed after permitted pre-checkpoint visibility-only assembly fixes.
Construction passed `1/1`, freezing checkpoint
`e2ef62f81675b5f31426161644b758097485ce0d` and tree
`85dc0e455872957fc829439cbb61fcf54d5719a7`.

Executable identity:

| File | SHA-256 |
| --- | --- |
| `decomposition.rs` | `43049c5a0202e506ba4a8239368ace714288223ae3f75016b432643bd4b143e7` |
| `mod.rs` | `da4ff44fb12f417fa6fc3cff2e92c90720c2d55816f61fe714fff46be84d186a` |
| `noise.rs` | `fbab0a741dac985256c20e8566004b290d5cbecd91a2c6c3d0b98efdd24498bb` |
| `plan.rs` | `3954451f0b44c849532f8c0e48425fdb4121aaf14a967d9f3cde84a91d554115` |
| `synthesis.rs` | `da166e1f44680c4a2c5ea926459451e88de8b9dc5cd855b0949af8b717e47134` |
| `tests.rs` / `EVIDENCE_SPEC` / `MEMORY_SPEC` | `f5fbe0d197c10a8e407744d8bf7d700558c1a5319d457b68855bd40d8bd7a69e` |
| `tonal.rs` | `3796e43a52993f35c7fcea923cff890d837e41d7137426f8e658686ca0538ab5` |
| `transient.rs` | `e244e148f276573a89d7585d6fa191fe16e4cf8e650c42c6998daa88a15a6dbb` |
| `Cargo.lock` | `e3848a40d2ea1ff88a0e036df40d1fefa56c7aca950a95262c1d8c5668fd394d` |

The checkpoint used `rustc 1.96.0 (ac68faa20 2026-05-25)`, host
`aarch64-apple-darwin`, LLVM `22.1.2`, on Darwin `25.5.0` arm64.

Structural admission ran once and finished `17/18`. `S01..S14` and
`S16..S18` passed. `S15` failed because exact-silence input produced
deterministic residual samples on the order of `1e-14`, not bit-exact zero.
No synthetic owner or rendered synthetic output exists, so there are no
synthetic rows or output digests. Listening did not open.

The dominant cause is contradictory residual and boundary ownership. The
residual rule interpolates `ln(a+eps)` and `ln(b+eps)` even when both endpoint
powers are zero; exponentiation therefore creates tiny positive power and
excitation. The same brief requires silence to be bit-exact zero. Repairing or
rerunning the frozen checkpoint is prohibited.

The candidate worktree, branch, checkpoint reference, private source, tests,
build state, receipt, and outputs were deleted. No candidate DSP entered
`main`. Geometry-audited v4 is rejected and no linked-STN candidate is ready.

## Batch 31.50 Exact-Silence Ownership Reconciliation

The v4 formula had no exact-zero residual state: adding `eps` before logarithm
turned two zero powers into positive synthesis power. The final boundary rule
required exact silence but did not freeze how residual descriptors,
coherence, excitation, signed zero, WOLA, and recombination preserved it.

Zero-preserving v5 owns that path completely:

- mono and each stereo diagonal use the sole `zlog` rule above
- zero-power coherence and cross-power are canonical complex positive zero
- inactive mono or mid/side components receive zero spectra directly
- duplicate, common-negation, anti-phase, and swap laws include zero states
- mapped zero envelope contributes no bed sample
- exact-zero accumulation converts to positive `0.0f32`
- every positive or mixed zero/positive power row keeps the v4 formula
- no threshold, denoiser, silence fast path, extra state, stochastic change,
  post-render repair, variable traversal, or quality-gate relaxation enters

The audit changes no capacity. Exact zero is derived from stored covariance
values and needs no mask or duration state. The `89 MiB` design ceiling,
`96 MiB` actual gate, two-pass schedule, fixed transform count, deterministic
order, and cost expression remain unchanged.

The strengthened `S12`, `S13`, `S15`, `S16`, and `S18` owners prove the new
rule without increasing the `18` structural or `10` synthetic owner count.
All synthetic sources, thresholds, PaulX comparator rows, concealed mono pack,
independent stereo pack, stop order, receipt identity, cleanup, and minimal
admission boundary remain unchanged.

This is fresh complete authority, not a repair or revival of checkpoint
`e2ef62f8`. `ZeroPreservingGeometryAuditedBoundedLinkedStnNoiseMorph` may be
implemented once from the canonical brief in the v5 worktree and branch. No
candidate DSP or product surface entered `main` during reconciliation.

## Batch 31.51 Candidate Outcome

The isolated implementation started from
`570da1604ba21204c1dccfb3aed6d2980ed239ac`. Compile passed without repair.
Construction passed `1/1`, freezing checkpoint
`959094513b6847cdeb8a3c0bf424efd09ce1fb6f` and tree
`080bea3698e5b70760edd6b38dcccb995697d2c2`.

Executable identity:

| File | SHA-256 |
| --- | --- |
| `decomposition.rs` | `2ae62457724047b32f40d93a25ab24d80779c5e9a30f89f4f52e8897e853b3a5` |
| `mod.rs` | `2cb89b0b01596c79b3f6cd79ac7569823a3c199c16b2d3b0c1287030a61a0389` |
| `noise.rs` | `47d5af7f3321aac33332fc5f441ad3bc145d4d405c089a6b165cee5292b49eb6` |
| `plan.rs` | `3ca6d28e4f12acd743e778218d7cac9b4446b1f2278eb071c00b0280bcdb1e29` |
| `synthesis.rs` | `d0ab6bc200b7d76c474c37c69024bb0fc08898888614e1f7f10237e435b3c138` |
| `tests.rs` / `EVIDENCE_SPEC` / `MEMORY_SPEC` | `28fc64c6c48040404df446689dda13e9daf3cf062dd19a56bb688d22f627b197` |
| `tonal.rs` | `b003284c0d4e47c7dfdfac7c13fcec9f1904bf6be07d16a4e136db41df3c06c7` |
| `transient.rs` | `bbfd17bfc50b87cbdbae8b6ada2112b5dcc8113f107852f0713e8c484a193410` |
| `Cargo.lock` | `e3848a40d2ea1ff88a0e036df40d1fefa56c7aca950a95262c1d8c5668fd394d` |

The checkpoint used `rustc 1.96.0 (ac68faa20 2026-05-25)`, host
`aarch64-apple-darwin`, LLVM `22.1.2`, on Darwin `25.5.0` arm64.

Structural admission ran once and finished `17/18`. `S01` and `S03..S18`
passed. `S02` failed its handwritten `F=8000` geometry vector: the checkpoint
asserted `Q_h=5`, while the frozen formula gives
`odd(round(0.240*8000/256))=odd(8)=9`. The renderer computed `9` correctly.
No synthetic owner or rendered synthetic output exists, so there are no
synthetic rows or output digests. Listening did not open.

The dominant cause is incomplete executable geometry authority. Construction
proved only exhaustive maxima and did not independently cross-check the
per-rate structural vector. Repairing or rerunning the frozen checkpoint is
prohibited.

The candidate worktree, branch, checkpoint reference, private source, tests,
build state, receipt, and outputs were deleted. No candidate DSP entered
`main`. Zero-preserving v5 is rejected and no linked-STN candidate is ready.

## Batch 31.52 Geometry-Vector Authority Audit

Two independent exact-integer evaluators reproduced every geometry row for all
`184001` supported integer sample rates. They agree on the complete binary
table SHA-256, FNV-1a fingerprint, transform transitions, positive-round tie
sets, upward odd counts, extent maxima, first witnesses, and every
geometry-derived `MEMORY_SPEC` maximum and first/last witness.

The incorrect Batch 31.51 8 kHz vector is replaced by the sole sentinel table
above: `Q_h=9`, not `5`. No other frozen geometry or capacity contradiction
was found. The formulas, renderer topology, exact-zero path, acoustic gates,
memory ceilings, and gate order remain unchanged.

Construction-bound v6 makes that audit executable. `GEOMETRY_SPEC` is the only
literal geometry table. A separately coded oracle checks the renderer across
the complete rate domain. Construction and `S02` call the same authority
assertion, so structural admission cannot introduce a different handwritten
row after checkpoint.

This is fresh complete authority, not a repair or revival of checkpoint
`95909451`. Batch 31.53 may implement it once in the named disposable worktree
and branch. No candidate DSP, test, harness, dependency, API, route, product,
Loophole, or Chorus surface entered `main` during Batch 31.52.

## Sources

- [Creative source triangulation](../research/specimen-dossiers/creative-stretch-source-triangulation.md)
- [Creative stretch study](./offline-creative-time-stretch-study.md)
- [Creative product contract](../contracts/085-creative-time-stretch-product-and-routing-contract.md)
- [SiTraNoStar pinned source](https://github.com/ollpu/SiTraNoStar/tree/2edf7b693040b5070116299973abf83dc5ba86e5)
- [Enhanced fuzzy STN decomposition](https://arxiv.org/abs/2210.14041)
- [Noise Morphing](https://arxiv.org/abs/2312.14586)
- [Extreme Audio Time Stretching Using Neural Synthesis](https://arxiv.org/abs/2211.16992)

## Next Task

Run Batch 31.53 only in the fresh construction-bound v6 worktree and branch.
Implement the complete private renderer once, run compile and construction,
freeze one immutable checkpoint only after construction passes `1/1`, then run
structural and synthetic admission in order. Stop, delete, and close on the
first miss. Do not repair Batch 31.51, change production code, merge, or push.
