# 2026-03-11 21:45:00 GMT - g02.005 transient shape and envelope summary tranche

Deepened `g02.005` by adding event-oriented transient and temporal-shape
summaries on top of the new descriptor-pack API in
`signal-analysis-character`.

This batch matters because Signal now exposes more than static timbral and
energy descriptors: downstream consumers can distinguish sharp versus slow
attacks, short versus long decays, and plateau-like events versus brief
impulses without rebuilding local envelope logic.

Implemented changes:

- added `TemporalShapeDescriptorPack` to `CharacterAnalysisResult` with:
  - `peak_transient_strength`
  - `median_transient_strength`
  - `attack_time_ms`
  - `decay_time_ms`
  - `sustain_plateau_ratio`
- extended `CharacterDescriptorReductionPolicy` so temporal-shape reductions are
  explicit, including event-median, event-mean, and strongest-event modes
- replaced the old one-off onset counting path with a reusable spectral-flux
  event series that:
  - detects flux peaks above the analyzer threshold
  - collapses nearby peaks into one representative transient marker
- added a frame-aligned RMS envelope path and used it to derive bounded
  `10 percent -> 90 percent` attack and decay spans around each detected event
- kept the existing temporal activity pack intact while making transient-marker
  strength and envelope-shape behavior first-class for later embedding and
  cataloging work
- expanded deterministic fixture coverage with ADSR-like synthetic pulses so the
  analyzer now distinguishes:
  - pulse trains from steady tones by transient strength
  - slower attacks from sharp attacks
  - longer decays from shorter decays
  - longer sustain plateaus from brief ones

Validation:

- `cargo fmt --all`
- `cargo test -p signal-analysis-character`

Remaining limits after this tranche:

- temporal shape is still summary-only; no event timeline or per-marker export
  is exposed yet
- spectral contrast remains a broadband summary rather than a multiband family
- `g02.005` still needs closeout evidence with explicit descriptor-pack examples
  and a final gap summary before rolling to `g02.006`

Next task:

Close `g02.005` by recording explicit descriptor-pack examples, remaining gaps,
and milestone-complete evidence before opening `g02.006` embedding and semantic
inference work.
