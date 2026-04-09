# Hardware Topology Diagnostics

Status: active
Updated: 2026-04-09

## Scenario

- manifest id: `signal.demo.hardware.topology-diagnostics`
- scenario id: `signal.demo.hardware.topology-diagnostics.default`

## Launch

- command: `effigy demo:hardware-topology-diagnostics`
- owner surface: `effigy-task`

## Expected Human Checks

- confirm the local host exports native CoreAudio device, stream, endpoint, and
  external-I/O posture through the existing summary line
- confirm the server host exports simulated Linux backend session posture,
  device-supervision truth, and backend-specific summaries through the existing
  summary line
- confirm the receipt keeps native-versus-simulated posture explicit rather
  than pretending the two sides are equivalent

## Environment Notes

- this surface runs the existing `signal-host-local` and `signal-host-server`
  binaries from the current workspace
- it does not require a new host UI or custom device scan root
- the Linux-side hardware posture is still simulated through the current server
  host summary output rather than claiming native Linux ownership

## Evidence Notes

- plugin capability browsing remains deferred and should stay explicit in the
  receipt rather than being implied by this hardware bootstrap surface
