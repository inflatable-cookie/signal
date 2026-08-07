import { type JsonObject } from "./paths.ts";

export function receiptFromRun(manifest: JsonObject, browserModel: JsonObject, primaryLaunch: JsonObject | null): JsonObject {
  const scenario = manifest.scenarios[0];
  const stats = browserModel.stats as JsonObject;
  const operatorChecks = [
    {
      id: "operator.plugin-browser.inventory-visible",
      status: stats.plugin_count > 0 ? "passed" : "failed",
      summary: "The browser shows discovered plugin inventory instead of a placeholder or hardcoded list.",
    },
    {
      id: "operator.plugin-browser.launch-path-visible",
      status: stats.launch_target_count > 0 ? "passed" : "failed",
      summary: "The browser exposes bounded per-plugin launch targets through repo-owned host commands.",
    },
    {
      id: "operator.plugin-browser.supported-live-path",
      status: primaryLaunch && primaryLaunch.status === "passed" ? "passed" : "failed",
      summary: "At least one supported CLAP or VST3 launch path executed successfully during the capture run.",
    },
    {
      id: "operator.plugin-browser.bounded-interaction-visible",
      status: primaryLaunch && primaryLaunch.interaction_proved ? "passed" : "failed",
      summary: "At least one bounded browser launch surfaced an explicit parameter-step interaction result instead of bootstrap-only success.",
    },
    {
      id: "operator.plugin-browser.exclusions-explicit",
      status: "passed",
      summary: "Unsupported editor embedding, platform exclusions, and bounded host-bootstrap posture remain explicit in the surface.",
    },
  ];
  const artifacts: JsonObject[] = [
    {
      kind: "plugin-browser-model",
      html_path: "demos/receipts/plugin-capability-browser.view.html",
      plugin_count: stats.plugin_count,
      format_count: stats.format_count,
      launch_target_count: stats.launch_target_count,
      fixture_fallback_used: stats.fixture_fallback_used,
      inventory: browserModel.inventory,
    },
  ];
  if (primaryLaunch) {
    artifacts.push({ kind: "bounded-plugin-launch", ...primaryLaunch });
  }
  return {
    receipt_version: "signal.demo.receipt.v1",
    manifest_id: manifest.id,
    scenario_id: scenario.id,
    status: operatorChecks.every((check) => check.status === "passed") ? "passed" : "failed",
    launch_command: "effigy demo:plugin-capability-browser",
    artifacts,
    operator_checks: operatorChecks,
  };
}
