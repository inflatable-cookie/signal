use std::process::Command;

fn run_tools_descriptor(flag: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_signal-supervisor-tools"))
        .args([flag, "--format=json"])
        .output()
        .expect("descriptor command should run");
    assert!(
        output.status.success(),
        "descriptor command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("descriptor output should be valid utf-8")
}

#[test]
fn public_release_packaging_boundary_is_consumable_without_private_scripts() {
    let packaging_manifest = run_tools_descriptor("--describe-packaging-manifest");
    assert!(packaging_manifest.contains("\"manifest\":\"signal.release.packaging-manifest\""));
    assert!(packaging_manifest
        .contains("\"acceptance_task\":\"effigy acceptance:release-packaging-consumer\""));
    assert!(packaging_manifest.contains(
        "\"path_or_command\":\"cargo run -p signal-supervisor-tools -- --describe-release-boundary --format=json\""
    ));
    assert!(
        packaging_manifest.contains("\"surface\":\"effigy acceptance:release-packaging-consumer\"")
    );
    assert!(packaging_manifest
        .contains("\"signed installers, notarization, and platform distribution packaging\""));

    let release_boundary = run_tools_descriptor("--describe-release-boundary");
    assert!(release_boundary.contains("\"boundary\":\"signal.release.boundary\""));
    assert!(release_boundary.contains("\"id\":\"publication-packaging-manifest\""));
    assert!(release_boundary.contains(
        "\"path_or_command\":\"cargo run -p signal-supervisor-tools -- --describe-packaging-manifest --format=json\""
    ));
}
