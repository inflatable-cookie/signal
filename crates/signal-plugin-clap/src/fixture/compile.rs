use super::source::{clap_fixture_source, clap_fixture_source_for_layout};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

/// Returns `true` when a `rustc` binary is invocable (fixture tests skip
/// gracefully when it is not).
pub fn rustc_available() -> bool {
    Command::new("rustc")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Compile the fixture cdylib into `directory`, returning the library path.
/// The library file is named after `plugin_name` with a `.clap` extension so
/// directory scans pick it up. Errors carry the rustc failure detail.
pub fn compile_clap_fixture(
    directory: &Path,
    plugin_type_id: &str,
    plugin_name: &str,
    midi_outputs: u16,
) -> Result<PathBuf, String> {
    std::fs::create_dir_all(directory)
        .map_err(|error| format!("fixture directory create failed: {error}"))?;
    let source_path = directory.join("fixture.rs");
    let library_path = directory.join(format!(
        "{}.clap",
        plugin_name.to_lowercase().replace(' ', "-")
    ));
    let source = clap_fixture_source(plugin_type_id, plugin_name, midi_outputs);
    std::fs::write(&source_path, source)
        .map_err(|error| format!("fixture source write failed: {error}"))?;
    let output = Command::new("rustc")
        .arg("--crate-type=cdylib")
        .arg("--edition=2021")
        .arg(&source_path)
        .arg("-o")
        .arg(&library_path)
        .output()
        .map_err(|error| format!("rustc invocation failed: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "clap fixture compilation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(library_path)
}

/// Compile the same real CLAP fixture as a MIDI-input, stereo-output
/// instrument with no audio input bus. Note velocity drives its generated
/// constant signal; note-off returns it to silence.
pub fn compile_clap_instrument_fixture(
    directory: &Path,
    plugin_type_id: &str,
    plugin_name: &str,
) -> Result<PathBuf, String> {
    std::fs::create_dir_all(directory)
        .map_err(|error| format!("fixture directory create failed: {error}"))?;
    let source_path = directory.join("instrument-fixture.rs");
    let library_path = directory.join(format!(
        "{}.clap",
        plugin_name.to_lowercase().replace(' ', "-")
    ));
    std::fs::write(
        &source_path,
        clap_fixture_source_for_layout(plugin_type_id, plugin_name, 0, true, 1),
    )
    .map_err(|error| format!("fixture source write failed: {error}"))?;
    let output = Command::new("rustc")
        .arg("--crate-type=cdylib")
        .arg("--edition=2021")
        .arg(&source_path)
        .arg("-o")
        .arg(&library_path)
        .output()
        .map_err(|error| format!("rustc invocation failed: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "clap fixture compilation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(library_path)
}

/// Compile an instrument fixture which requires the host to provide every
/// declared stereo output bus while rendering only its main output.
pub fn compile_clap_multi_output_instrument_fixture(
    directory: &Path,
    plugin_type_id: &str,
    plugin_name: &str,
    output_bus_count: u32,
) -> Result<PathBuf, String> {
    std::fs::create_dir_all(directory)
        .map_err(|error| format!("fixture directory create failed: {error}"))?;
    let source_path = directory.join("multi-output-instrument-fixture.rs");
    let library_path = directory.join(format!(
        "{}.clap",
        plugin_name.to_lowercase().replace(' ', "-")
    ));
    std::fs::write(
        &source_path,
        clap_fixture_source_for_layout(plugin_type_id, plugin_name, 0, true, output_bus_count),
    )
    .map_err(|error| format!("fixture source write failed: {error}"))?;
    let output = Command::new("rustc")
        .arg("--crate-type=cdylib")
        .arg("--edition=2021")
        .arg(&source_path)
        .arg("-o")
        .arg(&library_path)
        .output()
        .map_err(|error| format!("rustc invocation failed: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "clap fixture compilation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(library_path)
}
