mod runtime_core;
mod runtime_surfaces;

use crate::{CliMode, OutputFormat};
use runtime_core::print_runtime_core_boundary_mode;
use runtime_surfaces::print_runtime_surface_boundary_mode;

pub(super) fn print_boundary_describe_mode(mode: &CliMode, format: OutputFormat) -> bool {
    print_runtime_core_boundary_mode(mode, format)
        || print_runtime_surface_boundary_mode(mode, format)
}
