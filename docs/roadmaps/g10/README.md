# g10 Milestones

Status: `g10.035` active; Automatic exact replay Batch 35.5 ready
Updated: 2026-07-25

## Why this generation matters now

`g10` started as the 2026-06-11 audit-remediation generation: protect the real
audio path, remove simulated or narration-heavy surfaces, and rebuild only what
Signal needs as reusable runtime or DSP substrate.

Phase three added first-party stretch work after the operator chose a
Signal-owned time-stretch and pitch-shift engine rather than a Rubber Band
dependency. Signal can use external tools as clean-room benchmarks, but the DSP
implementation remains Signal-owned.

## Generation Runway

The 2026-07-19 consolidation reset is authoritative.

- The current OfflineHighQuality renderer, compression selector, and expansion
  selector are frozen as the competitive baseline.
- Rejected research implementations and candidate report surfaces were removed.
- `g10.029`, Contract `082`, and the detailed architecture ledger remain
  historical evidence. Batch 29.7BE is cancelled.
- `g10.030` and Contract `084` completed the successor decision. The first
  isolated end-to-end successor passed structural controls but failed
  anti-replica admission and was deleted. The event-sealed replacement then
  failed structural feasibility before implementation: its frozen impulse
  refinement is always `15` samples early. Its untouched worktree was deleted,
  the multiresolution phase-vocoder successor family is closed, and no
  candidate code entered `main`. A final non-phase-vocoder study found no
  family with a source-backed path through all whole-renderer gates. The
  program closed on the frozen production baseline.
- `g10.031` and Contract `085` open a separate offline creative-stretch path
  centered on `8x`. Batch 31.1 froze the product controls, range router, and
  comparator study without reopening the transparent successor lane. Batch
  31.2 captured the accessible comparator pack and froze a PaulX-centred
  parameter space with explicit spectral, rough, and cyclic character anchors.
  Batch 31.3 froze one complete `DiffuseSpectral` renderer brief without adding
  DSP or candidate surfaces to `main`. Batch 31.4 implemented that brief in a
  disposable worktree. Structural controls passed, but neutral `Dream` at
  `4x` raised deterministic-noise crest factor by `7.08 dB` against the frozen
  `6 dB` ceiling. The candidate was deleted before listening. Batch 31.5 kept
  the ceiling, closed independent-bin diffusion, and froze one complete
  continuous full-complex excitation replacement without changing DSP. Batch
  31.6 implemented it once. Twelve of thirteen structural controls passed, but
  common-polarity covariance missed by `0.0013287`; the candidate was deleted
  before crest admission. Batch 31.7 closed native angle subtraction and froze
  one final direct-complex relation brief. Batch 31.8 implemented it once, but
  coefficient proof exposed incompatible exact anti-phase negation/swap
  expectations and stopped before any renderer row. The candidate was deleted
  and the current diffusive owner closed. Batch 31.9 then paused the automatic
  spectral router, rejected the coherent baseline as a substitute core owner,
  and selected explicit cyclic expansion through `8x` as the narrower next
  promise. Batch 31.10 froze one complete `CyclicGrain` brief without changing
  DSP. Batch 31.11 implemented it once. All structural controls passed, but
  the first synthetic row measured `20.778` cents of pitch error against the
  `15`-cent ceiling. The candidate was deleted without correction or rerun.
  Batch 31.12 then selected one materially different correlation-aligned
  waveform family for a clean-room cyclic brief. Batch 31.13 froze its exact
  map, search, overlap, stereo, bounds, and gates without changing DSP. Batch
  31.14 implemented the similarity-aligned candidate once. Compile-only
  validation passed, but structural known-offset recovery chose source frame
  `6432` instead of the exact continuation at `6352`. The frozen coarse
  shortlist can hide an exact between-grid match from full refinement. The
  candidate was deleted without correction or rerun. Batch 31.15 docs-only
  cyclic ownership reassessment found no third materially different,
  source-backed whole-renderer path. SOLA/WSOLA variants repair the rejected
  search owner; pitch-/epoch-synchronous methods lack a full-mix linked period
  owner and `8x` evidence; other hybrids reopen closed seams or separate
  programs. Explicit `Cyclic` is closed without promotion. Batch 31.16 then
  reopened docs-only research by explicit operator decision and traced pinned
  PaulXStretch, CDP, and Potenza whole render paths. Neutral PaulXStretch uses
  long-window magnitude analysis, per-frame phase renewal, and frame crossfade
  rather than the recurrence and magnitude evolution tested by Signal's
  rejected spectral briefs. This is new source-backed evidence for one
  materially different neutral `Dream` family, `RenewalSpectral`. Batch 31.17
  froze its complete map, transform, phase renewal, linked mid/side law,
  pairwise synthesis, state bounds, comparator-calibrated gates, cleanup, and
  minimal admission without candidate DSP. Batch 31.18 implemented it once.
  Compile-only and structural admission passed, but the first neutral-`Dream`
  crest row measured `8.263162 dB` growth against the frozen `6 dB` ceiling.
  Complete independent phase renewal still left cross-bin waveform summation
  uncontrolled. The candidate was deleted before later synthetic or listening
  gates. Batch 31.19 reconciled that failure with the earlier `7.08 dB`
  `DiffuseSpectral` miss. Independent stochastic bin phase still has no
  intrinsic crest owner, and no materially different source-backed complete
  renderer remained. Batch 31.19 closed neutral `Dream`, but the operator
  superseded that closure: its PaulX crest calibration used long-form musical
  rows against a synthetic Signal stop row, and the matching PaulX synthetic
  suite had not run. Batch 31.20 then captured the matching pinned-core
  synthetics. PaulX uniform-noise crest growth is `9.932 dB` at `4x`, above
  Signal's rejected `8.263162 dB` row. The old `6 dB` ceiling was not
  target-relative. Batch 31.20 froze one complete
  `CompensatedRenewalSpectral` brief with Signal-derived overlap-statistics
  compensation and no candidate DSP. Batch 31.21 implemented it once, but
  compile-only validation failed on an unconstrained structural-test `Option`
  accumulator before the renderer executed. The candidate was deleted without
  correction or rerun. Its DSP remains untested. Batch 31.22 froze
  `VarianceCompensatedRenewalSpectral` as fresh complete authority and made
  compile completion a construction receipt before one-shot structural
  admission. Batch 31.23 compiled and passed seven structural tests. Its
  synthetic command returned green, but pre-listening review found missing
  impulse-train crest, secondary-region, exact-lag autocorrelation, and full
  discontinuity assertions. The receipt is invalid. The unopened listening
  pack and isolated candidate were deleted without repair or rerun. The
  topology remains untested. Batch 31.24 retained it under fresh identity
  `AuditedVarianceCompensatedRenewalSpectral` and froze `22` compile-linked
  gate owners, exact comparator measurements, allocation accounting, and one
  clean candidate boundary. Batch 31.25 passed construction `1/1`, structural
  `13/13`, synthetic `9/9`, and concealed mono listening as `15/15` ties from
  immutable checkpoint `97ee7056`. Linked-stereo review then rejected it: a
  source at `-0.4516 dB` right-minus-left became `+3.3660 dB` at neutral
  `space` because first-sample component orientation discarded the source
  mid/side relationship. The candidate was deleted. Batch 31.26 froze
  `SourceRelativeRenewalSpectral`: the passed mono renderer remains, while
  native left/right complex analysis and one explicit interchannel relation
  law own stereo. Batch 31.27 compiled and passed construction `1/1`, then
  failed structural admission at `14/15` because its frozen `mix64(1)` vector
  was transposed. The candidate was deleted without repair or rerun. Batch
  31.28 reproduced every counter vector independently and froze fresh verified
  authority. Batch 31.29 then passed compile, construction `1/1`, and
  structural `15/15` from immutable checkpoint `d94612dd`. Synthetic admission
  finished `7/9`: one `16x` row split into two replica regions, and two `4x`
  pitch rows measured about `10.96` cents outside their PaulX-relative
  ceilings. The candidate was deleted before listening. Batch 31.30 found that
  neither that brief nor Batch 31.25's passing mono brief froze the candidate
  seed. It withdrew the unsupported range diagnosis and froze
  `SeedAuditedSourceRelativeRenewalSpectral`. Batch 31.31 passed compile,
  construction `1/1`, and structural `15/15`, then failed `Y02` on the `8x`
  chord at `13.351828347` cents against an `11.331375778`-cent ceiling. `Y04`
  passed at all ratios. The candidate was deleted before listening. Two
  checkpoints fail the same tonal-pitch class. Batch 31.32 found that renewal
  has no persistent tonal phase owner and no eligible complete source-backed
  replacement remains. Renewal is closed without closing the PaulX-like
  product target. Other characters, routing, product exposure, and rejected
  branches remain closed or paused. The operator then changed the Contract
  `085` pitch gate: finite PaulX-relative pitch delta is diagnostic and
  concealed listening remains creative authority. Batch 31.33 froze one fresh
  listening-led source-relative candidate without changing DSP. Batch 31.34
  passed construction `1/1` and structural `15/15`, then synthetic `Y08`
  rejected exact-zero impulse hops at all ratios. The candidate was deleted.
  Batch 31.35 classified the complete-output dropout scan as executable
  evidence-construction failure and froze fresh support-audited authority.
  Batch 31.36 passed compile, construction `1/1`, structural `15/15`,
  synthetic `9/9`, and concealed mono as `15/15` ties. Valid same-source
  stereo admission then rejected `16x` local image stability: full-mix mapped
  windows reached `9.37..9.42 dB` balance error with channel-dominance
  reversal. The candidate was deleted. Batch 31.37 is a docs-only
  linked-stereo ownership reassessment. It found no materially different
  source-backed complete renewal owner and closed the family. The PaulX-like
  target remains. The operator then selected comparator-relative creative
  stereo promotion: local mapped-window source balance is diagnostic, while
  structural relationships, whole/band balance, and eligible independent
  listening remain terminal. Batch 31.38 froze one fresh complete
  `ComparatorAuditedRenewalSpectral` brief without candidate DSP. Batch 31.39
  then passed compile, construction `1/1`, and structural `15/15`, but
  synthetic admission finished `7/9`: `Y04` produced a second `16x` active
  replica region, and `Y09` reported linked-stereo swap failure at `4x` and `8x`.
  The candidate was deleted before listening. Because Batch 31.36 passed both
  owners under the nominally same renderer and seed, Batch 31.40 audited the
  evidence authority. It found no retained executable identity: construction
  froze inventory and selected constants, not helper bodies or assertions,
  and `Y09` lacks one canonical source-relative swap assertion. Both receipts
  remain checkpoint-local. Renewal is closed without closing the product
  target.
- The 2026-07-20 lifecycle reconciliation closes stale `g10.001` and
  `g10.003` active markers. It also records that Signal's `g10.017` capture and
  live-monitor implementation landed; that roadmap is paused only on explicit
  hardware alignment and consumer workflow evidence. No feature batch became
  ready through this correction.
- `g10.034` is complete. It admits continuous exact-target Cyclic over
  `2N..=8N` through public v4 after full private acoustic admission and exact
  public/private parity. No isolated candidate state remains.
- `g10.035` owns the current planning checkpoint. Batch 35.3 freezes one
  conformance-complete Automatic checkpoint, then stops at its first acoustic
  owner because byte-exact Transparent and a misplaced universal peak ceiling
  conflict. Batch 35.4 corrects gate ownership without changing candidate
  bytes. One exact replay is ready as Batch 35.5.
- `g10.028` RealtimePreview source-fill and all render-plane integration remain
  paused.

Do not start Loophole or Chorus planning from Signal internals.

## Milestone Map

- `g10.001` `complete`
  - audit adoption and generation open completed
- `g10.002` `complete`
  - render-plane declick and playback correctness
- `g10.003` `complete`
  - output stream hardening, cpal enumeration, and legacy CoreAudio retirement
- `g10.004` `complete`
  - hosting-domain demolition
- `g10.005` `complete`
  - runtime rescope to honest control plane
- `g10.006` `complete`
  - analysis pruning and measurement correctness
- `g10.007` `complete`
  - plugin-domain pruning to real foundations
- `g10.008` `complete`
  - DSP corrections and polyphase resampling
- `g10.009` `complete`
  - workspace consolidation and truthful front doors
- `g10.010` `complete`
  - graph-shaped plans and mixer realization
- `g10.011` `complete`
  - stable node identity and state handoff
- `g10.012` `complete`
  - parameter fast path and automation playback
- `g10.013` `complete`
  - DSP kit: biquads, pan law, limiter, denormals
- `g10.014` `done`
  - RT observability, metering, and callback health
- `g10.015` `complete`
  - WYSIWYG bounce on the render plane
- `g10.016` `complete`
  - output-time honesty and device lifecycle
- `g10.017` `paused`
  - recording capture and live monitoring landed; hardware alignment evidence
    remains an explicit operator gate
- `g10.018` `complete`
  - disk-streaming clip sources
- `g10.019` `complete`
  - transport regions, loop, click, count-in
- `g10.020` `complete`
  - Signal runtime endgame thin control library
- `g10.021` `complete`
  - stretch real corpus and benchmark evidence
- `g10.022` `paused`
  - OfflineHighQuality DSP depth; low-risk sustained candidates evidence-complete
- `g10.023` `paused`
  - stretch offline artifact scale and format depth
- `g10.024` `paused`
  - RealtimePreview stretch tier
- `g10.025` `deferred`
  - stretch product workflow contract checkpoint
- `g10.026` `complete`
  - RealtimePreview callback-safe state
- `g10.027` `complete`
  - RealtimePreview source-projected callback
- `g10.028` `paused`
  - RealtimePreview source fill contract
- `g10.029` `superseded`
  - historical correctness, listening, and rejected-successor ledger
- `g10.030` `complete`
  - stretch consolidated; candidate families closed; frozen baseline retained
- `g10.031` `complete`
  - exact fixed `4x`, `8x`, and `16x` neutral `Dream` is public; broader
    range, routing, cache, and product integration are deferred; historical
    path: Batch 31.36 passed all
    objective and mono gates, then failed source-relative stereo admission and
    was deleted; Batch 31.37 closed renewal after finding no different complete
    owner; Batch 31.38 records the operator's comparator-relative stereo policy
    and freezes one fresh complete candidate; Batch 31.39 rejected that fresh
    candidate at synthetic `Y04` and `Y09`; Batch 31.40 found no recoverable
    executable identity, closed renewed implementation, and retained the
    PaulX-like target; Batch 31.41 selected `LinkedStnNoiseMorph` for one
    complete docs-only brief; Batch 31.42 froze that renderer and its
    self-contained evidence authority; Batch 31.43 rejected its isolated
    implementation at structural `S17` for duration-derived working state;
    Batch 31.44 froze a two-pass bounded schedule and fresh candidate identity;
    Batch 31.45 passed compile but failed construction because the frozen
    first-residual formula exhaustively yields `53248` while its asserted row
    requires `59392`; the candidate was deleted; Batch 31.46 retained the
    per-geometry formula, corrected its maximum to `53248`, and froze fresh
    capacity-audited v3 identity; Batch 31.47 then found `R_v=59` at `F=8000`
    against the frozen `R_v<=57` bound and deleted the candidate before compile
    or checkpoint; Batch 31.48 corrected the exhaustive bound to `59`, retained
    every memory ceiling, and froze geometry-audited v4 identity; Batch 31.49
    passed construction but failed structural `S15` exact silence at `17/18`
    and was deleted before synthetic or listening; Batch 31.50 froze
    zero-preserving v5 across covariance, excitation, recombination, and
    output encoding without changing positive-power behavior; Batch 31.51
    passed construction but failed structural `S02` at `17/18` on an incorrect
    handwritten 8 kHz `Q_h` vector and was deleted before synthetic or
    listening; later Rule 11 work closed linked STN without acoustic evidence;
    Batch 31.64 found no unused fifth family; Batch 31.65 records the
    direct-renewal product-gate reset and freezes one complete
    `DirectRenewalDream` authority; Batch 31.66 passed the complete fixed-ratio
    candidate; Batch 31.67 admitted its private unrouted renderer; Batch 31.68
    retained the lower-overlap pause because Dream has no mandatory interior
    renders or shared coherent scheduler; Batch 31.69 froze one complete
    pointer-led granular `LayeredCloud` authority for fixed `16x..100x`;
    Batch 31.70 receipt is invalid across executable evidence ownership;
    Batch 31.71 authorizes one docs-first `AuditedLayeredCloud` identity;
    Batch 31.72 freezes its complete executable authority and makes one fresh
    isolated implementation ready; Batch 31.73 closes Cloud on contradictory
    occupancy authority; Batch 31.74 narrows executable creative coverage to
    private exact `4x`, `8x`, and `16x` Dream and defers the broader range;
    Batch 31.75 freezes one minimal public `CreativeStretch` wrapper; Batch
    31.76 admits it with byte-identical acoustic output;
    explicit `Cyclic` stays closed
- `g10.032` `complete`
  - deep Cyclic research reopened by operator decision; Batch 32.1 separates
    fixed Akai `CYCLIC` from adaptive `INTELL`, pins Potenza slow-anchor and
    SickoCV repeat/jump schedules; Batch 32.2 completes executable forensics
    without selecting a renderer; Batch 32.3 selects centred
    compressed-anchor behavior and corrects the gate; Batch 32.4 freezes one
    complete candidate and Rule 11 evidence brief; Batch 32.5 seals the
    isolated immutable checkpoint after two nominally clean conformance
    rounds; Batch 32.6 stops at evidence-invalid `Y01`; Batch 32.7 authorizes
    one fresh audited evidence identity; Batch 32.8 freezes a docs-only
    authority; Batch 32.9 finds it non-executable before source; Batch 32.10
    freezes reproducible manifests; Batch 32.11 freezes a clean isolated
    checkpoint after two byte-identical structural rounds; Batch 32.12 stops
    on a split receipt root at the first acoustic row; Batch 32.13 closes the
    identity; Batch 32.14 records the operator correction and authorizes one
    absolute-root replay of the unchanged checkpoint; Batches 32.15-32.17
    restore exact evidence, record a valid Y01 impulse rejection, and select
    sparse event-ledger ownership; Batches 32.18-32.19 freeze and implement
    fresh authority through two conformance rounds; Batches 32.20-32.21 prove
    its post-checkpoint evidence path non-executable and close the family
    under Rule 11; Batch 32.22 deletes all retained candidate state; Batches
    32.23-32.25 recover the acoustically unjudged implementation, repair its
    evidence path, and complete operator admission; Batches 32.26-32.28 admit
    the unchanged private and public exact-ratio renderer; Batch 32.29 closes
    the lane and selects continuous-range feasibility
- `g10.033` `complete`
  - Batch 33.1 audits exact coverage and owner compatibility, then selects one
    separately versioned `ContinuousDirectRenewalDream` direction over exact
    targets `4N <= T <= 16N`; Batch 33.2 freezes its complete implementation
    and evidence authority; Batch 33.3 admits the private continuous owner
    after complete objective and listening passage; Batch 33.4 freezes direct
    public `4N..=16N` Dream coverage with no same-character router; Batch 33.5
    admits that public surface; Batch 33.6 publishes the exact executable
    matrix and closes the lane; no cache, artifact, or consumer work is ready
- `g10.034` `complete`
  - Batch 34.1 selects `ContinuousEventLedgerCyclic` over every exact target
    `2N <= T <= 8N` through the unchanged admitted equations; Batch 34.2
    freezes its complete implementation and evidence authority; Batch 34.3
    passes and admits the private owner; Batch 34.4 freezes public v4; Batch
    34.5 admits the two-file wrapper with `12/12` public and `10/10` private
    focused tests; Batch 34.6 publishes the matrix and closes the lane
- `g10.035` `active`
  - Batch 35.1 audits current coverage, keeps Cyclic explicit, and selects one
    opt-in Transparent/Dream Automatic intent over exact `0.5N..=16N` with a
    `4N..=8N` transition; Batch 35.2 freezes the complete private route;
    Batch 35.3 produces one evidence-invalid checkpoint; Batch 35.4 corrects
    peak-gate ownership without candidate tuning; Batch 35.5 is ready as one
    exact replay; public work remains conditional

## Stretch Boundary

Current status:

- `Repitch`: implemented realtime-safe varispeed.
- `RealtimePreview`: bounded prototype with callback-state and allocation proof;
  direct callback support remains closed pending `g10.028`.
- `OfflineHighQuality`: the retained `2048/512` phase-vocoder baseline and two
  promoted short-window selectors are stable and fully regression-tested.
- Comparator evidence remains Signal-versus-external objective rows plus the
  five-family, three-ratio long-form blind listening pack.
- Rejected hybrid, H/R/P, phase-gradient, frequency-adaptive, tail, timeline,
  peak, and local phase variants are not active code or planning authority.

The retained OfflineHighQuality baseline is the only product-routed stretch
renderer. Contract `084` and `g10.030` are closed without promotion. A new
successor requires the whole-system evidence listed in the non-phase-vocoder
feasibility decision.

The separate public `CreativeStretch` path exposes exact fixed `4x`, `8x`, and
`16x` neutral `Dream` through the internal `DirectRenewalDream` renderer. It
has no automatic/product route, creative cache, artifact integration, dynamic
ratio, or wider range. Its automatic spectral route is paused after three
rejected and deleted candidates. The first
explicit cyclic candidate is also rejected and deleted after structural
admission and a first-row synthetic pitch miss. Correlation-aligned waveform
overlap also failed structural admission and was deleted. `RenewalSpectral`
passed structural admission but failed its first crest row and was deleted.
Matching PaulX synthetics later showed that the old absolute crest ceiling did
not describe the preferred reference. The complete compensated-renewal brief
was implemented once, but its compile-only validation failed before renderer
execution and the candidate was deleted. The fresh variance-compensated
implementation later produced an invalid synthetic receipt because required
assertions were absent. It was deleted before listening. The compensated-
renewal topology now has valid structural, synthetic, and mono listening
evidence. `AuditedVarianceCompensatedRenewalSpectral` was rejected and deleted
after its first-sample mid/side orientation law inverted source-relative
channel balance. Batch 31.26 froze the complete
`SourceRelativeRenewalSpectral` successor without adding DSP. Its native
left/right analysis preserves channel magnitudes and owns the interchannel
phase relation directly. Its first isolated checkpoint passed compile and
construction, then stopped at structural exact-vector proof because the
frozen `mix64(1)` assertion transposed the normative result. No synthetic or
listening result exists. The candidate was deleted.
Batch 31.28 independently reproduced the complete counter table in Python and
Ruby and froze fresh verified authority. Batch 31.29 passed construction and
all `15` structural owners, then failed two synthetic owners. Batch 31.30
corrected unfrozen seed authority. Batch 31.31 passed construction and all
structural owners under the audited seed, cleared the replica row, then failed
the `8x` chord pitch row. The isolated candidate was deleted before listening.
Batch 31.32 found no eligible complete renderer with both intrinsic tonal
coherence and the retained high-ratio diffusive character. Renewal is closed;
the PaulX-like target and comparator evidence remain.
The operator then made finite PaulX-relative pitch delta diagnostic rather than
terminal for creative `Dream`. Batch 31.33 froze a fresh source-relative
candidate with every hard, replica, level, boundary, and stereo gate retained.
At that checkpoint, creative stretch still had no renderer, public API,
harness surface, or product route on `main`. Batch 31.67 later admitted only
the private exact-ratio renderer.

The support-audited successor later passed every objective and concealed mono
gate. Its valid exact-source stereo gate preserved whole and band balance but
lost local mapped-window dominance at `16x`, reaching `9.418990 dB` error on
full mix. It was rejected and deleted before speaker or independent stereo
review. This repeats renewal's linked-stereo failure class at a different
scale, so the next work is architecture reassessment rather than another
relation-law variant.

Batch 31.37 completed that reassessment. The current-frame common rotation is
already present; independent renewal between adjacent frames leaves local
waveform interference unowned. Source-backed temporal corrections select
closed coherent/peak/oscillator families. Renewal is closed without closing
the PaulX-like target under the old stereo policy.

The operator then selected comparator-relative creative stereo promotion.
Batch 31.38 kept structural stereo relationships and whole/band balance hard,
made mapped local source balance a complete diagnostic, and required an
eligible independent listener for final image promotion. Batch 31.39's fresh
checkpoint passed construction and structural admission, then failed the
`16x` replica row and linked-stereo swap at `4x` and `8x`. It was deleted
before listening. Batch 31.36 passed those same owners under the nominally
same frozen renderer and seed. Batch 31.40 found that the briefs retained exact
seed, support, and inventory authority but not helper bodies, assertions,
per-row receipts, or output digests. `Y09` has no canonical executable
source-relative swap assertion. Neither deleted checkpoint can prove the
other. A third renewal candidate would create new authority rather than
reconcile evidence, so renewal is closed. The PaulX-like target remains.

Batch 31.41 executes the operator-authorized complete-owner study. Pinned
SiTraNoStar `v2.0.1` supplies a runnable classical sines/transients/noise plus
noise-morphing whole path, while the related papers supply reconstructing
two-stage decomposition, `4x`/`8x` listening, transient relocation, envelope,
and limited stereo evidence. The source is not production-ready: it is
GPL-3.0 clean-room evidence, mono-only, nondeterministic, full-file,
approximate-length, and not demonstrated at `16x`.

The study selects `LinkedStnNoiseMorph` for one complete clean-room brief.
Tonal peaks keep persistent linked phase, transient waveform events move once,
and only the separated residual uses continuous deterministic noise morphing.
One shared map, channel-symmetric decomposition, explicit residual spatial
owner, exact crop, bounded state, and the retained long-form listening order
remain mandatory.

Batch 31.42 freezes that complete system. The candidate uses
sample-rate-normalized long/short reconstructing masks, persistent linked
tonal state, one-shot native transient events, continuous covariance-shaped
residual excitation, mapped envelope correction, normalized WOLA, exact crop,
and a `96 MiB` duration-independent state cap. One compile-linked `28`-owner
specification and checkpoint/tree/file/output digests prevent another
multi-hop evidence-identity gap. No DSP or product surface entered `main`.

Batch 31.43 implemented the authority once in the named disposable worktree.
Compile and construction `1/1` passed. The immutable checkpoint then completed
structural admission at `17/18`: `S17` rejected full-duration component and
spectral arrays because working state must remain duration-independent under
`96 MiB`. Synthetic and listening gates did not open. The candidate and its
branch, checkpoint reference, tests, and worktree were deleted without repair
or rerun. No candidate code entered `main`.

Batch 31.44 found one feasible bounded schedule. A decomposition/event prepass
resolves the first-non-zero residual orientation scalars, then a clean render
pass advances fixed spectral, component, event, covariance, envelope, and
output rings. Every state has geometry-derived capacity and a monotonic last
consumer. The owned-state design ceiling is `89 MiB`; the terminal actual cap
remains `96 MiB`. Fresh `BoundedLinkedStnNoiseMorph` authority is frozen. No
DSP or product surface entered `main`.

Batch 31.45 passed compile, then construction failed `0/1` before checkpoint.
The frozen first-residual formula exhaustively reaches `53248`, while its
frozen maximum row requires `59392`. The larger row combines the global
`R_h=19` from a small transform with maximum transform geometry. No authority
was repaired and no structural, synthetic, or listening gate opened. The
candidate worktree, branch, source, tests, and build state were deleted.

Batch 31.46 retained the current-geometry formula and corrected its exhaustive
maximum to `53248`. The conservative short/source model becomes `9.700 MiB`;
the `12 MiB` category ceiling, `89 MiB` design sum, `7 MiB` reserve, and
`96 MiB` actual gate remain unchanged. Fresh
`CapacityAuditedBoundedLinkedStnNoiseMorph` identity was marked ready for one
isolated implementation. No DSP or product surface entered `main`.

Batch 31.47 found that identity was not executable authority. At `F=8000`,
the frozen short vertical formula and upward odd midpoint rule produce
`R_v=59`, contradicting the required exhaustive `R_v<=57` bound. Compile and
all gates stayed closed. The worktree, branch, private source, and tests were
deleted. This is an authority failure, not renderer-quality evidence.

Batch 31.48 independently reproduced `Q_h=17`, `Q_v=97`, `R_h=19`, and
`R_v=59` over the full supported-rate range. `Q_v` already dominates the
shared `97`-scalar median scratch, so no ring, packed model, category ceiling,
or cost class changes. Fresh `GeometryAuditedBoundedLinkedStnNoiseMorph`
identity is ready for one isolated implementation. No DSP or product surface
entered `main`.

Batch 31.49 passed compile and construction `1/1`, freezing checkpoint
`e2ef62f8` and tree `85dc0e45`. Structural admission stopped at `17/18`:
`S15` found deterministic residual output around `1e-14` for exact silence.
The frozen residual rule interpolates `ln(power+eps)` at zero endpoints while
the boundary rule requires bit-exact zero. Synthetic and listening stayed
closed. The candidate worktree, branch, checkpoint reference, source, tests,
build state, receipt, and outputs were deleted without repair or rerun. No DSP
or product surface entered `main`.

Batch 31.50 freezes the exact-zero state missing from v4. Two zero residual
power endpoints now produce canonical positive zero through diagonal
interpolation, coherence, mono and mid/side excitation, mapped-envelope
recombination, and final `f32` encoding. Mixed and positive power retain the
v4 formula. No threshold, mask, variable path, allocation, stochastic change,
memory change, or quality-gate relaxation enters. Fresh
`ZeroPreservingGeometryAuditedBoundedLinkedStnNoiseMorph` authority is ready
for one isolated implementation. No DSP or product surface entered `main`.

Batch 31.51 passed compile and construction `1/1`, freezing checkpoint
`95909451` and tree `080bea36`. Structural admission stopped at `17/18`:
`S02` asserted `Q_h=5` at 8 kHz, while the frozen formula and renderer produce
`odd(round(0.240*8000/256))=9`. The construction owner checked maxima but did
not cross-check that exact structural vector. Synthetic and listening stayed
closed. The candidate worktree, branch, checkpoint reference, source, tests,
build state, receipt, and outputs were deleted without repair or rerun. No DSP
or product surface entered `main`.

Batch 31.52 independently reproduced all `184001` supported-rate geometry
rows with two exact-integer evaluators. Transform transitions, every tie class,
extent maxima and witnesses, binary table fingerprints, and all
geometry-derived capacity maxima and witnesses agree. One compile-linked
`GEOMETRY_SPEC` is now the only literal geometry table; construction and
structural `S02` call the same exhaustive authority assertion. Fresh
`ConstructionBoundZeroPreservingLinkedStnNoiseMorph` v6 is ready for one
isolated implementation. No DSP or product surface entered `main`.

Batch 31.53 started from exact `main` head `fdad8432`, passed compile and
construction `1/1`, and froze checkpoint `366ac24b` with tree `68da7e43`.
Structural admission stopped at `16/18`: `S06` returned an extra equal-plateau
peak, and `S18` found forbidden `pub fn` source. Synthetic and listening stayed
closed. The checkpoint was not repaired or rerun. Its worktree, branch,
reference, private source, tests, and `3.4 GiB` build state were deleted. No
DSP or product surface entered `main`.

Batch 31.54 audited all `S01..S18` owners. Construction honestly covers only
fixed manifest, geometry, vector, primitive, and memory facts; most rows need
an executing renderer. Moving `S18` earlier is valid, but moving `S06` and the
remaining runtime rows makes construction duplicate structural admission or
moves the checkpoint. A locally corrected v7 is therefore protocol churn, not
a different renderer. After six implementation attempts and no synthetic or
listening evidence, `LinkedStnNoiseMorph` closes under Contract `084` Rule 7.
The PaulX-like neutral `Dream` target remains active and unadmitted.

Batch 31.55 freezes Contract `085` Rule 11. A working implementation may
iterate on compile, construction, and structural conformance against already
frozen authority. Only a clean tree passing all three becomes one immutable
acoustic checkpoint. Synthetic, concealed mono, and independent stereo then
run once in order and remain terminal. A local-only evidence ref retains exact
source and tests through reassessment without admitting rejected code to
`main`. Closed conformance-only families may receive one explicit docs-only
eligibility decision; acoustically rejected families remain closed.

Batch 31.56 classifies every closed owner under that rule. Diffusive spectral
and cyclic lineages reached synthetic rejection; their conformance-only
successors are superseded or contain architecture-level misses. Renewal reached
synthetic and concealed-mono admission before stereo rejection, with a later
checkpoint also failing synthetic admission. They remain closed. Linked STN
alone remained pre-acoustic with complete current authority across all six
attempts. Its material-separated brief and pinned source backing remain intact,
so it was selected once for fresh protocol binding. Batch 31.57 now freezes
`ConformanceBoundLinkedStnNoiseMorph`, exact isolated identity, complete
evidence authority, iterative conformance, and one later immutable acoustic
checkpoint. No DSP entered `main`. Batch 31.58 stopped before acoustic identity
when reconstructed impulse refinement exposed a missing numerical tie rule.
Batch 31.59 froze four-ULP earliest ownership, but Batch 31.60 proved it
incomplete on the frozen `0.65` train event and stopped pre-acoustic. Batch
31.61 froze one transform-bounded scale-relative rule. Batch 31.62 then passed
two complete conformance rounds and froze one exact acoustic checkpoint. Its
one-shot synthetic command stopped in the first selected owner, `Y09`, after
about `59` minutes of compute without a completed-owner result. Batch 31.63
then proved executable `Y09` omitted four canonical hard paths, construction
did not bind assertions to owners, and the unoptimized monolithic gate had no
frozen execution envelope or incremental receipt. The result is invalid
evidence, not an acoustic rejection or release-profile cost result. Repeated
incomplete executable authority closes linked STN. The evidence ref is deleted;
no renderer was admitted and no replacement owner was ready at that closeout.
Batch 31.64 then found no unused fifth family. Direct PaulX-style magnitude
renewal remains the smallest source-backed owner of the accepted sound. Batch
31.65 records the operator-authorized product-gate reset and freezes one fresh,
complete `DirectRenewalDream` renderer and executable evidence authority.
Batch 31.66 implemented it once at checkpoint `760da32d`. Two clean
conformance rounds, all `88` synthetic rows, concealed mono `15/15`, all `45`
stereo hard rows, all `15` trio rows, and all `1400` mapped diagnostics
completed. The operator accepted stereo on speakers and explicitly waived
eligible independent review for this effect. Contract `085` records the
one-ear limitation and scopes that product decision to this checkpoint. The
candidate passed. Batch 31.67 admitted its frozen minimal private fixed-ratio
surface. Analysis, plan, stereo, and synthesis remain
byte-identical to checkpoint `760da32d`; construction `1/1`, structural
`10/10`, and synthetic `88/88` rows with `76/76` renders pass after
integration. The module is private, production-compiled, exact-ratio only, and
unrouted. No public control, route, cache, dynamic ratio, other character, or
cross-repo surface opened. Batch 31.68 retained the `2x..4x` overlap pause:
Dream cannot render exact `2x` or interior probes, and the admitted renderers
do not share frame or boundary ownership. Neither renderer changed or failed.
Batch 31.69 then froze one complete `LayeredCloud` authority from pinned
Csound and SuperCollider architecture evidence. The clean-room renderer owns
one map, bounded unit-rate grains, linked-channel weights, validity
normalization, exact crop, and fixed `16x..100x` coverage. No Cloud DSP entered
`main`. The upper overlap remains paused because Dream has no interior
`16x..32x` render.
Batch 31.70 then stopped before candidate source when sub-hop input contradicted
the frozen validity-weight floor. Authority now rejects non-empty `L<H` before
allocation and owns `101` structural rows. After that docs correction, the
candidate passed two unchanged conformance rounds and froze checkpoint
`ee42f50c`. The apparent `Y01..Y05` green result is invalid because `Y05`
omitted frozen three-band and mapped-window natural-stereo diagnostics. No
comparator or listening stage opened, and no Cloud quality conclusion follows.
Batch 31.71 then audited the complete checkpoint without executing DSP. It
found missing spec owners, runner enforcement, structural assertions, truthful
stereo frame counts, Y02-Y05 diagnostics, and comparator/listening ownership.
This first evidence-integrity failure does not judge the small source-backed
renderer. One fresh `AuditedLayeredCloud` authority is justified; its brief
must be complete before implementation and a second evidence failure closes
the family.

Batch 31.72 froze the source-clean replacement brief without candidate DSP.
The pointer-led renderer is unchanged. Nine compile-linked specs now own every
formula, source/vector hash, row, assertion, diagnostic, receipt, deadline,
comparator capture, listening decision, cleanup action, and pass surface. Each
structural or synthetic row runs as a separate nextest process under an
enforced deadline. Component presence, centroids, cross-block dropout, final
remainder, whole/band/window stereo evidence, truthful frame counts, and all
`30` listening rows/`90` renders are explicit. Batch 31.73 opened one fresh
isolated implementation. A second evidence-integrity failure closes Cloud.

Batch 31.73 compiled that source-clean implementation and passed construction
`1/1`, then stopped before structural admission. The frozen `S03` equations
require strict `2|q|<D` with `D<=20H`, which permits at most `20` regular
launches plus one distinct terminal. The required exhaustive result `22` is
unreachable; the actual bound is `21`. No checkpoint, acoustic ref, synthetic
output, comparator output, or listening pack exists. This second evidence-
integrity failure closes Cloud without another rebinding. Batch 31.74 is a
docs-only high-range reassessment.

Batch 31.74 retains the accepted private `DirectRenewalDream` effect at exact
fixed `4x`, `8x`, and `16x`. Exact `16x` is not continuous-range evidence.
`16x..100x` is deferred research intent. The prior complete-owner audit found
no unused fifth family, so no materially different high-range owner or
implementation is ready. Automatic routing, both overlaps, dynamic ratio,
public controls, cache, and product exposure remain paused or absent.

Batch 31.75 freezes the smallest honest public crate boundary. It keeps
`DirectRenewalDream` internal and exposes only offline mono/stereo input, exact
target frames resolving to `4x`, `8x`, or `16x`, fixed `Dream`, and admitted
`space`. The wrapper uses the admitted fixed seed. It does not widen
`TimeStretcher`, tiers, cache identity, routing, dynamic ratio, pitch, motion,
detail, runtime, Loophole, or Chorus.

Batch 31.76 admits that wrapper and focused tests. The four acoustic renderer
files retain their frozen hashes; public/private output is byte-identical at
all admitted ratios and space anchors. Construction, structural, and synthetic
gates remain green. Cache, routing, tiers, runtime, and cross-repo work remain
closed.

Batch 31.77 repaired stale planning currentness and exposed one operator intent
checkpoint. With no named consumer or new source authority supplied, Batch
31.78 takes the recommended freeze path and closes `g10.031` on the admitted
public surface. Deferred scope creates no ready Batch 31.79.

`g10.032` now reopens Cyclic research only. New original-manual and pinned
source evidence distinguishes fixed `CYCLIC`, adaptive `INTELL`, Potenza's
slow-anchor two-grain schedule, and SickoCV's explicit repeat/jump cycle clock.
Batch 32.2 forensic evidence distinguishes the schedules, records
ReaReaRea's compressed-anchor-like replica scaling and separate centred map,
and invalidates the old absolute pitch gate. Batch 32.3 selects centred
compressed-anchor behavior, a fixed manual cycle, linked scheduling, and a
corrected integrity/diagnostic/listening gate. No renderer, public character,
routing, or cache is authorized. Batch 32.4 freezes
`CenteredCompressedAnchorCyclic`: one exact rational map, manual cycle,
two-read crossfade, linked schedule, bounded direct crop, comparator manifest,
and complete Rule 11 gate.

Batch 32.5 candidate `4600d228` passed release compile, construction `1/1`, and
structural `9/9` twice. Batch 32.6 then surfaced `unexpected dropout 1` in
`Y01`, but the owner failed before writing any receipt. Static audit found
pass-only, whole-owner receipt persistence plus incomplete diagnostic,
exact-`16x`, and listening ownership. No valid acoustic decision exists.
Batch 32.7 closes that evidence question and authorizes one fresh audited
identity. Old isolated state is deleted. Batch 32.8 freezes
`AuditedCenteredCompressedAnchorCyclic` without changing the renderer. Every
row is now a one-shot process with fail-durable receipts; exact `16x`,
comparator preparation, level policy, concealment, mono, and independent
stereo behavior are described, but their exact executable manifests are not
frozen.

Batch 32.9 instead stopped before worktree creation. The brief does not freeze
the exact expanded row/assertion manifests, comparator table and generator,
summary/runner/sentinel schemas, or listening decision manifests that it
requires construction to prove. Deleted checkpoint state remains prohibited.
Batch 32.10 closes those gaps with exact canonical encodings, `588` executable
rows, assertion/diagnostic bindings, runner and sentinel behavior, a fresh
reproducible `63`-row comparator set, and listening/reveal schemas. Batch 32.11
generates and binds that evidence before candidate source, implements every
frozen owner, and passes release compile, construction `1/1`, and all `339`
structural rows twice with byte-identical evidence. Clean checkpoint
`74a6d6d9` is frozen at the acoustic ref.

Batch 32.12 stops at the first `Y01` row. The test process writes a passing
two-line receipt below a crate-relative duplicate root, while the shell runner
looks below the repository-relative root and exits `66` for a missing receipt.
No valid row or summary exists, no later gate ran, and retry is prohibited.
This is the second incomplete-evidence checkpoint for the identity. Batch
32.13 closed the identity without an acoustic pass or rejection. Batch 32.14
supersedes that decision: the renderer did not fail, and exact checkpoint
`74a6d6d9` may replay `Y01` once with an absolute evidence root. No Cyclic
renderer is admitted. Batch 32.15 uses that root but stops before DSP because
Batch 32.13 cleanup removed the generated comparator assets. Every assertion
is `not_run`; no render or summary exists. Batch 32.16 restores and
hash-verifies the exact synthetic comparator environment. The unchanged
checkpoint passes `12` Y01 rows, then fails
`Y01-012-impulse-r2-c048000` on one unexpected dropout. No summary or later
gate exists. The checkpoint is rejected; the Cyclic product target remains
open. Batch 32.17 proves the failed window lies between commanded replicas:
continuous mapped-window activity is not sparse-event integrity. Select fresh
event-ledger evidence authority with unchanged DSP. Placeholder Y02, Y03, and
Y04 diagnostic owners in the rejected checkpoint prohibit code or harness
reuse.

Batch 32.18 freezes the sole
`EventLedgerAuditedCenteredCompressedAnchorCyclic` authority. All renderer
formulas remain unchanged. Six sparse Y01 rows move from continuous dropout
to the commanded event ledger; the other `24` retain the exact dropout gate.
The manifest stays at `588` rows. Construction now executes known answers
through every FFT, ledger, cadence, gap, stereo, level, and comparator owner.
No candidate or harness entered `main`.

Batch 32.19's pre-source audit stops the first clean isolation before evidence
or candidate files. It freezes missing ramp, numeric, band, correlation,
summary, and known-answer semantics in one correction. Recreate the unchanged
isolation from that correction commit; this is not a candidate repair.

Batch 32.19 then binds fresh evidence before source and implements the private
candidate. A Python 3.10 runner incompatibility stops the first checkpoint
before any row or render; one evidence-only correction creates checkpoint
`995ea516` without changing renderer or test logic. That checkpoint passes
release compile, construction `1/1`, and structural `339/339` twice with
byte-identical receipts and summaries. The acoustic ref is ready. No Y01 or
later row ran.

Batch 32.20 preflight finds no executable acoustic runner at the frozen
checkpoint. The tracked runner accepts conformance rounds only, while the
summary owner selects conformance rows and writes only the structural summary.
No Y01 row or render ran. This is incomplete executable evidence, not an
acoustic result. Keep the checkpoint and ref immutable for complete Rule 11
reassessment.

Batch 32.21 audits the whole post-checkpoint surface. Assertions auto-pass
from row success; Y01/Y03 use the wrong ledger oracle; later synthetic
diagnostics are absent, zero, or semantically wrong; comparator project
identity, acoustic summaries, concealment, decisions, and reveal are
non-executable. This repeats the incomplete-owner class the fresh identity was
created to correct. Contract `085` Rule 11 closes the family as protocol
churn. The event-ledger renderer has no acoustic judgment.

Batch 32.22 deletes the exact acoustic ref, candidate branch, worktree,
`562 MB` ignored build/evidence state, comparator assets, receipts, and
artifacts. `main` remains candidate-free. That closeout is superseded by the
operator correction in Batch 32.23.

Batch 32.23 records the operator correction: an evidence-system failure cannot
complete an acoustically unjudged product target. Checkpoint `995ea516`, tree
`fd42543b`, remains available as an unreferenced Git object. Contract `085`
now authorizes exact implementation recovery, iterative evidence repair, and
continued execution through valid synthetic and listening judgment. Batch
32.24 recovered the exact renderer and completed two `340/340` structural
rounds. Batch 32.25 has passed `183/183` synthetic rows, `5/5` exact-`16x`
rejection rows, and all `45` long-form mono renders. Its concealed operator
pack is ready. All `15` long-form linked-stereo rows also passed and their
concealed neutral stereo pack is ready at the same renderer checkpoint.
The operator judged the concealed outputs hard to distinguish and solid, with
no significant mono or stereo issue. After all hard stereo controls passed,
the operator explicitly waived independent review for exact checkpoint
`bab6ce96` at fixed `2x`, `4x`, and `8x`. The one-ear limitation remains
recorded and the exception does not generalize.

Batch 32.26 admits the unchanged acoustic renderer privately as
`creative_cyclic` in commit `81edaada`. Its four acoustic implementation files
are byte-identical to the accepted checkpoint. Candidate evidence and
listening scaffolding remain isolated. No public character, router, cache,
artifact, UI, Loophole, or Chorus surface changed.

Batch 32.27 freezes the public extension without changing code. Cyclic owns
exact `2x`, `4x`, and `8x`, optional `Duration` cycle in `5..90 ms`, a
`48 ms` default, character-specific control rejection, deterministic
microsecond canonicalization, and creative engine version v2. Batch 32.28
admits it in commit `e8948512` with `10/10` focused public tests and no
acoustic-file change. Batch 32.29 closes the lane. At that checkpoint,
creative coverage was two explicit fixed-ratio characters.

`g10.033` owns the current stretch decision. Batch 33.1 confirms that
OfflineHighQuality cannot supply a hidden Dream transition and that continuous
Cyclic is a separate character admission. It selects one unchanged-mechanism
Dream generalization over exact targets `4N <= T <= 16N`. Batch 33.2 freezes
that candidate end to end, including exact anchor parity and interior probes
at `4.5x`, `6x`, `10x`, and `15.5x`. Batch 33.3 passed two complete
conformance rounds, `154/154` acoustic rows, `138/138` candidate renders,
`20/20` concealed mono ties, and all stereo hard controls. The operator
accepted stereo and waived independent review for checkpoint `0e9969ab`.
Commit `73910aad` admits only the private `4N..=16N` target gate, internal v2
identity, and focused regression owners. Automatic character switching,
public widening, cache, artifacts, dynamic ratio, and named-consumer
integration remain unavailable.

Batch 33.4 freezes public Dream widening to every exact target in
`4N..=16N`. Discovery becomes a continuous Dream range or exact Cyclic list;
the misleading Dream exact-ratio list is removed and public behavior identity
advances to v3. Dream dispatch remains direct to one admitted owner. No hidden
router, overlap, blend, fallback, private DSP change, or consumer surface is
authorized.

Batch 33.5 admits that surface. Only `creative.rs` and `lib.rs` change.
Focused public tests pass `11/11`; all `18` retained private Dream owners pass.
Public/private output is byte-exact across anchors, interior targets,
one-frame boundaries, mono, stereo, and admitted `space` values. Cyclic stays
exact at `2x`, `4x`, and `8x`. Private renderer trees, routing, cache,
artifacts, dynamic ratio, runtime, UI, Loophole, and Chorus remain unchanged.

Batch 33.6 closes `g10.033` without code. Current public creative coverage is
continuous exact-target Dream over `4N..=16N` and exact-target Cyclic at
`2N`, `4N`, or `8N`. Both are whole-buffer deterministic mono or linked
stereo. They remain explicit characters with no route, blend, fallback,
dynamic ratio, cache, artifact, runtime, or consumer integration.

The next planning checkpoint is continuous Cyclic feasibility. Its private
general-target geometry is a research lead only. Lower Dream remains paused
because no compatible same-character owner exists.

`g10.034` opens that checkpoint. Batch 34.1 selects every exact target
`2N..=8N` for one `ContinuousEventLedgerCyclic` evidence candidate. The
admitted equations and files remain unchanged. Sub-`2x` is excluded because
identity is a bypass and no acoustic anchor owns the effect's emergence.
Batch 34.2 freezes one private entry, two complete conformance rounds, exact
anchor byte parity, low/middle/high interior synthetic and long-form evidence,
linked-stereo admission, Rule 11 repair, cleanup, and minimal private
admission. Public Cyclic stays exact at `2x`, `4x`, and `8x`. Batch 34.3 is
one isolated candidate only.

Batch 34.3 passes from acoustic checkpoint `264403b3`. Two complete
`334/334` conformance rounds are byte-identical. All `183` synthetic rows,
`15` long-form stereo rows, and `165` mapped stereo records pass. Concealed
mono is tied without artifact or usability failure; direct `5..90 ms` cycle
movement remains useful from metallic/ring motion through tremolo/echo motion.
The operator passes the complete speaker pack and makes the Rule 5 decision
for that exact checkpoint after all hard stereo controls pass. The hearing
limitation remains recorded; eligible independent review is not claimed.
Only the private `2N..=8N` entry, behavior identity, and focused regression
owners are admitted. Public Cyclic remains exact at `2x`, `4x`, and `8x`.
That made Batch 34.4 ready as documentation only.

Batch 34.4 freezes public Cyclic v4 over every exact target `2N..=8N`.
Discovery becomes `Continuous { minimum: 2, maximum: 8 }`; public constants
become Cyclic minimum and maximum bounds; the false `[2,4,8]` list is removed
without an alias. Public behavior identity advances to
`signal-creative-stretch-v4`.

Dispatch remains direct to private `render_continuous`. There is no router,
range branch, blend, fallback, cache schema, or integration. Dream, controls,
errors, duration canonicalization, and private renderer bytes remain
unchanged.

Batch 34.5 admits that frozen surface in commit `93758966`. Only
`creative.rs` and `lib.rs` change. Focused public tests pass `12/12`; retained
private continuous Cyclic tests pass `10/10`; all private renderer files remain
byte-identical.

Batch 34.6 closes the lane without code. Current public creative coverage is
continuous Dream `4N..=16N` and continuous Cyclic `2N..=8N`, both deterministic
whole-buffer mono or linked stereo. Character choice stays explicit. No
automatic route, blend, fallback, cache, artifact, dynamic ratio, runtime, UI,
Loophole, or Chorus integration is admitted. The isolated Batch 34.3
worktree, branch, acoustic ref, evidence surfaces, and generated assets are
absent.

`g10.035` Batch 35.1 selects one opt-in Automatic direction without code.
Transparent owns exact targets through `4N`; Transparent and neutral Dream
transition over `4N..=8N`; neutral Dream owns `8N..=16N`. Cyclic remains an
explicit effect and never enters Automatic. Explicit Transparent, Dream, and
Cyclic remain available.

Automatic exposes only exact duration. Its Dream contribution uses admitted
neutral defaults. Batch 35.2 freezes one complete
`ExactTargetTransparentDreamRouter`, including exact-target adaptation, map,
weight and level law, boundaries, linked stereo, deterministic identity,
bounded output staging, evidence, rejection, cleanup, and minimal admission.
Batch 35.3 passes normal-profile regression `204/204` and two unchanged
release conformance rounds, then stops at the first acoustic owner. Pure
Transparent `rademacher-noise` at `4N-1` is byte-exact but peaks at
`10.370356`, contradicting the brief's universal `8.0` ceiling. No later
acoustic or listening owner ran.

Batch 35.4 marks the checkpoint evidence-invalid. Pure owner controls inherit
their admitted owner integrity rules; interior route rows retain the
sample-aligned-arm peak bound. No renderer or evidence bytes change. Batch
35.5 is ready for one exact replay under new isolation identity. No public
route, cache, artifact, dynamic ratio, runtime, UI, Loophole, or Chorus work is
ready.

## Next Task

Execute `g10.035` Batch 35.5 only. Create the newly named disposable worktree,
restore and hash-prove the exact Batch 35.3 candidate source, pass conformance
twice, freeze the new acoustic checkpoint, and restart the corrected gate at
identity/parity.
