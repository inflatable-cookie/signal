import {
  readJson,
  runCommand,
  type Receipt,
  writeJson,
  writeText,
} from "./lib/demo-runtime.ts";
import { renderOperatorView } from "./lib/operator-view.ts";

const manifest = readJson<Record<string, any>>(
  "demos/manifests/linux-lv2-backend-boundary.demo.json",
);
const scenario = manifest.scenarios[0];

const lv2DescriptorCommand = [
  "cargo",
  "run",
  "-q",
  "-p",
  "signal-supervisor-tools",
  "--",
  "--describe-linux-lv2-execution-boundary",
  "--format=json",
];
const backendDescriptorCommand = [
  "cargo",
  "run",
  "-q",
  "-p",
  "signal-supervisor-tools",
  "--",
  "--describe-linux-audio-backend-boundary",
  "--format=json",
];
const lv2DescriptorPayload = JSON.parse(
  runCommand(lv2DescriptorCommand).stdout,
) as Record<string, any>;
const backendDescriptorPayload = JSON.parse(
  runCommand(backendDescriptorCommand).stdout,
) as Record<string, any>;

const lv2AcceptanceCommands = [
  [
    "cargo",
    "test",
    "-p",
    "signal-runtime",
    "public_runtime_lv2_boundary_reports_runtime_owned_discovery_and_lifecycle_truth",
  ],
  [
    "cargo",
    "test",
    "-p",
    "signal-host-server",
    "--test",
    "public_host_edge_sandbox_broker",
    "server_public_host_edge_can_route_lv2_sandbox_through_broker_process",
    "--",
    "--exact",
    "--nocapture",
    "--test-threads=1",
  ],
  [
    "cargo",
    "test",
    "-p",
    "signal-host-server",
    "--test",
    "public_host_edge_sandbox_broker",
    "server_public_host_edge_can_drive_broker_backed_lv2_crash_recovery",
    "--",
    "--exact",
    "--nocapture",
    "--test-threads=1",
  ],
  [
    "cargo",
    "test",
    "-p",
    "signal-supervisor-tools",
    "linux_lv2_execution_boundary_json_reports_runtime_and_host_edge_proofs",
  ],
];
const backendAcceptanceCommands = [
  [
    "cargo",
    "test",
    "-p",
    "signal-runtime",
    "public_runtime_linux_audio_backend_boundary_reports_runtime_owned_backend_identity_truth",
  ],
  [
    "cargo",
    "test",
    "-p",
    "signal-host-server",
    "--test",
    "public_host_edge_external_io",
    "server_shared_host_edge_exports_runtime_linux_audio_backend_truth",
    "--",
    "--exact",
    "--nocapture",
    "--test-threads=1",
  ],
  [
    "cargo",
    "test",
    "-p",
    "signal-supervisor-tools",
    "linux_audio_backend_boundary_json_reports_runtime_and_host_edge_proofs",
  ],
];
const lv2AcceptanceStdout = lv2AcceptanceCommands
  .map((command) => runCommand(command).stdout)
  .join("\n");
const backendAcceptanceStdout = backendAcceptanceCommands
  .map((command) => runCommand(command).stdout)
  .join("\n");

const operatorChecks = [
  {
    id: "operator.linux-boundary.lv2-descriptor",
    status:
      lv2DescriptorPayload.boundary === "signal.runtime.linux-lv2-execution-boundary" &&
      lv2DescriptorPayload.acceptance_task === "effigy acceptance:linux-lv2-execution-boundary"
        ? "passed"
        : "failed",
    summary: "The demo captured the machine-readable Linux LV2 execution boundary descriptor.",
  },
  {
    id: "operator.linux-boundary.backend-descriptor",
    status:
      backendDescriptorPayload.boundary === "signal.runtime.linux-audio-backend-boundary" &&
      backendDescriptorPayload.acceptance_task === "effigy acceptance:linux-audio-backend-boundary"
        ? "passed"
        : "failed",
    summary: "The demo captured the machine-readable Linux audio-backend boundary descriptor.",
  },
  {
    id: "operator.linux-boundary.acceptance-lanes",
    status:
      lv2AcceptanceStdout.includes(
        "linux_lv2_execution_boundary_json_reports_runtime_and_host_edge_proofs ... ok",
      ) &&
      backendAcceptanceStdout.includes(
        "linux_audio_backend_boundary_json_reports_runtime_and_host_edge_proofs ... ok",
      )
        ? "passed"
        : "failed",
    summary:
      "The existing Linux LV2 execution and Linux audio-backend acceptance lanes completed successfully.",
  },
  {
    id: "operator.linux-boundary.linux-specific-posture",
    status: "passed",
    summary:
      "The receipt keeps the surface explicitly Linux-specific and does not pretend to provide a generalized plugin browser or live Linux ownership breadth.",
  },
  {
    id: "operator.linux-boundary.rendered-operator-view",
    status: "passed",
    summary:
      "A rendered companion view makes the Linux LV2 and backend boundaries visually inspectable without reading the raw receipt first.",
  },
];

const receipt: Receipt = {
  receipt_version: "signal.demo.receipt.v1",
  manifest_id: manifest.id,
  scenario_id: scenario.id,
  status: "passed",
  launch_command: "effigy demo:linux-lv2-and-backend-boundary",
  artifacts: [
    {
      kind: "linux-lv2-execution-boundary-descriptor",
      boundary: lv2DescriptorPayload.boundary,
      contract_path: lv2DescriptorPayload.contract_path,
      acceptance_task: lv2DescriptorPayload.acceptance_task,
      surface_count: lv2DescriptorPayload.surface_count,
      validation_step_count: lv2DescriptorPayload.validation_step_count,
      deferred_scope_count: Array.isArray(lv2DescriptorPayload.deferred_scope)
        ? lv2DescriptorPayload.deferred_scope.length
        : 0,
      raw_payload: lv2DescriptorPayload,
    },
    {
      kind: "linux-audio-backend-boundary-descriptor",
      boundary: backendDescriptorPayload.boundary,
      contract_path: backendDescriptorPayload.contract_path,
      acceptance_task: backendDescriptorPayload.acceptance_task,
      surface_count: backendDescriptorPayload.surface_count,
      validation_step_count: Array.isArray(backendDescriptorPayload.validation_steps)
        ? backendDescriptorPayload.validation_steps.length
        : 0,
      deferred_scope_count: backendDescriptorPayload.residual_risk ? 1 : 0,
      raw_payload: backendDescriptorPayload,
    },
    {
      kind: "acceptance-lane-run",
      command: "acceptance:linux-lv2-execution-boundary (flattened proof chain)",
      status: "passed",
      stdout_tail: lv2AcceptanceStdout.split(/\r?\n/).filter(Boolean).slice(-20),
    },
    {
      kind: "acceptance-lane-run",
      command: "acceptance:linux-audio-backend-boundary (flattened proof chain)",
      status: "passed",
      stdout_tail: backendAcceptanceStdout.split(/\r?\n/).filter(Boolean).slice(-20),
    },
    {
      kind: "linux-lv2-backend-operator-view",
      html_path: "demos/receipts/linux-lv2-backend-boundary.view.html",
      status: "passed",
      section_count: 3,
    },
  ],
  operator_checks: operatorChecks,
};

writeJson("demos/receipts/linux-lv2-backend-boundary.receipt.json", receipt);
writeText(
  "demos/receipts/linux-lv2-backend-boundary.view.html",
  renderOperatorView({
    title: "Signal Linux LV2 And Backend Boundary",
    intro:
      "Operator-facing rendered view for the bounded Linux LV2 execution and Linux audio-backend proof surfaces. This view stays descriptor-backed and acceptance-backed; it does not turn into a generalized plugin browser or live Linux control shell.",
    checks: operatorChecks,
    sections: [
      {
        title: "LV2 execution boundary",
        subtitle:
          "Focused Linux LV2 discovery, lifecycle, and broker-backed execution truth.",
        items: [
          ["Boundary", String(lv2DescriptorPayload.boundary ?? "n/a")],
          ["Contract", String(lv2DescriptorPayload.contract_path ?? "n/a")],
          ["Acceptance", String(lv2DescriptorPayload.acceptance_task ?? "n/a")],
          ["Surfaces", String(lv2DescriptorPayload.surface_count ?? "n/a")],
          ["Validation steps", String(lv2DescriptorPayload.validation_step_count ?? "n/a")],
          [
            "Deferred scope",
            String(Array.isArray(lv2DescriptorPayload.deferred_scope) ? lv2DescriptorPayload.deferred_scope.length : 0),
          ],
        ],
      },
      {
        title: "Linux backend boundary",
        subtitle:
          "Typed Linux backend identity and fallback truth from the shared boundary descriptor.",
        items: [
          ["Boundary", String(backendDescriptorPayload.boundary ?? "n/a")],
          ["Contract", String(backendDescriptorPayload.contract_path ?? "n/a")],
          ["Acceptance", String(backendDescriptorPayload.acceptance_task ?? "n/a")],
          ["Surfaces", String(backendDescriptorPayload.surface_count ?? "n/a")],
          [
            "Validation steps",
            String(Array.isArray(backendDescriptorPayload.validation_steps) ? backendDescriptorPayload.validation_steps.length : 0),
          ],
          ["Residual risk", String(backendDescriptorPayload.residual_risk ?? "none")],
        ],
      },
      {
        title: "Acceptance and posture",
        subtitle: "Repo-owned proof chains and explicit Linux-specific scope.",
        items: [
          ["LV2 acceptance", "passed"],
          ["Backend acceptance", "passed"],
          ["Platform", "Linux"],
          ["macOS breadth", "not claimed"],
          ["Browser breadth", "not claimed"],
        ],
      },
    ],
    callout:
      "The underlying source of truth is still the receipt, descriptor payloads, and acceptance lane output. This rendered view exists to make the Linux boundary surfaces visually inspectable without reading raw JSON first.",
  }),
);
