#![allow(dead_code)]

use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::{Mutex, MutexGuard, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

pub(crate) struct TempPluginScanRoot {
    path: PathBuf,
}

impl TempPluginScanRoot {
    pub(crate) fn root(&self) -> String {
        self.path.display().to_string()
    }
}

impl Drop for TempPluginScanRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

pub(crate) struct DemoPluginEnvGuard {
    old_demo_format: Option<OsString>,
    old_demo_root: Option<OsString>,
    old_demo_plugin_type_id: Option<OsString>,
    _guard: MutexGuard<'static, ()>,
}

impl DemoPluginEnvGuard {
    pub(crate) fn enable_au(scan_root: &TempPluginScanRoot, plugin_type_id: &str) -> Self {
        let guard = demo_plugin_env_lock();
        let old_demo_format = std::env::var_os("SIGNAL_HOST_DEMO_PLUGIN_FORMAT");
        let old_demo_root = std::env::var_os("SIGNAL_HOST_DEMO_PLUGIN_ROOT");
        let old_demo_plugin_type_id = std::env::var_os("SIGNAL_HOST_DEMO_PLUGIN_TYPE_ID");

        unsafe {
            std::env::set_var("SIGNAL_HOST_DEMO_PLUGIN_FORMAT", "au");
            std::env::set_var("SIGNAL_HOST_DEMO_PLUGIN_ROOT", scan_root.root());
            std::env::set_var("SIGNAL_HOST_DEMO_PLUGIN_TYPE_ID", plugin_type_id);
        }

        Self {
            old_demo_format,
            old_demo_root,
            old_demo_plugin_type_id,
            _guard: guard,
        }
    }
}

impl Drop for DemoPluginEnvGuard {
    fn drop(&mut self) {
        restore_env(
            "SIGNAL_HOST_DEMO_PLUGIN_FORMAT",
            self.old_demo_format.as_ref(),
        );
        restore_env("SIGNAL_HOST_DEMO_PLUGIN_ROOT", self.old_demo_root.as_ref());
        restore_env(
            "SIGNAL_HOST_DEMO_PLUGIN_TYPE_ID",
            self.old_demo_plugin_type_id.as_ref(),
        );
    }
}

pub(crate) fn temp_public_local_vst3_scan_root() -> TempPluginScanRoot {
    let root = temp_scan_root("vst3");
    write_vst3_bundle(&root, "Signal Instrument.vst3", "plugin:vst3:instrument");
    write_vst3_bundle(
        &root,
        "Signal Multi Output Instrument.vst3",
        "plugin:vst3:multiout-instrument",
    );
    write_vst3_bundle(&root, "Signal Utility.vst3", "plugin:vst3:utility");
    write_vst3_bundle(&root, "Signal Bus FX.vst3", "plugin:vst3:bus-fx");
    TempPluginScanRoot { path: root }
}

pub(crate) fn temp_public_local_clap_scan_root() -> TempPluginScanRoot {
    let root = temp_scan_root("clap");
    write_clap_fixture_library(
        &root,
        "signal-local-default.clap",
        "plugin:clap:default",
        "Signal Default CLAP Plugin",
        1,
    );
    write_clap_fixture_library(
        &root,
        "signal-local-sandbox.clap",
        "plugin:clap:sandbox",
        "Signal Sandbox CLAP Plugin",
        1,
    );
    TempPluginScanRoot { path: root }
}

pub(crate) fn temp_public_local_au_scan_root() -> TempPluginScanRoot {
    let root = temp_scan_root("au");
    write_au_bundle(&root, "Signal Instrument.component", "plugin:au:instrument");
    write_au_bundle(
        &root,
        "Signal Multi Output Instrument.component",
        "plugin:au:multiout-instrument",
    );
    write_au_bundle(&root, "Signal Utility.component", "plugin:au:utility");
    write_au_bundle(&root, "Signal Bus FX.component", "plugin:au:bus-fx");
    TempPluginScanRoot { path: root }
}

pub(crate) fn temp_public_local_faulty_au_scan_root() -> TempPluginScanRoot {
    let root = temp_scan_root("au-fault");
    write_custom_au_bundle(
        &root,
        "Signal Fault.component",
        concat!(
            "plugin_type_id=plugin:au:render-context-fault\n",
            "component_type=aumu\n",
            "component_subtype=sigf\n",
            "manufacturer_code=sigl\n",
            "vendor=Signal\n",
            "name=Signal Fault AU Plugin\n",
            "version=0.1.0\n",
            "audio_inputs=0\n",
            "audio_outputs=2\n",
            "midi_inputs=1\n",
            "midi_outputs=0\n",
            "features=Instrument,Analyzer\n",
            "render_context_failure=unsupported_sample_rate\n"
        ),
    );
    TempPluginScanRoot { path: root }
}

fn temp_scan_root(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("signal-public-host-local-{label}-scan-{unique}"));
    fs::create_dir_all(&root).expect("public local scan root should be created");
    root
}

fn restore_env(key: &str, value: Option<&OsString>) {
    unsafe {
        if let Some(value) = value {
            std::env::set_var(key, value);
        } else {
            std::env::remove_var(key);
        }
    }
}

fn demo_plugin_env_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

mod bundles;
use bundles::{
    write_au_bundle, write_clap_fixture_library, write_custom_au_bundle, write_vst3_bundle,
};
