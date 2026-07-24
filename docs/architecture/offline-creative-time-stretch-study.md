# Offline Creative Time-Stretch Study

Status: continuous Dream and exact-ratio Cyclic public; `g10.034` audit ready
Owner: dsp
Updated: 2026-07-24
Contract: `085`
Roadmaps: `g10.031`, `g10.032`, `g10.033`

## Product Intent

Signal now exposes a `CreativeStretch` path centered on `8x` output duration.
The public offline whole-buffer API admits neutral `Dream` at every exact
target from `4x` through `16x` without automatic or product routing. It is an
offline sound-design renderer, not a replacement for
`OfflineHighQuality`, a Contract `084` successor, or a RealtimePreview path.

The product presents one stable intent surface while Signal routes between
renderer families by stretch range. Algorithm names and low-level transform
controls stay internal.

Initial product range:

- creative expansion only
- core spectral comparator ratios: `4x`, `8x`, and `16x`
- public Dream range: every exact target in `4N..=16N`
- public manual Cyclic ratios: exact `2x`, `4x`, and `8x`
- current executable creative coverage: continuous Dream and exact-ratio Cyclic
- planned routed range: `1x` through `100x`, deferred and unowned
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

Batch 31.19 attempted to close neutral `Dream`. Both independent-phase renderers
failed crest before listening: `DiffuseSpectral` at `7.08 dB` growth and
`RenewalSpectral` at `8.263162 dB`, against the same `6 dB` ceiling. Removing
the carrier, magnitude recurrence, and rolling overlap-add did not remove the
fault. Independent stochastic bin phase has no intrinsic bound on the crest of
the reconstructed waveform.

Joint low-crest multisine solvers and IAAFT can constrain periodic synthetic
waveforms, but they do not supply a source-mapped, nonstationary, linked-stereo,
bounded-cost musical stretcher. STN noise morphing owns a separated residual,
not the complete first-party renderer. Signal's bounded continuous-excitation
translation already failed linked-channel ownership. No materially different,
source-backed whole renderer clears the complete boundary. A limiter,
post-gain stage, phase/window choice, scalar sweep, or fusion of rejected
owners is not a replacement architecture.

## Operator Correction

The Batch 31.19 target closure is superseded. It promoted two candidate
failures into abandonment of the operator's stated PaulXStretch-like product
goal.

The decisive gate was also mismatched. PaulXStretch's `3.88 dB` figure came
from the 15 retained long-form musical rows. `RenewalSpectral` was stopped on a
synthetic uniform-noise row before the matching PaulXStretch synthetic suite
was rendered. That comparison can reject the candidate against an absolute
safety policy, but it cannot establish that the PaulX-like family missed its
reference.

The whole-path translation was incomplete as well. Pinned PaulXStretch uses a
raised-cosine blend between adjacent random-phase frames and an explicit
position-dependent compensation for blend interference and amplitude
modulation. `RenewalSpectral` substituted an equal-power blend and fixed gain.
Signal must derive its own bounded compensation law from overlap statistics;
it must not copy upstream constants or control flow.

`Dream` is active again. Rejected candidates stay deleted and remain useful
failure evidence. Work proceeds one complete candidate at a time, but a
candidate failure does not close the PaulX-like target. Metrics diagnose and
protect integrity. Long-form concealed listening remains the character
authority.

## PaulX Reference Recovery

Batch 31.20 rendered the frozen synthetics through the pinned PaulXStretch
`1.6.0` core at `4x`, `8x`, and `16x`. Worst-channel uniform-noise crest growth
was `9.932`, `11.899`, and `10.432 dB`. The rejected Signal candidate's
`8.263162 dB` `4x` row was below the matching PaulX value. The `6 dB` stop was
valid only for its frozen brief, not as a target-relative quality finding.

The recovered path separates two amplitude owners:

- a complementary raised-cosine adjacent-frame blend
- position-dependent compensation for the blend's energy modulation

Signal's clean-room law is derived independently. With frame weights `a` and
`b=1-a`, use `c=1/sqrt(a^2+b^2)`. It holds equal expected variance for two
equal-energy uncorrelated frames and stays in `[1,sqrt(2)]`. It does not cap
crest; matching-reference diagnostics and listening own that risk.

[Offline Creative VarianceCompensatedRenewalSpectral Renderer Brief](./offline-creative-variance-compensated-renewal-spectral-brief.md)
freezes the current complete candidate. The earlier
[CompensatedRenewalSpectral brief](./offline-creative-compensated-renewal-spectral-brief.md)
is rejected-at-compile evidence. No DSP is admitted.

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

## Cyclic Research Reopening

The operator reopened `Cyclic` research on 2026-07-23. This supersedes the
Batch 31.15 "no third family" conclusion at research level only.

The prior source set collapsed two historical Akai modes:

- fixed `CYCLIC`: one user-selected cycle duration
- adaptive `INTELL`: material-dependent decisions and correlation

It also missed pinned SickoCV's repeat/jump cycle clock. Potenza alternates two
forward grains around a slow source cursor; SickoCV repeatedly corrects one
forward cursor at explicit cycle boundaries and crossfades the jump. These are
different complete schedules.

The rejected `CyclicGrain` candidate remains deleted. Its `20.778`-cent first
row exceeded the frozen absolute ceiling by `5.778` cents, but no matching
ReaReaRea pitch delta or musical comparison ran. The receipt does not establish
target failure. `SimilarityAlignedCyclic` also stays deleted; it belongs to the
adaptive `INTELL` family and failed its frozen structural search.

The canonical reopening study is
[Cyclic Time-Stretch Source Architecture](../research/specimen-dossiers/cyclic-time-stretch-source-architecture.md).
Batch 32.2 completes ignored source-faithful forensics. ReaReaRea's replica
count and tail-support scaling are consistent with compressed-anchor overlap,
not raw whole-cycle repetition. ReaReaRea also centres replicas around mapped
events. Neither source probe owns that complete scheduler.

The same receipt invalidates the old absolute `15`-cent pitch gate for this
effect and confirms that one shared schedule preserves the tested stereo
relations.

Batch 32.3 selects centred compressed-anchor Cyclic behavior. One monotonic
ideal map owns event centres; one fixed render-wide user cycle owns the
metallic-to-tremolo direction; forward native-rate waveform reads create
ratio-dependent replica clusters around those centres; every linked channel
shares the schedule and exact boundary crop. Raw whole-cycle repeat/jump does
not match the measured target grammar. Similarity search remains adaptive
`INTELL`.

The corrected gate keeps integrity, boundaries, commanded replicas, and linked
mechanics hard. Pitch, cadence, replica distribution, crest, level, tail, and
local balance become complete diagnostics. Concealed listening across the five
musical families at `2x`, `4x`, and `8x` decides character and usefulness.

The canonical decision is
[Offline Creative Cyclic Behavioral Synthesis](./offline-creative-cyclic-behavioral-synthesis.md).

Batch 32.4 freezes
[CenteredCompressedAnchorCyclic](./offline-creative-centered-compressed-anchor-cyclic-brief.md).
One exact rational map, `5..90 ms` manual cycle, two forward unit-rate reads,
complementary raised-cosine crossfade, independent event ledger, linked
geometry, direct exact crop, bounded state, comparator manifest, and complete
Rule 11 gate now own the first candidate. No DSP entered `main`.

The first centred checkpoint later became evidence-invalid at `Y01`. A fresh
audited checkpoint passed conformance but split its first acoustic receipt
between two roots. Contract `085` Rule 11 closes that identity after the
second incomplete-evidence checkpoint. No valid acoustic judgment or admitted
Cyclic renderer exists.

The operator supersedes that closure in Batch 32.14. The renderer did not
fail; the caller supplied a relative evidence root. The unchanged checkpoint
may replay `Y01` once with the exact absolute root.

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
| `cycle` | short metallic Cyclic motion through long tremolo/echo motion | Cyclic only |
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

Shared controls retain one semantic direction wherever valid. Cyclic instead
uses one character-local `cycle`; `motion`, `detail`, and `space` are not
aliases for it. A consumer receives only the controls valid for an admitted
character, not an algorithm menu or disabled fictional controls.

`Cyclic` bypasses automatic coherent/diffusive/cloud selection because its
repetitions are an explicit musical choice. Both historical candidates remain
rejected; the character remains unavailable. The selected behavior targets
expansion above `1x` through `8x`; higher ratios require separate listening
before support is claimed.

## Range-Routed Architecture

All participating renderers consume one monotonic source/output map and the
same exact target frame count.

| Ratio | Current owner state | Product intent |
| --- | --- | --- |
| above `1x` to `8x` | cyclic owner closed; no implementation | commanded Akai-style repetition |
| `1x` to `2x` | coherent contribution retained, not creatively admitted | future source-readable lower range |
| `2x` to `4x` | overlap paused | no automatic route |
| `4x` to `16x` | private `DirectRenewalDream` admitted at exact `4x`, `8x`, and `16x` | no public route |
| above `16x` to `100x` | deferred; Cloud closed without acoustic judgment | no quality result, admitted owner, or active study |

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

At that checkpoint no neutral `Dream` implementation existed. The later
`DirectRenewalDream` reset and admission supersede that implementation-state
statement without reviving any rejected spectral candidate.

The studied upper `LayeredCloud` owner was one pointer-led granular renderer.
Its second executable-authority failure closed that family before acoustic
identity. It has no quality pass or rejection and does not own a production
range. Exact `16x` remains the admitted Dream endpoint; it does not imply
continuous `16x..32x` coverage or support above `16x`.

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

The evidence still supports the creative product intent, but not an admitted
renderer or the original automatic range router. All isolated spectral
fixed-ratio candidates are rejected and deleted. Batch 31.16 supplied source-backed
evidence for one materially different neutral `Dream` family. Batch 31.18
rejected its first translation at crest. Batch 31.19's family closure is
superseded by the operator correction above.

The explicit cyclic reserve still has operator value and a retained comparator,
but both complete owners are rejected and deleted. Batch 31.15 found no third
materially different, source-backed whole-renderer path. `Cyclic` is closed
with no promotion. `Spectral`, `Rough`, `Cloud`, automatic routing, dynamic
ratio, cache, and product integration remain paused. `Cyclic` stays closed.
The rejected `SourceRelativeRenewalSpectral` brief retains the passed mono
renewal renderer and replaces mid/side magnitude synthesis with native
left/right complex analysis and an explicit source-relative phase law. No
product surface was exposed.

Batch 31.27 implemented that brief once. Compile and construction passed, but
structural admission stopped at `14/15` because the frozen `mix64(1)` assertion
contained a transposed hexadecimal vector. The implementation matched the
normative formula. No synthetic or listening result exists. The candidate was
deleted without assertion repair or rerun.

Batch 31.28 independently reproduced every counter/tag/address vector in
Python and Ruby and froze fresh verified authority. Batch 31.29 passed
construction `1/1` and structural `15/15`, then failed one `16x` replica row
and two `4x` pitch rows. Its helpers chose seed `17`, while neither its brief
nor Batch 31.25's passing mono brief froze the candidate seed. Batch 31.30
therefore withdrew the range diagnosis: the failed checkpoint stays rejected,
but stochastic rows with differing or unknown request identity cannot select a
range switch. Pinned PaulX also retains one renewal path across all ratios.
Batch 31.30 froze `SeedAuditedSourceRelativeRenewalSpectral`, with the audited
address seed fixed for every synthetic and listening candidate render.

Batch 31.31 implemented that authority once. Checkpoint `790119b7` passed
compile, construction `1/1`, and structural `15/15`. The synthetic command
selected all nine owners. Six passed before `Y02` failed the `8x` chord at
`13.351828347` cents against its `11.331375778`-cent ceiling; `Y08` and `Y09`
were cancelled. `Y04` passed both impulse sources at all ratios. The candidate
was deleted before listening.

## Renewal Tonal-Coherence Closure

Batch 31.32 closes the renewal family without closing the PaulX-like product
goal. Batch 31.29 failed two `4x` pitch rows under seed `17`; Batch 31.31
failed the `8x` chord under the audited seed. Different seeds, material, and
ratios expose one repeated mechanism class: magnitude-only frame renewal has
no phase-continuous tonal state. It preserves pitch statistically, but cannot
own a deterministic finite-render pitch bound for sustained tones and chords.

Changing seed, transform, hop, window, blend, threshold, or scalar leaves that
ownership unchanged. Adding oscillator, peak, or phase recurrence would create
a different renderer rather than correct renewal. Contract `084` Rule 7 and
the one-complete-candidate rule therefore prohibit another renewal candidate.

The retained and newly checked source evidence supplies no eligible complete
replacement:

| Source family | Tonal owner | Blocking boundary |
| --- | --- | --- |
| Signalsmith Stretch | weighted horizontal and vertical phase prediction | upstream describes best quality at modest ratios and uses a `>2x` randomized-observation fallback identified by its author as a hack; it is not a complete PaulX-like extreme-stretch owner |
| Bungee and Rubber Band | tracked peak regions or material-state multiresolution phase | reopens explicitly rejected peak, H/R/P, frequency-adaptive, and hybrid work; neither is a clean replacement for the retained creative target |
| SBSMS | direct tracked partial oscillators | pinned complete source already failed Signal's mono, linked-stereo, and long-form feasibility controls |
| STN noise morphing and neural STN | oscillator, relocated-transient, and stochastic residual layers | separated hybrid owners, trained state, or unavailable complete source prevent one bounded first-party linked renderer |
| TSM-Net | learned whole-waveform decoder | public inference requires pretrained weights; training code and a usable repository licence are absent, and no intrinsic pitch, linked-stereo, or bounded-state law is exposed |
| public TSM toolboxes | classical PV, WSOLA, or HPSS combinations | reduce to previously assessed phase, waveform, or separated hybrid families and provide no new `4x`-`16x` whole-renderer evidence |

PaulXStretch remains the preferred audible reference and `Dream` remains valid
product vocabulary. This decision says only that Signal's clean-room renewal
translation is exhausted. Reopening requires a public complete renderer or
operator evidence that supplies intrinsic tonal coherence, diffusive musical
character at `4x`, `8x`, and `16x`, one linked-channel law, exact length,
determinism, and bounded state without reviving a rejected family.

## Listening-Led Gate Correction

The operator explicitly changed the governing Contract `085` boundary after
Batch 31.32. The old closure remains the correct result under its frozen gate,
and both rejected checkpoints stay rejected. Future creative admission no
longer treats the PaulX pitch error plus `2` cents as a terminal threshold.

The correction follows the strongest product evidence:

- Batch 31.25 passed concealed mono as `15/15` ties against PaulXStretch
- operator speaker review found the stereo output solid apart from the
  source-relative balance inversion
- the later native-channel relation law directly owns that balance defect
- its fresh checkpoints stopped before listening on finite comparator deltas,
  not a heard tonal failure

`ListeningLedSourceRelativeRenewalSpectral` retained the complete seed-audited
renderer and terminal controls. `Y02` measures every tone and chord row and
records candidate error, PaulX error, and signed delta. It rejects missing or
non-finite evidence, not a finite pitch delta. Hard integrity, replica, level,
discontinuity, dropout, boundary, deterministic-state, and linked-stereo gates
remain terminal. Concealed mono, operator speaker pre-screen, and eligible
independent stereo listening remain the default. Batch 31.66 later applies
Contract `085`'s checkpoint-scoped operator exception.

This was one fresh candidate authority. Batch 31.34 passed construction `1/1`
and structural `15/15`, then synthetic `Y08` rejected exact-zero impulse hops
at every ratio. Its executable range used complete impulse output where the
normative dropout text names mapped non-zero support. The candidate was
deleted. Batch 31.35 classified the complete-output dropout scan as executable
evidence-construction failure. The isolated impulse's mapped authored support
is only `4`, `8`, or `16` frames, shorter than `H=16384`; impulse spread and
replicas remain owned by `Y03` and `Y04`.

Fresh `SupportAuditedListeningLedSourceRelativeRenewalSpectral` authority
freezes one exact source-support table and separate discontinuity/dropout range
types. It changes no renderer formula, source, threshold, seed, listening pack,
or product surface.

Batch 31.36 passed compile, construction `1/1`, structural `15/15`, synthetic
`9/9`, and concealed mono as `15/15` ties. The operator reported only minor
extra low-end noise and opposite exterior energy weighting versus PaulX.
Valid same-source stereo admission then rejected `16x` local image stability:
bass mapped-window error was about `2.00 dB`; full mix reached
`9.37..9.42 dB` with channel-dominance reversal despite close whole-render and
band balance. The candidate was deleted before speaker or independent review.

This is the second complete renewal candidate to fail linked stereo. Batch
31.25 inverted global source balance; Batch 31.36 preserved global balance but
lost mapped local dominance. Contract `084` now requires architecture
reassessment before another implementation.

Batch 31.37 closes renewal under the current Contract `085` stereo boundary.
The native-channel law already applies one common current-frame phase rotation
while retaining per-channel magnitudes and exact relation at `space=0`.
Random common phase renewal then changes independently between adjacent
synthesis frames. Their waveform interference is not source-owned, so exact
coefficient relation and close global energy do not prevent local balance
drift after frame blending.

Every reviewed complete source-backed temporal owner changes family:
Bungee adds predecessor peak-region rotation, Signalsmith adds coherent
horizontal/vertical recurrence, Rubber Band adds tracked peak and material
states, and SBSMS adds paired oscillators but failed source feasibility.
PaulX itself uses independent per-channel phase draws and supplies no hard
source-relative image invariant. Post-hoc covariance, window gain, relation
smoothing, or another phase/`space` law is an unsupported repair.

No complete successor brief opens. The target remains valid because mono
listening repeatedly matched PaulX. The next move is an operator product-policy
decision: retain local source-relative stereo as terminal, or make it
diagnostic under comparator-relative independent listening. A changed gate
would require fresh authority and a fresh candidate; it cannot revive a
deleted checkpoint.

The operator selected the comparator-relative policy after Batch 31.37. This
does not erase either stereo failure. For neutral `Dream`, local mapped-window
source balance and dominance remain complete diagnostics, now paired with the
same PaulX-source measurements. Structural stereo relationships, whole-render
and three-band balance, deterministic integrity, exact boundaries, and bounded
state remain hard. An eligible independent stereo listener owns final image
promotion; the operator may reject a speaker pre-screen but cannot supply that
independent pass.

Batch 31.38 freezes one fresh complete
`ComparatorAuditedRenewalSpectral` brief. It retains the fully tested renderer
formulas rather than inventing a stereo repair, starts from fresh source, and
changes only the product-backed gate classification. The scorecards explicitly
retain the reported low-frequency noise and opposite entry/tail energy risks.
No candidate DSP or product surface enters `main` in this batch.

Batch 31.39 implemented that brief once from fresh source. Its immutable
checkpoint passed compile, construction `1/1`, and structural `15/15`, then
synthetic admission finished `7/9`. `Y04` found a second `16x` impulse region
at `-29.801787859 dB`. The frozen `-30 dB` activity threshold admitted that
region, so the required one-region / `None` result failed. `Y09` reported
linked-stereo swap failure at `4x` and `8x`. Objective rejection kept mono and
stereo listening closed, and the complete candidate was deleted without
repair or rerun.

Batch 31.36 passed both owners under the nominally same renderer formulas and
admission seed. The conflicting receipts are not evidence for a parameter
change or immediate reimplementation. Their frozen authority and executable
construction require the Batch 31.40 reconciliation below.

Batch 31.40 found that reconciliation is impossible from retained evidence.
Both candidates share exact counter, seed, source-support, and owner-inventory
authority, but construction did not freeze helper bodies, numeric execution,
row assembly, assertions, or output digests. `Y09` is decisive: the
source-relative law explicitly disclaims exact time-domain swap at one branch,
while the inherited gate prose never supplies one exact long-form swap
fixture, estimator, tolerance, and assertion.

Batch 31.36's pass and Batch 31.39's failure therefore remain decisions about
their own deleted checkpoints. Restoring executable identity would require a
new brief and a third renewal implementation. That is new candidate work, not
reconciliation, and no materially different source-backed renewal owner exists.
Close renewal again. Keep the PaulX-like target, comparator captures, and
operator findings; no Signal creative renderer is ready.

## Batch 31.41 Complete-Owner Study

The explicit research reopening selected one materially different family:
`LinkedStnNoiseMorph`. The source triangulation now includes the runnable
SiTraNoStar classical STN/noise-morphing path, its decomposition and synthesis
papers, and the related complete neural STN architecture. This evidence owns
the complete mono temporal topology at `4x` and `8x`; it does not transfer GPL
expression or authorize a neural dependency.

The selected Signal architecture is one renderer with three material lanes on
one exact map:

| Material | Analysis owner | Synthesis owner |
| --- | --- | --- |
| tonal | channel-symmetric long-resolution soft mask and persistent peak tracks | linked peak-region phase propagation with dormant/reactivation state |
| transient | shared short-resolution mask, event detection, and segmentation | native-channel waveform segment placed once at the exact mapped event |
| residual noise | remaining reconstructing mask plus time-varying channel spectrum | continuous deterministic multichannel excitation shaped by interpolated residual spectrum and explicit spatial relation |

One mapped source-envelope owner shapes the tonal-plus-noise bed before native
transient recombination. One normalized synthesis and exact crop own exterior
continuity, entry/tail energy, and target length. The final brief must make
analysis and synthesis tiling duration-independent, seed every excitation from
the request, and permit no working allocation after rendering starts.

This is not another renewal candidate. Phase-forgetting is confined to the
separated stochastic residual. Tonal pitch, transient uniqueness, event
placement, and stereo relations each retain a persistent owner. The design
therefore addresses the observed low-end noise, opposite entry/tail energy,
tonal instability, replica risk, and stereo drift together rather than through
post-render repair.

The source boundary remains strict:

- SiTraNoStar is mono-only, nondeterministic, full-file, and not exact-length
- the classical Noise Morphing listening evidence stops at short mono `8x`
- the neural STN path has no released complete training/weight authority
- `16x`, long-form music, deterministic bounded execution, and linked residual
  stereo are Signal-owned risks, not upstream claims

Batch 31.41 admits the family to brief-writing only. No renderer, candidate,
test, harness, fixture, API, route, cache, product surface, Loophole, or Chorus
work is authorized. Batch 31.42 must freeze one self-contained implementation
brief before any isolated candidate can become ready.

## Batch 31.42 Complete Renderer Freeze

[Offline Creative LinkedStnNoiseMorph Renderer Brief](./offline-creative-linked-stn-noise-morph-brief.md)
is now the sole candidate authority. It freezes one exact renderer:

- sample-rate-normalized `8192/1024` long/short analysis at `44.1` and
  `48 kHz`, with reconstructing channel-symmetric soft masks
- one signed-rational map and `512`-sample synthesis hop at those rates
- persistent linked tonal peak and bin oscillators with explicit dormancy and
  reactivation
- shared transient detection, class, segment, exact anchor, native unit-rate
  emission, and replica ledger
- continuous counter excitation shaped by interpolated native-channel residual
  covariance, with residual-only `space`
- mapped source-envelope correction, normalized WOLA, zero exterior, exact
  crop, and no arbitrary renderer fade
- `96 MiB` duration-independent working-state cap and fixed one-shot evidence
  order

The brief also repairs the evidence-authority failure without reviving renewal.
One compile-linked `28`-owner specification owns every helper input, source,
metric, threshold, and assertion. The closeout must retain checkpoint, tree,
file, toolchain, row, and output digests before candidate cleanup.

This is implementation authority, not admission. The candidate must still
prove component reconstruction, tonal and event behavior, residual
non-periodicity, linked stereo, `16x`, long-form mono, and eligible independent
stereo listening. No DSP or product surface entered `main` in Batch 31.42.

## Batch 31.43 Bounded-State Rejection

The isolated `LinkedStnNoiseMorph` checkpoint passed compile and construction
`1/1`. Its one-shot structural run completed `17/18`. `S17` rejected the
renderer because it allocated full-duration source component and spectral
arrays rather than the frozen monotonic analysis rings. Working capacity
therefore scaled with source duration and could not satisfy the `96 MiB`
duration-independent bound.

This is not evidence against STN sound quality: synthetic and listening gates
never opened. It is also not a tunable miss. The candidate failed one of the
complete architecture's ownership boundaries and was deleted without repair
or rerun. The next work is a docs-only feasibility reassessment of bounded
component production, consumer lookahead, eviction, event lifetime, and
synthesis support. At that closeout, another implementation was not ready.

## Batch 31.44 Bounded-State Decision

The complete material-owner graph has a bounded schedule. Symmetric medians,
WOLA support, `24`-frame event confirmation, capped segmentation, `5`-frame
covariance, mapped interpolation, and output normalization all have finite
geometry-derived lookahead and monotonic last consumers.

Residual orientation was the one non-causal dependency. It uses the first
exactly non-zero augmented-residual mid and side samples, which are known only
after decomposition and event reassignment. Bounded v2 resolves those two
scalars in a deterministic full-source orientation prepass, resets all state,
then performs the real render. It retains no prepass components and does not
change the map, masks, event decisions, tonal phase, residual covariance,
envelope, or stereo law.

The canonical brief now freezes packed spectral rings, component and claim
arenas, live event bounds, output finalization, envelope moments/deque,
eviction rules, and a compile-linked `MEMORY_SPEC`. Worst-geometry model rows
sum under category budgets whose owned-state design ceiling is `89 MiB`; the
terminal counting-allocator ceiling remains `96 MiB`. Only the returned
`Vec<f32>` may derive capacity from duration.

`BoundedLinkedStnNoiseMorph` is a fresh candidate identity, not a repair of the
deleted checkpoint. Batch 31.45 may implement it once. Creative quality,
`16x`, residual image, component leakage, entry/tail character, and cost remain
unproved until the full gate order runs.

## Batch 31.64 Simpler-Owner Decision

Linked STN later closed without acoustic evidence after repeated executable-
authority failure. The commissioned simpler-owner study found no unused fifth
family. CDP owns the separate vocoder-like `Spectral` target, waveform overlap
owns closed `Cyclic`, coherent phase propagation owns a different
source-readable character, and image or learned resynthesis adds unresolved
full-file, model, stereo, or cost ownership.

Direct PaulX-style magnitude renewal remains the smallest source-backed owner
of neutral `Dream`. It is not a new family. Batch 31.65 records the operator's
explicit product-gate reset because previous Signal renewal checkpoints are
closed. The reset retains hard integrity, level, long-form mono, and
independent stereo listening while making exact creative pitch,
impulse-region, local-image, and non-zero-`space` sample-algebra measurements
diagnostic.

Canonical decision:
[Offline Creative Direct-Renewal Owner Study](./offline-creative-direct-renewal-owner-study.md).

Complete candidate authority:
[Offline Creative DirectRenewalDream Renderer Brief](./offline-creative-direct-renewal-dream-brief.md).

## Batch 31.66 Candidate Decision

Immutable checkpoint `760da32d` passed two clean conformance rounds, all `88`
synthetic rows, concealed mono `15/15`, all `45` long-form stereo hard rows,
all `15` `space` trios, and all `1400` mapped diagnostics. Mono retained a
minor material-dependent entry/tail-envelope caveat. Stereo retained one large
but comparator-adjacent local `16x` full-mix diagnostic.

The operator accepted the stereo effect on speakers and explicitly removed
eligible independent review as a requirement for this fixed-ratio effect.
Contract `085` scopes that decision to checkpoint `760da32d`, records the
operator's one-ear limitation, and makes no independent-listening claim. The
candidate passes. Broader routing and product work remain closed.

## Batch 31.67 Admission Decision

Signal now contains the exact private fixed-ratio renderer, request, regression
owners, diagnostic schemas, and internal engine version. The four acoustic
implementation files remain byte-identical to checkpoint `760da32d`.
Construction passed `1/1`, structural passed `10/10`, and synthetic passed
`88/88` rows with `76/76` renders. Retained row evidence matches the checkpoint
after identity labels are excluded.

This is an unrouted internal admission, not product exposure. Exact `4x`, `8x`,
and `16x` are the only supported ratios. The paused lower overlap, other
characters, dynamic ratio, routing, cache, and consumer work remain separate.

## Batch 31.68 Lower-Overlap Decision

No complete `2x..4x` overlap architecture exists without changing an admitted
renderer. `OfflineHighQuality` accepts arbitrary positive fixed ratios, maps
target length as `round(input length * ratio)`, and owns a centered padded STFT
frame lattice with normalized overlap-add and exact crop. `DirectRenewalDream`
accepts only exact `4x`, `8x`, and `16x`, maps each synthesis-frame center
directly to one source position, and owns a different long-window lattice with
short head guard, long tail release, and exact-zero endpoints.

The mandatory exact `2x` and interior probes therefore have no `Dream` render.
Exact target length and determinism at `4x` do not create a shared scheduler or
boundary law. A hard switch at `4x` supplies no overlap band. Post-resampling a
`4x` Dream render changes pitch or the source/event map. A second stretch pass
introduces an unowned third timeline. An output crossfade cannot satisfy
Contract `085` Rule 4 when one owner cannot render the interior ratios and the
two owners do not share frame or exterior-boundary ownership.

The lower overlap stays paused. Neither renderer is rejected or changed.
Reopening requires either one complete lower creative owner that covers every
required `2x..4x` ratio on the Contract `085` map, or a separately versioned and
re-admitted generalized Dream renderer. An adapter, exact-`4x` blend, scalar
sweep, or post-process is insufficient.

## Batch 31.69 LayeredCloud Decision

Pinned Csound `sndwarpst` supplies the complete source-backed architecture
family: one pointer drives bounded overlapping unit-rate grains and the stereo
path shares its schedule and windowing. SuperCollider `Warp1` confirms the
family but owns randomized active grains per output channel, so it is not the
linked-stereo authority. Signal freezes its own map, launch lattice, grain
durations, counter addresses, validity normalization, exterior boundary law,
memory ceiling, synthetic gates, comparator capture, and listening policy.

The complete authority is
[Offline Creative LayeredCloud Renderer Brief](./offline-creative-layered-cloud-brief.md).
It supports every fixed ratio from `16x` through `100x`, including the Cloud
side of future upper-overlap probes. The upper overlap remains paused because
admitted `DirectRenewalDream` supports exact `16x` but no interior ratio.
Batch 31.70 was authorized to implement the Cloud brief once in its named
isolated worktree.

Batch 31.70's construction audit then stopped before source creation. The
brief admitted sub-hop sources that its unit-rate, zero-padded grain geometry
cannot cover without a zero denominator. Cloud now requires `L>=H`, where
`H=round_half_up(F/64)`, and rejects shorter non-empty input before allocation.
This was a request-boundary correction, not a scheduler or sound change. The
isolated worktree resumed under the corrected brief.

Batch 31.70 subsequently passed two unchanged complete conformance rounds and
froze checkpoint `ee42f50c4c338db4af8a7feaa89bb8b21e8d0860`. Its apparent
`Y01..Y05` green result is invalid: the compiled `Y05` helper omitted the
frozen three-band and mapped-window natural-stereo diagnostics. No comparator
or listening stage opened, so the pointer-led Cloud topology has neither a
quality pass nor a quality rejection. Batch 31.71 owns the docs-only evidence-
integrity decision; the checkpoint cannot be repaired or rerun.

Batch 31.71 found the executable gap broader than `Y05`: incomplete spec and
construction manifests, no tracked runner profile or enforceable row deadline,
missing structural vectors and algebra, false stereo frame counts, incomplete
Y02-Y05 diagnostics, and no comparator or listening owner. None is acoustic
evidence against the renderer.

The selected pointer-led topology remains the smallest source-backed owner of
the Cloud range and has no prior invalid-evidence identity or quality result.
One fresh docs-first `AuditedLayeredCloud` identity is justified under Contract
`085`. Batch 31.72 freezes that authority only. It cannot reuse checkpoint
source or output, change renderer behavior from acoustic evidence, or open
implementation before every executable edge is frozen. A second evidence-
integrity failure closes the family.

## Batch 31.73 Cloud Closure

The fresh source-clean candidate compiled and passed construction `1/1`, then
stopped before structural admission. Strict `2|q|<D` with `D<=20H` permits at
most `20` regular launches plus one distinct terminal. The frozen required
maximum `22` is unreachable; `21` is the exact bound.

No checkpoint, acoustic ref, synthetic output, comparator output, or listening
pack exists. This is Cloud's second evidence-integrity failure, so Contract
`085` closes the pointer-led family without another rebinding. It is not an
acoustic judgment.

## Batch 31.74 High-Range Decision

Signal's current executable creative envelope narrows to the admitted exact
fixed `4x`, `8x`, and `16x` neutral `Dream` renderer. The operator accepted
that effect. Exact `16x` is an endpoint, not a continuous-range or routing
claim.

The former `16x..100x` target is deferred research intent, not active work.
The complete-owner audit found no unused fifth family. Spectral, cyclic,
coherent, STN, image, and learned alternatives either own another character,
are closed, or lack a complete source-backed Signal boundary. Cloud cannot be
repaired or renamed into another candidate, and its unjudged output cannot
select one.

No implementation or further owner study is ready. Both overlaps, automatic
routing, dynamic ratio, public controls, cache, and product exposure remain
paused or absent. Reopening above `16x` requires explicit operator authority
and one materially different complete owner frozen docs-first.

## Batch 31.75 Public Surface Decision

The accepted fixed-ratio effect can cross the public crate boundary without
claiming the abandoned router. The frozen surface is
[Offline Creative Fixed-Ratio Public Surface](./offline-creative-fixed-ratio-public-surface.md).

The API is separate from `TimeStretcher`, `StretchBackendTier`, and
`OfflineHighQualityPath`. It accepts mono or interleaved stereo, sample rate,
exact target frames resolving to `4x`, `8x`, or `16x`, fixed `Dream`, and
normalized `space`. It is offline, whole-buffer, allocating, fallible, and
deterministic.

The public wrapper uses the admitted fixed seed. `motion`, `detail`, seed,
pitch, reverse, dynamic ratio, routing, cache, artifacts, runtime DTOs,
Loophole, and Chorus stay absent. The current transparent cache identity must
not identify creative output.

Batch 31.76 admits that wrapper. The four acoustic renderer files remain
byte-identical, public output matches the private renderer byte-for-byte, and
all retained construction, structural, and synthetic gates pass.

## Batch 32.26 Cyclic Private Admission

The recovered event-ledger renderer passed two `340/340` structural rounds,
`183/183` synthetic rows, `5/5` pre-allocation exact-`16x` rejection rows,
`45/45` long-form mono renders, and `15/15` linked-stereo renders at immutable
checkpoint `bab6ce96b0476e025dce5c957d91eab27e375fd6`.

The operator judged the concealed mono and speaker-stereo outputs hard to
distinguish from the comparator, similar, and solid. After all hard stereo
controls passed, the operator explicitly waived independent review for this
exact creative renderer. The one-ear limitation and candidate-scoped nature
of that waiver remain recorded in Contract `085`.

Commit `81edaada` admits the unchanged acoustic core privately as
`creative_cyclic`. It supports mono and linked stereo, exact `2x`, `4x`, and
`8x`, and one fixed render-wide cycle in `5..90 ms`; `48 ms` remains the
reviewed neutral point. No public request, automatic cycle selection, routing,
cache, artifact, UI, runtime, Loophole, or Chorus surface changed.

## Batch 32.27 Public Cyclic Surface

The existing fixed-ratio public API now freezes a source-compatible Cyclic
extension. It adds the `Cyclic` character, a separate exact `2x`/`4x`/`8x`
ratio list, optional `Duration` cycle in `5..90 ms`, a `48 ms` default,
character-control rejection, and deterministic microsecond canonicalization.
Public behavior identity advances to `signal-creative-stretch-v2` while Dream
output remains unchanged.

This is docs authority only. Public dispatch, cache, routing, artifacts,
runtime integration, Loophole, and Chorus remain unchanged.

## Batch 32.28 Public Cyclic Admission

Commit `e8948512` implements the frozen extension in `creative.rs` and
`lib.rs`. Dream remains byte-identical. Public Cyclic matches the admitted
private renderer for mono and linked stereo at every supported ratio and the
`5 ms`, default `48 ms`, and `90 ms` cycle anchors. Ten focused public tests
pass.

Automatic cycle selection, continuous ratios, routing, cache, artifacts,
runtime integration, Loophole, and Chorus remain unavailable.

## Batch 32.29 Lane Closeout

`g10.032` closes with two explicit public characters:

- Dream at exact `4x`, `8x`, and `16x`
- Cyclic at exact `2x`, `4x`, and `8x`

This is not continuous `2x..16x` coverage. Cyclic's commanded repetitions
cannot silently replace Dream's smooth smear at a ratio boundary. Character
selection remains a user decision; hidden range-owner selection is valid only
inside one character.

`g10.033` takes the prerequisite next step. It audits whether the admitted
mechanisms share enough map, schedule, boundary, normalization, stereo, and
deterministic-state ownership to support interior ratios or a same-character
overlap. It must select one complete source-backed direction or retain the
fixed-ratio surface. No implementation is ready through this closeout.

## Batch 33.1 Continuous-Range Compatibility Audit

### Executable coverage

`N` is the source frame count and `T` is the exact target frame count.

| Owner | Semantic lane | Public acceptance | Private acceptance | Admitted acoustic evidence |
| --- | --- | --- | --- | --- |
| `OfflineHighQuality` | `Transparent` | any finite positive fixed ratio; `T=round(N*r)` | same | frozen competitive baseline; not creative Dream evidence |
| `DirectRenewalDream` | `Dream` | `T` exactly `4N`, `8N`, or `16N` | same | exact `4x`, `8x`, and `16x` |
| event-ledger Cyclic | `Cyclic` | `T` exactly `2N`, `4N`, or `8N` | any `N <= T <= 8N`; identity is exact | exact `2x`, `4x`, and `8x` only |

Private acceptance is not promotion. Cyclic's current plan already has
frame-resolution ratio geometry through `8x`, but no interior acoustic or
public evidence exists. Dream's plan rejects every interior target before
allocation even though its map and synthesis equations take `T` directly.

### Ownership compatibility

| Boundary | `OfflineHighQuality` | `DirectRenewalDream` | event-ledger Cyclic |
| --- | --- | --- | --- |
| source map | fixed analysis lattice plus ratio-scaled synthesis hop | half-sample rational map evaluated at synthesis-block starts | the same half-sample rational ideal map, with two unit-rate local reads |
| scheduler | centered padded `2048/512` STFT; optional `1024/256` selector | sample-rate-scaled long FFT, fixed half-window output hop, two-frame blend | fixed user cycle, two compressed anchors, one output-sample loop |
| phase or sample state | tracked peak regions, phase recurrence, qualified transient reset | deterministic per-frame spectral phase renewal | no stochastic or spectral state |
| boundary | normalized overlap-add, centered crop; short input uses linear fallback | short head envelope, longer tail envelope, exact-zero endpoints | zero-valued exterior interpolation, direct exact crop, no Dream envelope |
| linked stereo | common mid/side schedule | common phase field plus symmetric `space` rotation | identical anchors, weights, and interpolation for every channel |
| normalization | samplewise overlap normalization | energy-compensated two-frame blend and fixed window gain | complementary raised-cosine weights |
| deterministic state | deterministic recurrence | fixed admitted seed and address law | deterministic, no seed |
| working bound | fixed transform state excluding source/output | `32 MiB` ceiling excluding source/output | `256 KiB` ceiling excluding source/output |

Dream and Cyclic use the same ideal half-sample source coordinate:

`((2o + 1)N - T) / (2T)`.

They do not share a scheduler, synthesis representation, boundary envelope, or
character. That common coordinate is useful structural evidence, not blend
authority.

### Contract 085 Rules 1-7

1. Each owner has one monotonic map. Only Dream and Cyclic express the exact
   target-frame map directly. `OfflineHighQuality` derives target length from a
   floating ratio and owns a different lattice.
2. Every current owner is deterministic. No routing version or overlap state
   exists.
3. Dream, Cyclic, and Transparent remain different user intent. Hidden
   Dream/Cyclic or Transparent/Dream substitution would violate the stable
   semantic vocabulary.
4. No current two-owner transition is seamless. The `2x..4x` and `16x..32x`
   overlaps remain paused. One Dream owner across `4x..16x` needs no internal
   transition.
5. All three owners make linked-channel decisions, but through incompatible
   representations. A channel-shared mix weight alone does not create shared
   synthesis ownership.
6. Dream retains the admitted fixed seed. Cyclic has no stochastic state.
   Ratio widening changes behavior identity even when anchor output stays
   byte-exact.
7. Every owner has exact output and bounded working state within its own
   contract. Interior Dream ratios still need explicit structural, synthetic,
   boundary, memory, and listening admission.

### Decision

Select one separately versioned `ContinuousDirectRenewalDream` candidate for
every exact fixed target satisfying `4N <= T <= 16N`.

The candidate must:

- keep the admitted transform, hop, source-centre equation, phase addresses,
  seed, `space` law, blend compensation, envelopes, memory ceiling, and
  synthesis code unchanged
- replace only the private exact-anchor ratio gate inside its isolated
  candidate identity
- keep exact `4x`, `8x`, and `16x` output byte-identical
- prove non-power-of-two interior targets, frame-adjacent endpoint probes, and
  target lengths not divisible by the synthesis hop
- run complete synthetic and concealed long-form mono/stereo admission at
  frozen interior ratios before any public widening
- advance engine and evidence identity if promoted

This is source-backed by PaulXStretch's one fractional source accumulator and
one renewal path across ratios. It is not a claim that interior Signal output
already passes.

Do not select a coherent/Dream blend. `OfflineHighQuality` remains
Transparent, and the `2x..4x` lower Dream gap remains open. Do not widen public
Cyclic in the same candidate. Its private continuous geometry is a separate
later admission choice. Cloud, ratios above `16x`, automatic routing, dynamic
ratio, cache, artifacts, and consumers remain unavailable.

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
- [Csound `sndwarpst` manual](https://csound.com/manual/opcodes/sndwarpst/)
- [Csound pointer and balance guidance](https://csound.com/manual/opcodes/sndwarp/)
- [Pinned Csound implementation](https://github.com/csound/csound/blob/0eaa07e3aee55f90e745f89294ddb52eec30345c/Opcodes/sndwarp.c)
- [SuperCollider `Warp1` manual](https://docs.supercollider.online/Classes/Warp1.html)
- [Pinned SuperCollider implementation](https://github.com/supercollider/supercollider/blob/2f0803bcd2e551564e3fef8d5075816cbb685cd4/server/plugins/GrainUGens.cpp)
- [Photosounder](https://www.photosounder.com/)
- [ARSS](https://arss.sourceforge.net/)
- [Noise Morphing for Audio Time Stretching](https://arxiv.org/abs/2312.14586)
- [Enhanced Fuzzy Decomposition of Sound Into Sines, Transients, and Noise](https://arxiv.org/abs/2210.14041)
- [SiTraNoStar classical STN/noise-morphing source](https://github.com/ollpu/SiTraNoStar/tree/2edf7b693040b5070116299973abf83dc5ba86e5)
- [Signalsmith Stretch source study](../research/specimen-dossiers/signalsmith-stretch.md)
- [Bungee source study](../research/specimen-dossiers/bungee-source-architecture.md)
- [Rubber Band source study](../research/specimen-dossiers/rubber-band-source-architecture.md)
- [SBSMS source study](../research/specimen-dossiers/sbsms-source-architecture.md)
- [TSM-Net paper](https://arxiv.org/abs/2210.17152)
- [TSM-Net public inference repository](https://github.com/tsmnet-mmasia23/tsmnet)
- [AudioLabs TSM toolbox](https://www.audiolabs-erlangen.de/resources/MIR/TSMtoolbox/)
- [Schroeder, low-peak-factor phase selection](https://doi.org/10.1109/TIT.1970.1054411)
- [Yang et al., arbitrary-spectrum crest minimization](https://pubmed.ncbi.nlm.nih.gov/25832418/)
- [Schreiber and Schmitz, IAAFT surrogate data](https://doi.org/10.1103/PhysRevLett.77.635)
- [PaulXStretch official repository](https://github.com/essej/paulxstretch)
- [Extreme Audio Time Stretching Using Neural Synthesis](https://arxiv.org/abs/2211.16992)
- [Verhelst and Roelands, WSOLA](https://doi.org/10.21437/Eurospeech.1993-59)
- [SoundTouch algorithm notes](https://soundtouch.surina.net/README.html)
- [SoundTouch source, studied revision `f738b113`](https://codeberg.org/soundtouch/soundtouch/commit/f738b1132ec1fd56efc90367898244cf52d9e6a5)
- [Moulines and Charpentier, PSOLA](https://doi.org/10.1016/0167-6393(90)90021-Z)
- [Rudresh et al., ESOLA](https://arxiv.org/abs/1801.06492)
- [Roberts and Paliwal, FESOLA](https://doi.org/10.1109/WASPAA.2019.8937258)

## Next Task

`g10.034` is active. Execute Batch 34.1 only: audit whether the admitted
Cyclic owner preserves character at interior fixed targets and select an exact
domain or close the lane. Keep lower Dream, routing, and implementation closed.
