use super::introspection::{metadata_descriptor, metadata_io_layout, read_vst3_bundle_snapshot};
use super::*;
use std::{
    env, fs, io,
    path::{Path, PathBuf},
    time::Instant,
};

/// Classification for one failed VST3 bundle discovery attempt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vst3DiscoveryDiagnosticKind {
    /// The bundle scan helper exceeded its watchdog deadline.
    TimedOut,
    /// The helper could not start or exited unsuccessfully.
    HelperFailed,
    /// Bundle metadata or helper output was invalid.
    InvalidData,
}

/// Bounded diagnostic for one VST3 bundle discovery attempt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vst3DiscoveryDiagnostic {
    /// Stable failure classification.
    pub kind: Vst3DiscoveryDiagnosticKind,
    /// Bundle path that was attempted.
    pub bundle_path: String,
    /// Bounded human-readable detail.
    pub detail: String,
    /// Wall-clock time spent on the bundle.
    pub elapsed_ms: u64,
}

/// Detailed result of VST3 discovery across explicit roots.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Vst3DiscoveryBatch {
    /// Successfully discovered plugin types.
    pub discovered: Vec<Vst3DiscoveredPluginType>,
    /// Bundle-level diagnostics in attempt order.
    pub diagnostics: Vec<Vst3DiscoveryDiagnostic>,
}

impl Vst3HostAdapter {
    /// Returns the default VST3 scan roots for the given platform.
    pub fn default_scan_roots(&self, platform: Vst3HostPlatform) -> Vec<Vst3ScanRoot> {
        match platform {
            Vst3HostPlatform::MacOs => vec![
                Vst3ScanRoot {
                    root: "~/Library/Audio/Plug-Ins/VST3".into(),
                    platform,
                    kind: Vst3ScanRootKind::UserBundleRoot,
                },
                Vst3ScanRoot {
                    root: "/Library/Audio/Plug-Ins/VST3".into(),
                    platform,
                    kind: Vst3ScanRootKind::SystemBundleRoot,
                },
            ],
            Vst3HostPlatform::Linux => vec![
                Vst3ScanRoot {
                    root: "~/.vst3".into(),
                    platform,
                    kind: Vst3ScanRootKind::UserBundleRoot,
                },
                Vst3ScanRoot {
                    root: "/usr/lib/vst3".into(),
                    platform,
                    kind: Vst3ScanRootKind::SystemBundleRoot,
                },
                Vst3ScanRoot {
                    root: "/usr/local/lib/vst3".into(),
                    platform,
                    kind: Vst3ScanRootKind::SystemBundleRoot,
                },
            ],
            Vst3HostPlatform::Windows => vec![
                Vst3ScanRoot {
                    root: "%LOCALAPPDATA%/Programs/Common/VST3".into(),
                    platform,
                    kind: Vst3ScanRootKind::UserBundleRoot,
                },
                Vst3ScanRoot {
                    root: "%COMMONPROGRAMFILES%/VST3".into(),
                    platform,
                    kind: Vst3ScanRootKind::SystemBundleRoot,
                },
            ],
        }
    }

    /// Scans the given filesystem roots for VST3 bundle directories and
    /// returns all discovered plugin types. Discovery reads `moduleinfo.json`
    /// first and falls back to factory introspection for older bundles. An
    /// empty root list scans nothing — system plugin directories are never
    /// scanned implicitly; pass [`Vst3HostAdapter::default_scan_roots`]
    /// explicitly to opt in.
    pub fn discover_plugins_for_roots(
        &self,
        platform: Vst3HostPlatform,
        roots: &[String],
    ) -> Vec<Vst3DiscoveredPluginType> {
        self.discover_plugins_for_roots_with_diagnostics(platform, roots)
            .discovered
    }

    /// Scans explicit roots and returns successful plugin types plus one
    /// bounded diagnostic for each bundle that could not be inspected.
    pub fn discover_plugins_for_roots_with_diagnostics(
        &self,
        platform: Vst3HostPlatform,
        roots: &[String],
    ) -> Vst3DiscoveryBatch {
        let mut batch = Vst3DiscoveryBatch::default();
        for root in roots {
            let expanded_root = expand_scan_root(root);
            if is_explicit_vst3_module_path(&expanded_root) {
                append_vst3_bundle_outcome(&mut batch, &expanded_root, platform);
                continue;
            }
            let Ok(entries) = fs::read_dir(&expanded_root) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                let file_name = entry.file_name();
                let Some(file_name) = file_name.to_str() else {
                    continue;
                };
                if !file_name.ends_with(".vst3") {
                    continue;
                }
                append_vst3_bundle_outcome(&mut batch, &path, platform);
            }
        }
        batch
    }
}

fn is_explicit_vst3_module_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("vst3") || extension.eq_ignore_ascii_case("bundle")
        })
}

fn append_vst3_bundle_outcome(
    batch: &mut Vst3DiscoveryBatch,
    path: &Path,
    platform: Vst3HostPlatform,
) {
    let started_at = Instant::now();
    let snapshot = match read_vst3_bundle_snapshot(path, platform) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            batch.diagnostics.push(diagnostic_from_error(
                path,
                error,
                started_at.elapsed().as_millis(),
            ));
            return;
        }
    };
    if snapshot.plugins.is_empty() {
        batch.diagnostics.push(Vst3DiscoveryDiagnostic {
            kind: Vst3DiscoveryDiagnosticKind::InvalidData,
            bundle_path: path.to_string_lossy().into_owned(),
            detail: "VST3 bundle exposed no eligible plugin classes".to_string(),
            elapsed_ms: elapsed_ms(started_at.elapsed().as_millis()),
        });
        return;
    }
    for metadata in snapshot.plugins {
        let plugin = Vst3DiscoveredPluginType {
            plugin_type_id: PluginTypeId(metadata.plugin_type_id.clone()),
            class_id: metadata.class_id.clone(),
            controller_class_id: metadata.controller_class_id.clone(),
            category: metadata.category.clone(),
            module_root: path.to_string_lossy().into_owned(),
            descriptor: metadata_descriptor(&metadata),
            default_io_layout: metadata_io_layout(&metadata),
        };
        if !batch
            .discovered
            .iter()
            .any(|existing: &Vst3DiscoveredPluginType| {
                existing.plugin_type_id == plugin.plugin_type_id
            })
        {
            batch.discovered.push(plugin);
        }
    }
}

fn diagnostic_from_error(
    path: &Path,
    error: io::Error,
    elapsed_millis: u128,
) -> Vst3DiscoveryDiagnostic {
    let kind = match error.kind() {
        io::ErrorKind::TimedOut => Vst3DiscoveryDiagnosticKind::TimedOut,
        io::ErrorKind::InvalidData => Vst3DiscoveryDiagnosticKind::InvalidData,
        _ => Vst3DiscoveryDiagnosticKind::HelperFailed,
    };
    Vst3DiscoveryDiagnostic {
        kind,
        bundle_path: path.to_string_lossy().into_owned(),
        detail: bounded_detail(&error.to_string()),
        elapsed_ms: elapsed_ms(elapsed_millis),
    }
}

fn bounded_detail(detail: &str) -> String {
    const MAX_CHARS: usize = 240;
    let normalized = detail.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.chars().count() <= MAX_CHARS {
        return normalized;
    }
    normalized.chars().take(MAX_CHARS).collect()
}

fn elapsed_ms(value: u128) -> u64 {
    value.min(u128::from(u64::MAX)) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_cubase_private_bundle_as_an_explicit_vst3_module() {
        assert!(is_explicit_vst3_module_path(Path::new(
            "/Applications/Cubase.app/Contents/Components/Modulation FX.bundle"
        )));
        assert!(is_explicit_vst3_module_path(Path::new(
            "/Library/Audio/Plug-Ins/VST3/Example.vst3"
        )));
        assert!(!is_explicit_vst3_module_path(Path::new(
            "/Library/Audio/Plug-Ins/VST3"
        )));
    }

    #[test]
    fn discovery_errors_map_to_stable_bounded_diagnostics() {
        let cases = [
            (
                io::ErrorKind::TimedOut,
                Vst3DiscoveryDiagnosticKind::TimedOut,
            ),
            (
                io::ErrorKind::InvalidData,
                Vst3DiscoveryDiagnosticKind::InvalidData,
            ),
            (
                io::ErrorKind::Other,
                Vst3DiscoveryDiagnosticKind::HelperFailed,
            ),
        ];

        for (error_kind, expected_kind) in cases {
            let detail = format!("detail\n{}", "x".repeat(300));
            let diagnostic = diagnostic_from_error(
                Path::new("/tmp/Example.vst3"),
                io::Error::new(error_kind, detail),
                17,
            );

            assert_eq!(diagnostic.kind, expected_kind);
            assert_eq!(diagnostic.elapsed_ms, 17);
            assert!(diagnostic.detail.chars().count() <= 240);
            assert!(!diagnostic.detail.contains('\n'));
        }
    }
}

fn expand_scan_root(root: &str) -> PathBuf {
    if let Some(stripped) = root.strip_prefix("~/") {
        if let Some(home) = env::var_os("HOME") {
            return PathBuf::from(home).join(stripped);
        }
    }
    if root.contains('%') {
        let mut expanded = root.to_string();
        for (key, value) in env::vars() {
            let pattern = format!("%{key}%");
            if expanded.contains(&pattern) {
                expanded = expanded.replace(&pattern, &value);
            }
        }
        return PathBuf::from(expanded);
    }
    PathBuf::from(root)
}
