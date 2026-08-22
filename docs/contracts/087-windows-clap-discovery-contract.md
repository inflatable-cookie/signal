# 087 Windows CLAP Discovery Contract

Status: active
Owner: core-product
Updated: 2026-08-22
Related contracts: `docs/contracts/086-linux-clap-discovery-contract.md`,
`docs/contracts/083-vst3-discovery-diagnostic-outcome-contract.md`
Consumer evidence: Soundcheck contract 031; Soundcheck
`docs/logs/2026-08/22-windows-scan-lane.md`

## Purpose

Extend CLAP filesystem discovery to Windows so Soundcheck can scan real
CLAP plugins there. Product-pulled backlog from Soundcheck, exactly like
086: no Signal generation, no hosting or sandbox change.

## Authority

- `signal-plugin-clap` owns CLAP scan roots, unit resolution, factory
  introspection, and typed discovery outcomes on every platform.
- Soundcheck and `soundcheck-library` must not hard-code Windows CLAP
  paths.

## Windows scan roots

Default Windows roots, in order (per the CLAP specification):

- `%COMMONPROGRAMFILES%\CLAP`
- `%LOCALAPPDATA%\Programs\Common\CLAP`

`CLAP_PATH` entries append after the defaults; the separator on Windows is
`;` (Linux/macOS keep `:`). An empty explicit root list still scans
nothing; consumers pass default roots to opt in.

Environment-variable expansion in root strings is resolved by the adapter
the same way `~` is on Unix platforms.

## Unit resolution

A Windows CLAP unit is a file whose name ends in `.clap` (a renamed DLL).
Scan it directly. Directory bundles are not part of the Windows CLAP
convention; a directory whose name ends in `.clap` without a recognized
platform layout is not a discovered Windows unit.

`ClapHostPlatform` gains a `Windows` variant; `current_clap_platform()`
maps `target_os = "windows"` to it. macOS and Linux behaviour is
unchanged.

## Out of scope

- hosting, GUI, sandbox lifecycle depth on Windows
- changing VST3 or LV2 discovery (VST3 already models Windows; the
  consumer verifies it with fixtures)
- Soundcheck scan orchestration
- a new Signal generation

## Acceptance

- Windows default roots are queryable from the adapter with `;`-separated
  `CLAP_PATH` append
- a flat `.clap` fixture is discovered on the Windows platform path
  (platform-parameterized tests; no Windows host required)
- macOS and Linux discovery tests remain green and unchanged
