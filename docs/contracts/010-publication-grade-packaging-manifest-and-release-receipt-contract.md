# 010 Publication-Grade Packaging Manifest And Release Receipt Contract

Status: active
Owner: core-product
Updated: 2026-03-13
Related contracts: `docs/contracts/002-supervisor-export-schema-and-report-boundary.md`, `docs/contracts/003-crate-maturity-and-public-runtime-boundary-baseline.md`, `docs/contracts/008-backend-neutral-plugin-capability-and-adapter-breadth-contract.md`, `docs/contracts/009-shared-host-convenience-api-and-consumer-edge-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first publication-grade packaging manifest and release-receipt
contract for `g05.003` so later release automation can stay repo-owned,
machine-readable, and aligned with the runtime/export/plugin and shared
host-edge boundaries already frozen in `g04`, `g05.001`, and `g05.002`.

## Authority hierarchy

Publication-grade packaging has one authority chain:

1. workspace version, changelog, and contract sources define the release claim:
   - Cargo package version sources
   - `CHANGELOG.md`
   - the frozen contract set under `docs/contracts/`
2. `signal-supervisor-tools` owns the machine-readable boundary descriptors that
   explain what a releasable Signal boundary currently includes:
   - `--describe-export`
   - `--describe-conformance-matrix`
   - `--describe-host-edge-boundary`
   - `--describe-release-boundary`
3. repo-owned Effigy tasks own the runnable validation spine for release
   automation:
   - `effigy acceptance:plugin-backend-breadth`
   - `effigy acceptance:host-edge-consumer`
   - `effigy acceptance:conformance`
   - `effigy acceptance:release-boundary`
   - `effigy health`
   - `effigy test`
   - `effigy validate`
4. downstream consumers may inspect or archive the resulting manifests and
   receipts, but they must not become the source of truth for release scope,
   supported boundaries, or validation policy

If a packaging claim cannot be explained in terms of repo-owned descriptors,
contracts, or Effigy tasks, it is not yet part of the shared Signal release
boundary.

## Manifest family

This milestone freezes a two-layer publication-grade packaging family.

### Boundary descriptors remain the semantic source

The existing machine-readable descriptors remain authoritative for their own
subdomains:

- export/report schema meaning comes from `--describe-export`
- runnable consumer proof coverage comes from `--describe-conformance-matrix`
- stable versus unstable shared host edges come from
  `--describe-host-edge-boundary`
- the baseline release inventory comes from `--describe-release-boundary`

A stronger packaging manifest may aggregate those descriptors, but it must not
replace or reinterpret them as a second semantic authority.

### Publication packaging manifest is the bundle inventory

The first publication-grade packaging manifest should bundle the currently
shippable Signal boundary into one machine-readable inventory that names:

- release identity and version source
- changelog location and release note source
- export schema name and version source
- stable shared boundary descriptors included in the release
- repo-owned validation tasks required before the package is considered valid
- supported packaging artifacts and example surfaces included in the release
- intentionally unstable or deferred publication scopes

This manifest is stronger than the `g04` release-boundary baseline because it
describes the full packageable boundary as a reusable bundle rather than a
lightweight descriptor plus prose references.

## Release receipt family

Publication-grade packaging also requires typed receipts that can prove how a
release bundle was assembled or validated without private scripts.

### Manifest-generation receipt

The first receipt class should record:

- which publication manifest identity and version were generated
- which repo-owned descriptor inputs were included
- which contract version or updated date anchored the bundle
- which artifacts or descriptor commands were emitted for consumers

### Validation receipt

The second receipt class should record:

- which repo-owned validation tasks were required
- which of those tasks were executed for the claimed release bundle
- whether any intentionally deferred publication channels were excluded
- which unstable scopes remain outside the package claim

Receipts may start narrow, but they must remain machine-readable, additive, and
repo-owned. They must not require private CI glue or consumer-local scripts to
reconstruct what Signal considered releasable.

## Packaging promises

The first publication-grade packaging contract keeps four promises.

### Packaging remains additive over frozen boundaries

Publication packaging may summarize the frozen runtime/export/plugin and
host-edge boundaries, but it must not loosen or supersede them. If the package
claims backend breadth or host-edge stability, those claims must point back to
contracts `008` and `009` and their repo-owned acceptance surfaces.

### Release automation stays repo-owned

Packaging automation must be explainable through repo-owned descriptors and
Effigy tasks. The contract does not allow a private release script, app-local
pipeline, or downstream wrapper to become the authoritative release recipe.

### Unsupported publication paths stay explicit

Publication-grade packaging must keep unsupported scopes visible rather than
hiding them behind a generic "release-ready" claim. Examples include:

- registry publication not yet promised by Signal
- platform-specific installers, notarization, or signing workflows
- app-specific distribution bundles
- broader backend breadth or host-edge surfaces not yet frozen by contract

### Receipts are for inspection, not reinvention

Consumers, CI, or later automation may archive or inspect release receipts, but
they should not need to recompute them from changelog text, cargo metadata, or
private helper scripts. Signal-owned manifests and receipts should be the
canonical inspection boundary.

## Canonical publication inputs

Consumers and maintainers should inspect release packaging inputs in this order:

1. the frozen contracts that define what Signal is willing to claim
2. the `signal-supervisor-tools` descriptors that expose those claims in
   machine-readable form
3. the repo-owned Effigy validation tasks that prove those claims remain true
4. the publication packaging manifest and release receipts that bundle the
   release-ready boundary for automation

The packaging layer is an aggregation and proof surface, not a separate policy
authority.

## Deferred publication breadth

This Batch 3.1 contract intentionally defers:

- crates.io, registry, or package-manager publication promises
- signed installers, notarization, or platform-distribution workflows
- app-specific bundle assembly or downstream-specific release wrappers
- long-running downstream soak and acceptance automation beyond the current
  focused conformance and release-boundary tasks
- generation-closeout bundling for `g05`, which belongs to `g05.005`

Those areas may build on this contract later, but they are not part of the
first publication-grade packaging baseline.

## Current baseline surfaces

The current repo-owned baseline that this contract builds on is:

- `cargo run -p signal-supervisor-tools -- --describe-export --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-conformance-matrix --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-host-edge-boundary --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-release-boundary --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-packaging-manifest --format=json`
- `effigy acceptance:plugin-backend-breadth`
- `effigy acceptance:host-edge-consumer`
- `effigy acceptance:conformance`
- `effigy acceptance:release-boundary`
- `effigy acceptance:packaging-manifest`
- `effigy acceptance:release-packaging-consumer`

## Next Task

Continue `g05.005` with Batch 5.1 by defining the combined `g05`
generation-closeout descriptor and task without weakening the packaging
manifest or release-receipt boundary.
