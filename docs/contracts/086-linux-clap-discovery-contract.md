# 086 Linux CLAP Discovery Contract

Status: active
Owner: core-product
Updated: 2026-08-21
Related contracts: `docs/contracts/022-backend-capability-parity-linux-plugin-support-and-cross-adapter-conformance-contract.md`,
`docs/contracts/072-real-plugin-hosting-discovery-and-sandbox-execution-contract.md`,
`docs/contracts/083-vst3-discovery-diagnostic-outcome-contract.md`
Consumer evidence: Soundcheck contract 030; Soundcheck
`docs/logs/2026-08/21-linux-scan-contract.md`

## Purpose

Freeze Linux CLAP filesystem discovery so Soundcheck can scan real CLAP
plugins on Linux. This is a product-pulled backlog item from Soundcheck. It
does not open a Signal generation, reopen hosting, or change sandbox
lifecycle.

## Authority

- `signal-plugin-clap` owns CLAP scan roots, bundle/file resolution, factory
  introspection, and typed discovery outcomes.
- Soundcheck and `soundcheck-library` own scan jobs and inventory meaning.
  They must not hard-code macOS CLAP library paths on Linux.
- Hosting, GUI, and sandbox remain existing CLAP adapter contracts. This file
  only covers find-and-describe.

## Linux scan roots

Default Linux roots, in order:

- `~/.clap`
- `/usr/lib/clap`
- `/usr/local/lib/clap`

`CLAP_PATH` entries, colon-separated, append after those defaults. An empty
explicit root list still scans nothing; consumers pass default roots to opt
in, same as VST3/LV2.

macOS roots stay `~/Library/Audio/Plug-Ins/CLAP` and
`/Library/Audio/Plug-Ins/CLAP`.

## Binary resolution

A CLAP unit is one of:

1. A file whose name ends in `.clap` (a cdylib). Scan it directly.
2. A directory whose name ends in `.clap`. Resolve the binary as the first
   existing file among:

   - `Contents/x86_64-linux/<stem>` or `<stem>.so`
   - `Contents/aarch64-linux/<stem>` or `<stem>.so`
   - `Contents/MacOS/<any file>` (macOS only)

Do not treat `Contents/MacOS` as the Linux layout. A Linux bundle that only
has `Contents/MacOS` is not a discovered Linux unit.

`compile_clap_fixture` already emits a flat `.clap` file. Tests must cover
both that file and a directory bundle using the Linux Contents layout for the
host architecture.

## Out of scope

- AU
- changing VST3 or LV2 discovery
- Soundcheck scan orchestration
- REAPER
- a new Signal generation
- claiming CLAP hosting depth beyond current Linux GUI/process support

## Acceptance

- Linux default roots are queryable from the adapter
- a flat `.clap` fixture is discovered
- an `aarch64-linux` or `x86_64-linux` bundle fixture matching the host is
  discovered
- a macOS-only bundle is not discovered as a Linux unit
