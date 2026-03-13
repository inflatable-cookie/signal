# 003 - Multi-Bus Graph Execution And Auxiliary Topology Depth

Status: planned
Owner: core-product
Created: 2026-03-13
Depends on: g07.001, g07.002
Vision tags: `ROUTING`, `GRAPH`, `EXECUTION`

## Problem

Signal's current graph work is credible for baseline buses and sends, but the
next Loophole feature wave needs stronger reusable truth for auxiliary buses,
multiple bus edges, and richer routing topology.

## Goals

- [ ] define runtime-owned multi-bus and auxiliary topology semantics
- [ ] support richer send, return, and bus execution without host-local patches
- [ ] keep graph, render, and diagnostics surfaces aligned on one topology model

## Non-Goals

- [ ] no product-specific console workflow work
- [ ] no final distributed routing behavior yet

## Execution Plan

### Batch 3.1 - Topology Contract

- [ ] define bus-role, auxiliary-path, and multi-bus connection semantics
- [ ] align the contract with graph snapshot and schedule expectations

### Batch 3.2 - Execution Depth

- [ ] implement stronger multi-bus execution and observation depth
- [ ] keep diagnostics and render surfaces aligned with the widened topology

### Batch 3.3 - Focused Proof

- [ ] add focused proofs for richer bus and auxiliary routing behavior

## Acceptance Criteria

- [ ] Signal has explicit multi-bus and auxiliary topology semantics
- [ ] later plugin-I/O and spatial work can reuse the same routing model
- [ ] hosts no longer need to flatten richer topology into baseline bus assumptions

## Risks And Mitigations

- Risk: bus expansion reopens product-local routing policy.
- Mitigation: freeze runtime topology meaning before host workflow breadth widens.

## Evidence Requirements

- [ ] log each meaningful multi-bus tranche
- [ ] run focused topology and execution validation
- [ ] record deferred topology cases explicitly

## Next Task

Continue `g07.004` by applying the widened routing model to complex plugin I/O
and multi-output instrument behavior.

