//! Select local FMA candidates for Signal stretch corpus listening runs.

use std::env;
use std::fs;
use std::process;

mod args;
mod report;
mod select;

#[cfg(test)]
mod tests;

use args::{parse_args, usage};
use report::{format_fma_selection_report, format_fma_selection_tsv};
use select::{review_seed_candidates, select_fma_candidates};

fn main() {
    let raw_args = env::args().skip(1).collect::<Vec<_>>();
    if raw_args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("{}", usage());
        return;
    }

    let args = match parse_args(raw_args) {
        Ok(args) => args,
        Err(message) => {
            eprintln!("{message}");
            eprintln!("{}", usage());
            process::exit(2);
        }
    };

    let candidates = match select_fma_candidates(&args) {
        Ok(candidates) => candidates,
        Err(message) => {
            eprintln!("{message}");
            process::exit(1);
        }
    };
    let report = format_fma_selection_report(&args, &candidates);

    if let Some(parent) = args.output.parent() {
        if let Err(error) = fs::create_dir_all(parent) {
            eprintln!("failed to create {}: {error}", parent.display());
            process::exit(1);
        }
    }
    if let Err(error) = fs::write(&args.output, report) {
        eprintln!("failed to write {}: {error}", args.output.display());
        process::exit(1);
    }
    println!("wrote {}", args.output.display());

    if let Some(tsv_output) = &args.tsv_output {
        let tsv = match format_fma_selection_tsv(&candidates) {
            Ok(tsv) => tsv,
            Err(message) => {
                eprintln!("{message}");
                process::exit(1);
            }
        };
        if let Some(parent) = tsv_output.parent() {
            if let Err(error) = fs::create_dir_all(parent) {
                eprintln!("failed to create {}: {error}", parent.display());
                process::exit(1);
            }
        }
        if let Err(error) = fs::write(tsv_output, tsv) {
            eprintln!("failed to write {}: {error}", tsv_output.display());
            process::exit(1);
        }
        println!("wrote {}", tsv_output.display());
    }

    if let Some(review_seed_tsv_output) = &args.review_seed_tsv_output {
        let review_seed = review_seed_candidates(&candidates, args.review_seed_per_family);
        let tsv = match format_fma_selection_tsv(&review_seed) {
            Ok(tsv) => tsv,
            Err(message) => {
                eprintln!("{message}");
                process::exit(1);
            }
        };
        if let Some(parent) = review_seed_tsv_output.parent() {
            if let Err(error) = fs::create_dir_all(parent) {
                eprintln!("failed to create {}: {error}", parent.display());
                process::exit(1);
            }
        }
        if let Err(error) = fs::write(review_seed_tsv_output, tsv) {
            eprintln!(
                "failed to write {}: {error}",
                review_seed_tsv_output.display()
            );
            process::exit(1);
        }
        println!("wrote {}", review_seed_tsv_output.display());
    }
}
