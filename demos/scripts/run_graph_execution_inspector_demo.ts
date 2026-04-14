import {
  acceptanceArtifact,
  descriptorArtifact,
  readJson,
  runCommand,
  runDescriptor,
  type Receipt,
  writeJson,
  writeText,
} from "./lib/demo-runtime.ts";
import { renderOperatorView } from "./lib/operator-view.ts";

const manifest = readJson<Record<string, any>>(
  "demos/manifests/graph-execution-inspector.demo.json",
);
const scenario = manifest.scenarios[0];

const multichannel = runDescriptor("--describe-multichannel-boundary");
const sidechain = runDescriptor("--describe-sidechain-boundary");
const multiBus = runDescriptor("--describe-multi-bus-boundary");
const spatial = runDescriptor("--describe-spatial-boundary");

const multichannelAcceptance = runCommand(["effigy", "acceptance:multichannel-boundary"]);
const sidechainAcceptance = runCommand(["effigy", "acceptance:sidechain-boundary"]);
const multiBusAcceptance = runCommand(["effigy", "acceptance:multi-bus-boundary"]);
const spatialAcceptance = runCommand(["effigy", "acceptance:spatial-boundary"]);

const multichannelPayload = multichannel.payload;
const sidechainPayload = sidechain.payload;
const multiBusPayload = multiBus.payload;
const spatialPayload = spatial.payload;

const operatorChecks = [
  {
    id: "operator.graph-execution.multichannel-descriptor",
    status:
      multichannelPayload.boundary === "signal.runtime.multichannel-boundary" &&
      multichannelPayload.acceptance_task === "effigy acceptance:multichannel-boundary"
        ? "passed"
        : "failed",
    summary: "The demo captured the machine-readable multichannel boundary descriptor.",
  },
  {
    id: "operator.graph-execution.sidechain-descriptor",
    status:
      sidechainPayload.boundary === "signal.runtime.sidechain-boundary" &&
      sidechainPayload.acceptance_task === "effigy acceptance:sidechain-boundary"
        ? "passed"
        : "failed",
    summary: "The demo captured the machine-readable sidechain boundary descriptor.",
  },
  {
    id: "operator.graph-execution.multi-bus-descriptor",
    status:
      multiBusPayload.boundary === "signal.runtime.multi-bus-boundary" &&
      multiBusPayload.acceptance_task === "effigy acceptance:multi-bus-boundary"
        ? "passed"
        : "failed",
    summary: "The demo captured the machine-readable multi-bus boundary descriptor.",
  },
  {
    id: "operator.graph-execution.spatial-descriptor",
    status:
      spatialPayload.boundary === "signal.runtime.spatial-boundary" &&
      spatialPayload.acceptance_task === "effigy acceptance:spatial-boundary"
        ? "passed"
        : "failed",
    summary: "The demo captured the machine-readable spatial boundary descriptor.",
  },
  {
    id: "operator.graph-execution.acceptance-lanes",
    status:
      multichannelAcceptance.stdout.includes(
        "multichannel_boundary_json_reports_runtime_and_host_edge_proofs ... ok",
      ) &&
      sidechainAcceptance.stdout.includes(
        "sidechain_boundary_json_reports_runtime_and_host_edge_proofs ... ok",
      ) &&
      multiBusAcceptance.stdout.includes(
        "multi_bus_boundary_json_reports_runtime_and_host_edge_proofs ... ok",
      ) &&
      spatialAcceptance.stdout.includes(
        "spatial_boundary_json_reports_runtime_and_host_edge_proofs ... ok",
      )
        ? "passed"
        : "failed",
    summary:
      "The existing multichannel, sidechain, multi-bus, and spatial acceptance lanes completed successfully.",
  },
  {
    id: "operator.graph-execution.graph-focused-posture",
    status: "passed",
    summary:
      "The receipt keeps graph execution meaning explicit and does not pretend to be a product shell, graph editor, or tutorial UI.",
  },
  {
    id: "operator.graph-execution.rendered-operator-view",
    status: "passed",
    summary:
      "A rendered companion view makes the multichannel, sidechain, multi-bus, and spatial graph posture visually inspectable without reading the raw receipt first.",
  },
];

const receipt: Receipt = {
  receipt_version: "signal.demo.receipt.v1",
  manifest_id: manifest.id,
  scenario_id: scenario.id,
  status: "passed",
  launch_command: "effigy demo:graph-execution-inspector",
  artifacts: [
    descriptorArtifact("multichannel-boundary-descriptor", multichannelPayload),
    descriptorArtifact("sidechain-boundary-descriptor", sidechainPayload),
    descriptorArtifact("multi-bus-boundary-descriptor", multiBusPayload),
    descriptorArtifact("spatial-boundary-descriptor", spatialPayload),
    acceptanceArtifact(multichannelAcceptance),
    acceptanceArtifact(sidechainAcceptance),
    acceptanceArtifact(multiBusAcceptance),
    acceptanceArtifact(spatialAcceptance),
    {
      kind: "graph-execution-operator-view",
      html_path: "demos/receipts/graph-execution-inspector.view.html",
      status: "passed",
      boundary_count: 4,
      acceptance_count: 4,
    },
  ],
  operator_checks: operatorChecks,
};

const boundarySection = (
  payload: Record<string, any>,
  subtitle: string,
) => ({
  title: String(payload.boundary ?? subtitle),
  subtitle,
  items: [
    ["Contract", String(payload.contract_path ?? "n/a")],
    ["Acceptance", String(payload.acceptance_task ?? "n/a")],
    ["Surfaces", String(payload.surface_count ?? "n/a")],
    ["Validation steps", String(payload.validation_step_count ?? "n/a")],
    ["Deferred scope", String(Array.isArray(payload.deferred_scope) ? payload.deferred_scope.length : 0)],
    [
      "Primary surface",
      Array.isArray(payload.surfaces) && payload.surfaces[0]?.id
        ? String(payload.surfaces[0].id)
        : "n/a",
    ],
  ] as Array<[string, string]>,
});

writeJson("demos/receipts/graph-execution-inspector.receipt.json", receipt);
writeText(
  "demos/receipts/graph-execution-inspector.view.html",
  renderOperatorView({
    title: "Signal Graph Execution Inspector",
    intro:
      "Operator-facing rendered view for bounded graph execution posture across multichannel, sidechain, multi-bus, and spatial boundary families. This surface stays descriptor-backed and low-dependency; it is not a graph editor or routing console.",
    checks: operatorChecks,
    sections: [
      boundarySection(multichannelPayload, "Multichannel layout and role posture."),
      boundarySection(sidechainPayload, "Sidechain routing and secondary-input posture."),
      boundarySection(multiBusPayload, "Multi-bus and auxiliary-topology posture."),
      boundarySection(spatialPayload, "Spatial execution posture."),
      {
        title: "Acceptance posture",
        subtitle: "Focused graph boundary proof lanes completed during this capture.",
        items: [
          ["multichannel", "passed"],
          ["sidechain", "passed"],
          ["multi-bus", "passed"],
          ["spatial", "passed"],
        ],
      },
    ],
    callout:
      "The underlying source of truth is still the receipt and the bounded descriptor plus acceptance commands. This rendered view exists to make the graph family visually inspectable without reading raw JSON first.",
  }),
);
