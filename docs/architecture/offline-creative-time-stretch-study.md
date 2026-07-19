# Offline Creative Time-Stretch Study

Status: complete; final fixed-ratio candidate brief frozen
Owner: dsp
Updated: 2026-07-19
Contract: `085`
Roadmap: `g10.031`

## Decision

Add a separate Signal-owned `CreativeStretch` product path centered on `8x`
output duration. It is an offline sound-design renderer, not a replacement for
`OfflineHighQuality`, not a reopened Contract `084` successor, and not a
RealtimePreview path.

The product presents one stable intent surface while Signal routes between
renderer families by stretch range. Algorithm names and low-level transform
controls stay internal.

Initial product range:

- creative expansion only
- comparator and first-candidate ratios: `4x`, `8x`, and `16x`
- planned routed range: `1x` through `100x`
- primary design point: `8x`
- ratios above `100x`: future texture/freeze work, not initial scope

`800%` means `8x` output duration. Public APIs continue to use the unambiguous
output/input ratio and explicit target frame count; a consuming UI may display
percent or resulting duration. Target frames are authoritative; the ratio is
derived or validated against them, and a mismatch is an invalid request.

## Product Surface

The consumer-facing model has two peers:

- `Transparent`: the existing `OfflineHighQuality` contract
- `Creative`: the new range-routed sound-design contract

Creative mode exposes:

| Control | Meaning | Primary UI |
| --- | --- | --- |
| `duration` | exact target duration or output/input ratio | yes |
| `character` | named creative intent: `Dream`, `Spectral`, `Rough`, `Cloud`, or `Cyclic` | yes |
| `motion` | stable spectral body to actively evolving detail | yes |
| `detail` | smeared attacks to more source-readable articulation | yes |
| `space` | preserve source image to widen/diffuse it under linked-channel rules | advanced |
| `seed` | deterministic variation identity; default derived from artifact identity | advanced reroll only |

Do not expose FFT size, window shape, grain size, overlap count, phase policy,
transient thresholds, renderer name, or transition weight. Those are engine
implementation details.

`character` names audible intent, not an algorithm. Signal may select or blend
internal owners to realize it without exposing that choice to consumers:

- `Dream`: smooth, fused, musical spectral smear; the default and primary
  PaulXStretch-backed target
- `Spectral`: deliberately exposed, vocoder-like spectral separation and
  decoherence, anchored by CDP `SPECTSTR`
- `Rough`: less smoothed, conspicuously processed polyphase texture, anchored
  by REAPER `Rrreeeaaa`; useful novelty rather than the default
- `Cloud`: dispersed, evolving upper-range texture; later owner
- `Cyclic`: explicit Akai-style repetition, anchored by REAPER `ReaReaRea`

`motion`, `detail`, and `space` refine every character with the same semantic
direction. A consumer gets one clean character selector and shared macro
controls, not an algorithm menu.

`Cyclic` bypasses automatic coherent/diffusive/cloud selection because its
repetitions are an explicit musical choice. Initial cyclic admission targets
creative expansion through `8x`; higher ratios require separate listening
before support is claimed.

## Range-Routed Architecture

All participating renderers consume one monotonic source/output map and the
same exact target frame count.

| Ratio | Internal owner | Product intent |
| --- | --- | --- |
| `1x` to `2x` | coherent slow-motion owner | source-readable lower range |
| `2x` to `4x` | coherent/diffusive overlap | continuous entry into creative smear |
| `4x` to `16x` | diffusive spectral owner | core `Dream` range; `8x` design point |
| `16x` to `32x` | diffusive/cloud overlap | continuous move from smear to texture |
| `32x` to `100x` | layered cloud owner | evolving spectral/granular soundscape |

The coherent lower owner begins from the retained `OfflineHighQuality`
renderer. That reuse does not widen its transparent-quality claim beyond its
admitted range; inside Creative mode it is one source-readable contribution.

The first independent-bin `DiffuseSpectral` topology was implemented once and
rejected for uncontrolled stochastic crest growth. That mechanism is closed.
The replacement family is `ContinuousExcitationSpectral`: long overlapping
spectral analysis, interpolated source envelopes, one bounded continuous
output-synchronous excitation, a linked coherent carrier, and exact normalized
synthesis. It intentionally relaxes transient placement and crisp phase
reconstruction in exchange for smooth, evolving, dreamy output without
constructing unrelated random phase in every bin.

That replacement was implemented once and rejected before its crest row.
Common-polarity covariance missed the structural bound because the per-bin
polar relation reconstruction was not value-stable enough. The waveform-level
excitation decision remains. The final brief replaces native angle subtraction
with a direct complex relation and explicit exact-cancellation law.

Its neutral `Dream` setting owns the PaulXStretch-like centre. The same complete
candidate must expose a useful, controlled path toward the `Spectral` and
`Rough` anchors. This is a parameter-space obligation, not permission to queue
independent detector, window, phase, or coefficient experiments. If one
topology cannot span those anchors without damaging `Dream`, reassess owner
boundaries before admission instead of averaging the references.

The upper `LayeredCloud` owner is a later spectral/granular renderer. It may
layer bounded voices around the common source cursor, but every voice remains
part of one renderer with one target length, normalization law, seed, and
linked-channel policy. It is not an arbitrary wet-effect stack.

## Seamless Selection

Fixed-ratio rendering comes first. Inside each overlap band, renderer weights
are constant for the whole render and follow a smoothstep curve over
`log2(ratio)`:

- coherent to diffusive: `2x` through `4x`
- diffusive to cloud: `16x` through `32x`

Both owners render against the same source map and exact output lattice.
Transition synthesis must:

- align latency and exterior boundaries before mixing
- use one channel-shared weight
- preserve constant-power intent without unbounded loudness correction
- keep the same target frame count on both sides
- remain deterministic for the same request and seed
- avoid a different routing decision from small ratio rounding differences

Dynamic ratio is not part of the first candidate. Later dynamic routing must
use the same logarithmic weight law, bounded weight slew, carried renderer
state, and one shared source cursor. Concatenating independent segment renders
does not satisfy this architecture.

## Stereo, Boundaries, And State

- analysis decisions, source-position variation, transition weights, and
  normalization are shared across linked channels
- channel-relative magnitude and phase relationships remain channel-local
  synthesis inputs; independently randomized left/right renderers are invalid
- mono input with neutral `space` remains mono
- duplicate stereo, channel swap, and polarity transforms must commute with
  the renderer where mathematically applicable
- exterior padding and cropping produce the exact requested frame count
- all stochastic choices come from a stable seeded generator
- working state is bounded independently of source duration, excluding source,
  output, and an explicitly bounded artifact writer

## Comparator Synthesis

The required no-purchase capture set is:

| Reference | Evidence sought |
| --- | --- |
| PaulXStretch | canonical static-to-evolving spectral dream behavior |
| REAPER `Rrreeeaaa` | polyphase big-stretch behavior without transient preservation |
| CDP `SPECTSTR` | spectral expansion with controlled decoherence and randomization |

Optional supplementary references are Sloom `Wide`/`Narrow` for extreme
spectral character, SoundHack `++spiralstretch` for layered PV/granular
ambience, and Ableton `Texture` for granular fluctuation. They do not block the
target freeze: the full Sloom product and SoundHack are paid, while Ableton is
unavailable to the operator.

Secondary references:

- Akaizer or REAPER `ReaReaRea` for the explicit cyclic/metallic reserve
- Photosounder or ARSS for logarithmic filterbank/image-resynthesis behavior
- noise-morphing STN research for shaped stochastic evolution
- neural STN research as an `8x` evidence ceiling, not an implementation plan

The first study uses the retained long-form musical source families: percussion,
bass, vocals, pads/sustains, and full mix. Render `4x`, `8x`, and `16x`.
Short attacks remain diagnostic only; long musical excerpts decide target
character.

The accessible capture is complete under ignored `target/`: PaulXStretch 1.6.0
default / FFT `16384`, REAPER 7.69 `Rrreeeaaa`, pinned CDP 8.0 `SPECTSTR`, and
REAPER `ReaReaRea` as the cyclic control. The CDP comparator uses `d-ratio=1`
and `d-rand=0.5`; those values define this external reference only. Its source
feed is reduced `18 dB` to avoid legacy synthesis clipping before all references
are cropped to exact length and RMS matched under a shared `0.95` peak ceiling.
Operator review froze the target as a controllable family, not a single winner:

- PaulXStretch is the default centre across all source families and ratios
  because its smoother output is the most consistently musical and useful
- CDP is a valid `Spectral` endpoint, but its vocoder-like character must never
  leak into neutral `Dream`
- `Rrreeeaaa` is a valid `Rough` endpoint and comparison anchor, but its novelty
  character must remain intentional rather than becoming the default
- `ReaReaRea` establishes a valuable `Cyclic` character through about `8x`

The goal is not sample-identical emulation or one compromised average. The
semantic controls must reach recognizably similar regions while preserving
Signal's structural rules.

## Admission

Structural rejection is immediate for:

- wrong length, non-finite samples, non-determinism, or unbounded working state
- DC growth, uncontrolled peak or loudness jumps, exterior clicks, or dropouts
- obvious frame-rate flutter, stutter, or static freeze outside the requested
  character
- channel collapse, unstable image motion, broken duplicate/mono behavior, or
  independently randomized channels
- discontinuous character across `2x`/`4x` or `16x`/`32x` overlap probes

Listening is the quality authority. Review records:

- dreaminess or textural usefulness
- source identity and tonal interest
- evolution versus frozen/static behavior
- objectionable grain, ringing, periodicity, or metallic replicas
- attack-detail usefulness rather than transparent transient accuracy
- level and stereo-image stability
- preference and intended use

Creative character admission additionally requires:

- neutral `Dream` remains smooth and musically useful on every retained family
  at `4x`, `8x`, and `16x`, without exposed vocoder colour or rough periodicity
- `Spectral` and `Rough` are recognizably distinct, useful destinations rather
  than accidental degradation, and do not cause clicks, dropouts, or arbitrary
  level changes
- movement from `Dream` toward either endpoint is stable and directionally
  consistent; a hard internal owner change cannot create an audible seam
- later `Cyclic` review treats commanded repetition as character, while still
  rejecting unstable timing, discontinuities, unbounded peaks, and unrelated
  channel motion

The operator may perform mono and character review. Linked-stereo promotion
still requires an independent eligible listener because the operator has
hearing in one ear.

## Research Resolution

The existing non-phase-vocoder closure remains correct for transparent
`0.5x` through `2x` replacement. Creative `4x` through `16x` work has different
success criteria: controlled smear, diffusion, and loss of transient precision
can be features. This study does not reopen the failed phase-vocoder successor
or relax `OfflineHighQuality` promotion rules.

The evidence still supports the product intent, but the range router has no
admitted diffusive owner. All three fixed-ratio candidates are rejected and
deleted. The final value-symmetric relation candidate stopped at coefficient
proof, closing the current diffusive owner rather than opening another variant.

## Sources

- [REAPER time-stretch engines](https://www.reaper.fm/about.php)
- [REAPER user guide](https://www.reaper.fm/userguide.php)
- [SoundHack Pvoc Kit and ++spiralstretch](https://www.soundhack.com/pvoc/)
- [SoundHack ++spiralstretch manual](https://www.soundhack.com/spiralstretch-manual/)
- [Ableton Warp Modes](https://www.ableton.com/en/live-manual/11/audio-clips-tempo-and-warping/)
- [CDP SPECTSTR](https://www.composersdesktop.com/docs/html/cstretch.htm)
- [Sloom](https://anemond.net/sloom/)
- [Akaizer](https://the-akaizer-project.blogspot.com/)
- [Photosounder](https://www.photosounder.com/)
- [ARSS](https://arss.sourceforge.net/)
- [Noise Morphing for Audio Time Stretching](https://arxiv.org/abs/2312.14586)
- [PaulXStretch official repository](https://github.com/essej/paulxstretch)
- [Extreme Audio Time Stretching Using Neural Synthesis](https://arxiv.org/abs/2211.16992)

## Next Task

Execute `g10.031` Batch 31.9 only. Reassess ownership of the creative `4x`
through `16x` range without implementation. Keep rejected diffusive families,
`Cloud`, `Cyclic`, and product routing closed.
