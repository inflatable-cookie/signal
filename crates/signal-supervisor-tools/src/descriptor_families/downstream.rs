mod automation;
mod fail_gates;

pub(crate) use automation::{render_downstream_automation_json, render_downstream_automation_text};
pub(crate) use fail_gates::{render_downstream_fail_gates_json, render_downstream_fail_gates_text};
