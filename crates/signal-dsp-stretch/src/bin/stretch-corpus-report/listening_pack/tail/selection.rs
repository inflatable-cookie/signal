use signal_dsp_stretch::{OfflineHighQualityPath, OfflineHighQualityStretcher, TimeStretcher};

use super::resolve_source;
use crate::{
    decode_listening_source_audio, ExternalBenchmarkQualityRender, StretchCorpusListeningSource,
};

pub(super) const REVIEW_ROWS: usize = 6;

#[derive(Clone)]
pub(super) struct TailReviewRow {
    pub(super) render_index: usize,
    pub(super) endpoint_correction: f64,
    pub(super) source_path: String,
}

pub(super) fn select_tail_review_rows(
    sources: &[StretchCorpusListeningSource],
    renders: &[ExternalBenchmarkQualityRender],
    frame_limit: usize,
    signal_path: OfflineHighQualityPath,
) -> Result<Vec<TailReviewRow>, String> {
    let mut rows = Vec::new();
    for (render_index, render) in renders.iter().enumerate() {
        let source = resolve_source(sources, render)?;
        let source_audio = decode_listening_source_audio(source.as_ref(), frame_limit)?;
        let mut stretcher = OfflineHighQualityStretcher::with_path(render.ratio, signal_path);
        let output = stretcher.stretch_mono(&source_audio.mono_samples());
        let endpoint_correction = output.last().copied().unwrap_or(0.0).abs() as f64;
        rows.push(TailReviewRow {
            render_index,
            endpoint_correction,
            source_path: source.source_path.clone(),
        });
    }
    bound_tail_review_rows(&mut rows);
    Ok(rows)
}

pub(super) fn bound_tail_review_rows(rows: &mut Vec<TailReviewRow>) {
    rows.sort_by(|left, right| {
        right
            .endpoint_correction
            .total_cmp(&left.endpoint_correction)
            .then_with(|| left.source_path.cmp(&right.source_path))
            .then_with(|| left.render_index.cmp(&right.render_index))
    });
    rows.truncate(REVIEW_ROWS.min(rows.len()));
}
