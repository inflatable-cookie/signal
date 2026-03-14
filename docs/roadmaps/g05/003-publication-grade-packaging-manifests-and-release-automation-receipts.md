# 003 - Publication-Grade Packaging Manifests And Release Automation Receipts

Status: complete
Owner: core-product
Created: 2026-03-12
Depends on: g05.001, g05.002
Vision tags: `RELEASE`, `PACKAGING`, `AUTOMATION`

## Problem

`g04` established the first host-free release boundary, but the current
baseline is still intentionally light: version source, changelog, export
description, conformance matrix, and closeout descriptors are explicit, yet
publication-grade manifests and stronger release receipts do not exist.

Without a dedicated packaging-depth milestone:

- release claims will outrun the actual machine-readable packaging evidence
- publication or distribution work will fall back to consumer-local scripts
- wider backend and host-edge claims will not be reflected in one stronger
  release manifest
- later downstream automation will not know which release receipts are
  canonical

## Goals

- [ ] define publication-grade packaging manifests and release receipts
- [ ] keep versioning and packaging policy repo-owned and inspectable
- [ ] avoid consumer-local release orchestration becoming the source of truth
- [ ] prepare one stronger machine-readable boundary for downstream automation

## Non-Goals

- [ ] no promise to publish to crates.io or another registry unless ready
- [ ] no app-specific distribution workflow
- [ ] no manual release checklist masquerading as a contract

## Execution Plan

### Batch 3.1 - Packaging Manifest Contract

- [x] define the first stronger packaging manifest and receipt family
- [x] align it with the existing export/conformance/release-boundary surfaces

### Batch 3.2 - Repo-Owned Release Automation Depth

- [x] wire the chosen manifests into repo-owned tasks or descriptors
- [x] keep intentionally unsupported publication paths explicit

### Batch 3.3 - Release-Proof Fixture

- [x] add a focused proof that the packaging manifest and release receipts stay
  consumable without private release scripts

## Progress Notes

- 2026-03-12: seeded `g05.003` so packaging depth grows from the closed `g04`
  baseline instead of being reinvented in downstream release flows.
- 2026-03-13: completed Batch 3.1 by freezing contract `010`, making the first
  publication-grade packaging manifest and release-receipt family explicit as
  an additive layer over the existing export, conformance, host-edge, and
  release-boundary descriptors rather than a new private release authority.
- 2026-03-13: completed Batch 3.2 by adding
  `signal-supervisor-tools --describe-packaging-manifest`, wiring
  `effigy acceptance:packaging-manifest`, and refreshing the older
  release-boundary descriptor so the repo-owned publication seam now includes
  the packaging manifest and explicit unsupported publication paths.
- 2026-03-13: completed Batch 3.3 by adding a downstream-style binary-facing
  proof in `crates/signal-supervisor-tools/tests/public_packaging_manifest_boundary.rs`
  and promoting the stronger acceptance surface to
  `effigy acceptance:release-packaging-consumer`, so packaging claims
  now stay consumable without private release scripts or app-local orchestration.

## Acceptance Criteria

- [x] Signal has a stronger publication-grade packaging manifest boundary
- [x] repo-owned release automation is more explicit than changelog plus prose
- [x] downstream consumers can inspect release receipts without private scripts

## Risks And Mitigations

- Risk: packaging work drifts into distribution-provider detail.
- Mitigation: keep the milestone to reusable manifests, receipts, and tasks.
- Risk: stronger manifests over-claim unsupported publication channels.
- Mitigation: keep intentionally unsupported paths explicit in the contract.

## Evidence Requirements

- [x] log each meaningful packaging tranche
- [x] run focused validation for release manifests or receipts
- [x] record which publication paths still remain deferred

## Next Task

COMPLETE. `g05.003` closed on 2026-03-13 after the packaging manifest
contract, repo-owned descriptor/acceptance wiring, and downstream-style
consumer proof landed. Continue with `g05.004` Batch 4.1.
