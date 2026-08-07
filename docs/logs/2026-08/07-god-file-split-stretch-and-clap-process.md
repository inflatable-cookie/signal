# God-File Split: stretch engine/resumable/callback + CLAP process

Status: complete
Created: 2026-08-07
Scope: `signal-dsp-stretch` stretch_engine, resumable, realtime_preview/callback;
`signal-plugin-clap` hosting/process

## Baseline

After LV2/capture batch: top highs included preview callback (653), resumable
(618), stretch_engine (616), clap process (590).

## What Changed

### `stretch_engine`

→ `stretch_engine/{limits,math,dynamic_ratio,render,pitch_window}`

### `resumable`

→ `resumable/{types,pitch,engine}`

### `realtime_preview/callback`

→ `callback/{accessors,projection,process}` (impl blocks; struct stays in contract)

### `signal-plugin-clap` `hosting/process`

→ `process/{events,buffers,session}`

Move-only. Public re-exports unchanged.

## After

Those four high-band production files cleared from the top of the scan.
Remaining criticals still tests/fixtures/demos only. Next highs include
render-plane `offline/stretch_artifact`, AU hosting instance, stretch_backend,
IPC shared_memory, clap discovery.

## Validation

- `cargo fmt` / `clippy -D warnings` on touched crates
- clap: 15 tests passed
- stretch: offline_high_quality_dynamic_ratio, short_window, resumable (+
  resumable_gates), realtime_preview filters green

## Next Task

Continue high-band prod shrinkage (`stretch_artifact`, `stretch_backend`,
AU instance, clap discovery / IPC) or stop for review.
