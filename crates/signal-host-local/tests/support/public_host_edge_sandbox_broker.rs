#![allow(dead_code)]

use std::{
    ffi::OsString,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Mutex, MutexGuard, OnceLock},
};

pub(crate) fn run_public_sandbox_broker(commands: &str) -> std::process::Output {
    let _guard = sandbox_broker_env_lock();
    let old_demo_format = std::env::var_os("SIGNAL_HOST_DEMO_PLUGIN_FORMAT");
    let old_demo_root = std::env::var_os("SIGNAL_HOST_DEMO_PLUGIN_ROOT");
    let old_demo_plugin_type_id = std::env::var_os("SIGNAL_HOST_DEMO_PLUGIN_TYPE_ID");
    restore_env("SIGNAL_HOST_DEMO_PLUGIN_FORMAT", None);
    restore_env("SIGNAL_HOST_DEMO_PLUGIN_ROOT", None);
    restore_env("SIGNAL_HOST_DEMO_PLUGIN_TYPE_ID", None);
    let mut child = Command::new("cargo")
        .current_dir(workspace_root())
        .args(["run", "-q", "-p", "signal-plugin-sandbox"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("sandbox broker should spawn");

    child
        .stdin
        .as_mut()
        .expect("sandbox broker stdin should be piped")
        .write_all(commands.as_bytes())
        .expect("sandbox broker commands should be written");

    let output = child
        .wait_with_output()
        .expect("sandbox broker should exit");
    restore_env("SIGNAL_HOST_DEMO_PLUGIN_FORMAT", old_demo_format.as_ref());
    restore_env("SIGNAL_HOST_DEMO_PLUGIN_ROOT", old_demo_root.as_ref());
    restore_env(
        "SIGNAL_HOST_DEMO_PLUGIN_TYPE_ID",
        old_demo_plugin_type_id.as_ref(),
    );
    output
}

pub(crate) struct SandboxBrokerEnvGuard {
    old_command: Option<OsString>,
    old_args: Option<OsString>,
    old_workdir: Option<OsString>,
    old_demo_format: Option<OsString>,
    old_demo_root: Option<OsString>,
    old_demo_plugin_type_id: Option<OsString>,
    _guard: MutexGuard<'static, ()>,
}

impl SandboxBrokerEnvGuard {
    pub(crate) fn enable_for_workspace_cargo_run() -> Self {
        let guard = sandbox_broker_env_lock();
        let old_command = std::env::var_os("SIGNAL_PLUGIN_SANDBOX_BROKER_COMMAND");
        let old_args = std::env::var_os("SIGNAL_PLUGIN_SANDBOX_BROKER_ARGS");
        let old_workdir = std::env::var_os("SIGNAL_PLUGIN_SANDBOX_BROKER_WORKDIR");
        let old_demo_format = std::env::var_os("SIGNAL_HOST_DEMO_PLUGIN_FORMAT");
        let old_demo_root = std::env::var_os("SIGNAL_HOST_DEMO_PLUGIN_ROOT");
        let old_demo_plugin_type_id = std::env::var_os("SIGNAL_HOST_DEMO_PLUGIN_TYPE_ID");

        unsafe {
            std::env::set_var("SIGNAL_PLUGIN_SANDBOX_BROKER_COMMAND", "cargo");
            std::env::set_var(
                "SIGNAL_PLUGIN_SANDBOX_BROKER_ARGS",
                "run -q -p signal-plugin-sandbox",
            );
            std::env::set_var(
                "SIGNAL_PLUGIN_SANDBOX_BROKER_WORKDIR",
                workspace_root().as_os_str(),
            );
        }

        Self {
            old_command,
            old_args,
            old_workdir,
            old_demo_format,
            old_demo_root,
            old_demo_plugin_type_id,
            _guard: guard,
        }
    }

    pub(crate) fn enable_for_workspace_demo_plugin(
        format: &str,
        scan_root: &str,
        plugin_type_id: &str,
    ) -> Self {
        let guard = sandbox_broker_env_lock();
        let old_command = std::env::var_os("SIGNAL_PLUGIN_SANDBOX_BROKER_COMMAND");
        let old_args = std::env::var_os("SIGNAL_PLUGIN_SANDBOX_BROKER_ARGS");
        let old_workdir = std::env::var_os("SIGNAL_PLUGIN_SANDBOX_BROKER_WORKDIR");
        let old_demo_format = std::env::var_os("SIGNAL_HOST_DEMO_PLUGIN_FORMAT");
        let old_demo_root = std::env::var_os("SIGNAL_HOST_DEMO_PLUGIN_ROOT");
        let old_demo_plugin_type_id = std::env::var_os("SIGNAL_HOST_DEMO_PLUGIN_TYPE_ID");

        unsafe {
            std::env::set_var("SIGNAL_PLUGIN_SANDBOX_BROKER_COMMAND", "cargo");
            std::env::set_var(
                "SIGNAL_PLUGIN_SANDBOX_BROKER_ARGS",
                "run -q -p signal-plugin-sandbox",
            );
            std::env::set_var(
                "SIGNAL_PLUGIN_SANDBOX_BROKER_WORKDIR",
                workspace_root().as_os_str(),
            );
            std::env::set_var("SIGNAL_HOST_DEMO_PLUGIN_FORMAT", format);
            std::env::set_var("SIGNAL_HOST_DEMO_PLUGIN_ROOT", scan_root);
            std::env::set_var("SIGNAL_HOST_DEMO_PLUGIN_TYPE_ID", plugin_type_id);
        }

        Self {
            old_command,
            old_args,
            old_workdir,
            old_demo_format,
            old_demo_root,
            old_demo_plugin_type_id,
            _guard: guard,
        }
    }
}

impl Drop for SandboxBrokerEnvGuard {
    fn drop(&mut self) {
        restore_env(
            "SIGNAL_PLUGIN_SANDBOX_BROKER_COMMAND",
            self.old_command.as_ref(),
        );
        restore_env("SIGNAL_PLUGIN_SANDBOX_BROKER_ARGS", self.old_args.as_ref());
        restore_env(
            "SIGNAL_PLUGIN_SANDBOX_BROKER_WORKDIR",
            self.old_workdir.as_ref(),
        );
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

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root should resolve")
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

fn sandbox_broker_env_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
