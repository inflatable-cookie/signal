use super::*;

pub(crate) fn json_runtime_control_snapshot(snapshot: &RuntimeControlSnapshot) -> String {
    let last_stop_reason = snapshot
        .last_stop_reason
        .map(|reason| format!("{reason:?}"));
    let last_reconfigure = snapshot.last_reconfigure.map(|request| {
        format!(
            "sample_rate={} block_size={} anticipative={} realtime_safe={}",
            request.sample_rate.0,
            request.block_size,
            request.anticipative_enabled,
            request.realtime_safe_mode
        )
    });
    format!(
        concat!(
            "{{",
            "\"handshaken\":{},",
            "\"configured\":{},",
            "\"running\":{},",
            "\"handshake_count\":{},",
            "\"configure_count\":{},",
            "\"start_count\":{},",
            "\"stop_count\":{},",
            "\"restart_count\":{},",
            "\"last_client_version\":{},",
            "\"last_stop_reason\":{},",
            "\"last_reconfigure\":{}",
            "}}"
        ),
        snapshot.handshaken,
        snapshot.configured,
        snapshot.running,
        snapshot.handshake_count,
        snapshot.configure_count,
        snapshot.start_count,
        snapshot.stop_count,
        snapshot.restart_count,
        json_option_string(snapshot.last_client_version.as_deref()),
        json_option_string(last_stop_reason.as_deref()),
        json_option_string(last_reconfigure.as_deref()),
    )
}
