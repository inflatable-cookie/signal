# 015 - Operator-Visible Interactive Demo And Plugin Browser Proof

Status: active
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
- [ ] remove Signal-specific metadata-file dependency from VST3 and AU
- [ ] make the plugin browser claim honest before the browser is implemented

#### Batch 15.2 Outcome

- landed a real CLAP root-scan path using actual `.clap` libraries through
  `signal-plugin-clap`
- rewired the local and server host CLAP scan/ensure/restart flow to use a
  scanned CLAP catalog instead of synthetic ids
- proved the CLAP path through direct host runs against compiled temporary
  `.clap` fixtures
- confirmed the remaining blocker precisely:
  - VST3 still depends on `signal-vst3-module.txt` and
    `signal-vst3-factory.txt`
  - AU still depends on `signal-au-component.txt`
  - LV2 scaffold-backed direct lookup remains in the adapter, but it is not the
    active host production discovery path
- promoted the next ready batch as
  `045-g09-015-vst3-au-real-introspection-burn-down.md`

### Batch 15.3 - VST3 And AU Real Introspection Burn-Down

- [ ] add an official operator-visible plugin capability browser
- [ ] remove VST3 `.txt` metadata-file discovery
- [ ] remove AU `.txt` metadata-file discovery
- [ ] keep the browser deferred until installed-plugin browsing is honest across
      CLAP, VST3, and AU

### Batch 15.4 - Plugin Browser And Interaction

- [ ] add an official operator-visible plugin capability browser
- [ ] let an operator browse installed plugins and launch supported live
      interaction paths through repo-owned commands
- [ ] keep unsupported formats or platform gaps explicit rather than implied

### Batch 15.5 - Wider Interactive Proof Uplift

- [ ] widen operator-visible proof across the remaining crate-family demos that
      still lean too heavily on receipts alone
- [ ] close `g09` again only once the intended interactive visibility scope is
      explicit and honest

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

Continue the reopened strict `g09` lane from
`docs/specs/batch-cards/045-g09-015-vst3-au-real-introspection-burn-down.md`.
