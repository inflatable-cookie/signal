# 085 Creative Time-Stretch Product And Routing Contract

Status: active cyclic-reassessment boundary; no implementation admitted
Owner: core-product
Updated: 2026-07-19
Related contracts: `046`, `048`, `084`
Related architecture: `docs/architecture/offline-creative-time-stretch-study.md`,
`docs/architecture/offline-creative-cyclic-grain-brief.md`
Roadmap: `g10.031`

## Purpose

Freeze one reusable Signal boundary for intentional long-form creative stretch
without weakening the transparent `OfflineHighQuality` contract or forcing a
consumer to expose renderer-specific controls.

## Authority

- Contract `046` remains authoritative for `Repitch`, `RealtimePreview`, and
  `OfflineHighQuality`.
- Contract `084` remains closed for transparent successor work. Its rejected
  candidate families do not become active through this contract.
- Contract `085` owns creative intent, range routing, transition behavior,
  deterministic variation, and creative admission.
- Signal owns the engine and its semantic parameter vocabulary. A consumer
  owns layout, labels, percent-versus-duration display, and workflow placement.
- External software is comparator evidence only. No external production
  dependency or copied implementation expression enters Signal.

## Product Vocabulary

`CreativeStretch` means offline pitch-preserving expansion whose goal is useful
dreamy, smeared, cyclic, or cloud-like synthesis rather than transparent event
reconstruction.

The stable semantic request contains:

- exact target frame count
- output/input duration ratio
- `character`: `Dream`, `Spectral`, `Rough`, `Cloud`, or `Cyclic`
- normalized `motion`
- normalized `detail`
- normalized `space`
- deterministic seed or request for the identity-derived default

Target frames are authoritative. Ratio is derived or validated against that
target; inconsistent values are rejected rather than rounded into different
routing decisions.

`Dream` remains the intended default if the automatic router reopens. It means
smooth, fused, musical spectral smear.
`Spectral` intentionally exposes vocoder-like separation and decoherence.
`Rough` intentionally exposes a less smoothed polyphase texture. `Cloud` means
dispersed upper-range evolution. `Cyclic` means commanded Akai-style
repetition. Its first complete candidate targeted expansion above `1x` through
`8x` and is rejected. The character now requires architecture reassessment.

No character is public today. A cyclic-only admission must not expose
unimplemented `Dream`, `Spectral`, `Rough`, or `Cloud` values, or imply that an
automatic range router exists.

These fields are intent, not transform controls. Public consumers must not
select FFT size, window, grain size, overlap, phase mode, internal renderer,
or transition weight.

`Transparent` and `CreativeStretch` are separate product choices. Creative
admission does not upgrade `OfflineHighQuality`, and transparent admission does
not authorize creative output.

## Rules

### Rule 1: one source/output map

Every internal owner uses the same monotonic source/output map and exact target
output lattice. Range selection may change synthesis character, not duration or
event-order truth.

### Rule 2: routing is versioned and deterministic

The automatic routed bands are paused:

- coherent: `1x` through `2x`
- coherent/diffusive overlap: `2x` through `4x`
- diffusive: `4x` through `16x`
- diffusive/cloud overlap: `16x` through `32x`
- cloud: `32x` through `100x`

These bands remain future product intent, not implementation authority. The
explicit `Cyclic` character bypasses them. Its first candidate accepts only
fixed expansion above `1x` through `8x`; `2x`, `4x`, and `8x` are mandatory
admission points.

If automatic routing reopens, overlap weights use smoothstep interpolation over
`log2(ratio)`. A fixed-ratio request uses one constant channel-shared weight for
the whole render.

Changing the band map or blend law changes the creative routing version and
cache identity.

### Rule 3: the UI vocabulary stays stable across renderer changes

`duration`, `character`, `motion`, `detail`, and `space` retain their audible
direction across every range. An internal renderer may implement the macro
differently, but increasing a control must not reverse its semantic meaning at
a routing boundary.

Character values are semantic anchor regions, not external algorithm names.
The required initial anchors are:

- `Dream`: PaulXStretch-like smoothness and musical usefulness
- `Spectral`: CDP-like vocoder/decoherence character
- `Rough`: `Rrreeeaaa`-like conspicuous polyphase texture
- `Cyclic`: `ReaReaRea`-like repetition through `8x`

Signal may use different internal owners or blends to reach those regions.
Until more than one character is admitted, a consumer receives only the
available character and the shared macros, not disabled or fictional choices.

`seed` is advanced variation identity, not a continuous quality knob.

### Rule 4: seamless means measured continuity

A transition is not seamless merely because it crossfades. Both owners must
share target length, source cursor, boundary alignment, linked-channel weight,
and deterministic state.

Boundary probes must cover values immediately below, inside, and above each
overlap. Reject audible level steps, image jumps, timing discontinuity, new
clicks, or abrupt changes in motion density.

Dynamic-ratio routing remains unsupported until fixed-ratio owners and overlap
bands pass. A later dynamic path must carry state and slew weights; independent
segment concatenation is forbidden.

### Rule 5: stereo variation stays linked

Analysis decisions, source-position variation, routing weights, and
normalization are shared across linked channels. Per-channel synthesis may
preserve source-relative detail, but left and right must not draw unrelated
random trajectories.

Neutral `space` preserves mono. Duplicate stereo, swap, and polarity mechanics
must remain explicit structural gates. Independent linked-stereo listening is
required before promotion.

### Rule 6: variation is reproducible

The same complete request and engine version produce byte-identical output on
the supported deterministic platform contract. Default variation derives from
the artifact identity. Rerolling produces a new explicit seed and artifact.

### Rule 7: exact boundaries and bounded state remain mandatory

Creative intent does not waive:

- exact target frame count
- finite output
- deterministic exterior padding and cropping
- bounded duration-independent working state, excluding source and output
- explicit chunk or artifact-writer bounds for long renders
- no audio-thread allocation, blocking, I/O, or execution

### Rule 8: cache identity includes creative intent

Before product-facing caching, identity includes at least:

- creative engine version
- routing version
- source content and channel layout
- exact target frames and ratio/map identity
- character, motion, detail, and space
- deterministic seed
- projection epoch and any pitch/warp inputs that affect output

Creative and transparent artifacts cannot collide.

### Rule 9: listening defines creative quality

Objective controls reject integrity and continuity failures. They do not decide
whether output is dreamy, evolving, musical, or useful.

The current cyclic lane requires concealed long-form listening at `2x`, `4x`,
and `8x`, with `4x` and `8x` primary. The pack covers percussion, bass, vocals,
pads/sustains, and full mix. `16x` remains a rejection-boundary probe, not a
supported target. Independent stereo review remains mandatory.

If the automatic router later reopens, its `Dream`, `Spectral`, and `Rough`
lane still requires `4x`, `8x`, and `16x`. `Dream` must remain the smoothest
and most generally musical centre. Exposed vocoder colour, rough periodicity,
or cyclic repetition in neutral `Dream` is rejection. `Spectral` and `Rough`
must remain deliberate, recognizable, stable destinations rather than one
degraded compromise.

Transparent transient-placement, replica, and tonal-fidelity limits are not
silently reused. Creative gates instead reject uncontrolled clicks, dropouts,
level changes, periodic flutter, metallic repetition outside `Cyclic`, static
freeze, stereo instability, and failure to map the semantic controls
consistently.

### Rule 10: one complete candidate at a time

The independent-bin candidate was rejected for crest growth. Its first
continuous-excitation replacement was rejected at linked-relation admission.
The final direct-complex replacement then stopped at coefficient proof because
its exact anti-phase test required incompatible negation and swap outcomes.
Every branch and its scaffolding was deleted. No distribution, window,
coefficient, phase, smoothing, seed, assertion repair, or scalar sweep follows
these rejections. The current diffusive owner is closed.

Range-owner reassessment rejects the retained coherent renderer as a substitute
for the PaulX-centred core and finds no new complete source-backed spectral
family. `Dream`, `Spectral`, `Rough`, `Cloud`, and automatic routing stay
closed.

The separate cyclic reserve has operator value, a retained ReaReaRea target,
and public two-grain Akai-style architecture evidence. It becomes the only
next owner study. Its first `CyclicGrain` candidate passed structural admission
but failed the first synthetic pitch row: `110 Hz` at `2x` measured
`111.328 Hz`, or `20.778` cents against the frozen `15`-cent ceiling. It was
deleted without correction or rerun. GPL source informs clean-room architecture
only; no expression, constant, or control flow enters Signal. Do not build the
full router as simultaneous experiments.

## Initial Promotion Sequence

1. Comparator capture and target-character freeze. Complete.
2. Rejected independent-bin brief and architecture reassessment. Complete.
3. Complete `ContinuousExcitationSpectral` replacement brief. Complete.
4. Isolated fixed-ratio structural candidate. Rejected on common-polarity
   covariance before creative synthetic controls.
5. Linked-relation architecture reassessment. Complete.
6. Final complete brief and isolated candidate. Rejected at relation proof.
7. Creative range-owner reassessment. Complete; automatic router paused and
   cyclic-first promise selected.
8. Freeze one complete clean-room cyclic-owner brief. Complete.
9. Implement one isolated cyclic candidate. Rejected at the first creative
   synthetic pitch row after structural admission.
10. Reassess cyclic ownership at architecture level or close the character.
    Next.
11. Run retained mono and independent stereo listening only if a future
    complete candidate clears structural and synthetic admission.
12. Admit only the minimal cyclic owner if every gate passes.
13. Reassess product exposure and cache identity after admission.

Core `Dream`/`Spectral`/`Rough`, coherent overlap, `LayeredCloud`, the upper
overlap, dynamic ratios, and automatic routing require a separate reopening
decision backed by new complete-system evidence.

## Current State

Three isolated spectral candidates and the first cyclic candidate are rejected
and deleted. The core `4x`-`16x` owner and automatic router are paused.
`CyclicGrain` passed structural admission but missed the first synthetic pitch
limit by `5.778` cents. Explicit `Cyclic` has no implementation and returns to
architecture reassessment. No public Rust enum, renderer, harness mode,
fixture, artifact schema, runtime route, or product-facing claim entered `main`.
`OfflineHighQuality` remains byte-exact and Contract `084` remains closed.

## Next Task

Run `g10.031` Batch 31.12 only. Reassess cyclic ownership at architecture level
or close the character. Do not tune or reimplement `CyclicGrain`. Keep core
spectral characters, Cloud, automatic routing, cache, and public APIs closed.
