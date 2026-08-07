use std::path::{Path, PathBuf};

use crate::ClapDiscoveredPluginType;

use super::entry;

pub(crate) fn clap_bundle_binary(bundle_root: &Path) -> Option<PathBuf> {
    let macos_root = bundle_root.join("Contents").join("MacOS");
    if let Ok(entries) = std::fs::read_dir(&macos_root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                return Some(path);
            }
        }
    }
    None
}

pub(super) fn scan_clap_root(
    root: &str,
    probe_capabilities: bool,
) -> Vec<ClapDiscoveredPluginType> {
    let root = expand_home(root);
    let Ok(metadata) = std::fs::metadata(&root) else {
        return Vec::new();
    };

    if metadata.is_file() {
        return entry::discover_from_clap_library(&root, probe_capabilities).unwrap_or_default();
    }
    if path_extension_matches(&root, "clap") {
        return clap_bundle_binary(&root)
            .and_then(|library| entry::discover_from_clap_library(&library, probe_capabilities))
            .unwrap_or_default();
    }

    collect_clap_candidates(&root)
        .into_iter()
        .flat_map(|candidate| {
            entry::discover_from_clap_library(&candidate, probe_capabilities).unwrap_or_default()
        })
        .collect()
}

fn collect_clap_candidates(root: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path_extension_matches(&path, "clap") {
                    if let Some(bundle_binary) = clap_bundle_binary(&path) {
                        candidates.push(bundle_binary);
                    }
                } else {
                    candidates.extend(collect_clap_candidates(&path));
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

fn expand_home(root: &str) -> PathBuf {
    if let Some(stripped) = root.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(stripped);
        }
    }
    PathBuf::from(root)
}
