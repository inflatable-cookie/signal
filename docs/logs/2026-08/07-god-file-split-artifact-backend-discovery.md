# God-File Split: stretch artifact/backend + CLAP discovery

Status: complete
Created: 2026-08-07
Scope: `signal-render-plane` offline/stretch_artifact; `signal-dsp-stretch`
stretch_backend; `signal-plugin-clap` discovery

## Baseline

After stretch/callback/clap-process batch: next highs included stretch_artifact
(597), stretch_backend (559), clap discovery (542).

## What Changed

### `offline/stretch_artifact`

→ `stretch_artifact/{types,errors,planning,rendering,build,bridge}`

### `stretch_backend`

→ `stretch_backend/{types,time_stretcher,phase_vocoder,realtime_preview,offline_high_quality}`

### `discovery`

→ `discovery/{paths,entry,build,probe,extensions,host,util}`

Move-only. Public re-exports unchanged.

## After

Those three high-band production files cleared. Remaining criticals still
tests/fixtures/demos/evidence only.

## Validation

- `cargo fmt` / `clippy -D warnings` on touched crates
- render-plane: stretch_artifact + offline:: filters green
- stretch: offline_high_quality + backend_plan green
- clap: discovery adapter tests green

## Next Task

Continue high-band prod shrinkage (AU hosting instance, IPC shared_memory,
VST3 wire host_application/stream, runtime client_session, cache_identity,
render-plane plan) or stop for review.
