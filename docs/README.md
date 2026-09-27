# Signal docs

Signal is the shared realtime audio library and runtime for the Loophole
ecosystem. The realtime render path, DSP, analysis, graph, and runtime crates
are live. CLAP, VST3, AU, and LV2 hosting runs through Signal's adapters and
sandbox; production host assembly and SharedSandbox multiplexing are present.
The transparent stretch renderer and exact-ratio creative `Dream` and
`Cyclic` surfaces are shipped. See the [glossary](reference/glossary.md).

## Read by topic

| Need | Owner |
| --- | --- |
| Purpose and boundaries | [Vision](knowledge/vision.md) |
| System shape, crates, shipped features | [Architecture](knowledge/architecture/README.md) |
| Durable behavior and policy | [Contracts](knowledge/contracts/contract-index.md) |
| Release procedure | [Release](knowledge/contracts/release.md) |
| Open questions and retired concepts | [Knowledge index](knowledge/README.md) |
| Priorities and product pull | Plan |
| Consumer setup and examples | [Reference](reference/consuming-signal.md) and [quick start](reference/quick-start.md) |
| Source studies and frozen evidence | [Research](research/master-index.md) |
| Unresolved leads | Triage |

Task briefs, validation results, review, and closeout live in Queue. Git
history holds removed process records. The repository records current truth
and product evidence.

## Validation

Run `effigy qa` for the full local board. For documentation changes, run
`effigy qa:docs` and `effigy qa:northstar`.
