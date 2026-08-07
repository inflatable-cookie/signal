# God-File Split: AU instance, IPC SHM, VST3 wire, cache identity, client session

Status: complete
Created: 2026-08-07
Scope: `signal-plugin-au` hosting/instance; `signal-ipc` shared_memory;
`signal-plugin-vst3` wire/host_application + wire/stream; `signal-dsp-stretch`
cache_identity; `signal-runtime` sandbox_broker_support/client_session

## Baseline

After artifact/backend/discovery batch: next highs included AU instance (561),
VST3 host_application (548) / stream (543), IPC shared_memory (546), runtime
client_session (539), stretch cache_identity (536).

## What Changed

### AU `hosting/instance`

→ `instance/{layout,format,params,hosted}`

### IPC `shared_memory`

→ `shared_memory/{error,region,broker,metadata,permissions}`

### VST3 `wire/host_application`

→ `host_application/{attribute_list,message,application,tests}`

### VST3 `wire/stream`

→ `stream/{memory_stream,state_envelope,constants,factory,component,process_types,edit_controller,component_handler,com_helpers}`

### `cache_identity`

→ `cache_identity/{types,input,identity,tests}`

### `client_session`

→ `client_session/{spawn,wire,session,plugin,editor,params}`

Move-only. Public re-exports unchanged. COM vtables stay with their callbacks.

## After

Those six high-band production files cleared from the top of the scan.
Remaining criticals still tests/fixtures/demos/evidence only.

## Validation

- `cargo fmt` / `clippy -D warnings` on touched crates
- AU hosting filters, IPC shared_memory, VST3 host_application + full vst3
  lib/hosting, stretch cache_identity, runtime sandbox_broker_support +
  plugin_hosting green

## Next Task

Continue high-band prod shrinkage (render-plane plan/binaural_bank/executor,
clap hosting instance, stretch transient_smear / resumable engine residue,
hardware-cpal input, bridge shm/vst3) or stop for review.
