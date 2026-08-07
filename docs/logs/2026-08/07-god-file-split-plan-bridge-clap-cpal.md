# God-File Split: plan/binaural/executor, CLAP instance, bridge shm, smear, cpal

Status: complete
Created: 2026-08-07
Scope: `signal-render-plane` plan, binaural_bank, plane/executor;
`signal-plugin-clap` hosting/instance; `signal-plugin-bridge` shm;
`signal-dsp-stretch` transient_smear; `signal-hardware-cpal` input

## Baseline

After AU/IPC/VST3/runtime batch: next highs included plan (528), binaural_bank
(510), clap instance (507), transient_smear (506), cpal input (503), executor
(500), bridge shm (497).

## What Changed

### `plan` → `plan/{types,compile,inherit}`
### `binaural_bank` → `binaural_bank/{types,bank,processor,tests}`
### `plane/executor` → `executor/{control,health,render}`
### CLAP `hosting/instance` → `instance/{layout,state_io,shape,hosted}`
### bridge `shm` → `shm/{budget,processor,tests}`
### `transient_smear` → `transient_smear/{types,detect,measure,features}`
### cpal `input` → `input/{types,enumerate,backend,stream,tests}`

Move-only. Public re-exports unchanged.

## After

Those seven high-band production files cleared. Remaining criticals still
tests/fixtures/demos/evidence only.

## Validation

- `cargo fmt` / `clippy -D warnings` on touched crates
- Targeted filters: plan/compile, binaural, clap lib, shm, transient_smear,
  cpal input, render-plane lib (145) green

## Next Task

Continue high-band prod shrinkage (bridge in_process/vst3, VST3 hosting
process/instance residue, stretch realtime_preview_stream / phase_vocoder,
sandbox child_gui, plan_render) or stop for review.
