mod boundaries;
mod lanes;

use crate::{CliMode, OutputFormat};
use boundaries::print_boundary_describe_mode;
use lanes::print_lane_describe_mode;

pub(super) fn print_surface(format: OutputFormat, text: fn() -> String, json: fn() -> String) {
    match format {
        OutputFormat::Text => println!("{}", text()),
        OutputFormat::Json => println!("{}", json()),
    }
}

pub(crate) fn print_describe_mode(mode: &CliMode, format: OutputFormat) -> bool {
    print_boundary_describe_mode(mode, format) || print_lane_describe_mode(mode, format)
}
