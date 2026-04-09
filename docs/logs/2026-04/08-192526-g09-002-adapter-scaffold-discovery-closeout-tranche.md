# 2026-04-08 19:25:26 - g09.002 adapter scaffold discovery closeout tranche

## Summary

Finished the last open item in `g09.002` Batch 2.2.

AU, VST3, and LV2 discovery no longer depend on crate-level `src/fixtures.rs`
modules. The adapter discovery metadata now lives in production-owned scaffold
modules under each adapter tree, and the old fixture files were removed.

## Files

- `crates/signal-plugin-vst3/src/vst3_host_adapter.rs`
- `crates/signal-plugin-vst3/src/vst3_host_adapter/discovery.rs`
- `crates/signal-plugin-vst3/src/vst3_host_adapter/scaffold.rs`
- `crates/signal-plugin-vst3/src/lib.rs`
- `crates/signal-plugin-au/src/au_host_adapter.rs`
- `crates/signal-plugin-au/src/au_host_adapter/discovery.rs`
- `crates/signal-plugin-au/src/au_host_adapter/scaffold.rs`
- `crates/signal-plugin-au/src/lib.rs`
- `crates/signal-plugin-lv2/src/lv2_host_adapter.rs`
- `crates/signal-plugin-lv2/src/lv2_host_adapter/discovery.rs`
- `crates/signal-plugin-lv2/src/lv2_host_adapter/scaffold.rs`
- `crates/signal-plugin-lv2/src/lib.rs`
- `docs/roadmaps/g09/002-shared-plugin-hosting-substrate-and-hardened-sandbox-execution.md`
- `docs/logs/2026-04/08-192526-g09-002-adapter-scaffold-discovery-closeout-tranche.md`

## Validation

- `cargo test -p signal-plugin-vst3 --lib`
- `cargo test -p signal-plugin-au --lib`
- `cargo test -p signal-plugin-lv2 --lib`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_vst3`
- `cargo test -p signal-host-server --test public_host_edge_vst3`
- `effigy health`

## Outcome

`g09.002` Batch 2.2 is now complete.

The remaining major scaffold in this milestone is the synthetic sandbox process
itself, so the next queue should move directly into Batch 2.3 instead of doing
more discovery churn.

## Next Task

Start `g09.002` Batch 2.3 by defining the typed sandbox broker lifecycle
receipts and replacing the synthetic `signal-plugin-sandbox` lifecycle shell
with one long-lived request-serving process boundary.
