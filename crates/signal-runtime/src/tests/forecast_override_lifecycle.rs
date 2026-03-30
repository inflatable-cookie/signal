use super::*;

#[test]
fn runtime_reconfigure_preserves_explicit_forecast_profile_request() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    runtime
        .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
            profile: RuntimePreworkForecastProfile::Server,
            target_window_blocks_override: Some(4),
        })
        .expect("set explicit forecast profile");

    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("reconfigure");

    let snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(
        snapshot.prework_forecast_requested_mode,
        RuntimePreworkForecastMode::ExplicitProfile
    );
    assert_eq!(
        snapshot.prework_forecast_mode,
        RuntimePreworkForecastMode::ExplicitProfile
    );
    assert_eq!(
        snapshot.prework_forecast_profile,
        Some(RuntimePreworkForecastProfile::Server)
    );
    assert_eq!(
        snapshot.prework_forecast_profile_target_window_override,
        Some(4)
    );
    assert_eq!(
        snapshot.prework_forecast_policy_target_window_blocks,
        Some(4)
    );
}

#[test]
fn runtime_restores_requested_explicit_forecast_mode_after_anticipative_reenable() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    runtime
        .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
            profile: RuntimePreworkForecastProfile::Server,
            target_window_blocks_override: Some(3),
        })
        .expect("set explicit forecast profile");

    let mut disabled_request = RuntimeConfigRequest::new(48_000, 256);
    disabled_request.anticipative_enabled = false;
    runtime
        .configure(disabled_request)
        .expect("disable anticipative");

    let disabled_snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(
        disabled_snapshot.prework_forecast_requested_mode,
        RuntimePreworkForecastMode::ExplicitProfile
    );
    assert_eq!(
        disabled_snapshot.prework_forecast_mode,
        RuntimePreworkForecastMode::Disabled
    );

    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("reenable anticipative");

    let restored_snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(
        restored_snapshot.prework_forecast_requested_mode,
        RuntimePreworkForecastMode::ExplicitProfile
    );
    assert_eq!(
        restored_snapshot.prework_forecast_mode,
        RuntimePreworkForecastMode::ExplicitProfile
    );
    assert_eq!(
        restored_snapshot.prework_forecast_profile,
        Some(RuntimePreworkForecastProfile::Server)
    );
    assert_eq!(
        restored_snapshot.prework_forecast_profile_target_window_override,
        Some(3)
    );
}

#[test]
fn runtime_restart_preserves_raw_forecast_override_request() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    runtime.start().expect("start runtime");

    runtime
        .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
            target_window_blocks: 5,
            prepare_budget_per_cycle: 2,
            buffer_seed_offset: 11,
            transport_playing: true,
            transport_tempo_bpm: 130.0,
            transport_loop_length_blocks: 12,
            parameter_target: "engine.test.raw".into(),
            parameter_cycle_length: 9,
        })
        .expect("set raw forecast policy");

    runtime
        .restart(RestartRequest { reconfigure: None })
        .expect("restart without reconfigure");

    let snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(
        snapshot.prework_forecast_requested_mode,
        RuntimePreworkForecastMode::RawPolicyOverride
    );
    assert_eq!(
        snapshot.prework_forecast_mode,
        RuntimePreworkForecastMode::RawPolicyOverride
    );
    assert_eq!(
        snapshot.prework_forecast_policy_target_window_blocks,
        Some(5)
    );
    assert_eq!(
        snapshot.prework_forecast_profile_source,
        Some(RuntimePreworkForecastProfileSource::RawPolicyOverride)
    );
}
