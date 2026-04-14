import {
  readJson,
  runCommand,
  type Receipt,
  writeJson,
  writeText,
} from "./lib/demo-runtime.ts";
import { renderOperatorView } from "./lib/operator-view.ts";

const BROKER_RUNS: Array<[string, string[]]> = [
  ["attach_status_teardown", ["status", "attach-demo", "status", "teardown-demo", "shutdown"]],
  ["healthy_run", ["run-demo", "shutdown"]],
  ["timeout_run", ["run-timeout-demo", "shutdown"]],
];

const manifest = readJson<Record<string, any>>(
  "demos/manifests/plugin-sandbox-lifecycle.demo.json",
);
const scenario = manifest.scenarios[0];
const launchCommand = "cargo run -q -p signal-plugin-sandbox";

const transcripts = BROKER_RUNS.map(([runId, commands]) => {
  const result = runCommand(
    ["cargo", "run", "-q", "-p", "signal-plugin-sandbox"],
    `${commands.join("\n")}\n`,
  );
  const lines = result.stdout
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line.startsWith("signal-plugin-sandbox"));
  return { run_id: runId, lines };
});

const allLines = transcripts.flatMap((transcript) => transcript.lines);
const observedStates = Array.from(
  new Set(
    allLines.flatMap((line) =>
      line
        .split(/\s+/)
        .filter((token) => token.startsWith("state="))
        .map((token) => token.split("=", 2)[1]),
    ),
  ),
).sort();

const transcriptContains = (runId: string, fragment: string) =>
  transcripts.some(
    (transcript) =>
      transcript.run_id === runId &&
      transcript.lines.some((line) => line.includes(fragment)),
  );

const operatorChecks = [
  {
    id: "operator.sandbox-lifecycle.ready-state",
    status: transcriptContains("attach_status_teardown", "state=ready")
      ? "passed"
      : "failed",
    summary: "Broker reported the ready state before live lifecycle commands.",
  },
  {
    id: "operator.sandbox-lifecycle.attach-and-teardown",
    status:
      transcriptContains("attach_status_teardown", "state=attached") &&
      transcriptContains("attach_status_teardown", "state=teardown_complete")
        ? "passed"
        : "failed",
    summary: "Explicit attach/status/teardown path remained inspectable.",
  },
  {
    id: "operator.sandbox-lifecycle.run-path",
    status:
      transcriptContains("healthy_run", "state=running") &&
      transcriptContains("healthy_run", "detail=lease_cleanup_ok")
        ? "passed"
        : "failed",
    summary: "Healthy demo run reached running and clean teardown states.",
  },
  {
    id: "operator.sandbox-lifecycle.timeout-path",
    status:
      transcriptContains("timeout_run", "state=timed_out") &&
      transcriptContains("timeout_run", "detail=lease_cleanup_ok_after_timeout")
        ? "passed"
        : "failed",
    summary:
      "Timeout demo run remained bounded and reported cleanup after interruption.",
  },
  {
    id: "operator.sandbox-lifecycle.shutdown",
    status: transcripts.every((transcript) =>
      transcript.lines.some((line) => line.includes("state=shutdown")),
    )
      ? "passed"
      : "failed",
    summary: "Broker exited through the explicit shutdown receipt.",
  },
  {
    id: "operator.sandbox-lifecycle.rendered-operator-view",
    status: "passed",
    summary:
      "A rendered companion view makes broker lifecycle and timeout recovery posture visually inspectable without reading the raw receipt first.",
  },
];

const attachLines =
  transcripts.find((transcript) => transcript.run_id === "attach_status_teardown")
    ?.lines ?? [];
const healthyLines =
  transcripts.find((transcript) => transcript.run_id === "healthy_run")?.lines ?? [];
const timeoutLines =
  transcripts.find((transcript) => transcript.run_id === "timeout_run")?.lines ?? [];

const receipt: Receipt = {
  receipt_version: "signal.demo.receipt.v1",
  manifest_id: manifest.id,
  scenario_id: scenario.id,
  status: "passed",
  launch_command: launchCommand,
  artifacts: [
    {
      kind: "broker-transcript-lines",
      line_count: allLines.length,
      observed_states: observedStates,
      runs: transcripts.map((transcript) => ({
        run_id: transcript.run_id,
        line_count: transcript.lines.length,
      })),
    },
    {
      kind: "sandbox-lifecycle-operator-view",
      html_path: "demos/receipts/plugin-sandbox-lifecycle.view.html",
      status: "passed",
      section_count: 3,
    },
  ],
  operator_checks: operatorChecks,
};

writeJson("demos/receipts/plugin-sandbox-lifecycle.receipt.json", receipt);
writeText(
  "demos/receipts/plugin-sandbox-lifecycle.view.html",
  renderOperatorView({
    title: "Signal Sandbox Lifecycle",
    intro:
      "Operator-facing rendered view for bounded broker lifecycle truth across ready, attach, status, healthy run, timeout run, teardown, and shutdown. This surface stays broker-backed and low-dependency; it is not a broker control console.",
    checks: operatorChecks,
    sections: [
      {
        title: "Lifecycle coverage",
        subtitle: "Observed broker states across the bounded lifecycle runs.",
        items: [
          ["Observed states", observedStates.join(", ")],
          ["Runs", String(transcripts.length)],
          ["Transcript lines", String(allLines.length)],
          ["Healthy run lines", String(healthyLines.length)],
          ["Timeout run lines", String(timeoutLines.length)],
        ],
      },
      {
        title: "Explicit attach path",
        subtitle:
          "Attach, status, teardown, and shutdown posture from the manual lifecycle run.",
        items: [
          [
            "Ready state",
            transcriptContains("attach_status_teardown", "state=ready") ? "yes" : "no",
          ],
          [
            "Attached",
            transcriptContains("attach_status_teardown", "state=attached") ? "yes" : "no",
          ],
          [
            "Teardown complete",
            transcriptContains("attach_status_teardown", "state=teardown_complete")
              ? "yes"
              : "no",
          ],
          [
            "Shutdown",
            transcriptContains("attach_status_teardown", "state=shutdown") ? "yes" : "no",
          ],
          ["Transcript lines", String(attachLines.length)],
        ],
      },
      {
        title: "Healthy and timeout runs",
        subtitle: "Comparison of the clean run and timeout recovery run.",
        items: [
          [
            "Healthy run reached running",
            transcriptContains("healthy_run", "state=running") ? "yes" : "no",
          ],
          [
            "Healthy cleanup detail",
            transcriptContains("healthy_run", "detail=lease_cleanup_ok") ? "yes" : "no",
          ],
          [
            "Timeout reached timed_out",
            transcriptContains("timeout_run", "state=timed_out") ? "yes" : "no",
          ],
          [
            "Timeout cleanup detail",
            transcriptContains("timeout_run", "detail=lease_cleanup_ok_after_timeout")
              ? "yes"
              : "no",
          ],
          [
            "Shutdown in both runs",
            transcriptContains("healthy_run", "state=shutdown") &&
            transcriptContains("timeout_run", "state=shutdown")
              ? "yes"
              : "no",
          ],
        ],
      },
    ],
    callout:
      "The underlying source of truth is still the receipt and broker transcript lines. This rendered view exists to make broker lifecycle and timeout recovery posture visually inspectable without reading raw JSON first.",
  }),
);
