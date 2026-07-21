# Signal Docs

Signal uses the Northstar documentation shape as the repo-owned authority layer
for the reusable library/runtime surface.

## Core Sections

- `vision/`
- `architecture/`
- `contracts/`
- `roadmaps/`
- `logs/`

## Optional Sections In Use

- `research/`
- `specs/` for closed strict-lane references and any future reopened strict
  lane

Signal is back in a baseline Northstar posture. There is currently no active
strict lane.

## Current Entry Points

- Vision: [vision/001-signal-vision.md](./vision/001-signal-vision.md)
- Architecture: [architecture/system-architecture.md](./architecture/system-architecture.md)
- Product guardrails: [architecture/product-guardrails.md](./architecture/product-guardrails.md)
- Package map: [architecture/package-map.md](./architecture/package-map.md)
- DSP and analysis feature reference: [architecture/dsp-analysis-feature-reference.md](./architecture/dsp-analysis-feature-reference.md)
- Offline time-stretch synthesis: [architecture/offline-time-stretch-synthesis.md](./architecture/offline-time-stretch-synthesis.md)
- Rejected offline stretch successor: [architecture/offline-time-stretch-successor-brief.md](./architecture/offline-time-stretch-successor-brief.md)
- Non-phase-vocoder feasibility: [architecture/offline-time-stretch-non-phase-vocoder-feasibility.md](./architecture/offline-time-stretch-non-phase-vocoder-feasibility.md)
- Creative time-stretch study: [architecture/offline-creative-time-stretch-study.md](./architecture/offline-creative-time-stretch-study.md)
- Rejected construction-bound LinkedStnNoiseMorph v6 brief: [architecture/offline-creative-linked-stn-noise-morph-brief.md](./architecture/offline-creative-linked-stn-noise-morph-brief.md)
- Rejected creative ComparatorAuditedRenewalSpectral brief: [architecture/offline-creative-comparator-audited-renewal-spectral-brief.md](./architecture/offline-creative-comparator-audited-renewal-spectral-brief.md)
- Rejected-under-old-stereo-policy SupportAuditedListeningLedSourceRelativeRenewalSpectral brief: [architecture/offline-creative-verified-source-relative-renewal-spectral-brief.md](./architecture/offline-creative-verified-source-relative-renewal-spectral-brief.md)
- Rejected-at-vector-proof creative SourceRelativeRenewalSpectral brief: [architecture/offline-creative-source-relative-renewal-spectral-brief.md](./architecture/offline-creative-source-relative-renewal-spectral-brief.md)
- Rejected creative AuditedVarianceCompensatedRenewalSpectral brief: [architecture/offline-creative-audited-variance-compensated-renewal-spectral-brief.md](./architecture/offline-creative-audited-variance-compensated-renewal-spectral-brief.md)
- Evidence-rejected creative VarianceCompensatedRenewalSpectral brief: [architecture/offline-creative-variance-compensated-renewal-spectral-brief.md](./architecture/offline-creative-variance-compensated-renewal-spectral-brief.md)
- Rejected-at-compile creative CompensatedRenewalSpectral brief: [architecture/offline-creative-compensated-renewal-spectral-brief.md](./architecture/offline-creative-compensated-renewal-spectral-brief.md)
- Rejected creative RenewalSpectral brief: [architecture/offline-creative-renewal-spectral-brief.md](./architecture/offline-creative-renewal-spectral-brief.md)
- Creative source triangulation: [research/specimen-dossiers/creative-stretch-source-triangulation.md](./research/specimen-dossiers/creative-stretch-source-triangulation.md)
- Rejected creative SimilarityAlignedCyclic brief: [architecture/offline-creative-similarity-aligned-cyclic-brief.md](./architecture/offline-creative-similarity-aligned-cyclic-brief.md)
- Rejected creative CyclicGrain brief: [architecture/offline-creative-cyclic-grain-brief.md](./architecture/offline-creative-cyclic-grain-brief.md)
- Rejected creative complex-relation brief: [architecture/offline-creative-continuous-excitation-complex-relation-brief.md](./architecture/offline-creative-continuous-excitation-complex-relation-brief.md)
- Rejected continuous-excitation brief: [architecture/offline-creative-continuous-excitation-spectral-brief.md](./architecture/offline-creative-continuous-excitation-spectral-brief.md)
- Rejected DiffuseSpectral brief: [architecture/offline-creative-diffuse-spectral-brief.md](./architecture/offline-creative-diffuse-spectral-brief.md)
- Graph and runtime feature reference: [architecture/graph-runtime-feature-reference.md](./architecture/graph-runtime-feature-reference.md)
- Working rules: [contracts/001-working-rules.md](./contracts/001-working-rules.md)
- Shared DSP boundary: [contracts/001-shared-dsp-and-host-boundary.md](./contracts/001-shared-dsp-and-host-boundary.md)
- Stretch candidate isolation and promotion: [contracts/084-stretch-candidate-isolation-and-promotion-contract.md](./contracts/084-stretch-candidate-isolation-and-promotion-contract.md)
- Creative stretch product and routing: [contracts/085-creative-time-stretch-product-and-routing-contract.md](./contracts/085-creative-time-stretch-product-and-routing-contract.md)
- Historical offline stretch synthesis policy: [contracts/082-offline-time-stretch-synthesis-policy-contract.md](./contracts/082-offline-time-stretch-synthesis-policy-contract.md)
- Supervisor export boundary: [contracts/002-supervisor-export-schema-and-report-boundary.md](./contracts/002-supervisor-export-schema-and-report-boundary.md)
- Roadmap index: [roadmaps/README.md](./roadmaps/README.md)
- Generation index: [roadmaps/generation-index.md](./roadmaps/generation-index.md)
- Stretch closeout roadmap: [roadmaps/g10/030-stretch-consolidation-and-completion.md](./roadmaps/g10/030-stretch-consolidation-and-completion.md)
- Active PaulX-like creative stretch roadmap: [roadmaps/g10/031-creative-time-stretch.md](./roadmaps/g10/031-creative-time-stretch.md)
- Strict-lane reference: [specs/001-g09-lane-first-strict-adoption.md](./specs/001-g09-lane-first-strict-adoption.md)
- Active strict-lane card: none
- Research index: [research/master-index.md](./research/master-index.md)

## Validation

- `effigy qa:docs`
- `effigy qa:northstar`

## Working Rule

- treat Signal docs as the canonical authority for reusable library/runtime
  building blocks
- keep Finch and Loophole wrapper notes outside Signal unless they affect the
  reusable library boundary
- keep section indexes aligned to Northstar conventions
- treat an active generation as a lane-first strict Northstar surface under
  `docs/specs/` only while that generation is explicitly open
- if there is no active strict lane, use the roadmap and contract front doors
  instead of reading old batch-card state as current authority
- in the strict lane, treat a bare `continue` as "follow the previous closeout's
  `Next Task`" rather than as permission to infer a new batch

## Next Task

Run `g10.031` Batch 31.54 as docs-only executable-authority reassessment.
Either bind every structural owner into one fresh construction authority or
close `LinkedStnNoiseMorph`. Do not recover the rejected checkpoint, implement
DSP, change production, routing, product exposure, Loophole, or Chorus. Only
drop into `specs/` when a strict lane is explicitly reopened.
