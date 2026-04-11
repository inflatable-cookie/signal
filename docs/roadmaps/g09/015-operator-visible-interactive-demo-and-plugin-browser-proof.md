# 015 - Operator-Visible Interactive Demo And Plugin Browser Proof

Status: complete
Owner: core-product
Created: 2026-04-10
Depends on: g09.014
Vision tags: `DEMOS`, `OPERATOR`, `VISIBILITY`
Contract refs: `079`, `081`

## Problem

`g09` closed the crate-readiness gate, but the current demo surfaces are still
too receipt-heavy to satisfy direct operator verification for every claimed
crate capability. The biggest remaining gap is plugin interaction: Signal can
host and validate bounded CLAP, VST3, AU, and LV2 paths, but there is not yet
one official low-dependency interactive surface where an operator can browse an
installed plugin and interact with it live. The deeper blocker is now explicit:
plugin discovery is still not real enough across formats for that browser claim
to be honest.

## Goals

- define the role-correct interaction model for operator-visible demos without
  introducing a heavyweight in-repo UI stack
- remove the remaining `.txt` and scaffold-backed plugin discovery shims that
  would make an installed-plugin browser dishonest
- turn the remaining planned demo gaps into explicit runnable surfaces
- prioritize plugin browsing and live plugin interaction first, then widen the
  same operator-visible posture across the rest of the demo suite

## Non-Goals

- no downstream product shell hidden inside Signal
- no broad UI framework adoption without separate justification
- no new plugin-hosting capability claims beyond what production code already
  supports

## Execution Plan

### Batch 15.1 - Interaction Strategy And Gap Inventory

- [ ] define the low-dependency interaction model for operator-visible demos
- [ ] inventory which existing demos are already sufficiently inspectable and
      which remain receipt-only
- [ ] promote the first honest follow-on batch after the strategy is known

#### Batch 15.1 Outcome

- defined the low-dependency interaction model for operator-visible demos
- confirmed the highest-value missing interactive surface is still the plugin
  capability browser
- corrected the execution order after adapter inspection:
  - the browser cannot honestly ship first because current CLAP discovery is
    still harness-backed and VST3/AU discovery still relies on Signal-specific
    metadata files
  - real discovery burn-down must happen before the browser becomes an honest
    official surface
- promoted the next ready batch as `044-g09-015-real-plugin-discovery-gap-burn-down.md`

### Batch 15.2 - Real Plugin Discovery Burn-Down

- [x] add real installed-plugin discovery for CLAP
- [x] narrow the remaining discovery blocker explicitly
- [x] remove Signal-specific metadata-file dependency from AU
- [ ] split and remove the remaining VST3 metadata-file dependency
- [ ] make the plugin browser claim honest before the browser is implemented

#### Batch 15.2 Outcome

- landed a real CLAP root-scan path using actual `.clap` libraries through
  `signal-plugin-clap`
- rewired the local and server host CLAP scan/ensure/restart flow to use a
  scanned CLAP catalog instead of synthetic ids
- proved the CLAP path through direct host runs against compiled temporary
  `.clap` fixtures
- completed the AU half of the discovery burn-down by replacing
  `signal-au-component.txt` with real `Info.plist` component metadata parsing
- narrowed the remaining blocker precisely:
  - VST3 still depends on `signal-vst3-module.txt` and
    `signal-vst3-factory.txt`
  - LV2 scaffold-backed direct lookup remains in the adapter, but it is not the
    active host production discovery path
- split the next ready batch explicitly as
  `046-g09-015-vst3-class-factory-discovery-burn-down.md`

### Batch 15.3 - VST3 Class-Factory Discovery Burn-Down

- [x] remove VST3 `.txt` metadata-file discovery
- [x] derive enough real module and class identity to support installed-plugin
      browser claims honestly
- [x] keep the browser deferred until installed-plugin browsing is honest across
      CLAP, VST3, and AU

#### Batch 15.3 Outcome

- replaced the remaining VST3 `.txt` shim with official `moduleinfo.json`
  parsing plus real module/class-factory fallback when moduleinfo is absent
- kept bounded Signal descriptor/io truth via `Info.plist` keys when present,
  while allowing real installed bundles to derive plugin identity from bundle
  and class data without private `.txt` files
- migrated the repo-owned VST3 fixture, test-support, public host-edge, and
  demo bundle writers onto real `Info.plist` plus `moduleinfo.json` surfaces
- validated the new shape through `signal-plugin-vst3` crate tests and focused
  local/server public VST3 host-edge proofs
- reactivated the next ready batch as
  `043-g09-015-plugin-capability-browser-bootstrap.md`

### Batch 15.4 - Plugin Browser And Interaction

- [x] add an official operator-visible plugin capability browser
- [x] let an operator browse installed plugins and launch supported live
      interaction paths through repo-owned commands
- [x] keep unsupported formats or platform gaps explicit rather than implied

#### Batch 15.4 Outcome

- added `signal.demo.plugin.capability-browser` as the first official
  operator-visible plugin browser surface
- added dedicated local/server scan examples that preserve per-plugin launch
  roots across CLAP, VST3, AU, and LV2 discovery
- wrapped the inventory with a lightweight browser-native HTML surface plus
  repo-owned launch wrapper rather than a heavyweight UI stack
- kept the browser honest about current limits:
  - bounded host bootstrap instead of editor embedding
  - local host launch remains macOS-only
  - fixture-backed proof mode is used for repeatable non-interactive validation
    because arbitrary installed plugins can still hang during discovery on this
    machine
- promoted the plugin family into the live demo coverage matrix instead of
  leaving `signal.demo.plugin.capability-browser` deferred

### Batch 15.5 - Honest Local Launch Targets

- [x] remove inferred local launch targets from the plugin browser
- [x] switch browser launch roots to exact per-plugin bundle/library roots
- [x] keep local launch buttons visible only for plugins returned by the local
      scan inventory

#### Batch 15.5 Outcome

- VST3 and AU discovery now accept exact bundle paths as scan roots rather than
  requiring only parent directory roots
- the local/server browser inventory examples now emit exact per-plugin launch
  roots instead of broad directory roots for VST3, AU, and LV2
- the browser no longer synthesizes local launch buttons from server-only
  inventory; local buttons are present only when the local scan surface
  actually returns that plugin
- fixture-backed browser proof remains green while live system mode stays
  explicit about local-scan failure or timeout instead of pretending every
  server-discovered plugin is locally launchable

### Batch 15.6 - Local Scan Containment And Visibility

- [x] keep the default interactive browser surface useful on real multi-plugin
      systems even when some local plugins misbehave
- [x] preserve honest local launch buttons without falling back to inferred
      server-only targets
- [x] keep local scan degradation explicit to the operator

#### Batch 15.6 Outcome

- replaced the browser's broad local scan dependence with bounded exact-root
  local probes so one problematic plugin no longer suppresses all local
  visibility
- kept local launch buttons tied to genuinely local scan truth by validating
  exact plugin roots one by one instead of synthesizing local targets from
  server-only discovery
- preserved the stable proof path under
  `effigy demo:plugin-capability-browser:proof`
- verified the real system-mode browser run now surfaces local launch targets
  on this machine and records a passed bounded local VST3 launch in the receipt

### Batch 15.7 - Browser Operator Posture Uplift

- [x] make local/server availability obvious at a glance in the browser
- [x] surface local probe containment and degradation posture more clearly
- [x] present bounded launch outcomes more clearly than raw JSON alone

#### Batch 15.7 Outcome

- added explicit availability chips so an operator can tell local-only,
  server-only, dual-surface, or no-launch posture at a glance
- made the interaction column read as operator posture instead of raw internal
  phrasing while still preserving the bounded host-bootstrap truth
- upgraded the launch area to show immediate launch state plus clear
  passed/failed/timeout posture before the raw JSON detail
- kept the surface browser-native and low-dependency instead of widening into a
  heavier UI runtime

### Batch 15.8 - Wider Interactive Proof Uplift

- [x] widen operator-visible proof across the remaining crate-family demos that
      still lean too heavily on receipts alone
- [ ] close `g09` again only once the intended interactive visibility scope is
      explicit and honest

#### Batch 15.8 Outcome

- analysis was the next honest seam because it already had bounded offline
  examples and structured output
- `signal.demo.analysis.feature-inspector` now emits a rendered companion view
  so rhythm, tonal, loudness, and character-semantic posture are visually
  inspectable instead of receipt-only
- the uplift stayed inside low-dependency presentation and did not require new
  runtime, device, or plugin-host capability

### Batch 15.9 - Plugin Browser Live-Scan Resilience

- [x] keep the interactive plugin browser useful on real machine plugin roots
- [x] contain scan interrupts, timeouts, and stale listeners so the browser
      does not become unkillable or all-or-nothing
- [x] keep the live scan bounded and honest instead of pretending to be an
      exhaustive crawler

#### Batch 15.9 Outcome

- browser scan wrappers now launch host scans in isolated process groups and
  tear them down explicitly on timeout or interrupt
- interactive system-mode scans now use bounded exact-root batches with
  smaller time budgets instead of broad directory-wide scans
- macOS interactive runs now prefer bounded local inventory first and then add
  server enrichment over locally confirmed roots instead of making server scan
  the single point of failure
- browser serve startup now auto-selects a free localhost port starting at
  `8765` instead of dying when an old listener is still around
- validated a real system-mode browser receipt on this machine with nonempty
  inventory and a passed bounded local VST3 launch

### Batch 15.10 - Plugin Browser Bounded Interaction Proof

- [x] deepen the browser from launch-only bootstrap into one bounded live
      interaction proof
- [x] keep the interaction host-owned and low-dependency rather than widening
      into editor embedding or persistent session control
- [x] make the browser result show plugin/event interaction truth beyond boot
      success alone

#### Batch 15.10 Outcome

- the browser launch path now injects one bounded host-owned `parameter-step`
  interaction through the existing demo override surface instead of remaining a
  pure bootstrap proof
- local and server host summaries now surface explicit interaction truth:
  interaction mode, applied automation value, parameter-event count, and
  generated event bytes
- the browser launch panel and receipt now treat bounded interaction visibility
  as a first-class operator check instead of leaving it buried in raw summary
  text
- validated both the fixture-backed proof path and a real system-mode run on
  this machine with a passed bounded local interaction result

### Batch 15.11 - Graph Execution Operator View

- [x] add a rendered operator companion for the graph execution inspector
- [x] keep the graph family grounded in existing descriptor and acceptance
      proof data
- [x] avoid widening into graph editing, routing mutation, or a product shell

#### Batch 15.11 Outcome

- `signal.demo.graph.execution-inspector` now emits a rendered companion view
  at `demos/receipts/graph-execution-inspector.view.html`
- the multichannel, sidechain, multi-bus, and spatial boundary families are
  now visually inspectable as operator cards instead of receipt-only JSON
- the graph uplift stayed presentation-only over existing descriptor and
  acceptance data
- while closing the batch, the inherited graph proof spine was corrected:
  multichannel, sidechain, multi-bus, and spatial acceptance lanes now use
  exact focused runtime proofs instead of loose `cargo test` filters that
  could execute the wrong runtime binaries or zero host-edge tests

### Batch 15.12 - DSP Processing Operator View

- [x] add a rendered operator companion for the DSP processing lab
- [x] keep the DSP family grounded in existing descriptor and acceptance proof
      data
- [x] avoid widening into waveform browsing, sample editing, or a product shell

#### Batch 15.12 Outcome

- `signal.demo.dsp.processing-lab` now emits a rendered companion view at
  `demos/receipts/dsp-processing-lab.view.html`
- the stretch, marker-analysis, and transform-artifact boundary families are
  now visually inspectable as operator cards instead of receipt-only JSON
- the DSP uplift stayed presentation-only over existing descriptor and
  acceptance data
- while closing the batch, the inherited DSP proof spine was corrected:
  stretch, marker-analysis, and transform-artifact acceptance lanes now use
  exact focused runtime proofs instead of loose `cargo test` filters that
  could execute the wrong runtime binaries or zero tests

### Batch 15.13 - Runtime Family Operator Views

- [x] add rendered operator companions for the runtime recovery inspector and
      the runtime supervisor boundary companion
- [x] keep the runtime family grounded in existing bounded reports and
      descriptor surfaces
- [x] avoid widening into runtime control, dashboards, or new supervisor
      behavior

#### Batch 15.13 Outcome

- `signal.demo.runtime.recovery-inspector` now emits a rendered companion view
  at `demos/receipts/runtime-recovery-inspector.view.html`
- `signal.demo.runtime.supervisor-boundary-companion` now emits a rendered
  companion view at
  `demos/receipts/runtime-supervisor-boundary-companion.view.html`
- the runtime family is no longer split between one rendered surface and one
  receipt-only companion; recovery, interruption, and fault-diagnostic posture
  are now visually inspectable at the operator layer
- both uplifts stayed presentation-only over existing supervisor-report and
  descriptor output rather than widening into a runtime console or host shell

### Batch 15.14 - Hardware And Host Operator Views

- [x] add rendered operator companions for hardware topology diagnostics and
      local-versus-server host comparison
- [x] keep both surfaces grounded in existing bounded host and topology truth
- [x] avoid widening into backend control panels or host session shells

#### Batch 15.14 Outcome

- `signal.demo.hardware.topology-diagnostics` now emits a rendered companion
  view at `demos/receipts/hardware-topology-diagnostics.view.html`
- `signal.demo.host.local-server-compare` now emits a rendered companion view
  at `demos/receipts/local-server-host-comparison.view.html`
- the hardware and host families are now visually inspectable instead of
  relying on receipts alone
- the host comparison uplift also corrected inherited readiness parsing so the
  operator surface reflects the actual host summary truth

### Batch 15.15 - Sandbox And Platform Operator Views

- [x] add rendered operator companions for sandbox lifecycle and the remaining
      macOS/Linux platform-boundary demos
- [x] keep those surfaces grounded in existing bounded lifecycle and platform
      proof data
- [x] avoid widening into broker consoles, native-device dashboards, or
      platform-specific product shells

#### Batch 15.15 Outcome

- `signal.demo.plugin.sandbox-lifecycle` now emits a rendered companion view
  at `demos/receipts/plugin-sandbox-lifecycle.view.html`
- `signal.demo.macos.au-coreaudio-boundary` now emits a rendered companion
  view at `demos/receipts/macos-au-coreaudio-boundary.view.html`
- `signal.demo.linux.lv2-backend-boundary` now emits a rendered companion view
  at `demos/receipts/linux-lv2-backend-boundary.view.html`
- the remaining dedicated plugin-lifecycle and platform-boundary surfaces are
  no longer receipt-only
- the platform demo runners were flattened onto direct proof commands so they
  no longer deadlock by recursively invoking nested `effigy acceptance:*`
  tasks from inside `effigy demo:*`

## Closeout Decision

- the plugin family now satisfies the operator-visible contract honestly:
  real installed-plugin browsing, bounded launch or attach truth, and one
  bounded live `parameter-step` interaction proof over supported host paths
- every live demo surface in the active crate set now provides direct operator
  inspection through either live browser-native interaction or a rendered
  companion view instead of receipt-only posture
- no additional operator-visible gap remains that would justify another
  `g09.015` batch without inventing scope beyond Contract `081`

### Final Outcome

- `g09.015` is complete under `core-product`
- operator-visible coverage is now live and inspectable across plugin browser,
  sandbox lifecycle, runtime, host comparison, hardware, graph, DSP, analysis,
  and the remaining platform boundaries
- `g09` is ready to close again at the generation front door because the
  reopened operator-visible demo scope is now complete and honest

## Acceptance Criteria

- the interaction strategy is explicit and contract-backed
- the plugin browser and live plugin interaction path are official repo-owned
  demo surfaces built on real discovery rather than Signal-specific metadata
  shims
- remaining non-interactive or deferred demo surfaces are explicit rather than
  implied

## Evidence Requirements

- [ ] log each meaningful interactive-demo planning or implementation batch
- [ ] keep `demos/coverage-matrix.*` aligned to the interactive proof posture
- [ ] run the actual demo, docs, and validation commands used to justify claims

## Next Task

COMPLETED: `g09.015` is closed. Re-enter planning at the next-generation
boundary before promoting another strict execution lane.
