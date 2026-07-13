# Research-to-Architecture Cross-Reference

Status: Draft
Owner:
Last updated:
Purpose: Map promoted research findings to architecture documents, identify gaps, and track promotion status.

## How To Use This File

- Add one section per translation memo or promoted research theme.
- Classify each pairing as `Aligned`, `Partially Aligned`, `Missing`, or `Conflicting`.
- Use this file to drive architecture updates and prototype ordering, not as a replacement for the architecture docs themselves.

## Gap Analysis Results

### Memo `003`: Non-Duplicating Stretch Ownership -> offline stretch synthesis

| Research finding | Architecture doc | Alignment | Gap description |
| --- | --- | --- | --- |
| Complementary subbands do not provide event-local resolution without coupled time-varying transitions | `architecture/offline-time-stretch-synthesis.md` | `Aligned` | Family rejected for the next proof |
| Generic coefficient-plane quilts do not provide a bounded exact local dual and phase topology together | `architecture/offline-time-stretch-synthesis.md` | `Aligned` | Family retained as reserve only |
| A time-adaptive painless NSG gives exact local duals and one event-local coefficient sequence | `architecture/offline-time-stretch-synthesis.md` | `Aligned` | Selected for Batch 29.6BP |

## Critical Gaps

| Gap | Related research | Architecture area | Status |
| --- | --- | --- | --- |
| Single-owner invariants are not yet expressed by the implementation proof | Memo 003 | offline stretch synthesis | open: Batch 29.6BP |

## Areas Already Aligned

| Finding | Research source | Architecture doc |
| --- | --- | --- |
| Adaptive resolution must share one global time map and linked-stereo decisions | Memo 003 | `architecture/offline-time-stretch-synthesis.md` |

## Prototype Dependency Ordering

### Tier 1: Architecture-blocking

1. Single-owner adaptive-frame proof - gates stretched phase work.

### Tier 2: Design-constraining

1. Study and global schedule attachment - proves one mapping across resolution changes.

### Tier 3: Refinement

1. Complete phase and synthesis proof - follows representation and mapping closure.

## Next Task

Execute Batch 29.6BP single-owner adaptive-frame proof.
