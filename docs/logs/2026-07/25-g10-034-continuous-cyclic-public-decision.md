# g10.034 Continuous Cyclic Public Decision

Date: 2026-07-25
Batch: 34.4
Status: complete

## Decision

Freeze public Cyclic v4 over every exact target `2N <= T <= 8N`.

- discovery: `Continuous { minimum: 2, maximum: 8 }`
- constants: Cyclic minimum `2`, maximum `8`
- remove the false `[2,4,8]` supported-ratios constant without an alias
- public behavior identity: `signal-creative-stretch-v4`
- validation: checked inclusive bounds before dispatch or allocation
- errors, controls, duration canonicalization, and empty success: unchanged
- dispatch: direct to private `render_continuous`
- direct cycle: `5..90 ms`, default `48 ms`
- Dream: unchanged over `4N..=16N`

No router, range branch, overlap, blend, fallback, cache schema, artifact,
dynamic ratio, runtime, UI, Loophole, or Chorus surface is admitted.

## Implementation Gate

Batch 34.5 may change only `creative.rs` and `lib.rs`. It must prove complete
small-domain validation, one-frame boundary rejection, byte-exact
public/private parity across anchors and interior targets, all three cycle
anchors, unchanged Dream behavior, unchanged linked relations and errors, and
byte-identical private renderer files.

No listening rerun is required. The public wrapper exposes the unchanged
private renderer already admitted in Batch 34.3.

## Next

Execute Batch 34.5 only. Admit the frozen v4 public range and stop before lane
closeout.
