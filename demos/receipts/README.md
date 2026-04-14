# Demo Receipts

Status: active
Updated: 2026-04-14

## Purpose

This folder holds the repo-owned proof artifacts for live Signal demos.

Each live demo currently owns one canonical pair:

- `<surface>.receipt.json`
- `<surface>.view.html`

## What Stays Tracked

- canonical receipt JSON for every live demo
- canonical rendered companion HTML for every live demo

These files are part of the proof contract behind the demo registry. They are
not disposable cache output.

## What Does Not Belong Here

- temporary browser server state
- local-only logs from exploratory runs
- transient scan failures or machine-specific scratch output

## Working Rule

- if a demo remains live in `demos/coverage-matrix.*`, its canonical receipt
  pair should stay tracked here
- if a demo is removed or replaced, remove or replace its proof artifacts in
  the same batch

## Next Task

Keep the receipt set aligned with the live demo registry and avoid adding
secondary artifact conventions unless the registry genuinely needs them.
