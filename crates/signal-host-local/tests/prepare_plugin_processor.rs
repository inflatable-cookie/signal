//! Host-owned bridge backend factory construction tests.

#[path = "support/public_host_edge_sandbox_broker.rs"]
mod public_host_edge_sandbox_broker_support;

use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use public_host_edge_sandbox_broker_support::SandboxBrokerEnvGuard;
use signal_host_local::LocalRuntimeHost;
use signal_plugin::{PluginFormat, PluginIsolationTier};
use signal_runtime::{
    PluginScanRequest, RuntimeConfig, RuntimeErrorKind, RuntimeSupervisorApi, SignalRuntime,
};

struct FixtureDir {
    path: PathBuf,
}

impl Drop for FixtureDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn unique_fixture_dir(label: &str) -> FixtureDir {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "signal-host-local-prepare-{label}-{}-{unique}",
        std::process::id()
    ));
    fs::create_dir_all(&path).expect("fixture directory should be created");
    FixtureDir { path }
}

fn booted_host() -> LocalRuntimeHost {
    LocalRuntimeHost::new(SignalRuntime::new(RuntimeConfig::local(48_000, 512)))
}

fn scan_root(host: &mut LocalRuntimeHost, root: &Path, format: PluginFormat) {
    host.start_plugin_scan(PluginScanRequest {
        roots: vec![root.display().to_string()],
        formats: vec![format],
    })
    .expect("fixture scan should succeed");
}

#[test]
fn prepare_in_process_clap_processor_from_compiled_fixture() {
    if !signal_plugin_clap::fixture::rustc_available() {
        eprintln!("skipping: rustc unavailable for the CLAP fixture");
        return;
    }
    let directory = unique_fixture_dir("clap");
    let plugin_type_id = "com.signal.host-factory-clap";
    signal_plugin_clap::fixture::compile_clap_fixture(
        &directory.path,
        plugin_type_id,
        "Signal Host Factory CLAP",
        0,
    )
    .expect("clap fixture should compile");
    let mut host = booted_host();
    scan_root(&mut host, &directory.path, PluginFormat::Clap);
    host.prepare_plugin_processor(plugin_type_id, PluginIsolationTier::InProcess)
        .expect("in-process CLAP construction should succeed");
}

#[test]
fn prepare_in_process_vst3_processor_from_compiled_fixture() {
    if !signal_plugin_vst3::fixture::rustc_available() {
        eprintln!("skipping: rustc unavailable for the VST3 fixture");
        return;
    }
    let directory = unique_fixture_dir("vst3");
    let plugin_type_id = "plugin:vst3:host-factory";
    signal_plugin_vst3::fixture::compile_vst3_fixture(
        &directory.path,
        plugin_type_id,
        "Signal Host Factory VST3",
    )
    .expect("vst3 fixture should compile");
    let mut host = booted_host();
    scan_root(&mut host, &directory.path, PluginFormat::Vst3);
    host.prepare_plugin_processor(plugin_type_id, PluginIsolationTier::InProcess)
        .expect("in-process VST3 construction should succeed");
}

#[test]
fn prepare_in_process_lv2_processor_from_compiled_fixture() {
    if !signal_plugin_lv2::fixture::rustc_available() {
        eprintln!("skipping: rustc unavailable for the LV2 fixture");
        return;
    }
    let directory = unique_fixture_dir("lv2");
    let plugin_uri = "https://signal.dev/fixtures/lv2/host-factory";
    signal_plugin_lv2::fixture::compile_lv2_fixture(
        &directory.path,
        plugin_uri,
        "Signal Host Factory LV2",
    )
    .expect("lv2 fixture should compile");
    let mut host = booted_host();
    scan_root(&mut host, &directory.path, PluginFormat::Lv2);
    host.prepare_plugin_processor(
        &format!("plugin:lv2:{plugin_uri}"),
        PluginIsolationTier::InProcess,
    )
    .expect("in-process LV2 construction should succeed");
}

#[cfg(target_os = "macos")]
#[test]
fn prepare_in_process_au_processor_from_registry_load_key() {
    let directory = unique_fixture_dir("au");
    let bundle = directory.path.join("AUDelay.component");
    fs::create_dir_all(bundle.join("Contents")).expect("au bundle contents should exist");
    fs::write(
        bundle.join("Contents").join("Info.plist"),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>AudioComponents</key>
  <array>
    <dict>
      <key>manufacturer</key>
      <string>appl</string>
      <key>name</key>
      <string>Apple: AUDelay</string>
      <key>subtype</key>
      <string>dely</string>
      <key>type</key>
      <string>aufx</string>
      <key>version</key>
      <integer>1</integer>
    </dict>
  </array>
  <key>CFBundleIdentifier</key>
  <string>plugin:au:host-factory-audelay</string>
  <key>CFBundleName</key>
  <string>AUDelay</string>
  <key>SignalPluginTypeId</key>
  <string>plugin:au:host-factory-audelay</string>
  <key>SignalAudioInputs</key>
  <integer>2</integer>
  <key>SignalAudioOutputs</key>
  <integer>2</integer>
</dict>
</plist>
"#,
    )
    .expect("au info plist should be written");
    let mut host = booted_host();
    scan_root(&mut host, &directory.path, PluginFormat::Au);
    host.prepare_plugin_processor(
        "plugin:au:host-factory-audelay",
        PluginIsolationTier::InProcess,
    )
    .expect("in-process AU construction should load AUDelay via registry key");
}

#[test]
fn prepare_dedicated_sandbox_attaches_from_real_broker_lease() {
    if !signal_plugin_clap::fixture::rustc_available() {
        eprintln!("skipping: rustc unavailable for the CLAP fixture");
        return;
    }
    let _guard = SandboxBrokerEnvGuard::enable_for_workspace_cargo_run();
    let directory = unique_fixture_dir("clap-sandbox");
    let plugin_type_id = "com.signal.host-factory-clap-sandbox";
    signal_plugin_clap::fixture::compile_clap_fixture(
        &directory.path,
        plugin_type_id,
        "Signal Host Factory CLAP Sandbox",
        0,
    )
    .expect("clap fixture should compile");
    let mut host = booted_host();
    scan_root(&mut host, &directory.path, PluginFormat::Clap);
    host.prepare_plugin_processor(plugin_type_id, PluginIsolationTier::DedicatedSandbox)
        .expect("DedicatedSandbox should attach from a real broker lease");
}

#[test]
fn prepare_plugin_processor_shared_sandbox_token() {
    let mut host = booted_host();
    let error = host
        .prepare_plugin_processor("unused", PluginIsolationTier::SharedSandbox)
        .expect_err("SharedSandbox is unimplemented");
    assert_eq!(error.kind, RuntimeErrorKind::UnsupportedCapability);
    assert!(error.message.contains("shared_sandbox_unimplemented"));
}
