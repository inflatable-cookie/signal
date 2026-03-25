use super::describe_flags::DESCRIBE_FLAG_SPECS;

pub(crate) fn print_usage() {
    let describe_flags = DESCRIBE_FLAG_SPECS
        .iter()
        .map(|spec| spec.flag)
        .collect::<Vec<_>>()
        .join("|");
    eprintln!(
        "usage: signal-supervisor-tools [--format text|json] [--include-payload] [{describe_flags}] <local|server> <default|timeout|crash|heartbeat|soak|mixed>"
    );
}
