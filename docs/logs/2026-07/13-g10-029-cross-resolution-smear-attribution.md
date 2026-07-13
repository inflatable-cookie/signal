# Cross-Resolution Smear Attribution

Date: 2026-07-13
Roadmap: `g10.029`
Batch: `29.6BM`
Status: complete

## Result

The rejected successor's smear comes from incoherent independent full-band
layer transport and recombination. It is present before event correction and
is not repaired by the current one-bin vertical policy.

## Evidence

- configurations: `3`
- development rows: `9`
- phase modes: `4`
- renders: `108`
- holdout reads: `0`
- maximum layer-sum error: `3.3306690738754696e-16`
- report: `target/stretch-successor-bm-smear-attribution.tsv`
- report SHA-256:
  `94d448013671e25eec89caa8d8f5fd544fd3c9774a19963566ec8e3c9dab7c1a`

| Mode | Mean arrival disagreement | Maximum | Correlation | Layer replicas | Combined replicas |
| --- | ---: | ---: | ---: | ---: | ---: |
| ordinary | `174.189394` | `507` | `0.123296` | `36.346591` | `38.414773` |
| event | `173.901515` | `507` | `0.128521` | `36.375000` | `38.505682` |
| vertical | `174.253788` | `507` | `0.221780` | `36.450758` | `38.346591` |
| complete | `172.776515` | `507` | `0.197448` | `36.348485` | `38.494318` |

The complete mode adds `2.145833` replicas per event over the mean individual
layer. Vertical policy improves whole-render correlation but leaves gross
arrival disagreement and replica growth.

## Decision

Retire independent synthesis-phase state per full-band resolution. Batch
29.6BN must solve one common physical-frequency phase field, apply event policy
once, and project the result back to every layer. Magnitudes, union dual, study,
schedule, and development material stay frozen.

If that proof cannot bring mean arrival disagreement below `8` frames,
correlation above `0.8`, and combined replica growth to zero, retire redundant
full-band union ownership and redesign the coefficient plane.

## Next Task

Execute Batch 29.6BN shared full-field phase proof. Keep holdout and tuning
closed.
