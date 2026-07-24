# Offline Creative Fixed-Ratio Public Surface

Status: Dream and Cyclic exact-ratio characters admitted
Owner: core-product
Updated: 2026-07-24
Contract: `085`
Roadmaps: `g10.031`, Batches 31.75-31.76; `g10.032`, Batches 32.27-32.28

## Decision

Expose the admitted exact-ratio neutral `Dream` effect through one small
Signal-owned offline API. Keep `DirectRenewalDream` an internal renderer name.
Do not expose automatic routing or unavailable creative controls.

Extend the same API with the admitted fixed-ratio `Cyclic` effect. Keep the
centred compressed-anchor event-ledger renderer internal. Expose one semantic
cycle duration, exact character-specific ratios, and no other Cyclic control.

This is a public library boundary, not a Loophole or Chorus integration plan.
It does not reopen `g10.025`.

## Public Shape

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

## Request Contract

- `input` is finite mono or interleaved stereo `Sample`
- `channels` is exactly `1` or `2`
- `sample_rate` is `8000..=192000`
- `target_frames` is authoritative
- source frame count is `input.len()/channels`
- Dream target frames equal source frames times `4`, `8`, or `16`
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
6. exact character-specific ratio and checked size

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
| duration | Dream: exact `400%`, `800%`, or `1600%`; Cyclic: exact `200%`, `400%`, or `800%` |
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
its admitted private renderer. Batch 32.28 may change only:

- `creative.rs` public types, validation, dispatch, error mapping, and focused
  tests
- `lib.rs` rustdoc and re-export wiring

The admitted Dream acoustic files `analysis.rs`, `plan.rs`, `stereo.rs`, and
`synthesis.rs` remain byte-identical. The admitted Cyclic acoustic files
`plan.rs`, `schedule.rs`, `interpolate.rs`, and `synthesis.rs` also remain
byte-identical. `creative_cyclic/mod.rs` and its production tests do not
change.

## Cache And Routing Boundary

Do not add `Creative` to `StretchBackendTier`, `StretchQuality`, or
`OfflineHighQualityPath`. `CreativeStretch` is a peer product choice, not a
transparent backend selector.

Do not use `StretchCacheIdentityInput` for creative output. Its current schema
does not own character, character-valid controls, fixed Dream seed, or creative
engine version without collision risk. Cache admission requires a later
contract and implementation batch.

A future creative cache key must include `signal-creative-stretch-v2`,
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

## Next Task

Keep this exact-ratio surface frozen. Execute `g10.033` Batch 33.2 only. Freeze
the complete `ContinuousDirectRenewalDream` candidate brief without starting
implementation or widening the public API.
