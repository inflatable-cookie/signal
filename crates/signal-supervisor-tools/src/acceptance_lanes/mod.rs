use super::*;

mod closeout;
mod grouped;
mod integrated_acceptance;
mod soak;
mod workflow;

pub(crate) use closeout::{render_generation_closeout_json, render_generation_closeout_text};
pub(crate) use grouped::{
    render_device_workflow_acceptance_lane_json, render_device_workflow_acceptance_lane_text,
    render_g07_acceptance_lane_json, render_g07_acceptance_lane_text,
    render_linux_live_acceptance_lane_json, render_linux_live_acceptance_lane_text,
};
pub(crate) use integrated_acceptance::{
    render_integrated_acceptance_lane_json, render_integrated_acceptance_lane_text,
};
pub(crate) use soak::{render_g06_soak_lane_json, render_g06_soak_lane_text};
pub(crate) use workflow::{
    render_control_preview_workflow_acceptance_lane_json,
    render_control_preview_workflow_acceptance_lane_text, render_immersive_acceptance_lane_json,
    render_immersive_acceptance_lane_text, render_integrated_live_workflow_acceptance_lane_json,
    render_integrated_live_workflow_acceptance_lane_text,
};

fn render_acceptance_lane_text(
    lane_label: &str,
    lane: &str,
    contract_path: &str,
    acceptance_task: &str,
    required_tasks: &[&str],
    advisory_tasks: &[&str],
    families: &[IntegratedAcceptanceFamily],
    validation_steps: &[IntegratedAcceptanceValidationStep],
    deferred_scope: &[&str],
) -> String {
    let mut rendered = format!(
        "{lane_label}: {lane}\ncontract_path: {contract_path}\nacceptance_task: {acceptance_task}\nrequired_tasks:\n"
    );
    for task in required_tasks {
        rendered.push_str(&format!("- {task}\n"));
    }
    rendered.push_str("advisory_tasks:\n");
    for task in advisory_tasks {
        rendered.push_str(&format!("- {task}\n"));
    }
    rendered.push_str("families:\n");
    for family in families {
        rendered.push_str(&format!(
            "- id: {}\n  title: {}\n  rationale: {}\n  required_tasks:\n",
            family.id, family.title, family.rationale
        ));
        for task in family.required_tasks {
            rendered.push_str(&format!("    - {task}\n"));
        }
        rendered.push_str("  advisory_tasks:\n");
        for task in family.advisory_tasks {
            rendered.push_str(&format!("    - {task}\n"));
        }
    }
    rendered.push_str("validation_steps:\n");
    for step in validation_steps {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in deferred_scope {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

fn render_acceptance_lane_json(
    lane: &str,
    contract_path: &str,
    acceptance_task: &str,
    required_tasks: &[&str],
    advisory_tasks: &[&str],
    families: &[IntegratedAcceptanceFamily],
    validation_steps: &[IntegratedAcceptanceValidationStep],
    deferred_scope: &[&str],
) -> String {
    let required_task_count = required_tasks.len();
    let advisory_task_count = advisory_tasks.len();
    let family_count = families.len();
    let validation_step_count = validation_steps.len();
    let required_tasks = required_tasks
        .iter()
        .map(|task| json_string(task))
        .collect::<Vec<_>>()
        .join(",");
    let advisory_tasks = advisory_tasks
        .iter()
        .map(|task| json_string(task))
        .collect::<Vec<_>>()
        .join(",");
    let families = families
        .iter()
        .map(|family| {
            let required = family
                .required_tasks
                .iter()
                .map(|task| json_string(task))
                .collect::<Vec<_>>()
                .join(",");
            let advisory = family
                .advisory_tasks
                .iter()
                .map(|task| json_string(task))
                .collect::<Vec<_>>()
                .join(",");
            format!(
                concat!(
                    "{{",
                    "\"id\":{},",
                    "\"title\":{},",
                    "\"rationale\":{},",
                    "\"required_tasks\":[{}],",
                    "\"advisory_tasks\":[{}]",
                    "}}"
                ),
                json_string(family.id),
                json_string(family.title),
                json_string(family.rationale),
                required,
                advisory,
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let validation_steps = validation_steps
        .iter()
        .map(|step| {
            format!(
                concat!(
                    "{{",
                    "\"id\":{},",
                    "\"command\":{},",
                    "\"rationale\":{}",
                    "}}"
                ),
                json_string(step.id),
                json_string(step.command),
                json_string(step.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let deferred_scope = deferred_scope
        .iter()
        .map(|scope| json_string(scope))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{",
            "\"lane\":{},",
            "\"contract_path\":{},",
            "\"acceptance_task\":{},",
            "\"required_task_count\":{},",
            "\"required_tasks\":[{}],",
            "\"advisory_task_count\":{},",
            "\"advisory_tasks\":[{}],",
            "\"family_count\":{},",
            "\"families\":[{}],",
            "\"validation_step_count\":{},",
            "\"validation_steps\":[{}],",
            "\"deferred_scope\":[{}]",
            "}}"
        ),
        json_string(lane),
        json_string(contract_path),
        json_string(acceptance_task),
        required_task_count,
        required_tasks,
        advisory_task_count,
        advisory_tasks,
        family_count,
        families,
        validation_step_count,
        validation_steps,
        deferred_scope,
    )
}
