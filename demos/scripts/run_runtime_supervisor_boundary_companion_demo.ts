import {
  readJson,
  runDescriptor,
  type Receipt,
  writeJson,
  writeText,
} from "./lib/demo-runtime.ts";
import { renderOperatorView } from "./lib/operator-view.ts";

const manifest = readJson<Record<string, any>>(
  "demos/manifests/runtime-supervisor-boundary-companion.demo.json",
);
const scenario = manifest.scenarios[0];

const interruption = runDescriptor("--describe-interruption-boundary");
const fault = runDescriptor("--describe-fault-diagnostic-boundary");
const interruptionPayload = interruption.payload;
const faultPayload = fault.payload;

const operatorChecks = [
  {
    id: "operator.runtime-supervisor.interruption-boundary",
    status:
      interruptionPayload.boundary === "signal.runtime.interruption-boundary" &&
      interruptionPayload.acceptance_task === "effigy acceptance:interruption-boundary"
        ? "passed"
        : "failed",
    summary:
      "The supervisor companion captured the machine-readable interruption boundary descriptor.",
  },
  {
    id: "operator.runtime-supervisor.fault-diagnostic-boundary",
    status:
      faultPayload.boundary === "signal.runtime.fault-diagnostic-boundary" &&
      faultPayload.acceptance_task === "effigy acceptance:fault-diagnostic-boundary"
        ? "passed"
        : "failed",
    summary:
      "The supervisor companion captured the machine-readable fault-diagnostic boundary descriptor.",
  },
  {
    id: "operator.runtime-supervisor.runtime-family-companion",
    status:
      manifest.id === "signal.demo.runtime.supervisor-boundary-companion"
        ? "passed"
        : "failed",
    summary:
      "The receipt keeps its relationship to the runtime recovery inspector explicit as a companion surface.",
  },
  {
    id: "operator.runtime-supervisor.rendered-operator-view",
    status: "passed",
    summary:
      "A rendered companion view makes interruption and fault-diagnostic boundary posture visually inspectable without reading the raw receipt first.",
  },
];

const receipt: Receipt = {
  receipt_version: "signal.demo.receipt.v1",
  manifest_id: manifest.id,
  scenario_id: scenario.id,
  status: "passed",
  launch_command: "effigy demo:supervisor-runtime-boundary-companion",
  artifacts: [
    {
      kind: "signal-supervisor-tools-runtime-boundaries",
      companion_to_manifest: "signal.demo.runtime.recovery-inspector",
      descriptors: [
        {
          boundary: interruptionPayload.boundary,
          contract_path: interruptionPayload.contract_path,
          acceptance_task: interruptionPayload.acceptance_task,
          surface_count: Array.isArray(interruptionPayload.surfaces)
            ? interruptionPayload.surfaces.length
            : 0,
          validation_step_count: Array.isArray(interruptionPayload.validation_steps)
            ? interruptionPayload.validation_steps.length
            : 0,
          deferred_scope_count: Array.isArray(interruptionPayload.deferred_scope)
            ? interruptionPayload.deferred_scope.length
            : 0,
          raw_payload: interruptionPayload,
        },
        {
          boundary: faultPayload.boundary,
          contract_path: faultPayload.contract_path,
          acceptance_task: faultPayload.acceptance_task,
          surface_count: Array.isArray(faultPayload.surfaces)
            ? faultPayload.surfaces.length
            : 0,
          validation_step_count: Array.isArray(faultPayload.validation_steps)
            ? faultPayload.validation_steps.length
            : 0,
          deferred_scope_count: Array.isArray(faultPayload.deferred_scope)
            ? faultPayload.deferred_scope.length
            : 0,
          raw_payload: faultPayload,
        },
      ],
    },
    {
      kind: "runtime-supervisor-operator-view",
      html_path: "demos/receipts/runtime-supervisor-boundary-companion.view.html",
      status: "passed",
      boundary_count: 2,
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
    [
      "Validation steps",
      String(
        payload.validation_step_count ??
          (Array.isArray(payload.validation_steps) ? payload.validation_steps.length : "n/a"),
      ),
    ],
    ["Deferred scope", String(Array.isArray(payload.deferred_scope) ? payload.deferred_scope.length : 0)],
    [
      "Primary surface",
      Array.isArray(payload.surfaces) && payload.surfaces[0]?.id
        ? String(payload.surfaces[0].id)
        : "n/a",
    ],
  ] as Array<[string, string]>,
});

writeJson(
  "demos/receipts/runtime-supervisor-boundary-companion.receipt.json",
  receipt,
);
writeText(
  "demos/receipts/runtime-supervisor-boundary-companion.view.html",
  renderOperatorView({
    title: "Signal Runtime Supervisor Boundary Companion",
    intro:
      "Operator-facing rendered view for bounded interruption and fault-diagnostic runtime boundary posture. This surface stays descriptor-backed and low-dependency; it complements the runtime recovery inspector instead of replacing it.",
    checks: operatorChecks,
    sections: [
      boundarySection(
        interruptionPayload,
        "Interruption taxonomy and resumability posture.",
      ),
      boundarySection(
        faultPayload,
        "Fault-diagnostic cause and evidence posture.",
      ),
    ],
    callout:
      "The underlying source of truth is still the receipt and the bounded supervisor descriptor commands. This rendered view exists to make the runtime companion surface visually inspectable without reading raw JSON first.",
  }),
);
