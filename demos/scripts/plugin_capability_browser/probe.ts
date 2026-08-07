import process from "node:process";
import { type JsonObject } from "./paths.ts";
import {
  LOCAL_PROBE_ATTEMPT_LIMIT,
  LOCAL_PROBE_SUCCESS_LIMIT,
  LOCAL_PROBE_TIMEOUT_SECONDS,
} from "./constants.ts";
import { renderedScanFormat, safeRunScanExample } from "./scan.ts";
import { exactLaunchRoot } from "./inventory.ts";

export function canRunLocalSurface(): boolean {
  return process.platform === "darwin";
}

function localProbeSortKey(plugin: JsonObject): [number, string, string, string] {
  const ranks: Record<string, number> = { Clap: 0, Vst3: 1, Au: 2 };
  return [
    ranks[String(plugin.format)] ?? 99,
    String(plugin.vendor),
    String(plugin.name),
    String(plugin.plugin_type_id),
  ];
}

export async function attachContainedLocalTargets(inventory: JsonObject[]): Promise<JsonObject> {
  const summary: JsonObject = {
    attempted: 0,
    succeeded: 0,
    failed: 0,
    limit_hit: false,
    failures: [],
  };
  if (!canRunLocalSurface()) {
    return summary;
  }
  const candidates = inventory
    .filter((plugin) => ["Clap", "Vst3", "Au"].includes(String(plugin.format)))
    .sort((left, right) => {
      const lhs = localProbeSortKey(left);
      const rhs = localProbeSortKey(right);
      return lhs.join("\0").localeCompare(rhs.join("\0"));
    });
  for (const plugin of candidates) {
    if (summary.succeeded >= LOCAL_PROBE_SUCCESS_LIMIT) {
      break;
    }
    if (summary.attempted >= LOCAL_PROBE_ATTEMPT_LIMIT) {
      summary.limit_hit = true;
      break;
    }
    const launchRoot = exactLaunchRoot(plugin);
    if (!launchRoot) {
      continue;
    }
    summary.attempted += 1;
    const [scan, error] = await safeRunScanExample(
      "signal-host-local",
      [renderedScanFormat(String(plugin.format))],
      [launchRoot],
      LOCAL_PROBE_TIMEOUT_SECONDS,
    );
    if (!scan) {
      summary.failed += 1;
      if (error && summary.failures.length < 5) {
        summary.failures.push(`${plugin.name} (${plugin.format}): ${error.split(/\r?\n/)[0]?.slice(0, 180)}`);
      }
      continue;
    }
    const discovered = new Set((scan.plugins as JsonObject[]).map((item) => String(item.plugin_type_id)));
    if (!discovered.has(String(plugin.plugin_type_id))) {
      summary.failed += 1;
      if (summary.failures.length < 5) {
        summary.failures.push(`${plugin.name} (${plugin.format}): local scan did not return the plugin type`);
      }
      continue;
    }
    if (!(plugin.launch_targets as JsonObject[]).some((target) => target.host_surface === "local")) {
      (plugin.launch_targets as JsonObject[]).push({
        host_surface: "local",
        launch_root: launchRoot,
        plugin_type_id: plugin.plugin_type_id,
        format: plugin.format,
      });
    }
    summary.succeeded += 1;
  }
  return summary;
}
