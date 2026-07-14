use std::path::PathBuf;

use signal_plugin_clap::fixture::compile_clap_instrument_fixture;

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let destination = PathBuf::from(args.next().ok_or_else(|| {
        "usage: install_instrument_fixture <destination> <plugin-id> <name>".to_string()
    })?);
    let plugin_id = args.next().ok_or_else(|| "missing plugin id".to_string())?;
    let name = args
        .next()
        .ok_or_else(|| "missing plugin name".to_string())?;
    if args.next().is_some() {
        return Err("unexpected extra arguments".to_string());
    }

    let parent = destination
        .parent()
        .ok_or_else(|| "destination has no parent directory".to_string())?;
    let compiled = compile_clap_instrument_fixture(parent, &plugin_id, &name)?;
    if compiled != destination {
        std::fs::rename(&compiled, &destination)
            .map_err(|error| format!("fixture install failed: {error}"))?;
    }
    println!("{}", destination.display());
    Ok(())
}
