# g10.032 Cyclic Public Surface

Batch 32.27 freezes the Cyclic extension of the existing fixed-ratio
`CreativeStretch` API.

Frozen boundary:

- `CreativeStretchCharacter::Cyclic`
- exact ratios `2x`, `4x`, and `8x`
- `cycle: Option<std::time::Duration>`
- inclusive `5..90 ms` range
- `48 ms` default
- deterministic nanosecond-to-microsecond round-half-up
- character-specific ratio discovery
- `InvalidCycle` and `UnsupportedCharacterControl`
- public behavior identity `signal-creative-stretch-v2`

The request remains source-compatible. Dream requires `cycle=None`. Cyclic
requires the existing `space` field to stay at its exact default bit pattern.
The consuming UI shows `space` only for Dream and `cycle` only for Cyclic.

Future creative cache identity uses character plus the active control:
`space` bits for Dream or effective integer microseconds for Cyclic. Cache
implementation remains unadmitted.

No Rust, renderer, routing, cache, artifact, runtime, Loophole, or Chorus code
changed in Batch 32.27. Batch 32.28 later admitted only `creative.rs` and
`lib.rs` in commit `e8948512` while preserving both renderers byte-for-byte.
