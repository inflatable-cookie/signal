use super::describe_flags::DESCRIBE_FLAG_SPECS;
use super::types::{CliArgs, CliMode, ExportDebugOptions, HostProfile, OutputFormat, Scenario};

fn describe_mode_from_flag(flag: &str) -> Option<CliMode> {
    DESCRIBE_FLAG_SPECS
        .iter()
        .find(|spec| spec.flag == flag)
        .map(|spec| spec.mode)
}

fn describe_positional_error(mode: CliMode) -> String {
    let flag = DESCRIBE_FLAG_SPECS
        .iter()
        .find(|spec| spec.mode == mode)
        .map(|spec| spec.flag)
        .unwrap_or("--describe-export");
    format!("`{flag}` does not accept <profile> <scenario> positionals")
}

pub(crate) fn parse_args(args: impl IntoIterator<Item = String>) -> Result<CliArgs, String> {
    let mut format = OutputFormat::Text;
    let mut debug = ExportDebugOptions::default();
    let mut describe_mode = None;
    let mut positional = Vec::new();

    for arg in args {
        if arg == "--json" {
            format = OutputFormat::Json;
            continue;
        }
        if arg == "--text" {
            format = OutputFormat::Text;
            continue;
        }
        if arg == "--include-payload" {
            debug.payload = true;
            continue;
        }
        if let Some(mode) = describe_mode_from_flag(&arg) {
            if describe_mode.replace(mode).is_some() {
                return Err("describe modes are mutually exclusive".into());
            }
            continue;
        }
        if let Some(value) = arg.strip_prefix("--format=") {
            format = OutputFormat::parse(value)?;
            continue;
        }
        positional.push(arg);
    }

    if let Some(mode) = describe_mode {
        if !positional.is_empty() {
            return Err(describe_positional_error(mode));
        }
        return Ok(CliArgs {
            format,
            debug,
            mode,
        });
    }

    if positional.len() != 2 {
        return Err("expected <profile> <scenario>".into());
    }

    Ok(CliArgs {
        format,
        debug,
        mode: CliMode::Run {
            profile: HostProfile::parse(&positional[0])?,
            scenario: Scenario::parse(&positional[1])?,
        },
    })
}
