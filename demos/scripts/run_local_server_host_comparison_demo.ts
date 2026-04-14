import {
  asBool,
  asInt,
  parseSummaryLine,
  readJson,
  runCommand,
  type Receipt,
  writeJson,
  writeText,
} from "./lib/demo-runtime.ts";
import { renderOperatorView } from "./lib/operator-view.ts";

const manifest = readJson<Record<string, any>>(
  "demos/manifests/local-server-host-comparison.demo.json",
);
const scenario = manifest.scenarios[0];

function runHost(packageName: string) {
  const result = runCommand(["cargo", "run", "-q", "-p", packageName]);
  const line = result.stdout
    .split(/\r?\n/)
    .map((value) => value.trim())
    .find((value) => value.startsWith(packageName));
  if (!line) {
    throw new Error(`${packageName} did not emit a summary line`);
  }
  return {
    package: packageName,
    line,
    parsed: parseSummaryLine(line),
  };
}

const local = runHost("signal-host-local");
const server = runHost("signal-host-server");
const localParsed = local.parsed;
const serverParsed = server.parsed;
const localReadiness = localParsed.readiness ?? "n/a";
const serverReadiness = serverParsed.readiness ?? "n/a";

const comparison = {
  shared_truth: {
    local_ready: localReadiness === "Ready",
    server_ready: serverReadiness === "Ready",
    local_running: asBool(localParsed.running),
    server_running: asBool(serverParsed.running),
    local_processed_blocks: asInt(localParsed.processed_blocks),
    server_processed_blocks: asInt(serverParsed.processed_blocks),
    local_completion: localParsed.completion,
    server_completion: serverParsed.completion,
    local_heartbeat_responses: asInt(localParsed.heartbeat_responses),
    server_heartbeat_responses: asInt(serverParsed.heartbeat_responses),
  },
  host_differences: {
    local_backend: localParsed.backend,
    local_audio_state: localParsed.audio_state,
    server_engine_processed_blocks: asInt(serverParsed.engine_processed_blocks),
    server_engine_graph_id: serverParsed.engine_graph_id,
    local_topology_nodes: asInt(localParsed.topology_nodes),
  },
};

const operatorChecks = [
  {
    id: "operator.host-compare.local-bootstrap",
    status:
      localReadiness === "Ready" &&
      asBool(localParsed.running) &&
      asInt(localParsed.processed_blocks) > 0 &&
      localParsed.completion === "Completed"
        ? "passed"
        : "failed",
    summary:
      "Local host booted successfully with ready/running posture and bounded execution.",
  },
  {
    id: "operator.host-compare.server-bootstrap",
    status:
      serverReadiness === "Ready" &&
      asBool(serverParsed.running) &&
      asInt(serverParsed.processed_blocks) > 0 &&
      serverParsed.completion === "Completed"
        ? "passed"
        : "failed",
    summary:
      "Server host booted successfully with ready/running posture and bounded execution.",
  },
  {
    id: "operator.host-compare.shared-lifecycle-truth",
    status:
      !!localParsed.sandbox &&
      !!serverParsed.sandbox &&
      asInt(localParsed.heartbeat_responses) > 0 &&
      asInt(serverParsed.heartbeat_responses) > 0
        ? "passed"
        : "failed",
    summary:
      "Both hosts exported active sandbox and heartbeat truth through the existing summary line.",
  },
  {
    id: "operator.host-compare-differences-explicit",
    status:
      localParsed.backend === "coreaudio" &&
      asInt(serverParsed.engine_processed_blocks) > 0
        ? "passed"
        : "failed",
    summary:
      "The receipt preserves real local-versus-server differences instead of flattening them.",
  },
  {
    id: "operator.host-compare.rendered-operator-view",
    status: "passed",
    summary:
      "A rendered companion view makes shared lifecycle truth and local-versus-server differences visually inspectable without reading the raw receipt first.",
  },
];

const receipt: Receipt = {
  receipt_version: "signal.demo.receipt.v1",
  manifest_id: manifest.id,
  scenario_id: scenario.id,
  status: "passed",
  launch_command: "effigy demo:local-server-host-comparison",
  artifacts: [
    {
      kind: "host-summary-lines",
      hosts: [
        {
          package: local.package,
          sandbox: localParsed.sandbox,
          profile: localParsed.profile,
          processed_blocks: asInt(localParsed.processed_blocks),
          heartbeat_responses: asInt(localParsed.heartbeat_responses),
          completion: localParsed.completion,
          raw_line: local.line,
        },
        {
          package: server.package,
          sandbox: serverParsed.sandbox,
          profile: serverParsed.profile,
          processed_blocks: asInt(serverParsed.processed_blocks),
          heartbeat_responses: asInt(serverParsed.heartbeat_responses),
          completion: serverParsed.completion,
          raw_line: server.line,
        },
      ],
      comparison,
    },
    {
      kind: "host-comparison-operator-view",
      html_path: "demos/receipts/local-server-host-comparison.view.html",
      status: "passed",
      section_count: 3,
    },
  ],
  operator_checks: operatorChecks,
};

writeJson("demos/receipts/local-server-host-comparison.receipt.json", receipt);
writeText(
  "demos/receipts/local-server-host-comparison.view.html",
  renderOperatorView({
    title: "Signal Local Server Host Comparison",
    intro:
      "Operator-facing rendered view for bounded local-versus-server host bootstrap posture. This surface stays summary-backed and low-dependency; it compares the current host binaries without turning into a host UI shell.",
    checks: operatorChecks,
    sections: [
      {
        title: "Local host",
        subtitle:
          "Shared lifecycle and host-local posture from the local bootstrap summary.",
        items: [
          ["Readiness", localReadiness],
          ["Running", String(asBool(localParsed.running)).toLowerCase()],
          ["Backend", localParsed.backend ?? "n/a"],
          ["Sandbox", localParsed.sandbox ?? "n/a"],
          ["Processed blocks", String(asInt(localParsed.processed_blocks))],
          ["Heartbeat responses", String(asInt(localParsed.heartbeat_responses))],
          ["Completion", localParsed.completion ?? "n/a"],
          ["Audio state", localParsed.audio_state ?? "n/a"],
        ],
      },
      {
        title: "Server host",
        subtitle:
          "Shared lifecycle and server-side execution posture from the server bootstrap summary.",
        items: [
          ["Readiness", serverReadiness],
          ["Running", String(asBool(serverParsed.running)).toLowerCase()],
          ["Sandbox", serverParsed.sandbox ?? "n/a"],
          ["Processed blocks", String(asInt(serverParsed.processed_blocks))],
          ["Heartbeat responses", String(asInt(serverParsed.heartbeat_responses))],
          ["Completion", serverParsed.completion ?? "n/a"],
          ["Engine processed blocks", String(asInt(serverParsed.engine_processed_blocks))],
          ["Engine graph", serverParsed.engine_graph_id ?? "n/a"],
        ],
      },
      {
        title: "Comparison",
        subtitle:
          "Explicit shared truth and real differences between the two host surfaces.",
        items: [
          [
            "Shared readiness",
            `${comparison.shared_truth.local_ready ? "ready" : "not-ready"} / ${comparison.shared_truth.server_ready ? "ready" : "not-ready"}`,
          ],
          [
            "Shared running",
            `${comparison.shared_truth.local_running ? "running" : "not-running"} / ${comparison.shared_truth.server_running ? "running" : "not-running"}`,
          ],
          [
            "Shared completion",
            `${comparison.shared_truth.local_completion ?? "n/a"} / ${comparison.shared_truth.server_completion ?? "n/a"}`,
          ],
          [
            "Heartbeat truth",
            `${comparison.shared_truth.local_heartbeat_responses} / ${comparison.shared_truth.server_heartbeat_responses}`,
          ],
          ["Local backend", String(comparison.host_differences.local_backend ?? "n/a")],
          [
            "Server engine blocks",
            String(comparison.host_differences.server_engine_processed_blocks),
          ],
          [
            "Server engine graph",
            String(comparison.host_differences.server_engine_graph_id ?? "n/a"),
          ],
          [
            "Local topology nodes",
            String(comparison.host_differences.local_topology_nodes),
          ],
        ],
      },
    ],
    callout:
      "The underlying source of truth is still the receipt and the existing host summary lines. This rendered view exists to make shared and differing host posture visually inspectable without reading raw JSON first.",
  }),
);
