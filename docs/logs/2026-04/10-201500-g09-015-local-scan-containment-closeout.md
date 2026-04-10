# 10-201500 - g09.015 Local Scan Containment Closeout

## Summary

Closed `048` by containing local plugin visibility per exact plugin root
instead of relying on one broad local scan over system plugin directories.

## Meaningful Changes

- replaced the browser's single broad local scan dependency with bounded
  exact-root local probes in
  `demos/scripts/run_plugin_capability_browser_demo.py`
- kept local launch buttons tied to genuine local scan truth by validating each
  plugin candidate against the local host example individually
- limited local probe cost with explicit attempt, success, and timeout bounds
  so one problematic plugin does not suppress all local visibility
- updated the interactive/default browser task wording so the operator-visible
  path and proof path remain explicit
- hardened the receipt capture to try a bounded set of launch candidates until
  one supported path succeeds, rather than letting one bad first pick fail the
  whole interactive run

## Validation

- `effigy demo:plugin-capability-browser:proof`
- `python3 demos/scripts/run_plugin_capability_browser_demo.py --scan-mode system`
- `effigy qa:docs`
- `effigy qa:northstar`

## Observed Outcome

- the system-mode browser run completed successfully on this machine
- the generated receipt recorded `515` discovered plugins and `6` locally
  validated launch targets
- the bounded launch artifact recorded a passed local VST3 launch:
  `plugin:vst3:com-wizoo-hybrid:97ff2b90447fa3bcc58a588bc331712b`

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether the next
honest `g09.015` seam is deeper live plugin interaction, broader plugin-browser
operator posture, or a planning pause.
