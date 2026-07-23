# Offline Creative Fixed-Ratio Public Surface

Status: admitted; Batch 31.76 complete
Owner: core-product
Updated: 2026-07-23
Contract: `085`
Roadmap: `g10.031`, Batches 31.75-31.76

## Decision

Expose the admitted exact-ratio neutral `Dream` effect through one small
Signal-owned offline API. Keep `DirectRenewalDream` an internal renderer name.
Do not expose automatic routing or unavailable creative controls.

This is a public library boundary, not a Loophole or Chorus integration plan.
It does not reopen `g10.025`.

## Public Shape

Add these public items to `signal-dsp-stretch`:

```rust
pub const CREATIVE_STRETCH_ENGINE_VERSION: &str = "signal-creative-stretch-v1";
pub const CREATIVE_STRETCH_SUPPORTED_RATIOS: [usize; 3] = [4, 8, 16];
pub const CREATIVE_STRETCH_DEFAULT_SPACE: f32 = 0.5;

#[non_exhaustive]
pub enum CreativeStretchCharacter {
    Dream,
}

#[non_exhaustive]
pub struct CreativeStretchRequest<'a> {
    pub input: &'a [Sample],
    pub channels: u16,
    pub sample_rate: SampleRate,
    pub target_frames: usize,
    pub character: CreativeStretchCharacter,
    pub space: f32,
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
}

#[non_exhaustive]
pub enum CreativeStretchError {
    InvalidChannelCount,
    UnsupportedSampleRate,
    PartialFrame,
    NonFiniteInput,
    InvalidSpace,
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
`new` sets `space` to `CREATIVE_STRETCH_DEFAULT_SPACE`.

`CREATIVE_STRETCH_ENGINE_VERSION` is the semantic public behavior version. The
renderer-specific `signal-creative-direct-renewal-dream-v1` identity remains
internal. The public version does not authorize use of the transparent cache
schema.

## Request Contract

- `input` is finite mono or interleaved stereo `Sample`
- `channels` is exactly `1` or `2`
- `sample_rate` is `8000..=192000`
- `target_frames` is authoritative
- source frame count is `input.len()/channels`
- checked `target_frames` must equal source frames times `4`, `8`, or `16`
- `character` is currently only `Dream`
- `space` is finite in `[0,1]`
- empty input with zero target returns empty; every other invalid combination
  returns the matching public error
- output preserves channel count, contains exactly `target_frames`, and is
  finite

No value is clamped. Unsupported targets return
`UnsupportedTargetFrames`; they never fall back to `OfflineHighQuality`.

The public wrapper always uses the admitted `ADMISSION_SEED`. A caller cannot
select or reroll seed. This preserves byte-deterministic output without
claiming unreviewed multi-seed character quality.

## UI Mapping

The honest consuming UI is small:

| Intent | Public meaning |
| --- | --- |
| mode | explicit `Creative` choice separate from `Transparent` |
| duration | exact `400%`, `800%`, or `1600%`, or the equivalent target duration |
| character | fixed `Dream`; hide the selector while it is the only value |
| space | optional preserve-to-widen control, normalized `0..1`, default `0.5` |

Do not show `motion`, `detail`, seed/reroll, algorithm, FFT, window, grain,
phase, routing, pitch, reverse, or dynamic-ratio controls.

## Execution Boundary

`render_creative_stretch` is whole-buffer, allocating, deterministic offline
work. It must not run on the audio thread. It returns the renderer output
directly; no resampling, second stretch pass, limiter, level correction, or
post-render fade is added.

The wrapper maps public request and error vocabulary onto the admitted private
renderer. It may change only:

- new `creative.rs` public boundary and focused tests
- `lib.rs` module and re-export wiring
- `creative_direct_renewal_dream/mod.rs` visibility of the admitted fixed seed
  and internal entry types needed by the wrapper

The admitted acoustic files `analysis.rs`, `plan.rs`, `stereo.rs`, and
`synthesis.rs` remain byte-identical.

## Cache And Routing Boundary

Do not add `Creative` to `StretchBackendTier`, `StretchQuality`, or
`OfflineHighQualityPath`. `CreativeStretch` is a peer product choice, not a
transparent backend selector.

Do not use `StretchCacheIdentityInput` for creative output. Its current schema
does not own character, `space`, fixed creative seed, or creative engine
version without collision risk. Cache admission requires a later contract and
implementation batch.

No automatic route, overlap, fallback, dynamic ratio, pitch, artifact writer,
promotion receipt, runtime DTO, Loophole, or Chorus surface enters this batch.

## Batch 31.76 Gate

Implementation passes only when:

1. only the three allowed source files above change
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

## Next Task

`g10.031` is complete. No follow-on implementation batch is ready. Reopen only
through named-consumer integration authority or renewed source-backed
high-range research.
