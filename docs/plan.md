# Signal plan

Signal's shipped substrate is the baseline. Preserve its realtime, plugin
isolation, and consumer boundaries as product needs pull more depth.

1. **Select the next Signal-owned product pull** (lane `product-pull`). The
   operator chooses a bounded need. Candidates are graph execution, device
   handling, or consumer release depth; see [Q-001](knowledge/questions.md#q-001--which-signal-owned-product-pull-comes-next)
   and [triage](triage/README.md). No candidate is ready solely by appearing
   here.
2. **Broaden analysis and substrate on demand** (lane `analysis-depth`).
   Beat tracking, higher-quality SRC, and multichannel/loudness depth need a
   named consumer or contract gap.
3. **Replace the temporary C++ compatibility island incrementally** (lane
   `runtime-consolidation`). A batch needs a trustworthy production
   integration path and an explicit consumer contract.
4. **Sweep removed records for rulings** (lane `record-rulings`). Check the
   historical task and log corpus in Git history for current decisions or
   procedures that were not already in an owning knowledge file. Update only
   that owner; do not restore process records.

Queue owns the briefs and outcomes for selected work.
