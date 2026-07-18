# g10.029 Bounded Normalized Sliced Integration

Date: 2026-07-18
Batch: 29.7AL
Status: complete; normalized Stage A selected

## Result

The fixed `16384/8192/512` sliced frame remains exact at `48 kHz` but fails as
the common integration boundary. Its physical span changes with sample rate;
at `8 kHz` it needs `2432/1217` signed/nonnegative atoms, above the frozen
`1344/673` capacity.

One alternative was compared and selected for proof only:

- common hop `H=F/100`
- transform/advance `32H/16H`
- supports `8H/4H/2H`
- crossover bins `240/1920`
- `32` coefficients per atom

The `8/44.1/48 kHz` rows use `380/191`, `1182/592`, and `1260/631`
signed/nonnegative atoms. All fit the existing capacity.

## Boundary

Identical outer sine windows and the inner painless canonical dual form one
synthesis law. A global common-lattice decision runs once, persists state
across slice retirement, and populates both active layers. No independent
normalizer, relation projection, state reset, or duplicate frequency decision
is selected.

Prepared coefficient storage is six source plus two output slabs:
`8CBK`, capped at `645120 Complex64` slots. Transform scratch, two-layer sample
overlap, a `19`-frame material halo, guided phase/energy slots, current/prior
regions, and static frame records have separate fixed formulas in Rule 31T.
None depends on render duration.

## Decision

Batch 29.7AM may implement normalized sliced identity, bounded mechanics,
overflow results, and one inert boundary-crossing state token. It may not add
the guided material policy or render stretched audio. No sound-quality,
objective, listening, or holdout result exists.

## Validation

- `effigy qa:docs`
- `effigy qa:northstar`
- `effigy health`
- `effigy validate`
- `git diff --check`

## Next Task

Run Batch 29.7AM Stage A under Rule 31T. Stop on any identity, ownership,
boundary-token, memory, work, repeat, or overflow miss.
