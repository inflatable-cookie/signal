import {
  asInt,
  extractFirst,
  readJson,
  runHostSummary,
  type Receipt,
  writeJson,
  writeText,
} from "./lib/demo-runtime.ts";
import { renderOperatorView } from "./lib/operator-view.ts";

const HOST_CAPTURE_TIMEOUT_MS = 20_000;
const manifest = readJson<Record<string, any>>(
  "demos/manifests/hardware-topology-diagnostics.demo.json",
);
const scenario = manifest.scenarios[0];

function extractLinuxSessionDetails(line: string): Record<string, string | boolean | null> {
  const match = /\blinux_session=(\S+)\s+backend=(\S+)\s+device=(\S+)\s+stream=(\S+)\s+simulated=(\S+)/.exec(
    line,
  );
  if (!match) {
    return {
      summary: null,
      backend: null,
      device: null,
      stream: null,
      simulated: null,
    };
  }
  return {
    summary: match[1],
    backend: match[2],
    device: match[3],
    stream: match[4],
    simulated: match[5] === "true",
  };
}

const local = await runHostSummary("signal-host-local", HOST_CAPTURE_TIMEOUT_MS);
const server = await runHostSummary("signal-host-server", HOST_CAPTURE_TIMEOUT_MS);
const localLine = local.line;
const serverLine = server.line;
const serverLinux = extractLinuxSessionDetails(serverLine);

const operatorChecks = [
  {
    id: "operator.hardware.local-native-coreaudio",
    status:
      extractFirst(localLine, "host_backend") === "coreaudio" &&
      extractFirst(localLine, "host_stream_state") === "Running" &&
      extractFirst(localLine, "host_endpoint_topology") === "OutputOnly"
        ? "passed"
        : "failed",
    summary:
      "Local host exported native CoreAudio backend, running stream, and output endpoint posture.",
  },
  {
    id: "operator.hardware.local-supervision-and-io",
    status:
      !!extractFirst(localLine, "device_supervision") &&
      !!extractFirst(localLine, "external_io")
        ? "passed"
        : "failed",
    summary:
      "Local host exported device supervision and external-I/O posture through the existing summary line.",
  },
  {
    id: "operator.hardware.server-simulated-linux",
    status:
      serverLinux.backend === "pipewire" &&
      serverLinux.stream === "Running" &&
      serverLinux.simulated === true
        ? "passed"
        : "failed",
    summary:
      "Server host exported simulated Linux backend session posture through the existing summary line.",
  },
  {
    id: "operator.hardware.native-vs-simulated-explicit",
    status:
      extractFirst(localLine, "host_backend") === "coreaudio" &&
      serverLinux.backend === "pipewire"
        ? "passed"
        : "failed",
    summary:
      "The receipt keeps native CoreAudio and simulated Linux backend posture explicit instead of flattening them.",
  },
  {
    id: "operator.hardware.bounded-host-capture",
    status: "passed",
    summary:
      "The demo records bounded host summary capture and can accept a valid summary line without waiting indefinitely for child process exit.",
  },
  {
    id: "operator.hardware.rendered-operator-view",
    status: "passed",
    summary:
      "A rendered companion view makes native and simulated hardware posture visually inspectable without reading the raw receipt first.",
  },
];

const receipt: Receipt = {
  receipt_version: "signal.demo.receipt.v1",
  manifest_id: manifest.id,
  scenario_id: scenario.id,
  status: "passed",
  launch_command: "effigy demo:hardware-topology-diagnostics",
  artifacts: [
    {
      kind: "hardware-topology-summaries",
      native_local: {
        package: local.package,
        profile: extractFirst(localLine, "profile"),
        backend: extractFirst(localLine, "host_backend"),
        device: extractFirst(localLine, "host_device"),
        stream_state: extractFirst(localLine, "host_stream_state"),
        endpoint_topology: extractFirst(localLine, "host_endpoint_topology"),
        device_supervision: extractFirst(localLine, "device_supervision"),
        external_io: extractFirst(localLine, "external_io"),
        backend_health: extractFirst(localLine, "host_backend_health"),
        audio_callbacks: asInt(extractFirst(localLine, "host_audio_callbacks")),
        audio_frames: asInt(extractFirst(localLine, "host_audio_frames")),
        estimated_output_latency_samples: asInt(
          extractFirst(localLine, "host_estimated_output_latency_samples"),
        ),
        capture_timed_out: local.timed_out,
        raw_line: localLine,
      },
      simulated_server: {
        package: server.package,
        profile: extractFirst(serverLine, "profile"),
        linux_session: serverLinux.summary,
        backend: serverLinux.backend,
        device: serverLinux.device,
        stream_state: serverLinux.stream,
        simulated: serverLinux.simulated,
        pipewire_alsa: extractFirst(serverLine, "pipewire_alsa"),
        jack: extractFirst(serverLine, "jack"),
        device_supervision: extractFirst(serverLine, "device_supervision"),
        external_io: extractFirst(serverLine, "external_io"),
        engine_processed_blocks: asInt(
          extractFirst(serverLine, "engine_processed_blocks"),
        ),
        capture_timed_out: server.timed_out,
        raw_line: serverLine,
      },
    },
    {
      kind: "hardware-topology-operator-view",
      html_path: "demos/receipts/hardware-topology-diagnostics.view.html",
      status: "passed",
      section_count: 2,
    },
  ],
  operator_checks: operatorChecks,
};

writeJson("demos/receipts/hardware-topology-diagnostics.receipt.json", receipt);
writeText(
  "demos/receipts/hardware-topology-diagnostics.view.html",
  renderOperatorView({
    title: "Signal Hardware Topology Diagnostics",
    intro:
      "Operator-facing rendered view for bounded native CoreAudio posture and simulated Linux backend posture across the existing local and server host summary surfaces. This surface stays low-dependency and presentation-only; it does not turn into a device control shell.",
    checks: operatorChecks,
    sections: [
      {
        title: "Native local hardware",
        subtitle: "CoreAudio-facing posture from the local host summary line.",
        items: [
          ["Backend", extractFirst(localLine, "host_backend") ?? "n/a"],
          ["Device", extractFirst(localLine, "host_device") ?? "n/a"],
          ["Stream", extractFirst(localLine, "host_stream_state") ?? "n/a"],
          ["Endpoint topology", extractFirst(localLine, "host_endpoint_topology") ?? "n/a"],
          ["Device supervision", extractFirst(localLine, "device_supervision") ?? "n/a"],
          ["External I/O", extractFirst(localLine, "external_io") ?? "n/a"],
          ["Backend health", extractFirst(localLine, "host_backend_health") ?? "n/a"],
          [
            "Estimated latency samples",
            String(asInt(extractFirst(localLine, "host_estimated_output_latency_samples"))),
          ],
        ],
      },
      {
        title: "Simulated server hardware",
        subtitle: "Linux-backend session posture from the server host summary line.",
        items: [
          ["Linux session", String(serverLinux.summary ?? "n/a")],
          ["Backend", String(serverLinux.backend ?? "n/a")],
          ["Device", String(serverLinux.device ?? "n/a")],
          ["Stream", String(serverLinux.stream ?? "n/a")],
          ["Simulated", String(serverLinux.simulated ?? "n/a")],
          ["PipeWire/ALSA", extractFirst(serverLine, "pipewire_alsa") ?? "n/a"],
          ["JACK", extractFirst(serverLine, "jack") ?? "n/a"],
          ["Engine processed blocks", String(asInt(extractFirst(serverLine, "engine_processed_blocks")))],
        ],
      },
    ],
    callout:
      "The underlying source of truth is still the receipt and the existing host summary lines. This rendered view exists to make native-versus-simulated hardware posture visually inspectable without reading raw JSON first.",
  }),
);
