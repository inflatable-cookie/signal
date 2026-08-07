# God-File Split: topology, MIDI, broker types, LV2 tests, hosted/session

Status: complete
Created: 2026-08-07
Scope: `signal-runtime` execution_topology_family + sandbox_broker_support/types;
`signal-hardware` midi_input; `signal-hardware-coremidi` backend;
`signal-plugin-lv2` lib tests extract; `signal-plugin-vst3` instance hosted
clusters; `signal-plugin-clap` process session clusters

## Baseline

After bridge/VST3/preview/GUI batch: next highs included topology (435), LV2
lib (434), clap process/session (418), midi_input (407), coremidi backend
(404), broker types (402), VST3 hosted residue (467).

## What Changed

### `execution_topology_family` → `{summaries,topology,build}`
### `midi_input` → `{types,ring,traits,fake,tests}`
### coremidi `backend` → `{cf,backend,subscription,tests}`
### broker `types` → `{receipt,plugin,session,wire}`
### LV2 `lib.rs` → thin root; tests → `tests.rs`
### VST3 `instance/hosted` → `{hosted,load,state,lifecycle,gui}`
### CLAP `process/session` → `{session,prepare,dispatch}`

Move-only. Public re-exports unchanged.

## After

High band largely cleared of production modules. Remaining highs at scan time:
evidence bin, resumable/engine residue, test support / gates / LV2 tests, and
turtle parser (~405). No non-test criticals.

## Validation

- `cargo fmt` / `clippy -D warnings` on touched crates
- hardware midi (10), coremidi (14), runtime sandbox_broker (7), LV2 lib (15),
  VST3 lib (26), clap lib (15) green

## Next Task

Clear remaining prod highs (`resumable/engine`, LV2 turtle `parser`) or stop
for review / doctor baseline reassessment.
