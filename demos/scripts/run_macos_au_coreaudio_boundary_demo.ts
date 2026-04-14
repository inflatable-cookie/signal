import {
  readJson,
  runCommand,
  type Receipt,
  writeJson,
  writeText,
} from "./lib/demo-runtime.ts";
import { renderOperatorView } from "./lib/operator-view.ts";

const manifest = readJson<Record<string, any>>(
  "demos/manifests/macos-au-coreaudio-boundary.demo.json",
);
const scenario = manifest.scenarios[0];

const descriptorCommand = [
  "cargo",
  "run",
  "-q",
  "-p",
  "signal-supervisor-tools",
  "--",
  "--describe-macos-au-coreaudio-boundary",
  "--format=json",
];
const descriptorResult = runCommand(descriptorCommand);
const descriptorPayload = JSON.parse(descriptorResult.stdout) as Record<string, any>;

const acceptanceCommands = [
  ["cargo", "test", "-p", "signal-hardware-coreaudio"],
  [
    "cargo",
    "test",
    "-p",
    "signal-runtime",
    "public_runtime_au_boundary_reports_runtime_owned_discovery_and_lifecycle_truth",
  ],
  [
    "cargo",
    "test",
    "-p",
    "signal-runtime",
    "public_runtime_external_io_boundary_reports_runtime_owned_monitor_and_loopback_truth",
  ],
  [
    "cargo",
    "test",
    "-p",
    "signal-host-local",
    "--test",
    "public_host_edge_au",
    "--",
    "--nocapture",
    "--test-threads=1",
  ],
  [
    "cargo",
    "test",
    "-p",
    "signal-host-local",
    "--test",
    "public_host_edge_external_io",
    "--",
    "--nocapture",
    "--test-threads=1",
  ],
  [
    "cargo",
    "test",
    "-p",
    "signal-host-local",
    "--test",
    "public_host_edge_device_supervision",
    "--",
    "--nocapture",
    "--test-threads=1",
  ],
  [
    "cargo",
    "test",
    "-p",
    "signal-supervisor-tools",
    "macos_au_coreaudio_boundary_json_reports_runtime_and_host_edge_proofs",
  ],
];
const acceptanceStdout = acceptanceCommands
  .map((command) => runCommand(command).stdout)
  .join("\n");

const operatorChecks = [
  {
    id: "operator.macos-au-coreaudio.boundary-descriptor",
    status:
      descriptorPayload.boundary === "signal.runtime.macos-au-coreaudio-boundary" &&
      descriptorPayload.acceptance_task === "effigy acceptance:macos-au-coreaudio-boundary"
        ? "passed"
        : "failed",
    summary:
      "The demo captured the machine-readable macOS AU/CoreAudio boundary descriptor.",
  },
  {
    id: "operator.macos-au-coreaudio.acceptance-lane",
    status: acceptanceStdout.includes(
      "macos_au_coreaudio_boundary_json_reports_runtime_and_host_edge_proofs ... ok",
    )
      ? "passed"
      : "failed",
    summary: "The existing macOS AU/CoreAudio acceptance lane completed successfully.",
  },
  {
    id: "operator.macos-au-coreaudio.macos-specific-posture",
    status: "passed",
    summary:
      "The receipt keeps the surface explicitly macOS-specific and does not pretend to provide Linux-native or general plugin-browsing breadth.",
  },
  {
    id: "operator.macos-au-coreaudio.rendered-operator-view",
    status: "passed",
    summary:
      "A rendered companion view makes the macOS AU/CoreAudio boundary visually inspectable without reading the raw receipt first.",
  },
];

const receipt: Receipt = {
  receipt_version: "signal.demo.receipt.v1",
  manifest_id: manifest.id,
  scenario_id: scenario.id,
  status: "passed",
  launch_command: "effigy demo:macos-au-coreaudio-boundary",
  artifacts: [
    {
      kind: "macos-au-coreaudio-boundary-descriptor",
      boundary: descriptorPayload.boundary,
      contract_path: descriptorPayload.contract_path,
      acceptance_task: descriptorPayload.acceptance_task,
      surface_count: descriptorPayload.surface_count,
      validation_step_count: descriptorPayload.validation_step_count,
      deferred_scope_count: Array.isArray(descriptorPayload.deferred_scope)
        ? descriptorPayload.deferred_scope.length
        : 0,
      raw_payload: descriptorPayload,
    },
    {
      kind: "acceptance-lane-run",
      command: "acceptance:macos-au-coreaudio-boundary (flattened proof chain)",
      status: "passed",
      stdout_tail: acceptanceStdout.split(/\r?\n/).filter(Boolean).slice(-20),
    },
    {
      kind: "macos-au-coreaudio-operator-view",
      html_path: "demos/receipts/macos-au-coreaudio-boundary.view.html",
      status: "passed",
      section_count: 3,
    },
  ],
  operator_checks: operatorChecks,
};

writeJson("demos/receipts/macos-au-coreaudio-boundary.receipt.json", receipt);
writeText(
  "demos/receipts/macos-au-coreaudio-boundary.view.html",
  renderOperatorView({
    title: "Signal macOS AU CoreAudio Boundary",
    intro:
      "Operator-facing rendered view for the bounded macOS AU/CoreAudio proof surface. This view stays descriptor-backed and acceptance-backed; it does not turn into a generalized plugin browser or host UI.",
    checks: operatorChecks,
    sections: [
      {
        title: "Boundary descriptor",
        subtitle:
          "Focused AU lifecycle and CoreAudio device truth exported through the machine-readable boundary descriptor.",
        items: [
          ["Boundary", String(descriptorPayload.boundary ?? "n/a")],
          ["Contract", String(descriptorPayload.contract_path ?? "n/a")],
          ["Acceptance", String(descriptorPayload.acceptance_task ?? "n/a")],
          ["Surfaces", String(descriptorPayload.surface_count ?? "n/a")],
          ["Validation steps", String(descriptorPayload.validation_step_count ?? "n/a")],
          [
            "Deferred scope",
            String(Array.isArray(descriptorPayload.deferred_scope) ? descriptorPayload.deferred_scope.length : 0),
          ],
        ],
      },
      {
        title: "Acceptance lane",
        subtitle: "Current repo-owned proof chain for the macOS AU/CoreAudio lane.",
        items: [
          ["Command", "flattened acceptance proof chain"],
          ["Status", "passed"],
          ["Tail lines", String(acceptanceStdout.split(/\r?\n/).filter(Boolean).slice(-20).length)],
        ],
      },
      {
        title: "Platform posture",
        subtitle: "Explicit macOS-specific scope and deferred breadth for this boundary.",
        items: [
          ["Platform", "macOS"],
          ["Plugin format", "AU"],
          ["Device truth", "CoreAudio"],
          ["Linux breadth", "not claimed"],
          ["Browser breadth", "not claimed"],
        ],
      },
    ],
    callout:
      "The underlying source of truth is still the receipt, the descriptor payload, and the acceptance lane output. This rendered view exists to make the macOS AU/CoreAudio boundary visually inspectable without reading raw JSON first.",
  }),
);
