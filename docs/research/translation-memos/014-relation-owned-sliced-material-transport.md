# Relation-Owned Sliced Material Transport

Status: promoted
Date: 2026-07-18
Roadmap: `g10.029`, Batch 29.7Z
Contract: `082`, Rule 31J

## Question

Attribute Batch 29.7Y's pre-operator linked-stereo failure and replace its
whole-source execution shape without tuning the rejected material law.

## Exact Attribution

Batch 29.7Y samples each channel independently:

`C_c(t) = polar_lerp(C_c[n], C_c[n + 1], alpha)`

It then applies one common material operator `O(t)`:

`Y_c(t) = C_c(t) O(t)`

The operator cancels from the linked-channel phase difference. The output
relation is whatever independent interpolation produced before material state.
For endpoint phase advances `a` and `b`, that relation advances by `b - a`.
The shortest path of the relation itself advances by `wrap(b - a)`. They agree
only when both channel unwrap branches are compatible.

One counterexample is sufficient. Let the reference advance from `0` to
`+170` degrees and the peer from `0` to `-170` degrees. Independent midpoint
interpolation produces a `-170` degree peer/reference relation. The endpoint
relations move from `0` to `+20` degrees, whose midpoint is `+10` degrees. The
error is `180` degrees before common rotation, noise diffusion, or synthesis.

This matches the completed report: IPD, mid/side, correlation, and aggregate
relation errors rise together while structural failures and silent-peer output
remain zero. A common later operator cannot repair an unconstrained earlier
relation.

## Primary Evidence

Dorran, Lawlor, and Coyle preserve multichannel phase by updating the greater-
magnitude peak first, then deriving the lesser peak from the original channel
relationship. Signalsmith independently selects the greatest-energy channel,
computes its phase result, and derives peers from the current input complex
relation while retaining peer energy. Both treat channel relation as an
explicit owned quantity, not an emergent result of two phase interpolators.

Holighaus et al. supply the execution correction. Their sliced
frequency-adaptive nonstationary Gabor transform uses fixed-size overlapping
slices, exact sliced reconstruction, precomputable duals, memory independent of
full signal length, and linear total complexity for fixed slice geometry. Its
coefficients approximate the whole-length transform; the sliced transform must
therefore be contracted as its own representation, not presented as a bit- or
coefficient-equivalent optimization of Batch 29.7Y.

## Selected Relation Law

For each output time and positive-frequency atom:

1. Use one source position, atom identity, material decision, and deterministic
   reference channel for all linked channels.
2. Sample the reference coefficient once.
3. Keep every peer's own interpolated magnitude.
4. At both enclosing source lattice points, form the unit peer/reference
   relation from the same selected reference.
5. Interpolate that relation once on the unit circle. Do not subtract two
   independently interpolated phases.
6. Form the peer base coefficient from reference phase, the interpolated
   relation, and peer magnitude.
7. Apply the same material gain, common-region rotation, and deterministic
   perturbation to every channel.
8. Keep exact silence exact. A missing reference relation resets from the
   current sampled pair; no relation history or peer magnitude is borrowed.

Relation interpolation has no threshold. Two defined endpoint relations take
the shortest circular path. One defined endpoint is held. If neither endpoint
has nonzero reference and peer magnitude, `UndefinedRelation` uses the current
sampled pair and is counted. An interpolated zero-magnitude peer stays exact
zero. Jointly zero energy is `Silent`, not `UndefinedRelation`.

The output invariant is explicit:

`unit(Y_peer conj(Y_ref)) = interpolated_source_relation`

Reference selection may change. The current relation is reconstructed at each
output atom, so a switch does not inherit a peer phase accumulator from the old
reference.

## Selected Sliced Frame

Replace whole-source FFT sizing with one fixed sliced representation:

- each transform spans `16384` frames, advances `8192`, and retains the
  `512`-frame common coefficient lattice
- identical analysis and synthesis windows use
  `h[n] = sin(pi (n + 0.5) / 16384)` and obey
  `h[n]^2 + h[n + 8192]^2 = 1`; at most two slices overlap
- the geometry is the passing Stage A transform span and twice the existing
  maximum `4096`-frame atom support, not a searched value
- every slice retains the proven long/middle/short atom ownership and inner
  painless canonical dual
- one outer slicing partition and its dual own overlap reconstruction
- the combined inner and outer dual is the only sliced-frame synthesis owner;
  no scale or slice normalizes independently
- overlapping slice layers share global output-frame identity, source position,
  material state, reference choice, relation, and perturbation key
- predecessor regions and material-analysis halo cross slice boundaries
- only the first and final slices use source reflection and final crop
- peak live coefficient memory is independent of source duration; analysis and
  synthesis work are linear in rendered frames for fixed geometry

Identity must be reproven for this sliced representation. Exact reconstruction
of the sliced frame does not imply that modified sliced coefficients equal the
rejected whole-source candidate.

## Rejected Alternatives

- Same operator after independent channel interpolation: already failed; the
  operator cancels from channel relation.
- Shared phase increment on prior outputs: preserves old output relation, not
  current source relation.
- Post-synthesis image repair: too late and already rejected in Batch 29.7K.
- Independent slice renders plus concatenation or crossfade: creates multiple
  state owners and repeats the rejected segment topology.
- Whole-source FFT with performance tuning: retains duration-dependent memory,
  `O(L log L)` work, and the five-hour report failure mode.
- Claiming sliced coefficients equal full-length coefficients: contradicted by
  the sliced-NSG approximation boundary.

## Proof Boundary

One further report-only proof is justified because both corrections are direct
architecture repairs with independent primary support. It is not a parameter
sweep.

Stage A proves sliced identity, exact crop, channel relations, boundary
reflection, duration-independent peak working memory, linear operation counts,
and repeat hashes before material transport runs.

Stage B changes only source coefficient relation ownership. Supports,
crossovers, fuzzy material law, transient law, diffusion curve, seed, peak map,
and objective gates stay frozen from 29.7Y. The calibrated stereo gate runs
before the long mono corpus and stops the candidate immediately on any miss.

## Decision

Retain `FrequencyAdaptiveMaterialPhase` for one final architecture-corrected
proof. Replace independent polar channel interpolation with relation-owned
transport and replace the whole-source frame with the fixed sliced frame. Any
identity, boundedness, stereo, mechanics, or mono miss closes this family.
Listening, dynamic ratio, realtime, routing, cache, production, and Batch 29.8
remain closed.

## Sources

- [Dorran, Lawlor, and Coyle, Multi-Channel Audio Time-Scale Modification](https://mural.maynoothuniversity.ie/id/eprint/8793/1/BL-Multi-channel-2005.pdf)
- [Signalsmith Stretch, pinned relationship-preserving recurrence](https://github.com/Signalsmith-Audio/signalsmith-stretch/blob/57b93f4e9206a089a45387eaa39bdc9f310d3308/signalsmith-stretch.h)
- [Holighaus et al., A Framework for Invertible, Real-Time Constant-Q Transforms](https://arxiv.org/abs/1210.0084)
- [Signal Batch 29.7Y evidence](../../logs/2026-07/18-g10-029-frequency-adaptive-material-phase-proof.md)
- [Signal linked-stereo recurrence memo](./006-linked-stereo-recurrence.md)

## Next Task

Stage A has passed the fixed sliced representation and boundedness proof. Run
Batch 29.7AA Stage B once. Add the frozen relation law and material operator,
then run synthetic and exact mechanics followed by the `48`-row calibrated
stereo gate. Stop before the long mono corpus on any miss.
