# Discovery Intake and Frontier Triage

Purpose: define how low-authority secondary channels feed into the research program without polluting the primary-source corpus.

## Why This Exists

The research program's source hierarchy requires academic papers, official docs, and source trees before secondary commentary. But secondary channels surface signals faster than primary-source indexing — new models, technique demonstrations, production insights often appear on blogs, YouTube, or forums before formal publication. Without an intake process, the program either misses timely signals or absorbs unvetted claims into the corpus.

## Discovery Channel Registry

### Tier A — Academic and Primary Sources (monthly check cadence)

These sources have peer review and reproducible results:

| Channel | Signal Type | Timeliness | Primary Failure Mode |
| --- | --- | --- | --- |
| ISMIR Proceedings | music IR papers, evaluation benchmarks | annual conference | academic focus may not match production constraints |
| AES Convention Papers | audio engineering, loudness standards | annual/biannual | industry-heavy, may lack open implementations |
| ICASSP | signal processing, ML audio | annual | broad scope, music-specific content mixed |
| arXiv cs.SD / eess.AS | preprints, new models | 0-14 days | unreviewed, quality variable |
| Essentia GitHub releases | library updates, new algorithms | per release | focused on Essentia's specific implementations |
| Librosa GitHub releases | Python audio analysis updates | per release | Python-only, performance constraints |

### Tier B — Production Tools and Analysis (event-driven check)

These channels provide empirical evidence about shipped implementations:

| Channel | Signal Type | Timeliness | Primary Failure Mode |
| --- | --- | --- | --- |
| Mixed In Key / Serato / Rekordbox releases | commercial feature updates, accuracy claims | per release | marketing framing, unverified accuracy claims |
| DJ TechTools | practitioner workflows, tool comparisons | weekly | affiliate revenue bias, shallow technical depth |
| Gearslutz / VI-Control forums | professional audio tool discussions | daily | anecdotal, uncontrolled comparisons |
| Landr / eMastered blogs | mastering/analysis workflows | weekly | service promotion bias |

### Tier C — Technical Explainers (as-needed reference)

These channels provide implementation-level education, not discovery:

| Channel | Signal Type | Timeliness | Primary Failure Mode |
| --- | --- | --- | --- |
| Music Information Retrieval Evaluation eXchange (MIREX) | benchmark results, algorithm comparisons | annual | leaderboard framing, may not reflect production needs |
| Coursera MIR courses (Columbia, NYU) | foundational concepts | not event-driven | curriculum-level, years behind frontier |
| API documentation (Essentia, Librosa) | implementation details | per release | library-specific, not comparative |

### Tier D — Community and Social (ephemeral signal, never cite directly)

| Channel | Signal Type | Primary Failure Mode |
| --- | --- | --- |
| Reddit r/WeAreTheMusicMakers, r/edmproduction | tool recommendations, workflow tips | extreme noise, beginner-heavy |
| Twitter/X MIR community | paper alerts, hot takes | extreme noise; vendor marketing indistinguishable |
| YouTube (production tutorials) | workflow demonstrations | affiliate bias, uncontrolled testing |

## Triage Rules

Every signal from a secondary channel must be triaged before it can enter the research corpus. Triage produces exactly one outcome per signal.

### Triage Outcomes

| Outcome | Meaning | What Happens Next |
| --- | --- | --- |
| `research now` | primary source exists, Finch-relevant, strong enough to enter a value track or memo | trace to primary source, add to relevant source map and value track |
| `lead only` | interesting signal but primary source is missing, incomplete, or unverified | record in the triage log with the claim and the missing primary source; do not add to corpus |
| `watch` | credible primary source exists but the technique is too early, too vendor-locked, or too uncertain to act on | record in the triage log with the primary source and a review trigger condition |
| `reject` | not Finch-relevant, or the claim does not survive primary-source tracing | record in the triage log with the reason for rejection; do not add to corpus |

### Triage Decision Tree

1. **Does a primary source exist?** (paper, official docs, source tree, reproducible benchmark)
   - No → `lead only` (record what the claim is and what source is missing)
   - Yes → continue

2. **Is the technique Finch-relevant?** (does it address a problem in signal analysis, classification, or workflow?)
   - No → `reject`
   - Yes → continue

3. **Is the primary source strong enough?** (peer-reviewed, open implementation, or production validation)
   - No → `lead only` (record the weak source and what would strengthen it)
   - Yes → continue

4. **Is the technique ready for Finch action?** (shipping in production, cross-validated, or has multiple independent implementations)
   - No → `watch` (record the technique, primary source, and what would make it ready)
   - Yes → `research now`

## Triage Log Format

Each triage entry records:

```
## [Signal Title]

- Source channel: [which secondary channel surfaced this]
- Date triaged: [YYYY-MM-DD]
- Claim: [what the signal claims, in one sentence]
- Primary source: [link to primary source if found, or "missing" with what would constitute one]
- Finch relevance: [which value track(s) this relates to, or "none"]
- Outcome: [research now | lead only | watch | reject]
- Reason: [one sentence explaining the triage decision]
- Review trigger: [for watch items only — what event would cause re-triage]
```

## Check Cadence

| Channel Tier | Check Frequency | Who |
| --- | --- | --- |
| Tier A (academic/primary) | monthly | research session |
| Tier B (production tools) | event-driven (major releases, ISMIR) | research session |
| Tier C (technical explainers) | as-needed when a topic requires background | research session |
| Tier D (community/social) | never systematically | research session |

## Next Task

Run the initial triage pass on current secondary signals to populate the triage log and validate the intake process. Focus on recent ISMIR 2024 papers and current commercial tool capabilities.
