use std::fs;
use std::path::Path;

use signal_dsp_stretch::{OfflineHighQualityPath, OfflineHighQualityStretcher, TimeStretcher};

use super::{tsv_writer, write_tsv};
use crate::{
    decode_listening_source_audio, source_for_external_quality_render,
    ExternalBenchmarkQualityRender, ExternalBenchmarkQualitySource, StretchCorpusListeningSource,
};

#[path = "tail/selection.rs"]
mod selection;
#[path = "tail/audio.rs"]
mod tail_audio;

use selection::select_tail_review_rows;
#[cfg(test)]
use selection::{bound_tail_review_rows, TailReviewRow, REVIEW_ROWS};
#[cfg(test)]
use tail_audio::PEAK_CEILING;
use tail_audio::{amplitude_dbfs, append_silence, shared_tail_gain, tail_excerpt};

const EXCERPT_SECONDS: usize = 1;
const POST_TAIL_SILENCE_MILLISECONDS: usize = 250;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum TailCandidate {
    Current,
    AdditiveZeroAnchor,
    MultiplicativeZeroFade,
}

impl TailCandidate {
    fn label(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::AdditiveZeroAnchor => "additive-zero-anchor",
            Self::MultiplicativeZeroFade => "multiplicative-zero-fade",
        }
    }
}

pub(crate) fn export_tail_listening_pack(
    sources: &[StretchCorpusListeningSource],
    renders: &[ExternalBenchmarkQualityRender],
    frame_limit: usize,
    signal_path: OfflineHighQualityPath,
    export_dir: &Path,
) -> Result<String, String> {
    if renders.is_empty() {
        return Err("tail listening pack requires benchmark render metadata".to_string());
    }

    let selected = select_tail_review_rows(sources, renders, frame_limit, signal_path)?;
    let trials_dir = export_dir.join("trials");
    fs::create_dir_all(&trials_dir)
        .map_err(|error| format!("failed to create {}: {error}", trials_dir.display()))?;

    let mut notes = tsv_writer(vec![
        "trial_id",
        "candidate_a",
        "candidate_b",
        "candidate_c",
        "click_pop",
        "pull_thump",
        "tail_continuity",
        "preference",
        "notes",
        "completed",
    ])?;
    let mut key = tsv_writer(vec![
        "trial_id",
        "case_id",
        "ratio",
        "source_path",
        "candidate_a_backend",
        "candidate_b_backend",
        "candidate_c_backend",
        "signal_path",
        "endpoint_correction",
        "current_endpoint_dbfs",
        "shared_gain_db",
        "excerpt_frames",
        "post_tail_silence_frames",
    ])?;

    for (index, row) in selected.iter().enumerate() {
        let render = &renders[row.render_index];
        let source = resolve_source(sources, render)?;
        let source_audio = decode_listening_source_audio(source.as_ref(), frame_limit)?;
        let source_mono = source_audio.mono_samples();
        let mut current_stretcher =
            OfflineHighQualityStretcher::with_path(render.ratio, signal_path);
        let mut zero_anchor_stretcher =
            OfflineHighQualityStretcher::with_path(render.ratio, signal_path);
        let mut multiplicative_fade_stretcher =
            OfflineHighQualityStretcher::with_path(render.ratio, signal_path);
        let current = current_stretcher.stretch_mono(&source_mono);
        let zero_anchor = zero_anchor_stretcher.stretch_zero_tail_anchor_review_mono(&source_mono);
        let multiplicative_fade = multiplicative_fade_stretcher
            .stretch_multiplicative_tail_fade_review_mono(&source_mono);

        let excerpt_frames = source_audio.sample_rate_hz as usize * EXCERPT_SECONDS;
        let current_tail = tail_excerpt(&current, excerpt_frames);
        let zero_anchor_tail = tail_excerpt(&zero_anchor, excerpt_frames);
        let multiplicative_fade_tail = tail_excerpt(&multiplicative_fade, excerpt_frames);
        let shared_gain =
            shared_tail_gain(current_tail, [zero_anchor_tail, multiplicative_fade_tail])?;
        let silence_frames =
            source_audio.sample_rate_hz as usize * POST_TAIL_SILENCE_MILLISECONDS / 1_000;
        let candidates = [
            append_silence(current_tail, shared_gain, silence_frames),
            append_silence(zero_anchor_tail, shared_gain, silence_frames),
            append_silence(multiplicative_fade_tail, shared_gain, silence_frames),
        ];
        let assignment = stable_tail_assignment(render, &row.source_path);
        let trial_id = format!("T{:03}", index + 1);
        let mut names = Vec::new();
        let mut backends = Vec::new();
        for (candidate_index, candidate) in assignment.iter().enumerate() {
            let letter = (b'A' + candidate_index as u8) as char;
            let name = format!("{trial_id}-{letter}.wav");
            super::write_float_wav(
                &trials_dir.join(&name),
                source_audio.sample_rate_hz,
                1,
                &candidates[candidate_slot(*candidate)],
            )?;
            names.push(format!("trials/{name}"));
            backends.push(candidate.label());
        }

        notes
            .write_record([
                trial_id.as_str(),
                names[0].as_str(),
                names[1].as_str(),
                names[2].as_str(),
                "",
                "",
                "",
                "",
                "",
                "false",
            ])
            .map_err(|error| format!("failed to write tail listening notes row: {error}"))?;
        key.write_record([
            trial_id.as_str(),
            render.case_id.as_str(),
            &format!("{:.6}", render.ratio),
            row.source_path.as_str(),
            backends[0],
            backends[1],
            backends[2],
            &format!("{signal_path:?}"),
            &format!("{:.9}", row.endpoint_correction),
            &format!("{:.6}", amplitude_dbfs(row.endpoint_correction)),
            &format!("{:.6}", 20.0 * shared_gain.log10()),
            &current_tail.len().to_string(),
            &silence_frames.to_string(),
        ])
        .map_err(|error| format!("failed to write tail listening key row: {error}"))?;
    }

    write_tsv(export_dir.join("tail-listening-notes.tsv"), notes)?;
    write_tsv(export_dir.join("tail-listening-key.tsv"), key)?;
    fs::write(
        export_dir.join("README.md"),
        tail_listening_readme(selected.len()),
    )
    .map_err(|error| format!("failed to write tail listening README: {error}"))?;

    Ok(format!(
        "tail_listening_pack export_dir={:?} status=ReadyForOperator trials={} candidates_per_trial=3 channels=1 post_tail_silence_ms={} notes={:?} key={:?}",
        export_dir.display().to_string(),
        selected.len(),
        POST_TAIL_SILENCE_MILLISECONDS,
        export_dir.join("tail-listening-notes.tsv").display().to_string(),
        export_dir.join("tail-listening-key.tsv").display().to_string(),
    ))
}

pub(super) fn resolve_source<'a>(
    sources: &'a [StretchCorpusListeningSource],
    render: &ExternalBenchmarkQualityRender,
) -> Result<std::borrow::Cow<'a, StretchCorpusListeningSource>, String> {
    match source_for_external_quality_render(sources, render) {
        ExternalBenchmarkQualitySource::Found(source) => Ok(source),
        ExternalBenchmarkQualitySource::Missing => Err(format!(
            "tail listening render {} at {:.6} has no source",
            render.case_id, render.ratio
        )),
        ExternalBenchmarkQualitySource::Ambiguous => Err(format!(
            "tail listening render {} at {:.6} has an ambiguous source",
            render.case_id, render.ratio
        )),
    }
}

fn candidate_slot(candidate: TailCandidate) -> usize {
    match candidate {
        TailCandidate::Current => 0,
        TailCandidate::AdditiveZeroAnchor => 1,
        TailCandidate::MultiplicativeZeroFade => 2,
    }
}

fn stable_tail_assignment(
    render: &ExternalBenchmarkQualityRender,
    source_path: &str,
) -> [TailCandidate; 3] {
    const PERMUTATIONS: [[TailCandidate; 3]; 6] = [
        [
            TailCandidate::Current,
            TailCandidate::AdditiveZeroAnchor,
            TailCandidate::MultiplicativeZeroFade,
        ],
        [
            TailCandidate::Current,
            TailCandidate::MultiplicativeZeroFade,
            TailCandidate::AdditiveZeroAnchor,
        ],
        [
            TailCandidate::AdditiveZeroAnchor,
            TailCandidate::Current,
            TailCandidate::MultiplicativeZeroFade,
        ],
        [
            TailCandidate::AdditiveZeroAnchor,
            TailCandidate::MultiplicativeZeroFade,
            TailCandidate::Current,
        ],
        [
            TailCandidate::MultiplicativeZeroFade,
            TailCandidate::Current,
            TailCandidate::AdditiveZeroAnchor,
        ],
        [
            TailCandidate::MultiplicativeZeroFade,
            TailCandidate::AdditiveZeroAnchor,
            TailCandidate::Current,
        ],
    ];
    let assignment = format!("{}|{:.9}|{source_path}", render.case_id, render.ratio);
    let hash = assignment
        .as_bytes()
        .iter()
        .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
        });
    PERMUTATIONS[hash as usize % PERMUTATIONS.len()]
}

fn tail_listening_readme(trial_count: usize) -> String {
    format!(
        "# Concealed Tail Listening Pack\n\nStatus: ready for operator notes\n\nTrials: {trial_count}\nCandidates per trial: 3\nAudio: mono, final one second, then 250 ms digital silence\n\n1. Keep `tail-listening-key.tsv` closed.\n2. For each row in `tail-listening-notes.tsv`, compare A, B, and C around the transition into silence.\n3. Record any click/pop, pull/thump, fade, or loss of tail continuity.\n4. Record a preference even when the differences are subtle.\n5. Set `completed=true` only after all fields were considered.\n6. Reveal the key only after notes are frozen.\n\nTrials are the six largest current endpoint jumps in the supplied render plan. The candidates are current Signal, the rejected additive zero anchor, and the multiplicative zero fade. One shared gain is applied per trial, targeting -16.48 dBFS RMS with a 0.95 peak ceiling; relative boundary amplitude is preserved. This is a local evidence artifact. Do not commit licensed audio.\n"
    )
}

#[cfg(test)]
#[path = "tail/tests.rs"]
mod tests;
