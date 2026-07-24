# g10.032 Cyclic Public Admission

Batch 32.28 admits the fixed-ratio Cyclic character in commit `e8948512`.

Changed:

- `CreativeStretchCharacter::Cyclic`
- exact Cyclic ratios `2x`, `4x`, and `8x`
- optional `Duration` cycle in `5..90 ms`, default `48 ms`
- integer microsecond canonicalization
- character-specific ratio discovery and typed control rejection
- public engine identity `signal-creative-stretch-v2`

Evidence:

- focused public tests: `10/10`
- Dream public/private parity: byte-exact
- Cyclic mono/stereo public/private parity: byte-exact
- cycle anchors: `5 ms`, default `48 ms`, `90 ms`
- linked duplicate and anti-phase relations: pass
- invalid control, ratio, empty, deterministic, and pre-dispatch `16x`
  behavior: pass
- both admitted private renderer trees: unchanged
- `effigy check:docs`, `effigy health`, and `effigy validate`: pass

Only `creative.rs` and `lib.rs` changed. No cache, route, tier, artifact,
runtime, Loophole, or Chorus surface entered.
