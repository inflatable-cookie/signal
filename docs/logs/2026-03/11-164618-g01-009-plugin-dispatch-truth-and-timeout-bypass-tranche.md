# 2026-03-11 16:46:18 GMT - g01.009 plugin dispatch truth and timeout bypass tranche

## Summary

Advanced `g01.009 / 009.3` by making the runtime-owned transport and parameter
forecast state drive the plugin block request that `signal-host-local` sends to
the sandboxed processor, then pinning that the bound plugin node stays attached
to the engine graph when a block times out and falls back to bypass.

This tranche closes the transport/parameter truth gap left by the earlier
plugin-node render bridge. The remaining `009.3` work is now concentrated on
recovery behavior: restart, overlap cleanup, and fallback continuity while the
plugin-backed node remains part of the same engine path as native nodes.

## What changed

- extended `crates/signal-runtime/src/interfaces.rs`,
  `crates/signal-runtime/src/lib.rs`, and
  `crates/signal-runtime/src/runtime.rs` with
  `RuntimePluginDispatchState` plus
  `SignalRuntime::prepare_plugin_dispatch_state_for_block(...)` so runtime owns
  the plugin-facing transport projection and parameter batch truth for each
  engine block
- updated `crates/signal-host-local/src/host.rs` so block dispatch now:
  - asks runtime for plugin dispatch state before sandbox processing
  - builds `PluginRenderContext` from runtime-owned transport truth
  - injects the runtime-owned automation value for the bound parameter lane into
    the plugin payload instead of trusting fixture-local values
  - keeps the later plugin-node render batch injection path unchanged, so
    sandbox output still enters the graph through the bound node seam
- added focused host-local coverage for:
  - plugin block request construction from runtime transport and parameter truth
  - timeout fallback that bypasses the plugin node without detaching its graph
    binding
- updated existing host-local watchdog/recovery expectations where automation
  assertions now reflect runtime-owned forecast values rather than earlier
  fixture-local event values

## Validation

- `cargo fmt`
- `cargo test -p signal-runtime`
- `cargo test -p signal-host-local`
- `git diff --check`
- `effigy validate`
- `effigy test`

## Follow-on

The next `009.3` batch should exercise watchdog restart, timeout fallback,
overlap recovery, and final fallback/reporting behavior while the runtime-owned
plugin dispatch state continues to feed the same bound plugin-node render path.
