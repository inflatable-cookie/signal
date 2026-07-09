use std::fs;
use std::path::{Path, PathBuf};

use signal_dsp_stretch::{OfflineHighQualityPath, OfflineHighQualityStretcher, TimeStretcher};

use super::{
    decode_external_benchmark_render_audio, decode_listening_source_audio,
    source_for_external_quality_render, ExternalBenchmarkQualityRender,
    ExternalBenchmarkQualitySource, StretchCorpusListeningSource,
};

#[path = "listening_pack/audio.rs"]
mod audio;
#[path = "listening_pack/note_status.rs"]
mod note_status;
#[path = "listening_pack/selection.rs"]
mod selection;

use audio::{gain_db, level_match_group, stable_assignment_is_signal_a, write_float_wav};
#[cfg(test)]
use audio::{level_stats, PEAK_CEILING};
pub(super) use note_status::format_blind_listening_note_status;
use selection::select_one_source_per_required_family;

const REQUIRED_FAMILIES: [&str; 5] = [
    "stretch:drums_percussion",
    "stretch:bass",
    "stretch:vocals",
    "stretch:pads_sustains",
    "stretch:full_mix",
];
const REQUIRED_RATIOS: [f64; 3] = [0.75, 1.25, 1.5];

pub(super) fn export_blind_listening_pack(
    sources: &[StretchCorpusListeningSource],
    renders: &[ExternalBenchmarkQualityRender],
    frame_limit: usize,
    signal_path: OfflineHighQualityPath,
    export_dir: &Path,
) -> Result<String, String> {
    if renders.is_empty() {
        return Err("blind listening pack requires external benchmark renders".to_string());
    }

    let selected = select_one_source_per_required_family(sources, renders)?;
    let references_dir = export_dir.join("references");
    let pairs_dir = export_dir.join("pairs");
    fs::create_dir_all(&references_dir)
        .map_err(|error| format!("failed to create {}: {error}", references_dir.display()))?;
    fs::create_dir_all(&pairs_dir)
        .map_err(|error| format!("failed to create {}: {error}", pairs_dir.display()))?;

    let mut notes = tsv_writer(vec![
        "pair_id",
        "case_id",
        "ratio",
        "source_reference",
        "candidate_a",
        "candidate_b",
        "transient",
        "tonal",
        "stereo",
        "formant",
        "boundary",
        "preference",
        "notes",
        "completed",
    ])?;
    let mut key = tsv_writer(vec![
        "pair_id",
        "case_id",
        "ratio",
        "source_path",
        "candidate_a_backend",
        "candidate_b_backend",
        "signal_path",
        "external_tool",
        "target_rms",
        "source_gain_db",
        "signal_gain_db",
        "external_gain_db",
    ])?;

    for (index, render) in selected.iter().enumerate() {
        let pair_id = format!("L{:03}", index + 1);
        let source = match source_for_external_quality_render(sources, render) {
            ExternalBenchmarkQualitySource::Found(source) => source,
            ExternalBenchmarkQualitySource::Missing => {
                return Err(format!("blind pair {pair_id} has no source"));
            }
            ExternalBenchmarkQualitySource::Ambiguous => {
                return Err(format!("blind pair {pair_id} has an ambiguous source"));
            }
        };
        let source_audio = decode_listening_source_audio(source.as_ref(), frame_limit)?;
        let external_audio = decode_external_benchmark_render_audio(render)?;
        if source_audio.sample_rate_hz != external_audio.sample_rate_hz {
            return Err(format!(
                "blind pair {pair_id} sample-rate mismatch: source={} external={}",
                source_audio.sample_rate_hz, external_audio.sample_rate_hz
            ));
        }
        if source_audio.channels != external_audio.channels
            || !matches!(source_audio.channels, 1 | 2)
        {
            return Err(format!(
                "blind pair {pair_id} requires matching mono or stereo channels: source={} external={}",
                source_audio.channels, external_audio.channels
            ));
        }

        let mut stretcher = OfflineHighQualityStretcher::with_path(render.ratio, signal_path);
        let signal_samples = if source_audio.channels == 1 {
            stretcher.stretch_mono(&source_audio.samples)
        } else {
            stretcher.stretch_interleaved_stereo(&source_audio.samples)
        };
        let matched = level_match_group(
            &source_audio.samples,
            &signal_samples,
            &external_audio.samples,
        )?;
        let signal_is_a = stable_assignment_is_signal_a(render, signal_path);
        let (candidate_a, candidate_b, candidate_a_backend, candidate_b_backend) = if signal_is_a {
            (
                &matched.signal,
                &matched.external,
                format!("Signal::{signal_path:?}"),
                render.tool_name.clone(),
            )
        } else {
            (
                &matched.external,
                &matched.signal,
                render.tool_name.clone(),
                format!("Signal::{signal_path:?}"),
            )
        };

        let source_name = format!("{pair_id}-source.wav");
        let candidate_a_name = format!("{pair_id}-A.wav");
        let candidate_b_name = format!("{pair_id}-B.wav");
        write_float_wav(
            &references_dir.join(&source_name),
            source_audio.sample_rate_hz,
            source_audio.channels,
            &matched.source,
        )?;
        write_float_wav(
            &pairs_dir.join(&candidate_a_name),
            source_audio.sample_rate_hz,
            source_audio.channels,
            candidate_a,
        )?;
        write_float_wav(
            &pairs_dir.join(&candidate_b_name),
            source_audio.sample_rate_hz,
            source_audio.channels,
            candidate_b,
        )?;

        notes
            .write_record([
                pair_id.as_str(),
                render.case_id.as_str(),
                &format!("{:.6}", render.ratio),
                &format!("references/{source_name}"),
                &format!("pairs/{candidate_a_name}"),
                &format!("pairs/{candidate_b_name}"),
                "",
                "",
                "",
                "",
                "",
                "",
                "",
                "false",
            ])
            .map_err(|error| format!("failed to write blind notes row: {error}"))?;
        key.write_record([
            pair_id.as_str(),
            render.case_id.as_str(),
            &format!("{:.6}", render.ratio),
            source.source_path.as_str(),
            candidate_a_backend.as_str(),
            candidate_b_backend.as_str(),
            &format!("{signal_path:?}"),
            render.tool_name.as_str(),
            &format!("{:.9}", matched.target_rms),
            &format!("{:.6}", gain_db(matched.source_gain)),
            &format!("{:.6}", gain_db(matched.signal_gain)),
            &format!("{:.6}", gain_db(matched.external_gain)),
        ])
        .map_err(|error| format!("failed to write blind key row: {error}"))?;
    }

    write_tsv(export_dir.join("blind-listening-notes.tsv"), notes)?;
    write_tsv(export_dir.join("blind-listening-key.tsv"), key)?;
    fs::write(
        export_dir.join("README.md"),
        listening_readme(selected.len()),
    )
    .map_err(|error| format!("failed to write blind pack README: {error}"))?;

    Ok(format!(
        "blind_listening_pack export_dir={:?} status=ReadyForOperator pairs={} families={} signal_path={:?} level_policy=rms-matched-with-0p95-peak-ceiling notes={:?} key={:?}",
        export_dir.display().to_string(),
        selected.len(),
        REQUIRED_FAMILIES.len(),
        signal_path,
        export_dir.join("blind-listening-notes.tsv").display().to_string(),
        export_dir.join("blind-listening-key.tsv").display().to_string(),
    ))
}

fn tsv_writer(headers: Vec<&str>) -> Result<csv::Writer<Vec<u8>>, String> {
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .from_writer(Vec::new());
    writer
        .write_record(headers)
        .map_err(|error| format!("failed to write TSV headers: {error}"))?;
    Ok(writer)
}

fn write_tsv(path: PathBuf, writer: csv::Writer<Vec<u8>>) -> Result<(), String> {
    let bytes = writer
        .into_inner()
        .map_err(|error| format!("failed to finish {}: {error}", path.display()))?;
    fs::write(&path, bytes).map_err(|error| format!("failed to write {}: {error}", path.display()))
}

fn listening_readme(pair_count: usize) -> String {
    format!(
        "# Blind Stretch Listening Pack\n\nStatus: ready for operator notes\n\nPairs: {pair_count}\nFamilies: percussion, bass, vocals, pads/sustains, full mix\n\n1. Keep `blind-listening-key.tsv` closed.\n2. For each row in `blind-listening-notes.tsv`, hear the source reference, then A and B.\n3. Record transient, tonal, stereo, formant, and boundary findings.\n4. Set `completed=true` only after all fields were considered.\n5. Reveal the key after notes are frozen.\n\nAll WAVs are RMS matched to one common per-pair target with a 0.95 peak ceiling. This is a local evidence artifact; do not commit licensed source audio.\n"
    )
}

#[cfg(test)]
#[path = "listening_pack/tests.rs"]
mod tests;
