# 003 Crate Maturity And Public Runtime Boundary Baseline

Status: active
Owner: core-product
Updated: 2026-03-12
Related contracts: `docs/contracts/002-supervisor-export-schema-and-report-boundary.md`
Related package map: `docs/architecture/package-map.md`

## Purpose

Freeze the first explicit crate-maturity inventory and identify the minimum
Signal-owned public boundary that downstream consumers may treat as the
starting stable contract for `g04`.

## Crate maturity inventory

### Public

These crates are the current reusable library baseline and are intended to be
consumed directly without requiring product-local host ownership:

- `signal-primitives`
- `signal-dsp`
- `signal-dsp-resample`
- `signal-dsp-spectral`
- `signal-analysis`
- `signal-analysis-rhythm`
- `signal-analysis-tonal`
- `signal-analysis-loudness`
- `signal-analysis-character`
- `signal-analysis-embed`

### Consumer-facing but unstable

These crates are visible to downstream consumers and already carry meaningful
behavior, but their public shape is not yet broadly frozen beyond the explicit
boundary named below:

- `signal-graph`
- `signal-runtime`
- `signal-ipc`
- `signal-plugin`
- `signal-hardware`
- `signal-host-local`
- `signal-host-server`
- `signal-supervisor-tools`

### Internal

These crates currently exist to support backend, adapter, or platform depth and
should not be treated as independent public contracts:

- `signal-plugin-clap`
- `signal-plugin-sandbox`
- `signal-hardware-coreaudio`

## First boundary to stabilize

The first frozen `g04` boundary is narrow on purpose. Signal does not yet
declare every consumer-facing crate stable. The minimum boundary that now
requires stronger contract discipline is:

1. the typed runtime/export/report surface intentionally re-exported by
   `signal-runtime`
2. the versioned supervisor export envelope frozen by contract `002`
3. the consumer-facing proof path in `signal-supervisor-tools` that serializes
   that export family without reading private runtime internals

That boundary includes the runtime-owned report and receipt families already
promoted into `signal-runtime`, including:

- supervisor/observation summaries
- profiling and soak receipts
- offline render, freeze, stem, manifest, and delegated-execution receipts
- runtime-owned plugin recall and execution-topology export

## Explicitly deferred from the first freeze

The following surfaces remain outside the first frozen boundary and may still
change more freely during later `g04` work:

- `signal-host-local` and `signal-host-server` convenience APIs beyond the
  shared runtime/export contract
- backend crates and adapters such as `signal-hardware-coreaudio`,
  `signal-plugin-clap`, and `signal-plugin-sandbox`
- CLI flags, presentation details, and non-schema UX in
  `signal-supervisor-tools`
- internal graph scheduling, orchestration, and hardware policy that is not yet
  promoted into typed runtime/export receipts

## Maturity-tier promises

These promises are the `g04.001` policy baseline. They are stricter than “best
effort” documentation, but they are not yet a crates.io publication policy.

### `public`

- the crate is intended for direct downstream use as part of Signal's reusable
  library surface
- additive API growth is allowed when existing public meanings stay intact
- removing, renaming, or materially redefining documented public API requires a
  migration note and an explicit roadmap/log mention
- if a `public` crate is reclassified, that reclassification must be recorded
  explicitly rather than implied by code churn

### `consumer-facing but unstable`

- the crate may be used by early consumers, but it is not broadly frozen beyond
  any narrower contract called out explicitly in docs/contracts
- APIs may still be reorganized, split, or replaced without a semver-like
  compatibility promise
- changes that affect the frozen runtime/export boundary, or that change the
  crate's maturity classification, still require roadmap/log documentation

### `internal`

- no direct downstream contract is promised
- names, module structure, and APIs may change freely when they do not widen or
  break an explicit public contract
- internal crates should not be referenced as required integration points in
  consumer-facing guidance

## Schema and versioning policy

1. `signal.supervisor.export` remains the only numerically versioned
   machine-readable export envelope in the current baseline. Its schema
   identity stays:
   - `schema = "signal.supervisor.export"`
   - `schema_version = 1`
2. The typed runtime/export/report DTOs re-exported by `signal-runtime` are the
   Rust authority surface for the first frozen boundary. JSON export is a
   projection of that authority surface, not a separate host-local model.
3. Contract `002` remains the canonical rule set for the supervisor export
   envelope. In particular:
   - additive export fields may extend `schema_version = 1` when existing field
     meaning stays intact
   - breaking export shape or meaning changes require a new `schema_version`
4. The same additive-first rule applies to typed runtime-owned report and
   receipt families inside the frozen boundary:
   - new typed surfaces may be added
   - existing typed fields, variants, and meanings should not be silently
     removed, renamed, or repurposed
5. Host-local convenience APIs may evolve outside this schema policy unless
   they are part of the explicit runtime/export boundary. They must not become
   the authority source for runtime-owned report or receipt data.
6. `signal-supervisor-tools` remains the canonical host-free serializer and
   describer for the frozen export envelope. Changes to export defaults or
   supported debug sections should be treated as contract-facing changes, not
   incidental CLI cleanup.

## Migration-note triggers

The following changes now require an explicit migration note plus roadmap/log
recording:

- reclassifying a crate between `public`, `consumer-facing but unstable`, and
  `internal`
- removing, renaming, or materially redefining a typed runtime/export/report
  surface inside the frozen boundary
- changing runtime-owned data so that a host-local layer becomes the authority
  for fields previously owned by `signal-runtime`
- changing export defaults, supported debug sections, or schema meaning in a
  way that can affect automation or parser assumptions

If a change also breaks the JSON export envelope, it must bump
`schema_version` according to contract `002`.

## Consumer-facing proof

The current proof path for this frozen boundary is intentionally narrow:

- `crates/signal-runtime/tests/public_contract_boundary.rs` compiles as an
  external integration test and exercises `SignalRuntime`,
  `RuntimeObservationReport`, `RuntimeSupervisorReport`,
  `RuntimeProfilingReceipt`, and `RuntimeSoakReceipt` through public re-exports
  only
- `crates/signal-runtime/examples/supervisor_report_demo.rs` remains the
  human-readable example for downstream users who want to inspect the same
  boundary without reading crate-private modules

That proof is deliberately limited to the frozen runtime/export/report seam.
The deferred host convenience APIs, backend adapters, and CLI presentation
surfaces listed above remain outside this stability promise.

## Rule

For the remainder of `g04.001`, new stability promises should extend this
boundary only by explicit contract or roadmap note. Consumer convenience alone
is not enough to widen the public freeze.

## Next Task

Keep this boundary stable while `g04.002` deepens multicore scheduling and
anticipative execution on top of the now-explicit public/runtime contract.
