# `g09.003` closeout audit and promotion

## Summary

Closed `g09.003` after removing the last production scaffold holdover from the
VST3 lane and confirming the remaining bounded behavior is deliberate rather
than an accidental fallback.

## Code landed

- removed the scaffold-backed production `discover_plugin_type(...)` shortcut
  from
  `/Users/betterthanclay/Dev/projects/signal/crates/signal-plugin-vst3/src/vst3_host_adapter/discovery.rs`
- moved the VST3 scaffold module behind `#[cfg(test)]` in
  `/Users/betterthanclay/Dev/projects/signal/crates/signal-plugin-vst3/src/vst3_host_adapter.rs`
- trimmed the old unused production helper wall out of the test-only scaffold
  file in
  `/Users/betterthanclay/Dev/projects/signal/crates/signal-plugin-vst3/src/vst3_host_adapter/scaffold.rs`

## Validation

Passed:

- `cargo check -p signal-plugin-vst3`
- `cargo test -p signal-plugin-vst3 --lib`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `effigy health`

## Closeout notes

- the remaining bounded VST3 behavior is the explicit contract of this
  milestone: metadata-driven bundle discovery and bounded adapter/broker
  execution proof
- what was removed here was the stray production scaffold fallback that no
  longer belonged in that contract

## Next

Start `g09.004` with one meaningful baseline batch: audit the current AU
adapter and `signal-hardware-coreaudio` surfaces for the biggest remaining
scaffold seams, then land the first real production-depth pass on whichever is
more central, most likely CoreAudio device truth or AU bundle discovery.
