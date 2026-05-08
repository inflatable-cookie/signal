# g09.015 - Plugin Browser Bounded Interaction Closeout

Status: complete
Date: 2026-04-10
Card: `docs/roadmaps/g09/batch-cards/052-g09-015-plugin-browser-bounded-interaction-proof.md`
Roadmap: `docs/roadmaps/g09/015-operator-visible-interactive-demo-and-plugin-browser-proof.md`

## Summary

Closed the bounded browser interaction batch by promoting launch-only host
bootstrap into one explicit host-owned `parameter-step` interaction proof.

The local and server host demo launch paths now honor bounded interaction env
overrides, apply an automation step to the protocol automation parameter, and
report explicit interaction truth in the host summary line. The plugin browser
now passes that interaction contract during launch, parses the returned summary,
and surfaces bounded interaction state in the operator launch panel and receipt.

## Code And Surface Updates

- added one bounded interaction contract through
  `SIGNAL_HOST_DEMO_INTERACTION_MODE=parameter-step` and
  `SIGNAL_HOST_DEMO_INTERACTION_VALUE`
- local host path now applies the interaction step through the existing payload
  override seam and reports the resulting automation value
- server host path now mirrors that payload override seam and surfaces the same
  interaction truth
- browser launch results now record:
  - `interaction_mode`
  - `interaction_value`
  - `parameter_event_count`
  - `generated_event_bytes`
- browser operator checks now require one bounded interaction proof rather than
  launch success alone

## Validation Run

- `cargo check -p signal-host-local -p signal-host-server`
- `python3 demos/scripts/run_plugin_capability_browser_demo.py --scan-mode fixture`
- `effigy demo:plugin-capability-browser:proof`
- `python3 demos/scripts/run_plugin_capability_browser_demo.py --scan-mode system`
- `effigy qa:docs`
- `effigy qa:northstar`

## Validation Outcome

- fixture-backed browser proof passed and recorded bounded interaction truth
- real system-mode browser run passed on this machine
- current system-mode receipt records:
  - `plugin_count=18`
  - `launch_target_count=23`
  - `launch_status=passed`
  - `launch_package=signal-host-local`
  - `interaction_mode=parameter-step`
  - `parameter_event_count=2`
  - `generated_event_bytes=268`

## Result

- `052-g09-015-plugin-browser-bounded-interaction-proof.md` is complete
- `g09.015` has no current ready card
- the strict lane is back at a planning pause rather than inventing another
  implementation seam

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether the next
honest `g09.015` seam is another crate-family operator-view uplift, deeper live
plugin interaction, or a planning pause.
