# Complete-System Development Rejection

Date: 2026-07-13
Roadmap: `g10.029`
Batch: `29.6BK`
Status: rejected before holdout

## Result

Operator notes were frozen before the concealed key opened. All three
successors lose every explicitly ranked row to current Signal or Rubber Band.
They also share one broad temporal-smear defect: reverb-like blur or multiple
source copies at very small delays.

No successor can reach the required `6/9`. Four explicit losses leave at most
five possible wins. Batch 29.6BL stays closed. Holdout remains unread.

## Frozen Findings

| Row | Preferred | Usable | Successor result |
| --- | --- | --- | --- |
| `L001 0.75x` | current Signal | current Signal, Rubber Band | all three unusably blurred |
| `L002 1.25x` | Rubber Band | Rubber Band, current Signal | all three unusably blurred |
| `L004 0.75x` | current Signal | current Signal, Rubber Band | all three unusably blurred |
| `L005 1.25x` | current Signal | current Signal, Rubber Band | all three unusably blurred |
| remaining five | no unique winner stated | varies | same broad theme reported |

`L001` current Signal also has a visible transient pop. This does not alter the
successor gate.

Frozen notes SHA-256:
`e18a56c5af3e546d034fff74c2fc737b6115816372a0e89fa0e5cf5c9ecf8dda`

## Key Resolution

Each rejected successor occupies the unusable set on all four explicit rows:

- `g512-sr-u0-rc-v1`: `B`, `D`, `C`, `A`
- `g512-sr-u1-rc-v1`: `C`, `C`, `B`, `B`
- `g512-sc-u1-rc-v0`: `D`, `B`, `E`, `C`

The letters above follow row order `L001`, `L002`, `L004`, `L005`.

Current Signal is `E`, `A`, `D`, `D`. Rubber Band is `A`, `E`, `A`, `E`.

## Attribution Boundary

The shared implementation independently phase-transports three full-band STFT
layers, then sums them through the union dual. Current vertical alignment
changes one dominant bin per frame. Identity reconstruction proves only the
unmodified union frame; it does not prove coherent modified layers.

This is a causal hypothesis, not a conclusion. Batch 29.6BM must export
per-layer and combined evidence under frozen phase modes. It decides whether
the blur already exists within each layer or appears when otherwise usable
layers recombine.

No parameter sweep, holdout read, product promotion, or comparator dependency
opens from this result.

## Next Task

Execute Batch 29.6BM cross-resolution smear attribution on the frozen nine-row
development set and three rejected configurations.
