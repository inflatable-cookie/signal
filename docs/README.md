# Signal Docs

Signal is a shared realtime audio library and runtime for the Loophole
ecosystem. This docs tree is where its long-term direction, architecture,
frozen boundaries, delivery plan, and evidence live. The docs follow the
Northstar shape: Vision → Architecture → Contracts → Roadmaps → Logs.

If a term below is unfamiliar, look it up in the
[glossary](./reference/glossary.md) — it translates the shorthand used
throughout these docs.

## Start here

Pick the path that matches what you're doing:

- **I'm new to Signal.** Read the [Vision](./vision/001-signal-vision.md) (2
  minutes) and [System Architecture](./architecture/system-architecture.md)
  (10 minutes). That gives you what Signal is, what it owns, and what it
  doesn't.
- **I want to use Signal in another project.** Start with
  [Consuming Signal](./reference/consuming-signal.md), the runbook for
  depending on Signal crates, and the
  [Quick Start](./reference/quick-start.md) for working code you can run
  today. Then read the [system inventory](./architecture/system-inventory.md)
  and the
  [DSP and analysis feature reference](./architecture/dsp-analysis-feature-reference.md)
  to see what is actually available.
- **I'm working on Signal.** Read the [Architecture section](./architecture/README.md),
  find the contract that owns your seam in the [contract index](./contracts/contract-index.md),
  and check the [active generation roadmap](./roadmaps/g10/README.md) before
  starting anything.
- **I need the current status in one paragraph.** Each section README below
  opens with a short "in plain words" summary. The
  [roadmaps README](./roadmaps/README.md) is the single best status snapshot.
- **I need to know what stretch (time-stretch / pitch-shift) is shipped.**
  Read the [stretch summary](./architecture/offline-time-stretch-synthesis.md)
  and the [creative stretch surface](./architecture/offline-creative-fixed-ratio-public-surface.md).
  Everything else in this tree about stretch is archived research.

## How the docs are organized

Each layer answers one question. Read them top to bottom for full context, or
jump straight to the layer that matches your question.

| Layer | Answers | Status |
| --- | --- | --- |
| [Vision](./vision/README.md) | What are we building and why? | Active |
| [Architecture](./architecture/README.md) | How does it fit together? | Active |
| [Contracts](./contracts/README.md) | What boundaries are frozen? | Active |
| [Roadmaps](./roadmaps/README.md) | What is being built, and in what order? | Active (`g11`) |
| [Logs](./logs/README.md) | What happened, with evidence? | Archive |
| [Research](./research/master-index.md) | What did we learn before deciding? | Active |
| [Reference](./reference/glossary.md) | Runbooks and plain-language guides | Active |
| [Policy](./policy/internal-writing-style.md) | How the docs themselves are written | Active |

## Key entry points

**Vision**

- [001 Signal Vision](./vision/001-signal-vision.md) — the long-horizon
  statement: one reusable audio stack across products.

**Architecture**

- [System Architecture](./architecture/system-architecture.md) — the top-level
  stack: primitives, DSP, analysis, graph, runtime, host-edge adapters.
- [System Inventory](./architecture/system-inventory.md) — every workspace
  crate, what it does, and where it sits.
- [Product Guardrails](./architecture/product-guardrails.md) — what Signal
  must always and never be.
- [DSP and Analysis Feature Reference](./architecture/dsp-analysis-feature-reference.md) —
  what the DSP and analysis crates actually expose today.
- [Graph and Runtime Feature Reference](./architecture/graph-runtime-feature-reference.md) —
  what the runtime and graph crates actually expose today.
- [Production Host-Assembly Integration](./architecture/production-host-assembly-integration.md) —
  the `g11.001` authority map from host assembly to bridge backends
- [Offline Time-Stretch Synthesis](./architecture/offline-time-stretch-synthesis.md) —
  how the shipped stretch renderer works.
- [Creative Time-Stretch Study](./architecture/offline-creative-time-stretch-study.md) —
  the decision record for the creative stretch product path.

**Contracts**

- [Contract Index](./contracts/contract-index.md) — the searchable front door
  to all 85 contracts, grouped by boundary family.
- [001 Working Rules](./contracts/001-working-rules.md) — how work is run in
  this repository.
- [001 Shared DSP and Host Boundary](./contracts/001-shared-dsp-and-host-boundary.md) —
  the boundary that defines what Signal owns.
- [072 Real Plugin Hosting, Discovery, and Sandbox Execution](./contracts/072-real-plugin-hosting-discovery-and-sandbox-execution-contract.md) —
  the authority for shipped CLAP/VST3/AU/LV2 hosting and remaining integration seams.
- [084 Stretch Candidate Isolation and Promotion](./contracts/084-stretch-candidate-isolation-and-promotion-contract.md) —
  the rules that govern stretch evidence and admission.
- [085 Creative Time-Stretch Product and Routing](./contracts/085-creative-time-stretch-product-and-routing-contract.md) —
  the creative stretch product vocabulary and gates.

**Roadmaps**

- [Roadmap Index](./roadmaps/README.md) — generation history and current
  posture.
- [Strategic Runway](./roadmaps/strategic-runway.md) — long-horizon sequencing
  after the `g10` stretch audit.
- [Active Generation g11](./roadmaps/g11/README.md) — current queue.
- [Generation g10](./roadmaps/g10/README.md) — closing generation; stretch audit
  complete.
- [Generation Index](./roadmaps/generation-index.md) — the history of
  generations `g01`…`g11`.

**Research**

- [Master Index](./research/master-index.md) — the map from research outputs
  to crate planning and consumers. This is also where archived stretch
  research lives (specimen dossiers, translation memos, rejected briefs).

**Reference**

- [Quick Start](./reference/quick-start.md) — hear it, analyze it, stretch it:
  complete worked examples from the repo.
- [Consuming Signal](./reference/consuming-signal.md) — the canonical runbook
  for depending on Signal crates from another repository.
- [Glossary](./reference/glossary.md) — plain-English translations of the
  shorthand used across these docs.

## Working Rule

- treat Signal docs as the canonical authority for reusable library/runtime
  building blocks
- keep Finch and Loophole wrapper notes outside Signal unless they affect the
  reusable library boundary
- keep section indexes aligned to Northstar conventions
- treat an active generation as a lane-first strict Northstar surface under
  `docs/specs/` only while that generation is explicitly open
- if there is no active strict lane, use the roadmap and contract front doors
  instead of reading old batch-card state as current authority
- in the strict lane, treat a bare `continue` as "follow the previous closeout's
  `Next Task`" rather than as permission to infer a new batch

## Validation

- `effigy qa:docs`
- `effigy qa:northstar`

## Next Task

Execute
`docs/roadmaps/g11/batch-cards/001-g11-001-bridge-backend-factory.md`
using `docs/architecture/production-host-assembly-integration.md` as authority.
Do not infer Automatic, RealtimePreview, integration, Loophole, or Chorus work
from the completed stretch lane.
