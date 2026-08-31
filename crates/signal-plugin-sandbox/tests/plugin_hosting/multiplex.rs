use super::*;

fn attach_processor(lease: &signal_runtime::SandboxPluginAudioLease) -> Arc<ShmPluginProcessor> {
    Arc::new(
        ShmPluginProcessor::attach(
            &lease.region_id,
            &lease.shm_path,
            lease.shm_bytes,
            lease.max_frames,
            lease.channels,
            SAMPLE_RATE_HZ,
        )
        .expect("parent should attach the member audio block region"),
    )
}

fn require_activated(
    outcome: SandboxPluginActivateOutcome,
) -> signal_runtime::SandboxPluginAudioLease {
    match outcome {
        SandboxPluginActivateOutcome::Activated(lease) => lease,
        SandboxPluginActivateOutcome::LayoutUnsupported { detail } => {
            panic!("stereo fixture rejected: {detail}")
        }
    }
}

#[test]
fn two_instances_of_the_same_type_process_through_one_child() {
    let _slot = sandbox_child_slot();
    if !rustc_available() {
        eprintln!("skipping: rustc unavailable for the CLAP fixture");
        return;
    }
    let directory = unique_fixture_dir();
    let library = compile_clap_fixture(
        &directory.path,
        FIXTURE_PLUGIN_ID,
        "Signal Sandbox Hosting Fixture",
        0,
    )
    .expect("fixture should compile");
    let library_path = library.display().to_string();

    let mut client = SandboxBrokerClientSession::spawn_command(
        env!("CARGO_BIN_EXE_signal-plugin-sandbox"),
        &[],
        &SandboxBrokerSpawnConfig::default(),
    )
    .expect("broker child should spawn");
    client
        .read_startup_receipts()
        .expect("startup receipts should arrive");

    let first = client
        .load_plugin_instance("member-a", &library_path, FIXTURE_PLUGIN_ID)
        .expect("first instance should load");
    assert_eq!(first.parameters.len(), 2);
    let duplicate = client
        .load_plugin_instance("member-a", &library_path, FIXTURE_PLUGIN_ID)
        .expect_err("duplicate instance_id must fail");
    assert!(
        format!("{duplicate:?}").contains("plugin_already_loaded"),
        "typed token expected: {duplicate:?}",
    );
    client
        .load_plugin_instance("member-b", &library_path, FIXTURE_PLUGIN_ID)
        .expect("second instance should load");

    let lease_a = require_activated(
        client
            .activate_plugin_instance("member-a", SAMPLE_RATE_HZ, 1, MAX_FRAMES)
            .expect("activate member-a"),
    );
    let lease_b = require_activated(
        client
            .activate_plugin_instance("member-b", SAMPLE_RATE_HZ, 1, MAX_FRAMES)
            .expect("activate member-b"),
    );
    assert_ne!(lease_a.lease_id, lease_b.lease_id);
    assert_ne!(lease_a.region_id, lease_b.region_id);
    assert_ne!(lease_a.shm_path, lease_b.shm_path);

    client
        .start_processing()
        .expect("one child audio thread should start");

    let processor_a = attach_processor(&lease_a);
    let processor_b = attach_processor(&lease_b);
    let handle_a = RenderPluginProcessor::new(Arc::clone(&processor_a) as Arc<_>);
    let handle_b = RenderPluginProcessor::new(Arc::clone(&processor_b) as Arc<_>);

    let frames = 128usize;
    let reference: Vec<f32> = (0..frames * 2).map(|index| index as f32 / 512.0).collect();
    let mut scratch_a = reference.clone();
    let mut scratch_b = reference.clone();
    process_offline(&handle_a, &mut scratch_a, frames, 2);
    process_offline(&handle_b, &mut scratch_b, frames, 2);
    for (index, (output, input)) in scratch_a.iter().zip(reference.iter()).enumerate() {
        assert!(
            (output - input * CLAP_FIXTURE_GAIN).abs() < 1e-7,
            "member-a sample {index}: {output} vs {input} * {CLAP_FIXTURE_GAIN}",
        );
    }
    for (index, (output, input)) in scratch_b.iter().zip(reference.iter()).enumerate() {
        assert!(
            (output - input * CLAP_FIXTURE_GAIN).abs() < 1e-7,
            "member-b sample {index}: {output} vs {input} * {CLAP_FIXTURE_GAIN}",
        );
    }

    assert!(client.is_alive(), "shared child should still be alive");
    client.stop_processing().expect("stop-processing");
    client
        .deactivate_plugin_instance("member-a")
        .expect("deactivate member-a");
    client
        .deactivate_plugin_instance("member-b")
        .expect("deactivate member-b");
    client
        .unload_plugin_instance("member-a")
        .expect("unload member-a");
    client
        .unload_plugin_instance("member-b")
        .expect("unload member-b");
    client.shutdown().expect("shutdown");
}

#[test]
fn load_while_processing_rejects_until_boundary_stop() {
    let _slot = sandbox_child_slot();
    if !rustc_available() {
        eprintln!("skipping: rustc unavailable for the CLAP fixture");
        return;
    }
    let directory = unique_fixture_dir();
    let library = compile_clap_fixture(
        &directory.path,
        FIXTURE_PLUGIN_ID,
        "Signal Sandbox Hosting Fixture",
        0,
    )
    .expect("fixture should compile");
    let library_path = library.display().to_string();

    let mut client = SandboxBrokerClientSession::spawn_command(
        env!("CARGO_BIN_EXE_signal-plugin-sandbox"),
        &[],
        &SandboxBrokerSpawnConfig::default(),
    )
    .expect("broker child should spawn");
    client
        .read_startup_receipts()
        .expect("startup receipts should arrive");

    client
        .load_plugin_instance("member-a", &library_path, FIXTURE_PLUGIN_ID)
        .expect("first instance should load");
    let _lease_a = require_activated(
        client
            .activate_plugin_instance("member-a", SAMPLE_RATE_HZ, 1, MAX_FRAMES)
            .expect("activate member-a"),
    );
    client
        .start_processing()
        .expect("child audio thread should start");

    let refused = client
        .load_plugin_instance("member-b", &library_path, FIXTURE_PLUGIN_ID)
        .expect_err("load while processing must fail");
    assert!(
        format!("{refused:?}").contains("already_processing"),
        "typed token expected: {refused:?}",
    );

    client
        .stop_processing()
        .expect("stop-processing unlocks lifecycle mutation");
    client
        .load_plugin_instance("member-b", &library_path, FIXTURE_PLUGIN_ID)
        .expect("second instance should load after stop");
    let _lease_b = require_activated(
        client
            .activate_plugin_instance("member-b", SAMPLE_RATE_HZ, 1, MAX_FRAMES)
            .expect("activate member-b"),
    );
    client
        .start_processing()
        .expect("boundary may start again after sequential add");
    assert!(client.is_alive(), "shared child should still be alive");
    client.stop_processing().expect("stop-processing");
    client
        .deactivate_plugin_instance("member-a")
        .expect("deactivate member-a");
    client
        .deactivate_plugin_instance("member-b")
        .expect("deactivate member-b");
    client
        .unload_plugin_instance("member-a")
        .expect("unload member-a");
    client
        .unload_plugin_instance("member-b")
        .expect("unload member-b");
    client.shutdown().expect("shutdown");
}

#[test]
fn default_slot_rejects_a_second_load_plugin() {
    let _slot = sandbox_child_slot();
    if !rustc_available() {
        eprintln!("skipping: rustc unavailable for the CLAP fixture");
        return;
    }
    let directory = unique_fixture_dir();
    let library = compile_clap_fixture(
        &directory.path,
        FIXTURE_PLUGIN_ID,
        "Signal Sandbox Hosting Fixture",
        0,
    )
    .expect("fixture should compile");
    let library_path = library.display().to_string();

    let mut client = SandboxBrokerClientSession::spawn_command(
        env!("CARGO_BIN_EXE_signal-plugin-sandbox"),
        &[],
        &SandboxBrokerSpawnConfig::default(),
    )
    .expect("broker child should spawn");
    client
        .read_startup_receipts()
        .expect("startup receipts should arrive");
    client
        .load_plugin(&library_path, FIXTURE_PLUGIN_ID)
        .expect("default slot should load");
    let error = client
        .load_plugin(&library_path, FIXTURE_PLUGIN_ID)
        .expect_err("second default-slot load-plugin must fail");
    assert!(
        format!("{error:?}").contains("plugin_already_loaded"),
        "typed token expected: {error:?}",
    );
    client.unload_plugin().expect("unload-plugin");
    client.shutdown().expect("shutdown");
}
