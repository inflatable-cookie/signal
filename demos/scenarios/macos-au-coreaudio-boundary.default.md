# macOS AU CoreAudio Boundary

Status: active
Updated: 2026-04-10

## Scenario

- manifest id: `signal.demo.macos.au-coreaudio-boundary`
- scenario id: `signal.demo.macos.au-coreaudio-boundary.default`

## Launch

- command: `effigy demo:macos-au-coreaudio-boundary`
- owner surface: `effigy-task`

## Expected Human Checks

- confirm the receipt captures the current
  `--describe-macos-au-coreaudio-boundary --format=json` descriptor output
- confirm the receipt records that the existing
  `effigy acceptance:macos-au-coreaudio-boundary` lane completed
- confirm the receipt stays explicitly macOS-specific and does not imply Linux
  or general plugin-browsing coverage

## Environment Notes

- this surface depends on the current local macOS AU/CoreAudio proof lane and
  is not intended to run as a cross-platform demo
- it reuses the existing `signal-supervisor-tools` boundary descriptor and
  acceptance task directly instead of inventing a new binary mode

## Evidence Notes

- machine-readable run output is captured in
  `demos/receipts/macos-au-coreaudio-boundary.receipt.json`
- plugin capability browsing remains deferred and should stay explicit rather
  than being implied by this macOS-specific bootstrap surface
