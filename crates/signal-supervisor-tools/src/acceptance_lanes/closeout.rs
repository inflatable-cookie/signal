use super::*;

fn generation_closeout_validation_steps() -> &'static [GenerationCloseoutValidationStep] {
    &[
        GenerationCloseoutValidationStep {
            id: "integrated-acceptance-base",
            command: INTEGRATED_LIVE_WORKFLOW_ACCEPTANCE_TASK,
            rationale:
                "The final g08 closeout gate must build on the already-closed integrated live-ownership and workflow lane instead of replacing Linux live, device workflow, immersive, and preview evidence with prose-only summary.",
        },
        GenerationCloseoutValidationStep {
            id: "closeout-descriptor-proof",
            command:
                "cargo test -p signal-supervisor-tools generation_closeout_json_reports_combined_boundary_and_next_queue",
            rationale:
                "The closeout descriptor itself must remain covered as a machine-readable repo-owned surface so the recorded promotion verdict cannot drift away from the runnable gate.",
        },
        GenerationCloseoutValidationStep {
            id: "generation-closeout-description",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-generation-closeout --format=json",
            rationale:
                "Consumers and maintainers need one machine-readable g08 closeout record tying together grouped acceptance, required validation, and the final bounded downstream workflow readiness verdict.",
        },
        GenerationCloseoutValidationStep {
            id: "repo-validation",
            command: "effigy validate",
            rationale:
                "The closeout gate still requires the repo-owned configure/build/test chain to stay green.",
        },
    ]
}

fn generation_closeout_residual_risks() -> &'static [&'static str] {
    &[
        "broader repeated-run and environment-specific acceptance depth remain outside the bounded g08 closeout fast path and are now explicit post-g08 backlog work instead of implied follow-up",
        "product-local controller, browser, immersive, certification, and downstream launch workflows remain deferred instead of becoming implicit g08 closeout requirements",
        "post-g08 work should open from the recorded backlog item only when maintainers choose to promote stronger rerun depth or broader shared environment matrices into a new active queue",
    ]
}

fn generation_closeout_next_queue_summary() -> &'static str {
    "g08 is closed. The next likely queue is recorded as explicit post-g08 backlog work for repeated-run confidence, environment matrices, and stronger shared downstream workflow depth rather than a new active generation."
}

fn generation_closeout_readiness_areas() -> &'static [GenerationReadinessArea] {
    &[
        GenerationReadinessArea {
            id: "linux-live-and-guarded-ownership-surface",
            status: "sufficient-for-closeout",
            rationale:
                "Closed Linux live ownership, JACK coordination, PipeWire and ALSA parity, and bounded integrated acceptance now give Signal one reusable live backend substrate strong enough to close g08 without widening into distro-specific or daemon-specific certification matrices.",
        },
        GenerationReadinessArea {
            id: "immersive-render-and-monitoring-surface",
            status: "sufficient-for-closeout",
            rationale:
                "Closed immersive room-policy, deployment-monitoring, renderer-export, and grouped immersive acceptance now give Signal bounded immersive evidence strong enough to close g08 without expanding into renderer-vendor or product-console depth.",
        },
        GenerationReadinessArea {
            id: "device-protocol-and-workflow-surface",
            status: "sufficient-for-closeout",
            rationale:
                "Closed external MIDI, controller-expression, control-surface, advanced hardware, and grouped device workflow acceptance now give Signal bounded device-workflow evidence strong enough to close g08 without promoting vendor-private control workflows into shared runtime policy.",
        },
        GenerationReadinessArea {
            id: "preview-and-workflow-service-surface",
            status: "sufficient-for-closeout",
            rationale:
                "Closed preview-device policy, preview-workflow posture, transform persistence, and grouped control-preview workflow acceptance now give Signal bounded preview-workflow evidence strong enough to close g08 without promoting browser-local queue UX or downstream launch workflow policy.",
        },
    ]
}

pub(crate) fn render_generation_closeout_text() -> String {
    let mut rendered = format!(
        "generation_closeout: {GENERATION_CLOSEOUT}\ngeneration: {GENERATION_CLOSEOUT_GENERATION}\ncontract_path: {GENERATION_CLOSEOUT_CONTRACT_PATH}\nroadmap_path: {GENERATION_CLOSEOUT_ROADMAP_PATH}\ncloseout_task: {GENERATION_CLOSEOUT_TASK}\npromotion_decision: {GENERATION_CLOSEOUT_PROMOTION_DECISION}\ncloseout_gate_status: {GENERATION_CLOSEOUT_GATE_STATUS}\ng08_integrated_acceptance_lane_command: {G08_INTEGRATED_ACCEPTANCE_LANE_COMMAND}\nnext_queue_path: {GENERATION_CLOSEOUT_NEXT_QUEUE_PATH}\nnext_queue_status: {GENERATION_CLOSEOUT_NEXT_QUEUE_STATUS}\nvalidation_steps:\n"
    );
    for step in generation_closeout_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("downstream_workflow_readiness_areas:\n");
    for area in generation_closeout_readiness_areas() {
        rendered.push_str(&format!(
            "- id: {}\n  status: {}\n  rationale: {}\n",
            area.id, area.status, area.rationale,
        ));
    }
    rendered.push_str("residual_risks:\n");
    for risk in generation_closeout_residual_risks() {
        rendered.push_str(&format!("- {risk}\n"));
    }
    rendered.push_str(&format!(
        "next_queue_summary: {}\n",
        generation_closeout_next_queue_summary()
    ));
    rendered
}

pub(crate) fn render_generation_closeout_json() -> String {
    let validation_steps = generation_closeout_validation_steps()
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
    let readiness_areas = generation_closeout_readiness_areas()
        .iter()
        .map(|area| {
            format!(
                concat!(
                    "{{",
                    "\"id\":{},",
                    "\"status\":{},",
                    "\"rationale\":{}",
                    "}}"
                ),
                json_string(area.id),
                json_string(area.status),
                json_string(area.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let residual_risks = generation_closeout_residual_risks()
        .iter()
        .map(|risk| json_string(risk))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{",
            "\"closeout\":{},",
            "\"generation\":{},",
            "\"contract_path\":{},",
            "\"roadmap_path\":{},",
            "\"closeout_task\":{},",
            "\"promotion_decision\":{},",
            "\"closeout_gate_status\":{},",
            "\"g08_integrated_acceptance_lane_command\":{},",
            "\"next_queue_path\":{},",
            "\"next_queue_status\":{},",
            "\"validation_steps\":[{}],",
            "\"downstream_workflow_readiness_areas\":[{}],",
            "\"residual_risks\":[{}],",
            "\"next_queue_summary\":{}",
            "}}"
        ),
        json_string(GENERATION_CLOSEOUT),
        json_string(GENERATION_CLOSEOUT_GENERATION),
        json_string(GENERATION_CLOSEOUT_CONTRACT_PATH),
        json_string(GENERATION_CLOSEOUT_ROADMAP_PATH),
        json_string(GENERATION_CLOSEOUT_TASK),
        json_string(GENERATION_CLOSEOUT_PROMOTION_DECISION),
        json_string(GENERATION_CLOSEOUT_GATE_STATUS),
        json_string(G08_INTEGRATED_ACCEPTANCE_LANE_COMMAND),
        json_string(GENERATION_CLOSEOUT_NEXT_QUEUE_PATH),
        json_string(GENERATION_CLOSEOUT_NEXT_QUEUE_STATUS),
        validation_steps,
        readiness_areas,
        residual_risks,
        json_string(generation_closeout_next_queue_summary()),
    )
}
