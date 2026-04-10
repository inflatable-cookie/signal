# 079 Interactive Demo Binary And Crate-Capability Proof Contract

Status: active
Owner: core-product
Updated: 2026-04-10
Related contracts: `docs/contracts/011-shared-downstream-conformance-and-release-acceptance-automation-contract.md`, `docs/contracts/071-generation-closeout-and-downstream-workflow-readiness-gate-contract.md`
Related architecture: `docs/architecture/system-architecture.md`, `docs/architecture/system-inventory.md`

## Purpose

Freeze the contract for repo-owned interactive demo binaries that show and
verify what each Signal crate claims, without turning demos into product-local
UI shells.

## Authority hierarchy

1. each crate owns the behavior its demo or proof path claims
2. shared demo substrate owns manifest, launch, scenario, and evidence
   conventions
3. Effigy owns task-level launch and validation entry points
4. demos remain proof and inspection surfaces, not canonical API authority

## Required shared guarantees

- every active crate must map to either:
  - a dedicated demo binary, or
  - a named scenario inside a shared domain demo binary
- demos must be runnable through repo-owned commands, not tribal knowledge
- demos must emit machine-readable manifests or receipts describing:
  - covered crates
  - covered scenarios
  - known exclusions
  - validation commands and expected human checks

## Rules

- demos must stay focused on shared Signal-owned substrate, not downstream app
  shells
- one demo may cover multiple closely related crates when that better matches
  the actual operator workflow
- demos do not replace unit, integration, or acceptance tests; they supplement
  them with inspectable live proof

## Required proof surfaces

- a crate-to-demo coverage matrix in repo docs
- Effigy demo tasks or equivalent repo-owned launch commands
- machine-readable manifest export for each demo binary or scenario bundle

## Next Task

Use this contract while executing
Next-generation planning. `g09` is closed.
