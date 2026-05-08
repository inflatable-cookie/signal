# 2026-04-10 - g09.015 AU Info.plist Migration And VST3 Split

## Summary

Closed the AU half of the real plugin discovery burn-down by replacing the
remaining `signal-au-component.txt` production discovery shim with real
`Contents/Info.plist` component metadata parsing, then split the still-deeper
VST3 discovery seam into its own ready batch.

## Work Completed

- replaced AU production discovery metadata loading in
  `crates/signal-plugin-au/src/au_host_adapter/introspection.rs` with real
  `Info.plist` parsing via `plist`
- preserved bounded AU capability and failure truth by reading custom Signal
  keys when present and deriving sensible defaults from the real
  `AudioComponents` tuple otherwise
- migrated AU fixture writers in the plugin crate, local/server host test
  support, and local/server public host-edge support from
  `signal-au-component.txt` to generated `Contents/Info.plist`
- confirmed the AU `.txt` shim no longer appears in the active AU and host
  proof roots
- corrected the active strict lane so `045` closes on the AU landing and `046`
  becomes the new ready card for VST3 class-factory discovery

## Why The Batch Split

Installed AU bundles on the active machine expose the real component tuple in
`Contents/Info.plist`, so AU introspection was honestly batchable now.
Installed VST3 bundles did not expose an equivalent cheap metadata-only path
for factory and controller truth, so keeping AU and VST3 in one batch would
have hidden a materially deeper VST3 seam.

## Validation Run

- `cargo check -p signal-plugin-au`
- `cargo test -p signal-plugin-au --lib`
- `cargo run -q -p signal-host-local` with:
  - `SIGNAL_HOST_DEMO_PLUGIN_FORMAT=au`
  - `SIGNAL_HOST_DEMO_PLUGIN_ROOT=<temporary .component root>`
  - `SIGNAL_HOST_DEMO_PLUGIN_TYPE_ID=plugin:au:instrument`
- `cargo run -q -p signal-host-server` with:
  - `SIGNAL_HOST_DEMO_PLUGIN_FORMAT=au`
  - `SIGNAL_HOST_DEMO_PLUGIN_ROOT=<temporary .component root>`
  - `SIGNAL_HOST_DEMO_PLUGIN_TYPE_ID=plugin:au:instrument`
- `effigy qa:docs`
- `effigy qa:northstar`

## Notes

Focused public AU exact-test binaries still showed the same machine-local test
launcher instability seen in earlier lanes, so direct local/server host runs
against temporary `.component` bundles were used as the honest closeout truth
source for this batch.

## Next Task

Continue the reopened strict `g09` lane from
`docs/roadmaps/g09/batch-cards/046-g09-015-vst3-class-factory-discovery-burn-down.md`.
