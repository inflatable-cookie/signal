# 076 Low-Level Correctness, Safety, And Protocol Hardening Contract

Status: active
Owner: core-product
Updated: 2026-04-08
Related contracts: `docs/contracts/001-shared-dsp-and-host-boundary.md`, `docs/contracts/002-supervisor-export-schema-and-report-boundary.md`, `docs/contracts/014-plugin-isolation-policy-transport-rebind-and-shared-sandbox-continuity-contract.md`
Related architecture: `docs/architecture/system-architecture.md`

## Purpose

Freeze the contract for hardening low-level correctness and process-boundary
behavior across graph routing, primitive invariants, CLAP sandbox handling, and
shared-memory IPC.

## Required shared guarantees

- unsupported graph layout adaptation must surface explicit error or degraded
  receipts instead of silent zeroing
- primitive constructors must reject invalid or lossy states
- protocol handlers must return typed failures instead of panic-oriented
  `expect(...)` paths
- shared-memory ownership, cleanup, and permission posture must be explicit and
  inspectable

## Rules

- production protocol paths may not rely on panic for expected drift handling
- correctness fixes must preserve real-time safety on audio-thread paths
- degraded behavior must be typed and reportable, not implicit

## Required proof surfaces

- focused negative tests for invalid layout, buffer, and protocol inputs
- IPC lifecycle tests for stale-region cleanup and ownership loss
- stable runtime or supervisor receipts for degraded-path behavior where
  recovery is allowed

## Next Task

Use this contract for the active strict `g09.008` lane. If no further bounded
hardening seam remains, stop and hand the lane back to planning before
continuing into `g09.009`.
