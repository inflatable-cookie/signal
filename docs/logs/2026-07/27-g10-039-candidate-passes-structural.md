# 2026-07-27 g10.039 Batch 39.3 Candidate Passes Structural Admission

Status: structural gates green; acoustic checkpoint not yet opened

## Gate Results

| gate | before | after |
| --- | --- | --- |
| `G1` chunk-size independence, static ratio | failed at chunk `1024` | identical across four partitions |
| `G1b` chunk-size independence, dynamic ratio | failed at chunk `2048` | identical across three partitions |
| `G2` memory ceiling, duration independence | `11665468 B` against `8388608 B` | `8519740 B` against `9437184 B` |
| `G3` output length matches target | passed | passed |
| `G4` correlation against a whole render | `-0.082711` | **`1.000000`** |

`G4` is the acceptance target inherited from `g10.036`, where the segmented
path measured `0.034`. The resumable renderer is bit-identical, not merely
correlated.

## The Actual Defect

Not emission scheduling, which is where the first analysis pointed. The input
ring holds `ring_frames` of source, and `render` pushed an entire caller chunk
into it regardless of size. A `144000`-frame whole-source call against an
`8192`-frame ring overran it seventeen times over, so source was overwritten
before the analysis cursor reached it.

That inverts the reading of the first result. The chunked renders were not
wrong; the whole-render *reference* was the most corrupted case of all, because
it pushed the most at once. `G3` passing throughout was the clue: frame
scheduling and output length were always right, and only the samples were
destroyed.

`render` and `flush` now feed the ring in slices bounded by its free capacity,
draining between slices. The ring never holds source the analysis cursor has
not reached.

## Ceiling Correction

`G2` failed for a second, unrelated reason: the implementation allocated rings
at four times the window where the brief said twice.

Fixing that exposed an error in the brief itself. The Batch 39.2 inventory
counted the overlap-add and normalization rings and omitted the input ring
entirely, so its `8 MiB` ceiling was derived from an incomplete list. Three
rings of twice the window is the real cost: `8519740 B` at maximum geometry,
which the original ceiling could never have covered.

Contract `046` is corrected to `9 MiB` with `917444 B` of headroom. The ceiling
moved to match the measurement rather than the renderer being squeezed to meet a
number that was wrong.

Duration independence holds exactly: a `1000`-frame source and a ten-minute
source both measure `266300 B` at the retained geometry.

## What This Does Not Yet Prove

The five gates are structural. No acoustic evidence exists, and the renderer is
not wired into any production path, so:

- `segmented_render_matches_whole_render_at_constant_ratio` in
  `transparent_correctness_owners` stays `#[ignore]`d. It measures the shipped
  segmented path, which is unchanged until Batch 39.4 adopts the new renderer
- the seam pulse the `g10.036` listening rounds heard is not yet removed from
  anything a listener would hear
- `A18` remains untriaged

Contract `084` Rule 11 governs what comes next: a clean tree passing compile,
construction, and every structural gate may become one immutable acoustic
checkpoint, and synthetic and listening evidence then run once in order.

## Validation Run

- `cargo test -p signal-dsp-stretch --test resumable_gates`: `5` passed
- `cargo test --workspace`: green
- `cargo clippy -p signal-dsp-stretch --all-targets --all-features`: pre-existing
  warning set unchanged
- `effigy qa:docs`

## Next Task

Execute `g10.039` Batch 39.4: replace per-chunk stretcher construction in
`signal-render-plane` with the resumable renderer, remove
`smooth_artifact_chunk_boundaries_interleaved` and the crate's internal seam
smoother once measurement shows they are no longer needed, prove artifact output
against the frozen seam metric, and keep the chunk plan as the memory-bounding
authority.

Adoption changes rendered output for every dynamic-ratio render and every source
longer than one chunk, so it carries a behavior version advance and Contract
`084` Rule 5 listening evidence.
