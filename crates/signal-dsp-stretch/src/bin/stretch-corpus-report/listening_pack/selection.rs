use std::collections::HashMap;

use super::{
    source_for_external_quality_render, ExternalBenchmarkQualityRender,
    ExternalBenchmarkQualitySource, StretchCorpusListeningSource, REQUIRED_FAMILIES,
};

pub(super) fn select_one_source_per_required_family<'a>(
    sources: &[StretchCorpusListeningSource],
    renders: &'a [ExternalBenchmarkQualityRender],
) -> Result<Vec<&'a ExternalBenchmarkQualityRender>, String> {
    let mut selected_source_by_family = HashMap::<&str, String>::new();
    let mut selected = Vec::new();
    for render in renders {
        let Some(family) = REQUIRED_FAMILIES
            .iter()
            .copied()
            .find(|family| *family == render.case_id)
        else {
            continue;
        };
        let source = match source_for_external_quality_render(sources, render) {
            ExternalBenchmarkQualitySource::Found(source) => source,
            ExternalBenchmarkQualitySource::Missing => continue,
            ExternalBenchmarkQualitySource::Ambiguous => continue,
        };
        let source_id = source.source_path.clone();
        let selected_source = selected_source_by_family
            .entry(family)
            .or_insert_with(|| source_id.clone());
        if *selected_source == source_id {
            selected.push(render);
        }
    }
    for family in REQUIRED_FAMILIES {
        if !selected_source_by_family.contains_key(family) {
            return Err(format!(
                "blind listening pack is missing required family {family}"
            ));
        }
    }
    selected.sort_by(|left, right| {
        family_rank(&left.case_id)
            .cmp(&family_rank(&right.case_id))
            .then_with(|| left.ratio.total_cmp(&right.ratio))
            .then_with(|| left.rendered_path.cmp(&right.rendered_path))
    });
    Ok(selected)
}

fn family_rank(case_id: &str) -> usize {
    REQUIRED_FAMILIES
        .iter()
        .position(|family| *family == case_id)
        .unwrap_or(usize::MAX)
}
