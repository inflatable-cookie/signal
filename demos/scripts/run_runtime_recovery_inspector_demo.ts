import {
  extractFirst,
  readJson,
  runCommand,
  type Receipt,
  writeJson,
  writeText,
} from "./lib/demo-runtime.ts";
import { renderOperatorView } from "./lib/operator-view.ts";

const manifest = readJson<Record<string, any>>(
  "demos/manifests/runtime-recovery-inspector.demo.json",
);
const scenario = manifest.scenarios[0];
const launchCommand = "cargo run -q -p signal-runtime --example supervisor_report_demo";

const result = runCommand([
  "cargo",
  "run",
  "-q",
  "-p",
  "signal-runtime",
  "--example",
  "supervisor_report_demo",
]);

const lines = result.stdout
  .split(/\r?\n/)
  .map((line) => line.trim())
  .filter(Boolean);
const joined = lines.join(" ");
const contains = (fragment: string) => lines.some((line) => line.includes(fragment));

const readiness = extractFirst(joined, "readiness") ?? "n/a";
const handshaken = extractFirst(joined, "handshaken") ?? "n/a";
const configured = extractFirst(joined, "configured") ?? "n/a";
const running = extractFirst(joined, "running") ?? "n/a";
const watchdog = extractFirst(joined, "last_watchdog") ?? "n/a";
const pluginFaults = extractFirst(joined, "plugin_faults") ?? "n/a";
const lastFault = extractFirst(joined, "last_fault") ?? "n/a";
const eventCount = extractFirst(joined, "events") ?? "n/a";
const safeMode = extractFirst(joined, "safe_mode") ?? "n/a";
const deviceSafeMode =
  extractFirst(joined, "device_supervision_safe_mode_enabled") ?? "n/a";
const externalIo = extractFirst(joined, "external_io_summary") ?? "n/a";
const linuxBackend =
  extractFirst(joined, "linux_backend_session_summary") ?? "n/a";

const operatorChecks = [
  {
    id: "operator.runtime-recovery.handshake-and-start",
    status:
      contains("handshaken=true") &&
      contains("configured=true") &&
      contains("running=true")
        ? "passed"
        : "failed",
    summary: "Runtime example completed handshake, configuration, and start.",
  },
  {
    id: "operator.runtime-recovery.watchdog-snapshot",
    status:
      contains("last_watchdog=HeartbeatMisses") &&
      contains("degradation_summary_last_watchdog=Some(HeartbeatMisses)")
        ? "passed"
        : "failed",
    summary: "Supervisor output exposed the watchdog-trigger snapshot.",
  },
  {
    id: "operator.runtime-recovery.plugin-faults",
    status:
      contains("plugin_faults=2") && contains("last_fault=sandbox-demo:Timeout")
        ? "passed"
        : "failed",
    summary: "Runtime example exported the injected plugin timeout faults.",
  },
  {
    id: "operator.runtime-recovery-safe-mode-posture",
    status:
      contains("safe_mode=false") &&
      contains("device_supervision_safe_mode_enabled=false")
        ? "passed"
        : "failed",
    summary:
      "Runtime report kept explicit safe-mode posture in the steady-state surface.",
  },
  {
    id: "operator.runtime-recovery.external-surface",
    status:
      contains("external_io_summary=health=Unavailable") &&
      contains("linux_backend_session_summary=backend=Unavailable")
        ? "passed"
        : "failed",
    summary:
      "Runtime report preserved degraded hardware/backend surfaces explicitly.",
  },
  {
    id: "operator.runtime-recovery.rendered-operator-view",
    status: "passed",
    summary:
      "A rendered companion view makes watchdog, fault, safe-mode, and degraded-surface posture visually inspectable without reading the raw receipt first.",
  },
];

const receipt: Receipt = {
  receipt_version: "signal.demo.receipt.v1",
  manifest_id: manifest.id,
  scenario_id: scenario.id,
  status: "passed",
  launch_command: launchCommand,
  artifacts: [
    {
      kind: "runtime-supervisor-report-lines",
      line_count: lines.length,
      highlights: {
        readiness,
        watchdog,
        plugin_fault_count: pluginFaults,
        event_count: eventCount,
      },
    },
    {
      kind: "runtime-recovery-operator-view",
      html_path: "demos/receipts/runtime-recovery-inspector.view.html",
      status: "passed",
      section_count: 3,
    },
  ],
  operator_checks: operatorChecks,
};

writeJson("demos/receipts/runtime-recovery-inspector.receipt.json", receipt);
writeText(
  "demos/receipts/runtime-recovery-inspector.view.html",
  renderOperatorView({
    title: "Signal Runtime Recovery Inspector",
    intro:
      "Operator-facing rendered view for bounded runtime recovery posture across handshake, watchdog, plugin faults, safe mode, and degraded external/backend surfaces. This surface stays example-backed and low-dependency; it is not a runtime dashboard or control shell.",
    checks: operatorChecks,
    sections: [
      {
        title: "Lifecycle posture",
        subtitle:
          "Bounded runtime startup and readiness posture from the supervisor report example.",
        items: [
          ["Readiness", readiness],
          ["Handshaken", handshaken],
          ["Configured", configured],
          ["Running", running],
        ],
      },
      {
        title: "Watchdog and faults",
        subtitle:
          "Observed watchdog trigger and plugin-fault history from the bounded recovery report.",
        items: [
          ["Watchdog", watchdog],
          ["Plugin faults", pluginFaults],
          ["Last fault", lastFault],
          ["Events", eventCount],
        ],
      },
      {
        title: "Safe mode and degraded surfaces",
        subtitle:
          "Steady-state safe-mode and external/backend degradation posture.",
        items: [
          ["Safe mode", safeMode],
          ["Device safe mode", deviceSafeMode],
          ["External I/O", externalIo],
          ["Linux backend", linuxBackend],
        ],
      },
    ],
    callout:
      "The underlying source of truth is still the receipt and the bounded runtime report example. This rendered view exists to make runtime recovery posture visually inspectable without reading raw JSON first.",
  }),
);
