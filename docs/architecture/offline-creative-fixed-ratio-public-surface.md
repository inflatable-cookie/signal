# Offline Creative Public Surface

Status: public v3 admitted; continuous Cyclic v4 frozen
Owner: core-product
Updated: 2026-07-25
Contract: `085`
Roadmaps: `g10.031`, Batches 31.75-31.76; `g10.032`, Batches 32.27-32.28;
`g10.033`, Batches 33.4-33.5; `g10.034`, Batches 34.4-34.5

## Decision

Expose the admitted exact-ratio neutral `Dream` effect through one small
Signal-owned offline API. Keep `DirectRenewalDream` an internal renderer name.
Do not expose automatic routing or unavailable creative controls.

Extend the same API with the admitted fixed-ratio `Cyclic` effect. Keep the
centred compressed-anchor event-ledger renderer internal. Expose one semantic
cycle duration, exact character-specific ratios, and no other Cyclic control.

This is a public library boundary, not a Loophole or Chorus integration plan.
It does not reopen `g10.025`.

## Continuous Dream Decision

Batch 33.4 widens public `Dream` to every exact target satisfying
`4L <= T <= 16L`. `L` is source frames and `T` is authoritative target
frames. Every integer `T` in that closed interval is valid. Ratios need not be
integral, power-of-two, hop-divisible, or named by a floating value.

One private `ContinuousDirectRenewalDream` owner already covers the complete
interval with one map, scheduler, boundary law, linked-stereo law, and
deterministic state. Public Dream therefore dispatches directly to that owner.
It has no range branch, hidden router, overlap, blend, or fallback.

`Cyclic` remains a separate explicit character at exact `2x`, `4x`, and `8x`.
The shared `4x` and `8x` targets do not route or crossfade between characters.
`OfflineHighQuality` remains a separate Transparent product choice.

## Continuous Cyclic Decision

Batch 34.4 freezes public `Cyclic` widening to every exact target satisfying
`2L <= T <= 8L`. Every integer target frame count in that closed interval is
valid. The target need not be an integral ratio, named anchor, cycle multiple,
or power of two.

The admitted private `ContinuousEventLedgerCyclic` owner already covers the
complete interval with one map, schedule, cycle law, boundary law,
linked-channel geometry, and deterministic state. Public Cyclic therefore
dispatches directly to private `render_continuous`. It has no range branch,
same-character router, overlap, blend, fallback, or transition identity.

Exact `2x`, `4x`, and `8x` remain mandatory byte-parity anchors. The direct
`5..90 ms` cycle control remains the sole Cyclic control and retains its
`48 ms` default. Short values move toward metallic/ring motion; long values
move toward tremolo/echo motion.

Dream stays unchanged over `4L..=16L`. At shared targets, Dream and Cyclic
remain explicit, distinct user choices. No automatic character selection is
authorized.

## Executable Coverage

Let `N` be source frames and `T` the requested output frames. This is the
complete public creative execution matrix after Batch 33.5:

| Character | Accepted target | Channels | Public control | Execution |
| --- | --- | --- | --- | --- |
| `Dream` | every integer `T` with `4N <= T <= 16N` | mono or linked stereo | `space` in `0..=1`, default `0.5` | deterministic whole-buffer offline |
| `Cyclic` | exactly `T=2N`, `T=4N`, or `T=8N` | mono or linked stereo | `cycle` in `5..=90 ms`, default `48 ms` | deterministic whole-buffer offline |

The matrix is target-frame exact. `Dream` is continuous over integer output
lengths, not merely floating-point ratios. `Cyclic` is not continuous. Both
characters reject unsupported targets before render allocation and never
substitute the other character or Transparent.

This remains the executable v3 matrix until Batch 34.5 implements the frozen
v4 Cyclic range.

Adjacent Signal stretch owners are not hidden creative range owners:

| Owner | Executable posture | Boundary |
| --- | --- | --- |
| `Repitch` | implemented realtime-safe varispeed with dynamic ratio | tempo and pitch remain coupled |
| `RealtimePreview` | control-side prototype with static, pitch, and stepwise dynamic-ratio entry points | `audio_thread_processing_supported=false`; no callback product path |
| `OfflineHighQuality` | deterministic mono and linked-stereo Transparent renderer; positive finite static ratios and stepwise dynamic ratios are accepted | callable breadth is not a blanket acoustic promotion; it never serves as creative fallback |
| `CreativeStretch` | the Dream and Cyclic rows above | whole-buffer offline only; separate from the backend-tier enum |

Current non-coverage is explicit:

- Dream below `4x` or above `16x`
- Cyclic interior targets between `2x`, `4x`, and `8x`
- creative dynamic ratio, character automation, reverse, or pitch
- automatic same-character or cross-character routing
- creative cache identity, artifact writing, runtime DTOs, UI integration,
  Loophole, or Chorus integration

The quality statement is narrower than raw API acceptance.
`OfflineHighQuality` remains the frozen competitive Transparent baseline, with
the retained listening program concentrated on compression through long
expansion. Dream `4x..16x` and Cyclic `2x`/`4x`/`8x` carry their own completed
creative admission. No row claims universal professional parity from metrics
alone.

## Frozen Public V4 Shape

Batch 34.5 may admit only this shape:

```rust
pub const CREATIVE_STRETCH_ENGINE_VERSION: &str = "signal-creative-stretch-v4";
pub const CREATIVE_STRETCH_DREAM_MIN_RATIO: usize = 4;
pub const CREATIVE_STRETCH_DREAM_MAX_RATIO: usize = 16;
pub const CREATIVE_STRETCH_CYCLIC_MIN_RATIO: usize = 2;
pub const CREATIVE_STRETCH_CYCLIC_MAX_RATIO: usize = 8;

#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreativeStretchRatioDomain {
    Continuous {
        minimum: usize,
        maximum: usize,
    },
    Exact(&'static [usize]),
}

impl CreativeStretchCharacter {
    pub const fn ratio_domain(self) -> CreativeStretchRatioDomain;
}
```

`Dream::ratio_domain()` remains
`Continuous { minimum: 4, maximum: 16 }`. `Cyclic::ratio_domain()` becomes
`Continuous { minimum: 2, maximum: 8 }`.

Remove `CREATIVE_STRETCH_CYCLIC_SUPPORTED_RATIOS`. It becomes false after
widening. Do not retain a compatibility alias or reinterpret `[2, 4, 8]` as
recommendations. `CreativeStretchRatioDomain::Exact` remains valid public
domain vocabulary but is returned by no current character.

The request, character enum, controls, render entry, and error variants remain
unchanged. Public behavior identity becomes `signal-creative-stretch-v4`
because discovery and the accepted Cyclic request set change. Private renderer
identity remains internal.

## Continuous Cyclic Request Contract

- preserve the existing validation order through controls and empty/target
  relationship
- reject source or target frames above `2^53-1` as `SizeOverflow`
- compute `minimum=checked_mul(L,2)` and `maximum=checked_mul(L,8)`
- accept Cyclic exactly when `minimum <= T <= maximum`
- reject Cyclic outside that interval as `UnsupportedTargetFrames`
- reject arithmetic overflow as `SizeOverflow`
- validate the complete public domain before render dispatch or output
  allocation
- call private `render_continuous` once with the exact target and canonical
  cycle
- preserve empty-input plus zero-target success
- never clamp, round, route, crossfade, or fall back

The public wrapper repeats the checked domain decision for stable public error
precedence. The private continuous owner remains final geometry authority.

## Batch 34.5 Gate

Only `creative.rs` and `lib.rs` may change.

Required focused ownership:

1. v4 constants, exports, rustdoc, and ratio discovery match the frozen shape
2. validation accepts every target in `2L..=8L` for source lengths
   `1`, `2`, `3`, and `257`
3. `2L-1` and `8L+1` reject as `UnsupportedTargetFrames` before dispatch or
   output allocation
4. public mono and stereo match private `render_continuous` byte-for-byte at
   `2x`, `2x+1 frame`, `2.5x`, `3x`, `4x-1 frame`, `4x`, `4x+1 frame`,
   `5x`, `6x`, `7.5x`, `8x-1 frame`, and `8x`
5. parity covers explicit `5 ms`, default `48 ms`, and explicit `90 ms`
6. exact `2x`, `4x`, and `8x` public output remains byte-identical
7. Dream discovery, validation, output, `space`, seed, and errors remain
   unchanged
8. Cyclic cycle canonicalization, control ownership, error mapping,
   determinism, exact length, finiteness, and linked-stereo relations remain
   unchanged
9. every private Dream and Cyclic file remains byte-identical
10. no router, cache, artifact, dynamic ratio, runtime, UI, Loophole, or
    Chorus surface changes

No listening rerun is required. The public wrapper exposes output already
admitted from the unchanged private owner. Any private-file or parity
difference stops the batch.

## Admitted Public V3 Shape

Batch 33.5 admits this shape:

```rust
pub const CREATIVE_STRETCH_ENGINE_VERSION: &str = "signal-creative-stretch-v3";
pub const CREATIVE_STRETCH_DREAM_MIN_RATIO: usize = 4;
pub const CREATIVE_STRETCH_DREAM_MAX_RATIO: usize = 16;
pub const CREATIVE_STRETCH_CYCLIC_SUPPORTED_RATIOS: [usize; 3] = [2, 4, 8];

#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreativeStretchRatioDomain {
    Continuous {
        minimum: usize,
        maximum: usize,
    },
    Exact(&'static [usize]),
}

impl CreativeStretchCharacter {
    pub const fn ratio_domain(self) -> CreativeStretchRatioDomain;
}
```

`Dream::ratio_domain()` returns `Continuous { minimum: 4, maximum: 16 }`.
`Cyclic::ratio_domain()` returns
`Exact(&CREATIVE_STRETCH_CYCLIC_SUPPORTED_RATIOS)`.

Remove `CREATIVE_STRETCH_SUPPORTED_RATIOS` and
`CreativeStretchCharacter::supported_ratios()`. They describe Dream as a
discrete list and become false after widening. Do not retain a compatibility
alias or reinterpret the three old values as recommendations.

The request, character enum, controls, render entry, and error variants remain
unchanged. Update their documentation from exact-ratio wording to
character-specific target-domain wording.

The public behavior version became `signal-creative-stretch-v3` because the
accepted request set and discovery surface change. Renderer-specific identity
stays internal. This does not admit a creative cache schema.

## Continuous Dream Request Contract

- validate channels, framing, sample rate, input finiteness, controls, and
  empty/target relationship in the existing order
- reject source or target frames above `2^53-1` as `SizeOverflow`
- compute `minimum=checked_mul(L,4)` and `maximum=checked_mul(L,16)`
- accept Dream exactly when `minimum <= T <= maximum`
- reject Dream outside that interval as `UnsupportedTargetFrames`
- retain exact Cyclic multiplication against `2`, `4`, and `8`
- reject before output allocation; never clamp, round, route, or fall back
- preserve empty-input plus zero-target success
- preserve exact target length, finite output, deterministic repeat, and
  admitted-seed ownership

The public wrapper performs the same checked domain decision as the private
Dream owner. The private owner remains the final geometry authority.

## Batch 33.5 Gate

Only `creative.rs` and `lib.rs` may change.

Required focused ownership:

1. public ratio-domain constants and introspection match the frozen v3 shape
2. preallocation validation accepts every target in `4L..=16L` for
   representative small `L`, and rejects `4L-1` and `16L+1`
3. public mono and stereo match the private Dream renderer byte-for-byte at
   `4x`, `4x+1 frame`, `4.5x`, `6x`, `10x`, `15.5x`, `16x-1 frame`, and
   `16x`
4. existing `4x`, `8x`, and `16x` public output remains byte-identical
5. every Dream `space` value already admitted passes unchanged
6. Cyclic ratios, duration canonicalization, output, errors, and discovery
   remain unchanged
7. unsupported targets fail before render dispatch or output allocation
8. all private Dream acoustic files and private Cyclic files remain
   byte-identical
9. the admitted private continuous structural and synthetic owners remain
   green
10. no router, cache, artifact, dynamic ratio, runtime, UI, Loophole, or
    Chorus surface changes

No listening rerun is required. The public wrapper adds no acoustic behavior.
Any private-file or output difference stops the batch.

## Admitted V2 Shape

The Batch 32.28 public shape is:

```rust
use std::time::Duration;

pub const CREATIVE_STRETCH_ENGINE_VERSION: &str = "signal-creative-stretch-v2";
pub const CREATIVE_STRETCH_SUPPORTED_RATIOS: [usize; 3] = [4, 8, 16];
pub const CREATIVE_STRETCH_CYCLIC_SUPPORTED_RATIOS: [usize; 3] = [2, 4, 8];
pub const CREATIVE_STRETCH_DEFAULT_SPACE: f32 = 0.5;
pub const CREATIVE_STRETCH_MIN_CYCLE: Duration = Duration::from_millis(5);
pub const CREATIVE_STRETCH_DEFAULT_CYCLE: Duration = Duration::from_millis(48);
pub const CREATIVE_STRETCH_MAX_CYCLE: Duration = Duration::from_millis(90);

#[non_exhaustive]
pub enum CreativeStretchCharacter {
    Dream,
    Cyclic,
}

impl CreativeStretchCharacter {
    pub const fn supported_ratios(self) -> &'static [usize];
}

#[non_exhaustive]
pub struct CreativeStretchRequest<'a> {
    pub input: &'a [Sample],
    pub channels: u16,
    pub sample_rate: SampleRate,
    pub target_frames: usize,
    pub character: CreativeStretchCharacter,
    pub space: f32,
    pub cycle: Option<Duration>,
}

impl<'a> CreativeStretchRequest<'a> {
    pub fn new(
        input: &'a [Sample],
        channels: u16,
        sample_rate: SampleRate,
        target_frames: usize,
        character: CreativeStretchCharacter,
    ) -> Self;

    pub fn with_space(self, space: f32) -> Self;
    pub fn with_cycle(self, cycle: Duration) -> Self;
}

#[non_exhaustive]
pub enum CreativeStretchError {
    InvalidChannelCount,
    UnsupportedSampleRate,
    PartialFrame,
    NonFiniteInput,
    InvalidSpace,
    InvalidCycle,
    UnsupportedCharacterControl,
    EmptyInput,
    ZeroTargetFrames,
    UnsupportedTargetFrames,
    SizeOverflow,
    AllocationFailed,
    NonFiniteOutput,
}

pub fn render_creative_stretch(
    request: CreativeStretchRequest<'_>,
) -> Result<Vec<Sample>, CreativeStretchError>;
```

The enums and request derive the ordinary `Clone`, `Copy`, `Debug`, and
equality traits supported by their fields. Request fields stay readable.
`new` sets `space` to `CREATIVE_STRETCH_DEFAULT_SPACE` and `cycle` to `None`.
`with_cycle` stores `Some(cycle)`.

`CREATIVE_STRETCH_ENGINE_VERSION` is the semantic public behavior version.
Batch 32.28 changes it from `signal-creative-stretch-v1` to
`signal-creative-stretch-v2` because character dispatch and request semantics
change. Dream output remains byte-identical. Renderer-specific identities
remain internal. The public version does not authorize use of the transparent
cache schema.

The existing `CREATIVE_STRETCH_SUPPORTED_RATIOS` remains the Dream ratio list.
`CREATIVE_STRETCH_CYCLIC_SUPPORTED_RATIOS` owns Cyclic. Callers that branch on
character use `CreativeStretchCharacter::supported_ratios()` rather than
forming a union. Exact `4x` and `8x` belong to both characters but produce
deliberately different effects.

## Public Request Contract

- `input` is finite mono or interleaved stereo `Sample`
- `channels` is exactly `1` or `2`
- `sample_rate` is `8000..=192000`
- `target_frames` is authoritative
- source frame count is `input.len()/channels`
- Dream target frames satisfy `4 * source_frames <= target_frames <=
  16 * source_frames`
- Cyclic target frames equal source frames times `2`, `4`, or `8`
- Dream requires `cycle=None` and finite `space` in `[0,1]`
- Cyclic requires `space` to remain bit-exact
  `CREATIVE_STRETCH_DEFAULT_SPACE`
- Cyclic `cycle=None` resolves to `CREATIVE_STRETCH_DEFAULT_CYCLE`
- Cyclic `cycle=Some` is within the inclusive
  `CREATIVE_STRETCH_MIN_CYCLE..=CREATIVE_STRETCH_MAX_CYCLE` range
- empty input with zero target returns empty; every other invalid combination
  returns the matching public error
- output preserves channel count, contains exactly `target_frames`, and is
  finite

No value is clamped. Unsupported targets return
`UnsupportedTargetFrames`; they never fall back to `OfflineHighQuality`.

Dream always uses the admitted `ADMISSION_SEED`. A caller cannot select or
reroll it. Cyclic has no stochastic state. Both characters are
byte-deterministic for one complete request.

### Cycle Canonicalization

`cycle` is a semantic duration, not a grain or window control. The wrapper
converts its integer nanoseconds to the nearest microsecond with integer
round-half-up:

```text
effective_cycle_us = (cycle.as_nanos() + 500) / 1000
```

The raw duration must first be inside the inclusive `5..90 ms` public range.
The canonical microsecond value is then passed unchanged to the admitted
private renderer. The default resolves to `48,000 us`. Future cache identity
uses this effective microsecond value, so sub-microsecond requests that round
to the same value share behavior identity.

### Error Precedence

The wrapper validates without allocation in this order:

1. channel count and partial interleaved frame
2. sample rate and finite input
3. character-control ownership
4. active control range
5. empty/target relationship
6. character-specific target domain and checked size

For Dream, any `cycle=Some` returns `UnsupportedCharacterControl`. For Cyclic,
any `space` value other than the default bit pattern returns
`UnsupportedCharacterControl`, including non-finite values. An owned but
out-of-range control returns `InvalidSpace` or `InvalidCycle`.

The Cyclic private errors map exactly:

| Private error | Public error |
| --- | --- |
| `InvalidChannels` | `InvalidChannelCount` |
| `PartialFrame` | `PartialFrame` |
| `InvalidSampleRate` | `UnsupportedSampleRate` |
| `NonFiniteInput` | `NonFiniteInput` |
| `InvalidCycle` | `InvalidCycle` |
| `InvalidEmptyTarget` | `EmptyInput` or `ZeroTargetFrames` from request geometry |
| `UnsupportedCompression`, `UnsupportedRatio` | `UnsupportedTargetFrames` |
| `ExactIntegerLimit`, `ArithmeticOverflow`, `AllocationOverflow` | `SizeOverflow` |

`AllocationFailed` and `NonFiniteOutput` remain shared public outcomes owned
by the Dream renderer. The Cyclic wrapper adds no post-render gain, fallback,
or output mutation.

## UI Mapping

The honest consuming UI is small:

| Intent | Public meaning |
| --- | --- |
| mode | explicit `Creative` choice separate from `Transparent` |
| duration | Dream: continuous `400%..=1600%`; Cyclic v4: continuous `200%..=800%` |
| character | explicit `Dream` or `Cyclic` |
| space | Dream only; optional preserve-to-widen control, normalized `0..1`, default `0.5` |
| cycle | Cyclic only; `5..90 ms`, default `48 ms`; short is metallic/ring-like, long is tremolo/echo-like |

Show only the control valid for the selected character. Do not show
`motion`, `detail`, seed/reroll, algorithm, FFT, window, grain, phase,
routing, pitch, reverse, or dynamic-ratio controls.

## Execution Boundary

`render_creative_stretch` is whole-buffer, allocating, deterministic offline
work. It must not run on the audio thread. It returns the renderer output
directly; no resampling, second stretch pass, limiter, level correction, or
post-render fade is added.

The Batch 31.76 Dream wrapper mapped public request and error vocabulary onto
its admitted private renderer. Batch 32.28 added Cyclic. Batch 33.5 widened
Dream. Batch 34.5 may change only:

- `creative.rs` public types, validation, dispatch, error mapping, and focused
  tests
- `lib.rs` rustdoc and re-export wiring

The admitted Dream acoustic files `analysis.rs`, `plan.rs`, `stereo.rs`, and
`synthesis.rs` remain byte-identical. The admitted Cyclic files `mod.rs`,
`plan.rs`, `schedule.rs`, `interpolate.rs`, `synthesis.rs`, and `tests.rs`
also remain byte-identical.

## Cache And Routing Boundary

Do not add `Creative` to `StretchBackendTier`, `StretchQuality`, or
`OfflineHighQualityPath`. `CreativeStretch` is a peer product choice, not a
transparent backend selector.

Do not use `StretchCacheIdentityInput` for creative output. Its current schema
does not own character, character-valid controls, fixed Dream seed, or creative
engine version without collision risk. Cache admission requires a later
contract and implementation batch.

A future creative cache key must include `signal-creative-stretch-v4`,
character, exact target frames, and the active character control: `space` bits
for Dream or effective `cycle_us` for Cyclic. It must not include the inactive
control. This freezes identity semantics only; it does not admit caching.

No automatic route, overlap, fallback, dynamic ratio, pitch, artifact writer,
promotion receipt, runtime DTO, Loophole, or Chorus surface enters this batch.

## Batch 31.76 Gate

Implementation passes only when:

1. only `creative.rs`, `lib.rs`, and
   `creative_direct_renewal_dream/mod.rs` change
2. all new public items carry complete rustdoc and pass missing-doc checks
3. public mono and stereo output matches the private renderer byte-for-byte at
   `4x`, `8x`, and `16x`
4. `space=0`, `0.5`, and `1` map without alteration
5. the public wrapper uses the exact admitted seed
6. every private error maps to the frozen public error
7. exact length, finiteness, deterministic repeat, empty success, and invalid
   request behavior pass through the public entry
8. the four acoustic files remain byte-identical
9. existing construction, structural `10/10`, and synthetic `88/88` with
   `76/76` renders remain green
10. no cache, route, tier, dynamic-ratio, report, fixture, product-integration,
    Loophole, or Chorus surface changes

No new listening round is required for a byte-identical wrapper using the
admitted seed and control domain. Any output difference stops the batch.

## Batch 31.76 Result

The frozen wrapper is public through `signal-dsp-stretch`. Public and private
mono/stereo renders are byte-identical at `4x`, `8x`, and `16x`; `space=0`,
`0.5`, and `1` pass unchanged. All frozen errors map without clamping or
fallback.

The four acoustic files retain their Batch 31.75 hashes. Integrated
construction `1/1`, structural `10/10`, and synthetic `88/88` with `76/76`
renders remain green. No cache, route, tier, dynamic-ratio, report, fixture,
runtime, Loophole, Chorus, or cross-repo surface changed.

## Batch 32.28 Gate

Implementation passes only when:

1. only `creative.rs` and `lib.rs` change
2. all new public items carry complete rustdoc and pass missing-doc checks
3. existing Dream output remains byte-identical at every admitted ratio and
   `space=0`, `0.5`, and `1`
4. public Cyclic mono and stereo output matches the private renderer
   byte-for-byte at `2x`, `4x`, and `8x`
5. Cyclic parity covers explicit `5 ms`, default `48 ms`, and explicit
   `90 ms`
6. absent Cyclic cycle resolves exactly to `48,000 us`
7. duration canonicalization uses the frozen integer rule and never floating
   conversion
8. wrong-character controls, invalid active controls, unsupported ratios,
   empty/target combinations, and size failures return the frozen public error
9. validation and exact `16x` Cyclic rejection occur before output allocation
10. exact length, finiteness, deterministic repeat, linked-stereo algebra, and
    no-fallback behavior pass through the public entry
11. both sets of acoustic files remain byte-identical
12. no cache, route, tier, dynamic-ratio, report, fixture, product integration,
    Loophole, or Chorus surface changes

No listening rerun is required. The wrapper must be byte-identical to the
accepted private renderer for every exposed ratio and the three reviewed cycle
anchors. Any acoustic-file or output difference stops the batch.

## Batch 32.28 Result

Commit `e8948512` admits the frozen Cyclic extension through
`signal-dsp-stretch`. Only `creative.rs` and `lib.rs` changed.

Focused public tests pass `10/10`:

- Dream mono and stereo remain byte-identical to the private renderer
- Cyclic mono and stereo match the private renderer byte-for-byte at exact
  `2x`, `4x`, and `8x`
- explicit `5 ms`, default `48 ms`, and explicit `90 ms` match unchanged
- sub-microsecond cycle values follow integer round-half-up
- duplicate and anti-phase stereo relations pass through
- wrong-character controls, invalid cycles, unsupported ratios, and Cyclic
  `16x` return the frozen errors before render dispatch
- both characters remain deterministic; empty/zero succeeds
- both private renderer trees remain byte-identical

Missing-doc checks, focused nextest, Effigy health, and Effigy validation pass.
No cache, route, tier, artifact, runtime, Loophole, or Chorus surface changed.

## Batch 33.5 Result

The public surface now accepts every Dream target in `4L..=16L` and reports
that interval through `CreativeStretchRatioDomain::Continuous`. Cyclic still
reports `Exact(&[2, 4, 8])`. Public behavior identity is
`signal-creative-stretch-v3`.

Only `creative.rs` and `lib.rs` changed. The public wrapper still dispatches
Dream directly to the one private owner. No router, blend, fallback, or
private DSP change entered the batch.

Focused public tests pass `11/11`:

- complete validation domains for source lengths `1`, `2`, `3`, and `257`
- rejection immediately below `4L` and above `16L`
- byte-exact public/private mono and stereo parity at `4x`, `4x+1 frame`,
  `4.5x`, `6x`, `8x`, `10x`, `15.5x`, `16x-1 frame`, and `16x`
- unchanged Dream `space`, Cyclic ratios, Cyclic cycle anchors, control
  ownership, error mapping, determinism, and empty success

All `18` retained private Dream construction, structural, synthetic, and
continuous owners pass. Both private renderer trees remain byte-identical.
No listening rerun was required because public output matches the admitted
private renderer exactly.

## Next Task

Execute `g10.034` Batch 34.5 only. Admit the frozen v4 public Cyclic range in
`creative.rs` and `lib.rs`, run the focused parity gate, and keep every private
renderer, integration, and inactive product surface unchanged.
