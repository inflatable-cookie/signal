use std::collections::HashSet;

use signal_dsp_stretch::{OfflineHighQualityPath, OfflineHighQualityStretcher, TimeStretcher};

use super::resolve_source;
use crate::tail_features::measure_tail_spectral_centroid_hz;
use crate::{
    decode_listening_source_audio, ExternalBenchmarkQualityRender, StretchCorpusListeningSource,
};

pub(super) const REVIEW_ROWS: usize = 6;
const CLASSIFIER_ROWS_PER_BAND: usize = 3;
const CLASSIFIER_CENTROID_THRESHOLD_HZ: f64 = 2_000.0;
const LABELED_SOURCE_STEMS: [&str; 3] = [
    "0002-drums_percussion-000169.wav",
    "0013-pads_sustains-000870.wav",
    "0017-full_mix-000153.wav",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TailCentroidBand {
    BelowThreshold,
    AtOrAboveThreshold,
}

impl TailCentroidBand {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::BelowThreshold => "below-2000-hz",
            Self::AtOrAboveThreshold => "at-or-above-2000-hz",
        }
    }
}

#[derive(Clone)]
pub(super) struct TailReviewRow {
    pub(super) render_index: usize,
    pub(super) endpoint_correction: f64,
    pub(super) source_path: String,
    pub(super) spectral_centroid_hz: f64,
    pub(super) centroid_band: TailCentroidBand,
}

pub(super) fn select_tail_review_rows(
    sources: &[StretchCorpusListeningSource],
    renders: &[ExternalBenchmarkQualityRender],
    frame_limit: usize,
    signal_path: OfflineHighQualityPath,
) -> Result<Vec<TailReviewRow>, String> {
    let mut rows = measure_tail_review_rows(sources, renders, frame_limit, signal_path)?;
    bound_tail_review_rows(&mut rows);
    Ok(rows)
}

fn measure_tail_review_rows(
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
        let spectral_centroid_hz =
            measure_tail_spectral_centroid_hz(source_audio.sample_rate_hz, &output);
        rows.push(TailReviewRow {
            render_index,
            endpoint_correction,
            source_path: source.source_path.clone(),
            spectral_centroid_hz,
            centroid_band: centroid_band(spectral_centroid_hz),
        });
    }
    Ok(rows)
}

pub(super) fn select_tail_classifier_validation_rows(
    sources: &[StretchCorpusListeningSource],
    renders: &[ExternalBenchmarkQualityRender],
    frame_limit: usize,
    signal_path: OfflineHighQualityPath,
) -> Result<Vec<TailReviewRow>, String> {
    let mut rows = measure_tail_review_rows(sources, renders, frame_limit, signal_path)?;
    rows.retain(|row| {
        !LABELED_SOURCE_STEMS
            .iter()
            .any(|stem| row.source_path.ends_with(stem))
    });
    let selected = bound_classifier_validation_rows(rows);
    let below = selected
        .iter()
        .filter(|row| row.centroid_band == TailCentroidBand::BelowThreshold)
        .count();
    let above = selected.len() - below;
    if below != CLASSIFIER_ROWS_PER_BAND || above != CLASSIFIER_ROWS_PER_BAND {
        return Err(format!(
            "tail classifier validation requires {CLASSIFIER_ROWS_PER_BAND} distinct sources per centroid band; found below={below} above={above}"
        ));
    }
    Ok(selected)
}

pub(super) fn bound_classifier_validation_rows(mut rows: Vec<TailReviewRow>) -> Vec<TailReviewRow> {
    rows.sort_by(|left, right| {
        right
            .endpoint_correction
            .total_cmp(&left.endpoint_correction)
            .then_with(|| left.source_path.cmp(&right.source_path))
            .then_with(|| left.render_index.cmp(&right.render_index))
    });
    let mut source_paths = HashSet::new();
    let mut below = Vec::new();
    let mut above = Vec::new();
    for row in rows {
        let band = match row.centroid_band {
            TailCentroidBand::BelowThreshold => &mut below,
            TailCentroidBand::AtOrAboveThreshold => &mut above,
        };
        if band.len() < CLASSIFIER_ROWS_PER_BAND && source_paths.insert(row.source_path.clone()) {
            band.push(row);
        }
    }
    below.extend(above);
    below
}

fn centroid_band(spectral_centroid_hz: f64) -> TailCentroidBand {
    if spectral_centroid_hz < CLASSIFIER_CENTROID_THRESHOLD_HZ {
        TailCentroidBand::BelowThreshold
    } else {
        TailCentroidBand::AtOrAboveThreshold
    }
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
