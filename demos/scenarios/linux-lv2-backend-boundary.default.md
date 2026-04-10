# Linux LV2 And Backend Boundary

Status: active
Updated: 2026-04-10

## Scenario

- manifest id: `signal.demo.linux.lv2-backend-boundary`
- scenario id: `signal.demo.linux.lv2-backend-boundary.default`

## Launch

- command: `effigy demo:linux-lv2-and-backend-boundary`
- owner surface: `effigy-task`

## Expected Human Checks

- confirm the receipt captures the current
  `--describe-linux-lv2-execution-boundary --format=json` descriptor output
- confirm the receipt captures the current
  `--describe-linux-audio-backend-boundary --format=json` descriptor output
- confirm the receipt records that the existing
  `effigy acceptance:linux-lv2-execution-boundary` and
  `effigy acceptance:linux-audio-backend-boundary` lanes completed
- confirm the receipt stays explicitly Linux-specific and does not imply a
  generalized plugin browser or live Linux device-ownership breadth

## Environment Notes

- this surface depends on the current `signal-supervisor-tools` Linux boundary
  descriptors and their existing acceptance lanes
- it does not require a new host UI, owned demo scan roots, or a plugin browse
  shell
- the backend half of the surface proves runtime-owned Linux backend identity
  and fallback posture, not broader live Linux ownership

## Evidence Notes

- machine-readable run output is captured in
  `demos/receipts/linux-lv2-backend-boundary.receipt.json`
- plugin capability browsing remains deferred and should stay explicit rather
  than being implied by this Linux boundary bootstrap surface
