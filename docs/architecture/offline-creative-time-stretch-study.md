# Offline Creative Time-Stretch Study

Status: `RenewalSpectral` rejected; crest-ownership reassessment ready
Owner: dsp
Updated: 2026-07-20
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
- core spectral comparator ratios: `4x`, `8x`, and `16x`
- retained cyclic comparator ratios: identity, `2x`, `4x`, and `8x`; explicit
  cyclic implementation is closed
- planned routed range: `1x` through `100x`, currently paused
- primary design point: `8x`
- ratios above `100x`: future texture/freeze work, not initial scope

`800%` means `8x` output duration. Public APIs continue to use the unambiguous
output/input ratio and explicit target frame count; a consuming UI may display
percent or resulting duration. Target frames are authoritative; the ratio is
derived or validated against them, and a mismatch is an invalid request.

## Source-Triangulation Reassessment

Batch 31.16 reopens docs-only `Dream` research by explicit operator decision.
Pinned end-to-end study of PaulXStretch 1.6.0, CDP8 `SPECTSTR`, and Potenza
changes the architecture evidence:

- neutral PaulXStretch uses long-window magnitude analysis, deliberate input
  phase loss, new stochastic phase per output frame, and frame crossfade
- the retained PaulX default disables onset handling and optional spectral
  processors; continuous phase recurrence and magnitude evolution do not own
  its preferred sound
- CDP interpolates amplitude and instantaneous-frequency analysis frames, then
  optionally perturbs lower-energy frequency tracks before phase-vocoder
  synthesis; this is a separate `Spectral` owner
- Potenza confirms that Akai-style cyclic colour comes from two unit-rate
  waveform grains with compressed anchor advance; it does not supply pitch,
  stereo, exact-length, or similarity-search ownership

The rejected `DiffuseSpectral` brief was not a direct test of the PaulX path.
It added an instantaneous-frequency carrier, correlated diffusion,
log-magnitude evolution, and a different overlap topology. The two
continuous-excitation replacements moved farther from deliberate per-frame
phase forgetting. Those candidates stay rejected, but they do not close the
new source-backed family.

Batch 31.16 selected `RenewalSpectral` for the next complete brief. It owns neutral
`Dream` at `4x`, `8x`, and `16x` through one long-window magnitude view,
deterministic frame phase renewal, bounded frame combination, exact crop, and a
new Signal-owned linked-channel excitation law. It does not own `Spectral`,
`Rough`, `Cyclic`, `Cloud`, routing, or blends.

The canonical source study is
[Creative Stretch Source Triangulation](../research/specimen-dossiers/creative-stretch-source-triangulation.md).

Batch 31.17 freezes the selected renderer in
[Offline Creative RenewalSpectral Renderer Brief](./offline-creative-renewal-spectral-brief.md).
The private candidate owns `space`; `motion` and `detail` remain unsupported
until the neutral core passes. No DSP is admitted by the brief.

Batch 31.18 implemented that brief once. Compile-only and structural admission
passed, but the first crest row measured `8.263162 dB` growth against the
frozen `6 dB` ceiling. The candidate was deleted before later synthetic or
listening gates. No DSP was admitted.

## Range-Owner Reassessment

Batch 31.9 narrows the first deliverable. Do not assign `4x` through `16x` to
the retained coherent renderer or another spectral-diffusion variant.

- `OfflineHighQuality` has no PaulX-centred listening evidence at these ratios.
  Reusing it as the core owner would provide source-readable phase-vocoder
  expansion, not the frozen `Dream`, `Spectral`, and `Rough` character space.
- Adding smear, stochastic excitation, or a spectral wet layer would recreate
  the closed diffusive owner under another boundary.
- Granular cloud, image resynthesis, STN, and learned synthesis do not have one
  complete source-backed path through the core range, linked stereo, exact
  length, deterministic state, and retained musical targets.

The automatic coherent/diffusive/cloud router is paused. `Dream`, `Spectral`,
`Rough`, and `Cloud` remain useful target vocabulary and comparator evidence,
but no implementation or public availability is claimed.

The last bounded path studied explicit `Cyclic` expansion above `1x` through
`8x`. It is a narrower product promise, not the replacement core owner:

- the operator found the ReaReaRea/Akai-style result useful through about `8x`
- the retained comparator pack contains ReaReaRea at `4x`, `8x`, and `16x`,
  with `16x` retained as a rejection boundary rather than a target
- public Potenza source independently demonstrates a two-overlapping-grain,
  moving-source, crossfaded Akai-style topology
- cyclic waveform grains are materially separate from the rejected spectral
  diffusion and continuous-excitation families

Potenza is GPL-3.0 architecture evidence at revision
`ddb44a8f949b3f49320932e1d2e997b3a02149bb`. Signal must not copy its
expression, constants, or control flow. Akaizer 2.5 is now paid and provides
no public source backing; it is optional behavioral context, not implementation
backing or a required comparator.

Batch 31.10 froze one clean-room Signal-owned `CyclicGrain` renderer before any
candidate work. It owns the sample-domain map, deterministic two-grain
scheduling, normalized crossfade and boundary law, unit-rate pitch
preservation, linked channels, exact length, bounded state, semantic macro
mapping, gate order, listening pack, rejection, and cleanup. No raw grain size
or implementation algorithm is promised to consumers.

Batch 31.11 implemented that brief once. All structural controls passed, then
neutral `110 Hz` at `2x` measured `111.328 Hz`, or `20.778` cents against the
frozen `15`-cent synthetic pitch ceiling. The candidate was deleted without
correction or rerun. At that stop, explicit `Cyclic` had no owner and required
architecture reassessment before another complete candidate.

## Cyclic Ownership Reassessment

Batch 31.12 selects one materially different family for a new complete brief:
`SimilarityAlignedCyclic`. It is a clean-room, waveform-domain,
correlation-aligned overlap-add renderer. This selection does not admit an
implementation.

The architecture changes the failed owner, not a constant inside it:

- rejected `CyclicGrain` used fixed ideal-map anchors, then crossfaded
  source-offset unit-rate reads whether their waveforms agreed or not
- `SimilarityAlignedCyclic` keeps a regular output launch lattice but selects
  each source segment inside one bounded neighbourhood of the ideal map by
  similarity to the prior segment's natural continuation
- selected source anchors form one strictly increasing path; every correction
  is relative to the ideal map and cannot accumulate into a free-running
  cursor
- one channel-symmetric score, anchor, window, and normalization law is shared
  across linked channels; synthesis still reads each native channel
- all segment reads remain forward and unit-rate; correlation alignment, not a
  spectral or pitch-shift stage, owns join continuity and pitch preservation

This directly addresses the rejected pitch mechanism. The WSOLA literature
derives local segment displacement as the means of restoring natural waveform
continuity at overlap joins. SoundTouch independently demonstrates a maintained
time-domain implementation shape with one overlap search, one selected offset,
and shared multichannel processing. Signal may use those sources for
architecture and validation only. Their expression, constants, search order,
tables, and tuning do not transfer.

The historical WSOLA closure remains valid for transparent replacement. One
waveform lag cannot guarantee arbitrary-polyphonic transparency, and published
or maintained implementations warn about echoing and drift as sequence and
search support grow. Those faults are risks here, but intentional cyclic
repetition is the target rather than a transparent failure. The complete
candidate must still clear the frozen tone and chord pitch ceiling, integrity
controls, five-family mono pack, exact `16x` rejection, linked mechanics, and
independent stereo listening.

Pitch-synchronous OLA is not selected: it requires a credible dominant-period
owner that does not generalize to the retained full-mix pack. Plain SOLA does
not add a stronger source-backed path than waveform-similarity selection.
Fixed chunk repetition and another unaligned grain lattice retain the rejected
join problem. Spectral correction would reopen the closed spectral families.

Batch 31.13 freezes that complete brief. It owns the exact rational ideal map,
strict realized path, fixed output lattice, bounded two-stage correlation
search, score and ties, `0.25` low-confidence fallback, segment and overlap
geometry, semantic macros, anti-replica law, linked synthesis, exact length,
`4 MiB` state cap, deterministic cost, gate order, retained comparator pack,
cleanup, and minimal admission. No item remains a candidate-time choice.

## Final Cyclic Ownership Reassessment

Batch 31.14 implemented `SimilarityAlignedCyclic` once. Compile-only
validation passed, but structural admission failed: the exact continuation at
source frame `6352` lay between coarse samples and never reached full
refinement, which selected frame `6432`. The candidate was deleted without
correction or rerun.

Batch 31.15 closes explicit `Cyclic`. No remaining family supplies a genuinely
different, source-backed path through the complete Signal gate set:

| Family | Evidence | Blocking boundary | Decision |
| --- | --- | --- | --- |
| fixed or unaligned cyclic overlap-add | Potenza demonstrates the Akai-style two-grain shape | `CyclicGrain` already failed pitch; another lattice, window, or hop repairs the rejected owner | closed |
| SOLA, WSOLA, SoundTouch-style search | one selected waveform lag and overlap join | exhaustive search, denser coarse sampling, a larger shortlist, or another score repairs `SimilarityAlignedCyclic`; SoundTouch documents the same sequence/search/overlap trade between echo and drift | closed |
| TD-PSOLA, ESOLA, FESOLA | pitch- or epoch-synchronous speech and strong-fundamental processing | no channel-shared full-mix epoch owner; ESOLA evidence covers speech at `0.5x` through `2x`, not the retained mixed-program `8x` target | specialist only |
| transient-managed or component overlap-add | can protect selected attacks or tonal components | reopens detector timing or separately modified component recombination without one complete linked-stereo law | closed for this lane |
| spectral, sinusoidal, or learned hybrid | other representations can own different material classes | reopens closed spectral/component families or requires a separate trained-model product program; no retained cyclic whole-system path | separate research only |

The two candidate failures are different, but every remaining waveform option
is either a direct repair of one of them or requires an unowned period,
component, stereo, or model boundary. A third implementation is not ready.
`Cyclic` remains useful comparator vocabulary, not an available character.

## Product Surface

The planned consumer-facing model has two peers:

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
repetitions are an explicit musical choice. Its first candidate is rejected;
the character remains unavailable. Any reopening still targets expansion above
`1x` through `8x`; higher ratios require separate listening before support is
claimed.

## Range-Routed Architecture

All participating renderers consume one monotonic source/output map and the
same exact target frame count.

| Ratio | Current owner state | Product intent |
| --- | --- | --- |
| above `1x` to `8x` | cyclic owner closed; no implementation | commanded Akai-style repetition |
| `1x` to `2x` | coherent contribution retained, not creatively admitted | future source-readable lower range |
| `2x` to `4x` | overlap paused | no automatic route |
| `4x` to `16x` | `RenewalSpectral` rejected; no implementation | neutral `Dream` owner under reassessment |
| `16x` to `32x` | overlap closed | no automatic route |
| `32x` to `100x` | cloud owner closed | future texture research only |

The coherent lower owner begins from the retained `OfflineHighQuality`
renderer. That reuse does not widen its transparent-quality claim beyond its
admitted range; inside Creative mode it is one source-readable contribution.

The first independent-bin `DiffuseSpectral` topology was implemented once and
rejected for uncontrolled stochastic crest growth. That mechanism is closed.
The replacement family was `ContinuousExcitationSpectral`: long overlapping
spectral analysis, interpolated source envelopes, one bounded continuous
output-synchronous excitation, a linked coherent carrier, and exact normalized
synthesis. It intentionally relaxes transient placement and crisp phase
reconstruction in exchange for smooth, evolving, dreamy output without
constructing unrelated random phase in every bin.

That replacement was implemented once and rejected before its crest row.
Common-polarity covariance missed the structural bound because the per-bin
polar relation reconstruction was not value-stable enough. The waveform-level
excitation decision remained. The final direct-complex replacement then
stopped at coefficient proof because its test required incompatible exact
anti-phase negation and negated-swap outcomes. The frozen stop rule closed the
candidate and current diffusive owner.

No neutral `Dream` implementation now exists. A future reopening must still
span `Dream`, `Spectral`, and `Rough` without averaging their distinct targets,
but it requires new complete-system evidence rather than another local
spectral variant.

The upper `LayeredCloud` owner is a later spectral/granular renderer. It may
layer bounded voices around the common source cursor, but every voice remains
part of one renderer with one target length, normalization law, seed, and
linked-channel policy. It is not an arbitrary wet-effect stack.

## Seamless Selection

The following remains a future router law, not an admitted implementation.
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

The evidence still supports the creative product intent, but not the original
automatic range router. All three prior spectral fixed-ratio candidates are
rejected and deleted. Batch 31.16 supplies new source-backed evidence for one
materially different neutral `Dream` family. It does not reopen the deleted
branches or admit candidate DSP.

The explicit cyclic reserve still has operator value and a retained comparator,
but both complete owners are rejected and deleted. Batch 31.15 found no third
materially different, source-backed whole-renderer path. `Cyclic` is closed
with no promotion. `Spectral`, `Rough`, `Cloud`, automatic routing, dynamic
ratio, cache, and product integration remain paused. `Dream` is open only for
docs-level crest-ownership reassessment after the `RenewalSpectral` rejection.

## Sources

- [REAPER time-stretch engines](https://www.reaper.fm/about.php)
- [REAPER user guide](https://www.reaper.fm/userguide.php)
- [SoundHack Pvoc Kit and ++spiralstretch](https://www.soundhack.com/pvoc/)
- [SoundHack ++spiralstretch manual](https://www.soundhack.com/spiralstretch-manual/)
- [Ableton Warp Modes](https://www.ableton.com/en/live-manual/11/audio-clips-tempo-and-warping/)
- [CDP SPECTSTR](https://www.composersdesktop.com/docs/html/cstretch.htm)
- [Sloom](https://anemond.net/sloom/)
- [Akaizer](https://the-akaizer-project.blogspot.com/)
- [Potenza Akai-style time-stretch source](https://github.com/dar-io-p/potenza-time-stretch/tree/ddb44a8f949b3f49320932e1d2e997b3a02149bb)
- [Photosounder](https://www.photosounder.com/)
- [ARSS](https://arss.sourceforge.net/)
- [Noise Morphing for Audio Time Stretching](https://arxiv.org/abs/2312.14586)
- [PaulXStretch official repository](https://github.com/essej/paulxstretch)
- [Extreme Audio Time Stretching Using Neural Synthesis](https://arxiv.org/abs/2211.16992)
- [Verhelst and Roelands, WSOLA](https://doi.org/10.21437/Eurospeech.1993-59)
- [SoundTouch algorithm notes](https://soundtouch.surina.net/README.html)
- [SoundTouch source, studied revision `f738b113`](https://codeberg.org/soundtouch/soundtouch/commit/f738b1132ec1fd56efc90367898244cf52d9e6a5)
- [Moulines and Charpentier, PSOLA](https://doi.org/10.1016/0167-6393(90)90021-Z)
- [Rudresh et al., ESOLA](https://arxiv.org/abs/1801.06492)
- [Roberts and Paliwal, FESOLA](https://doi.org/10.1109/WASPAA.2019.8937258)

## Next Task

Run Batch 31.19 only. Reassess neutral-`Dream` crest ownership at architecture
level or close the owner. Do not restore a rejected branch or reopen another
character or product surface.
