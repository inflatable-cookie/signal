# 2026-03-10 15:35:00 - Host assembly-driven plugin bindings

## Summary

- moved plugin-backed node bindings in `signal-host-local` and `signal-host-server` behind host assembly descriptions that declare:
  - graph projection
  - plugin sandbox inventory
  - plugin-backed node bindings
- updated boot flows and host test setup helpers to consume the same assembly seam instead of separate graph and binding helpers
- kept runtime-owned plugin-constrained scheduler behavior intact while making the binding source reflect real host assembly inputs

## Why

- the previous runtime binding projection work proved the scheduler could react to bound active/degraded/missing sandbox state
- hosts were still feeding that path through separate demo helpers, which left the plugin-backed ownership seam more artificial than it needed to be
- this batch makes the next plugin/runtime integration step cleaner by collapsing graph, sandbox, and binding ownership into one host assembly input

## Validation

- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime runtime_plugin_bindings_project_into_snapshot_and_track_bound_sessions -- --nocapture`
- `cargo test -p signal-host-local local_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-supervisor-tools --no-run`

## Next

- drive plugin-backed node bindings from more real host/plugin assembly data, such as plugin instance descriptors or broker-owned sandbox metadata, so runtime scheduling no longer depends on demo node-to-sandbox naming conventions at all
