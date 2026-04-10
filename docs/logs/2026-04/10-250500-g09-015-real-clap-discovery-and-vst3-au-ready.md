# 10-250500 - g09.015 Real CLAP Discovery And VST3 AU Ready

## Summary

Completed the first real plugin-discovery burn-down slice for `g09.015`.
`signal-plugin-clap` now scans real `.clap` libraries, and the local/server
host scan and sandbox bring-up paths now carry that scanned CLAP catalog
instead of depending on harness-only `plugin:clap:*` discovery. This closes the
first meaningful realism gap and narrows the remaining discovery blocker to the
VST3 and AU `.txt` metadata shims.

## Code Reality

- added real CLAP discovery in
  `crates/signal-plugin-clap/src/discovery.rs`
- updated `signal-plugin-clap` adapter state to keep a scanned CLAP catalog and
  serve real discovered plugin types during host ensure/restart
- rewired local/server host scan and CLAP sandbox lifecycle paths to use the
  scanned CLAP catalog
- moved the CLAP adapter and public host parity helpers onto compiled temporary
  `.clap` fixtures instead of synthetic scan roots
- confirmed LV2 still has scaffold-backed direct adapter lookup, but it is not
  the active host production discovery path

## Validation

- `effigy tasks`
- `cargo check -p signal-plugin-clap`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-plugin-clap --lib --no-run`
- direct host validation with compiled temporary `.clap` fixtures:
  - `cargo run -q -p signal-host-local`
    - `SIGNAL_HOST_DEMO_PLUGIN_FORMAT=clap`
    - `SIGNAL_HOST_DEMO_PLUGIN_ROOT=<temp_clap_root>`
    - `SIGNAL_HOST_DEMO_PLUGIN_TYPE_ID=plugin:clap:default`
  - `cargo run -q -p signal-host-server`
    - `SIGNAL_HOST_DEMO_PLUGIN_FORMAT=clap`
    - `SIGNAL_HOST_DEMO_PLUGIN_ROOT=<temp_clap_root>`
    - `SIGNAL_HOST_DEMO_PLUGIN_TYPE_ID=plugin:clap:server`
- `effigy health`

## Notes

- focused Rust test binaries for the new real-root CLAP proof lane still showed
  the same machine-local startup instability seen earlier, so I did not use
  that launcher as the closeout truth source for this batch
- the direct host runs prove the new CLAP scan path itself is working against
  actual `.clap` roots
- the next honest discovery seam is the VST3 and AU `.txt` metadata-file
  removal, now promoted as the ready card

## Next Task

Continue the reopened strict `g09` lane from
`docs/specs/batch-cards/045-g09-015-vst3-au-real-introspection-burn-down.md`.
