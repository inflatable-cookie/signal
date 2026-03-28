mod device_workflow;
mod g07;
mod linux_live;

pub(crate) use device_workflow::{
    render_device_workflow_acceptance_lane_json, render_device_workflow_acceptance_lane_text,
};
pub(crate) use g07::{render_g07_acceptance_lane_json, render_g07_acceptance_lane_text};
pub(crate) use linux_live::{
    render_linux_live_acceptance_lane_json, render_linux_live_acceptance_lane_text,
};
