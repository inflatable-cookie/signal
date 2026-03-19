# 2026-03-19 - g07.020 Closeout Gate Surface Tranche

## Summary

Implemented the runnable `g07` closeout surface by retargeting the shared
generation-closeout descriptor to the current generation and wiring the matching
repo-owned Effigy gate.

## Why this tranche matters

`g07.020` could not finish on contract text alone. This tranche turns the
frozen closeout policy into one typed gate that Batch 20.3 can review, instead
of forcing the final Loophole-facing verdict to invent its own validation
surface or rely on prose-only judgment.

## What changed

- updated `crates/signal-supervisor-tools/src/main.rs` so the shared
  generation-closeout descriptor now reports `g07`-specific contract, roadmap,
  grouped-acceptance, provisional readiness, and residual-risk state
- added the repo-owned `acceptance:g07-closeout` task in `effigy.toml`
- rolled the roadmap, contract, architecture, and shared next-task pointers
  forward so Batch 20.3 is now the explicit next queue

## Validation

- `cargo fmt --all`
- `cargo test -p signal-supervisor-tools generation_closeout_json_reports_combined_boundary_and_next_queue -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-generation-closeout --format=json`
- `effigy acceptance:g07-closeout --repo .`
- `git diff --check`
- `effigy health --repo .`
- `effigy validate --repo .`
- `effigy qa:docs --repo .`

## Residual risk

The final Loophole-facing readiness verdict and any `g08`-or-backlog decision
still belong to Batch 20.3, and richer Linux live-backend, immersive-routing,
vendor-protocol hardware, and product-local preview workflows remain explicitly
deferred beyond the bounded `g07` closeout gate.

## Next task

Continue `g08.001` with Batch 1.1 by freezing the runtime-owned live Linux
audio backend ownership and session-lifecycle contract before deeper ALSA,
JACK, and PipeWire runtime realization widens.
