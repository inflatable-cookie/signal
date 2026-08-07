//! Out-of-process VST3 factory scan helper orchestration.

use crate::vst3_host_adapter::Vst3HostPlatform;
use std::{
    ffi::OsString,
    io,
    io::Read,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

#[cfg(not(target_os = "macos"))]
use libloading::Library;

use super::factory::load_vst3_factory_classes_from_module;
use super::paths::{preflight_vendor_scan_access, read_vst3_bundle_info};
use super::types::*;

pub(crate) fn run_vst3_scan_helper<I>(args: I) -> i32
where
    I: IntoIterator<Item = OsString>,
{
    let mut args = args.into_iter();
    let Some(first) = args.next() else {
        eprintln!("missing VST3 scan helper platform");
        return 64;
    };
    let platform_arg = if first == crate::vst3_host_adapter::VST3_SCAN_HELPER_ARG {
        let Some(platform) = args.next() else {
            eprintln!("missing VST3 scan helper platform");
            return 64;
        };
        platform
    } else {
        first
    };
    let Some(bundle_root) = args.next() else {
        eprintln!("missing VST3 scan helper bundle path");
        return 64;
    };
    let Some(platform) = parse_platform_arg(&platform_arg) else {
        eprintln!("unsupported VST3 scan helper platform");
        return 64;
    };
    let bundle_root = PathBuf::from(bundle_root);
    if let Err(error) =
        read_vst3_bundle_info(&bundle_root).and_then(|bundle| preflight_vendor_scan_access(&bundle))
    {
        eprintln!("{error}");
        return 65;
    }
    match load_vst3_factory_classes_from_module(&bundle_root, platform) {
        Ok((vendor, classes)) => {
            let payload = Vst3FactorySnapshotWire { vendor, classes };
            match serde_json::to_string(&payload) {
                Ok(json) => {
                    println!("{json}");
                    0
                }
                Err(error) => {
                    eprintln!("failed to encode VST3 scan helper result: {error}");
                    70
                }
            }
        }
        Err(error) => {
            eprintln!("{error}");
            65
        }
    }
}

pub(crate) fn load_vst3_factory_classes_with_helper(
    bundle_root: &Path,
    platform: Vst3HostPlatform,
) -> io::Result<(Option<String>, Vec<Vst3FactoryClass>)> {
    let mut command = scan_helper_command()?;
    command
        .arg(crate::vst3_host_adapter::VST3_SCAN_HELPER_ARG)
        .arg(platform_arg(platform))
        .arg(bundle_root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let child = command.spawn().map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("failed to start VST3 scan helper: {error}"),
        )
    })?;
    read_vst3_scan_helper_child(child, scan_helper_timeout())
}

pub(crate) fn read_vst3_scan_helper_child(
    mut child: Child,
    timeout: Duration,
) -> io::Result<(Option<String>, Vec<Vst3FactoryClass>)> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child.try_wait()? {
            let mut stdout = Vec::new();
            if let Some(mut output) = child.stdout.take() {
                output.read_to_end(&mut stdout)?;
            }
            let mut stderr = String::new();
            if let Some(mut output) = child.stderr.take() {
                output.read_to_string(&mut stderr)?;
            }
            if !status.success() {
                let detail = stderr.split_whitespace().collect::<Vec<_>>().join(" ");
                let message = format!(
                    "VST3 scan helper exited with status {status}{}",
                    if detail.is_empty() {
                        String::new()
                    } else {
                        format!(": {detail}")
                    }
                );
                return Err(if status.code() == Some(65) {
                    io::Error::new(io::ErrorKind::InvalidData, message)
                } else {
                    io::Error::other(message)
                });
            }
            let snapshot = decode_scan_helper_snapshot(&stdout)?;
            return Ok((snapshot.vendor, snapshot.classes));
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "VST3 scan helper timed out",
            ));
        }
        thread::sleep(Duration::from_millis(10));
    }
}

pub(crate) fn decode_scan_helper_snapshot(stdout: &[u8]) -> io::Result<Vst3FactorySnapshotWire> {
    stdout
        .split(|byte| *byte == b'\n')
        .rev()
        .find_map(|line| serde_json::from_slice::<Vst3FactorySnapshotWire>(line).ok())
        .ok_or_else(|| {
            let error = serde_json::from_slice::<Vst3FactorySnapshotWire>(stdout)
                .err()
                .expect("unmatched helper output should remain invalid");
            io::Error::new(io::ErrorKind::InvalidData, error.to_string())
        })
}

#[cfg(all(test, unix))]
mod scan_helper_tests {
    use super::super::paths::is_native_instruments_bundle;
    use super::*;

    fn shell_child(script: &str) -> Child {
        Command::new("/bin/sh")
            .args(["-c", script])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn deterministic scan helper fixture")
    }

    #[test]
    fn scan_helper_timeout_kills_and_reaps_child() {
        let child = shell_child("sleep 5");
        let error = read_vst3_scan_helper_child(child, Duration::from_millis(20))
            .expect_err("slow helper should time out");

        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    }

    #[test]
    fn scan_helper_abnormal_exit_is_reported() {
        let child = shell_child("echo fixture-reason >&2; exit 7");
        let error = read_vst3_scan_helper_child(child, Duration::from_secs(1))
            .expect_err("failed helper should be reported");

        assert_eq!(error.kind(), io::ErrorKind::Other);
        assert!(error.to_string().contains("status"));
        assert!(error.to_string().contains("fixture-reason"));
    }

    #[test]
    fn scan_helper_inspection_failure_is_invalid_not_crashed() {
        let child = shell_child("echo invalid-fixture >&2; exit 65");
        let error = read_vst3_scan_helper_child(child, Duration::from_secs(1))
            .expect_err("inspection failure should be reported");

        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(error.to_string().contains("invalid-fixture"));
    }

    #[test]
    fn scan_helper_invalid_output_is_reported() {
        let child = shell_child("printf not-json");
        let error = read_vst3_scan_helper_child(child, Duration::from_secs(1))
            .expect_err("invalid helper output should be reported");

        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn scan_helper_ignores_plugin_logs_around_json_payload() {
        let payload = r#"{"vendor":"Example","classes":[]}"#;
        for script in [
            format!("printf 'plugin log\\n%s\\n' '{payload}'"),
            format!("printf '%s\\nplugin shutdown log\\n' '{payload}'"),
        ] {
            let child = shell_child(&script);
            let (vendor, classes) = read_vst3_scan_helper_child(child, Duration::from_secs(1))
                .expect("embedded helper payload");
            assert_eq!(vendor.as_deref(), Some("Example"));
            assert!(classes.is_empty());
        }
    }

    #[test]
    fn native_instruments_bundle_detection_is_vendor_scoped() {
        assert!(is_native_instruments_bundle(
            "com.native-instruments.Raum.vst3"
        ));
        assert!(!is_native_instruments_bundle("com.example.Raum.vst3"));
    }
}

pub(crate) fn scan_helper_command() -> io::Result<Command> {
    if let Some(path) = std::env::var_os(VST3_SCAN_HELPER_ENV).filter(|path| !path.is_empty()) {
        return Ok(Command::new(path));
    }
    if let Some(path) = nearby_scan_helper_binary()? {
        return Ok(Command::new(path));
    }
    Ok(Command::new(std::env::current_exe()?))
}

pub(crate) fn nearby_scan_helper_binary() -> io::Result<Option<PathBuf>> {
    let current_exe = std::env::current_exe()?;
    let Some(current_dir) = current_exe.parent() else {
        return Ok(None);
    };
    let candidates = [
        current_dir.join(helper_binary_name()),
        current_dir
            .parent()
            .map(|parent| parent.join(helper_binary_name()))
            .unwrap_or_else(|| current_dir.join(helper_binary_name())),
    ];
    Ok(candidates.into_iter().find(|path| path.is_file()))
}

pub(crate) fn helper_binary_name() -> String {
    #[cfg(target_os = "windows")]
    {
        format!("{VST3_SCAN_HELPER_BINARY}.exe")
    }
    #[cfg(not(target_os = "windows"))]
    {
        VST3_SCAN_HELPER_BINARY.to_string()
    }
}

pub(crate) fn scan_helper_timeout() -> Duration {
    std::env::var(VST3_SCAN_HELPER_TIMEOUT_MS_ENV)
        .ok()
        .and_then(|raw| raw.parse::<u64>().ok())
        .map(Duration::from_millis)
        .unwrap_or(VST3_SCAN_HELPER_DEFAULT_TIMEOUT)
}

pub(crate) fn platform_arg(platform: Vst3HostPlatform) -> &'static str {
    match platform {
        Vst3HostPlatform::MacOs => "macos",
        Vst3HostPlatform::Linux => "linux",
        Vst3HostPlatform::Windows => "windows",
    }
}

pub(crate) fn parse_platform_arg(value: &OsString) -> Option<Vst3HostPlatform> {
    match value.to_str()? {
        "macos" => Some(Vst3HostPlatform::MacOs),
        "linux" => Some(Vst3HostPlatform::Linux),
        "windows" => Some(Vst3HostPlatform::Windows),
        _ => None,
    }
}
