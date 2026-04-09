# 075 Runtime Public Interface Decomposition And Internal Assembly Boundary Contract

Status: active
Owner: core-product
Updated: 2026-04-09
Related contracts: `docs/contracts/003-crate-maturity-and-public-runtime-boundary-baseline.md`, `docs/contracts/009-shared-host-convenience-api-and-consumer-edge-contract.md`, `docs/contracts/015-offline-render-recovery-and-resumability-contract.md`
Related architecture: `docs/architecture/system-architecture.md`, `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the contract for breaking `signal-runtime`'s oversized public interface
surface into smaller versionable families with explicit internal assembly
layers.

## Authority hierarchy

1. `signal-runtime` public contract families own stable consumer-facing DTOs,
   requests, receipts, and summaries.
2. internal assembly modules may compose those DTOs but are not part of the
   public contract by default.
3. host crates and downstream consumers depend on public families, not on
   assembly helpers or incidental module layout.

## Required shared guarantees

- public interface families must be grouped by semantic domain, not by current
  source-file convenience.
- large preview, offline-render, and observation builders must separate:
  - validation
  - resolution
  - policy derivation
  - result formatting
- test trees must align with those public families rather than one global
  import wall.

## Rules

- new public DTOs must land in the narrowest contract family that owns them.
- internal-only helpers must not leak through broad root reexports by default.
- test-fixture convenience must not drive public API shape.

## Required proof surfaces

- public API diff review for each extraction tranche
- compile-only conformance for downstream-style imports
- reduced root-file and import-wall pressure in `signal-runtime`

## Next Task

Use this contract for the active `g09.007` runtime decomposition lane, starting
with the remaining internal assembly wall and then normalizing the runtime test
surface against the same family boundaries.
