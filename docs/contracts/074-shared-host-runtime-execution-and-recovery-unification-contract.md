# 074 Shared Host/Runtime Execution And Recovery Unification Contract

Status: draft
Owner: core-product
Updated: 2026-04-08
Related contracts: `docs/contracts/012-runtime-interruption-taxonomy-and-resumability-contract.md`, `docs/contracts/014-plugin-isolation-policy-transport-rebind-and-shared-sandbox-continuity-contract.md`, `docs/contracts/015-offline-render-recovery-and-resumability-contract.md`, `docs/contracts/025-device-supervision-restart-state-machine-and-fault-boundary-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the contract for removing duplicated execution-block, watchdog, broker,
and recovery policy from `signal-host-local` and `signal-host-server` by
promoting shared behavior into runtime-owned or host-support-owned substrate.

## Authority hierarchy

1. `signal-runtime` owns execution context, recovery state, continuity meaning,
   and typed receipts.
2. shared host-support modules own reusable orchestration helpers that stay
   outside the real-time graph path.
3. `signal-host-local` and `signal-host-server` own only their environment-
   specific ingress, transport, and process-boundary adaptations.

## Required shared guarantees

- equivalent recovery cases must pass through one reusable policy surface.
- local and server hosts must not diverge on completion-slot, lingering
  session, or broker-failure meaning.
- realtime block processing must stay allocation-safe and bounded while shared
  logic moves out of host-specific roots.

## Rules

- duplicate behavior may remain only where the environment truly differs.
- host-specific wrappers may not redefine runtime fault or continuity meaning.
- new recovery or execution features must land in the shared seam first.

## Required proof surfaces

- paired local/server conformance tests over the same scenarios
- duplicate-block scan reduction evidence
- one interactive local-vs-server continuity demo under contract `079`

## Next Task

Use this contract to drive the `g09` host/runtime unification milestone before
adding more host-specific recovery depth.
