# Local Server Host Comparison

Status: active
Updated: 2026-04-09

## Scenario

- manifest id: `signal.demo.host.local-server-compare`
- scenario id: `signal.demo.host.local-server-compare.default`

## Launch

- command: `effigy demo:local-server-host-comparison`
- owner surface: `effigy-task`

## Expected Human Checks

- confirm both host binaries start without falling back to the old explicit
  CLAP unsupported-path error
- confirm the comparison receipt keeps shared lifecycle truth explicit:
  readiness, running posture, active sandbox, processed blocks, and completion
- confirm the receipt also preserves real local-versus-server differences such
  as backend and engine-output posture instead of pretending the hosts are
  identical

## Environment Notes

- this surface runs the existing `signal-host-local` and `signal-host-server`
  binaries from the current workspace
- it does not require a separate demo scan root because the default host demo
  assemblies now carry explicit CLAP plugin ids

## Evidence Notes

- plugin capability browsing and hardware diagnostics remain deferred and should
  stay explicit in the receipt rather than being implied by this comparison
  surface
