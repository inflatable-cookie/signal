# 020 - Signal Runtime Endgame Thin Control Library

Status: planned
Owner: core-product
Created: 2026-06-11
Depends on: g10.015
Vision tags: `DEMOLITION`, `ARCHITECTURE`

## Problem

After WYSIWYG bounce retires the simulation offline path, signal-runtime's
last real jobs are the media/decode pipeline pieces pulse uses, broker/
sandbox sessions, and the observation slice host-local serves. The
engine-block simulation, topology narration snapshots, prework scheduler
welded to the simulated engine, and transport-session concurrency model
have no remaining product consumer. host-local (7.5k) + runtime (52.6k)
should shrink to under ~10k combined.

## Goals

- [ ] inventory pulse's true consumption (signal_bridge + host-local surface) and freeze it as the target API
- [ ] delete the engine-block simulation path and its snapshot/observation surfaces beyond the consumed slice
- [ ] extract keepers into right-sized homes: media decode/analysis pipeline, broker sessions, plugin contracts consumption
- [ ] prework scheduler: record the policy vocabulary in a design note, then delete — re-derive against the render plane when anticipative rendering is scheduled
- [ ] host-local collapses into the thin bridge pulse actually needs (possibly merged into one crate)
- [ ] workspace member count and README updated; CHANGELOG entry

## Execution Plan

### Batch 20.1 - Freeze And Cut

- [ ] consumption inventory; simulation deletion; keeper extraction

### Batch 20.2 - Collapse

- [ ] host-local merge/shrink; front-door truth pass

## Acceptance Criteria

- [ ] pulse builds and passes against the reduced surface unchanged
- [ ] runtime+host-local combined under ~10k LoC
- [ ] no simulation execution path remains in the workspace

## Next Task

Generation g10 closure becomes a real conversation after this lands.
