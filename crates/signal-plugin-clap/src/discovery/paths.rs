use std::path::{Path, PathBuf};

use crate::{ClapDiscoveredPluginType, ClapHostPlatform};

use super::entry;

pub(crate) fn clap_bundle_binary_for_platform(
    bundle_root: &Path,
    platform: ClapHostPlatform,
) -> Option<PathBuf> {
    match platform {
        ClapHostPlatform::MacOs => {
            let macos_root = bundle_root.join("Contents").join("MacOS");
            std::fs::read_dir(macos_root)
                .ok()?
                .flatten()
                .map(|entry| entry.path())
                .find(|path| path.is_file())
        }
        ClapHostPlatform::Linux => {
            let stem = bundle_root.file_stem()?.to_str()?;
            ["x86_64-linux", "aarch64-linux"]
                .into_iter()
                .flat_map(|architecture| {
                    let directory = bundle_root.join("Contents").join(architecture);
                    [directory.join(stem), directory.join(format!("{stem}.so"))]
                })
                .find(|path| path.is_file())
        }
        // Windows CLAP units are flat `.clap` DLLs. A directory whose name
        // ends in `.clap` is not a recognized Windows unit.
        ClapHostPlatform::Windows => None,
    }
}

pub(super) fn scan_clap_root(
    root: &str,
    platform: ClapHostPlatform,
    probe_capabilities: bool,
) -> Vec<ClapDiscoveredPluginType> {
    let root = expand_scan_root(root);
    let Ok(metadata) = std::fs::metadata(&root) else {
        return Vec::new();
    };

    if metadata.is_file() {
        return entry::discover_from_clap_library(&root, platform, probe_capabilities)
            .unwrap_or_default();
    }
    if path_extension_matches(&root, "clap") {
        return clap_bundle_binary_for_platform(&root, platform)
            .and_then(|library| {
                entry::discover_from_clap_library(&library, platform, probe_capabilities)
            })
            .unwrap_or_default();
    }

    collect_clap_candidates(&root, platform)
        .into_iter()
        .flat_map(|candidate| {
            entry::discover_from_clap_library(&candidate, platform, probe_capabilities)
                .unwrap_or_default()
        })
        .collect()
}

fn collect_clap_candidates(root: &Path, platform: ClapHostPlatform) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path_extension_matches(&path, "clap") {
                    if let Some(bundle_binary) = clap_bundle_binary_for_platform(&path, platform) {
                        candidates.push(bundle_binary);
                    }
                } else {
                    candidates.extend(collect_clap_candidates(&path, platform));
                }
            } else if path_extension_matches(&path, "clap") {
                candidates.push(path);
            }
        }
    }
    candidates
}

fn path_extension_matches(path: &Path, extension: &str) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case(extension))
}

fn expand_scan_root(root: &str) -> PathBuf {
    if let Some(stripped) = root.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(stripped);
        }
    }
    if root.contains('%') {
        let mut expanded = root.to_string();
        for (key, value) in std::env::vars() {
            let pattern = format!("%{key}%");
            if expanded.contains(&pattern) {
                expanded = expanded.replace(&pattern, &value);
            }
        }
        return PathBuf::from(expanded);
    }
    PathBuf::from(root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_scan_root_replaces_percent_env_vars_like_unix_home() {
        let Ok(home) = std::env::var("HOME") else {
            return;
        };
        assert_eq!(
            expand_scan_root("%HOME%/CLAP"),
            PathBuf::from(home).join("CLAP")
        );
    }

    #[test]
    fn windows_platform_does_not_resolve_directory_bundles() {
        let root = std::env::temp_dir().join(format!(
            "signal-clap-windows-bundle-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
        ));
        let bundle = root.join("Example.clap");
        let macos_binary = bundle.join("Contents").join("MacOS").join("Example");
        let linux_binary = bundle
            .join("Contents")
            .join("x86_64-linux")
            .join("Example.so");
        let windows_binary = bundle
            .join("Contents")
            .join("x86_64-win")
            .join("Example.clap");
        std::fs::create_dir_all(macos_binary.parent().expect("macos parent")).expect("macos dirs");
        std::fs::create_dir_all(linux_binary.parent().expect("linux parent")).expect("linux dirs");
        std::fs::create_dir_all(windows_binary.parent().expect("windows parent"))
            .expect("windows dirs");
        std::fs::write(&macos_binary, b"fixture").expect("macos binary");
        std::fs::write(&linux_binary, b"fixture").expect("linux binary");
        std::fs::write(&windows_binary, b"fixture").expect("windows binary");

        assert!(clap_bundle_binary_for_platform(&bundle, ClapHostPlatform::Windows).is_none());
        assert_eq!(
            clap_bundle_binary_for_platform(&bundle, ClapHostPlatform::MacOs),
            Some(macos_binary)
        );
        assert_eq!(
            clap_bundle_binary_for_platform(&bundle, ClapHostPlatform::Linux),
            Some(linux_binary)
        );

        let _ = std::fs::remove_dir_all(root);
    }
}
