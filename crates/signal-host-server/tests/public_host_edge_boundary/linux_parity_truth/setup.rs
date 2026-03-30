#[path = "setup/discovery.rs"]
mod discovery;
#[path = "setup/lifecycle.rs"]
mod lifecycle;

use super::super::*;

pub(super) fn build_server_linux_parity_report() -> signal_runtime::RuntimeSupervisorReport {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-server-linux-plugin-parity".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public server linux parity handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public server linux parity configure should succeed");

    discovery::record_server_linux_parity_discovery(&mut runtime);
    lifecycle::record_server_linux_parity_lifecycle(&mut runtime);

    ServerRuntimeHost::new(runtime).supervisor_report()
}
