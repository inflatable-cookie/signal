# God-File Split: bridge VST3, hosting process/instance, preview/PV, GUI, plan_render

Status: complete
Created: 2026-08-07
Scope: `signal-plugin-bridge` in_process/vst3; `signal-plugin-vst3` hosting/process,
hosting/instance (thin mod), gui; `signal-dsp-stretch` realtime_preview_stream +
phase_vocoder; `signal-plugin-sandbox` child_gui; `signal-render-plane` plan_render

## Baseline

After plan/bridge-shm/clap/cpal batch: next highs included bridge in_process/vst3
(489), VST3 process (484) / instance/mod (478), realtime_preview_stream (472),
phase_vocoder/mod (465), child_gui (465), plan_render (457), VST3 gui (413).

## What Changed

### bridge `in_process/vst3` → `vst3/{editor,processor,block}`
### VST3 `hosting/process` → `process/{buffers,session,tests}`
### VST3 `hosting/instance` — thin `mod.rs`; body → `hosted.rs`; state → `layout.rs`
### `realtime_preview_stream` → `{constants,types,state_*}`
### `phase_vocoder` → `{entry,run,config,engine,wrap_phase}` (+ existing tests)
### sandbox `child_gui` → `{types,handle,service,editor,macos}`
### `plan_render` → `{envelope,events,interpolate,clips}`
### VST3 `gui` → `{constants,types,view,frame,session,tests}`

Move-only. Public re-exports unchanged. COM vtables stay with callbacks.

## After

Those high-band production files cleared. Remaining criticals still
tests/fixtures/demos/evidence only.

## Validation

- `cargo fmt` / `clippy -D warnings` on touched crates
- bridge vst3, VST3 hosting/gui, stretch phase_vocoder + preview stream gates,
  sandbox editor, render-plane clip_window_gain green

## Next Task

Continue high-band prod shrinkage (runtime execution_topology_family, hardware
midi_input, LV2 lib/parser residue, clap process/session, hardware-coremidi
backend) or stop for review.
