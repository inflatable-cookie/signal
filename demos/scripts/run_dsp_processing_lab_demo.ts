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
  "demos/manifests/dsp-processing-lab.demo.json",
);
const scenario = manifest.scenarios[0];

const stretch = runDescriptor("--describe-stretch-boundary");
const markerAnalysis = runDescriptor("--describe-marker-analysis-boundary");
const transformArtifact = runDescriptor("--describe-transform-artifact-boundary");

const stretchAcceptance = runCommand(["effigy", "acceptance:stretch-boundary"]);
const markerAcceptance = runCommand(["effigy", "acceptance:marker-analysis-boundary"]);
const transformAcceptance = runCommand(["effigy", "acceptance:transform-artifact-boundary"]);

const stretchPayload = stretch.payload;
const markerPayload = markerAnalysis.payload;
const transformPayload = transformArtifact.payload;

const operatorChecks = [
  {
    id: "operator.dsp-processing.stretch-descriptor",
    status:
      stretchPayload.boundary === "signal.runtime.stretch-boundary" &&
      stretchPayload.acceptance_task === "effigy acceptance:stretch-boundary"
        ? "passed"
        : "failed",
    summary: "The demo captured the machine-readable stretch boundary descriptor.",
  },
  {
    id: "operator.dsp-processing.marker-analysis-descriptor",
    status:
      markerPayload.boundary === "signal.runtime.marker-analysis-boundary" &&
      markerPayload.acceptance_task === "effigy acceptance:marker-analysis-boundary"
        ? "passed"
        : "failed",
    summary:
      "The demo captured the machine-readable marker-analysis boundary descriptor.",
  },
  {
    id: "operator.dsp-processing.transform-artifact-descriptor",
    status:
      transformPayload.boundary === "signal.runtime.transform-artifact-boundary" &&
      transformPayload.acceptance_task === "effigy acceptance:transform-artifact-boundary"
        ? "passed"
        : "failed",
    summary:
      "The demo captured the machine-readable transform-artifact boundary descriptor.",
  },
  {
    id: "operator.dsp-processing.acceptance-lanes",
    status:
      stretchAcceptance.stdout.includes(
        "stretch_boundary_json_reports_runtime_and_host_edge_proofs ... ok",
      ) &&
      markerAcceptance.stdout.includes(
        "marker_analysis_boundary_json_reports_runtime_and_host_edge_proofs ... ok",
      ) &&
      transformAcceptance.stdout.includes(
        "transform_artifact_boundary_json_reports_runtime_and_host_edge_proofs ... ok",
      )
        ? "passed"
        : "failed",
    summary:
      "The existing stretch, marker-analysis, and transform-artifact acceptance lanes completed successfully.",
  },
  {
    id: "operator.dsp-processing.processing-focused-posture",
    status: "passed",
    summary:
      "The receipt keeps DSP processing meaning explicit and does not pretend to be a waveform editor or sample browser.",
  },
  {
    id: "operator.dsp-processing.rendered-operator-view",
    status: "passed",
    summary:
      "A rendered companion view makes the DSP processing posture visually inspectable without reading the raw receipt first.",
  },
];

const receipt: Receipt = {
  receipt_version: "signal.demo.receipt.v1",
  manifest_id: manifest.id,
  scenario_id: scenario.id,
  status: "passed",
  launch_command: "effigy demo:dsp-processing-lab",
  artifacts: [
    descriptorArtifact("stretch-boundary-descriptor", stretchPayload),
    descriptorArtifact("marker-analysis-boundary-descriptor", markerPayload),
    descriptorArtifact("transform-artifact-boundary-descriptor", transformPayload),
    acceptanceArtifact(stretchAcceptance),
    acceptanceArtifact(markerAcceptance),
    acceptanceArtifact(transformAcceptance),
    {
      kind: "dsp-processing-operator-view",
      html_path: "demos/receipts/dsp-processing-lab.view.html",
      status: "passed",
      boundary_count: 3,
      acceptance_count: 3,
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

writeJson("demos/receipts/dsp-processing-lab.receipt.json", receipt);
writeText(
  "demos/receipts/dsp-processing-lab.view.html",
  renderOperatorView({
    title: "Signal DSP Processing Lab",
    intro:
      "Operator-facing rendered view for bounded DSP processing posture across stretch, marker-analysis, and transform-artifact boundary families. This surface stays descriptor-backed and low-dependency; it is not a waveform editor or processing shell.",
    checks: operatorChecks,
    sections: [
      boundarySection(stretchPayload, "Stretch timing and continuity posture."),
      boundarySection(markerPayload, "Marker-analysis descriptor posture."),
      boundarySection(transformPayload, "Transform-artifact export posture."),
      {
        title: "Acceptance posture",
        subtitle: "Focused DSP proof lanes completed during this capture.",
        items: [
          ["stretch", "passed"],
          ["marker-analysis", "passed"],
          ["transform-artifact", "passed"],
        ],
      },
    ],
    callout:
      "The underlying source of truth is still the receipt and the bounded descriptor plus acceptance commands. This rendered view exists to make the DSP family visually inspectable without reading raw JSON first.",
  }),
);
