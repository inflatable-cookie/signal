//! Emit the retained Signal stretch comparator and blind-listening evidence.

mod alloc_tracker;
mod args;
mod external;
mod listening;
mod listening_pack;
mod manifest;
mod quality;

use std::env;
use std::fs;

use args::{exit_with_error, parse_args, ParseOutcome, ReportArgs};
use external::{load_external_benchmark_metadata, load_external_benchmark_quality_renders};
use listening::load_listening_sources;
use listening_pack::{export_blind_listening_pack, format_blind_listening_note_status};
use quality::format_external_benchmark_quality_metrics;
use signal_dsp_stretch::{
    build_stretch_corpus_comparison_report_with_sources, format_stretch_corpus_comparison_report,
};

fn main() {
    let args = match parse_args(env::args().skip(1)) {
        Ok(ParseOutcome::Run(args)) => *args,
        Ok(ParseOutcome::Help) => {
            println!("{}", args::usage());
            return;
        }
        Err(message) => exit_with_error(&message, true),
    };
    run(args);
}

fn run(args: ReportArgs) {
    let sources =
        load_listening_sources(&args).unwrap_or_else(|message| exit_with_error(&message, false));
    let quality_renders = load_external_benchmark_quality_renders(&args)
        .unwrap_or_else(|message| exit_with_error(&message, false));
    let external_renders = load_external_benchmark_metadata(&quality_renders)
        .unwrap_or_else(|message| exit_with_error(&message, false));

    let report = build_stretch_corpus_comparison_report_with_sources(
        &args.report_name,
        &args.projection_epoch,
        &external_renders,
        &sources,
    );
    let mut formatted = format_stretch_corpus_comparison_report(&report);
    if args.measure_external_benchmark_quality {
        let quality = format_external_benchmark_quality_metrics(
            &sources,
            &quality_renders,
            args.frame_limit,
            args.signal_path,
        )
        .unwrap_or_else(|message| exit_with_error(&message, false));
        if !quality.is_empty() {
            formatted.push('\n');
            formatted.push_str(&quality);
        }
    }
    if let Some(export_dir) = &args.export_blind_listening_pack {
        let pack = export_blind_listening_pack(
            &sources,
            &quality_renders,
            args.frame_limit,
            args.signal_path,
            export_dir,
        )
        .unwrap_or_else(|message| exit_with_error(&message, false));
        formatted.push('\n');
        formatted.push_str(&pack);
    }
    for notes in &args.blind_listening_note_manifests {
        let status = format_blind_listening_note_status(notes)
            .unwrap_or_else(|message| exit_with_error(&message, false));
        formatted.push('\n');
        formatted.push_str(&status);
    }

    if let Some(path) = &args.output {
        fs::write(path, format!("{formatted}\n")).unwrap_or_else(|error| {
            exit_with_error(
                &format!("failed to write {}: {error}", path.display()),
                false,
            )
        });
    } else {
        println!("{formatted}");
    }
}
