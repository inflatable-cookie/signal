#[path = "linux_parity_truth/assertions.rs"]
mod assertions;
#[path = "linux_parity_truth/setup.rs"]
mod setup;

use super::*;

#[test]
fn server_shared_host_edge_exports_runtime_linux_plugin_parity_truth() {
    let report = setup::build_server_linux_parity_report();
    assertions::assert_server_linux_parity_report(&report);
}
