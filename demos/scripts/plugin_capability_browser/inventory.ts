import { type JsonObject } from "./paths.ts";
import { INTERACTIVE_SERVER_FOLLOWUP_LIMITS } from "./constants.ts";
import { renderedScanFormat } from "./scan.ts";
import { runLaunch } from "./launch.ts";

export function exactLaunchRoot(plugin: JsonObject): string | null {
  for (const target of plugin.launch_targets as JsonObject[]) {
    if (target.host_surface === "server") {
      return String(target.launch_root);
    }
  }
  for (const target of plugin.launch_targets as JsonObject[]) {
    return String(target.launch_root);
  }
  return null;
}

export function combineInventory(scans: JsonObject[]): JsonObject[] {
  const combined = new Map<string, JsonObject>();
  for (const scan of scans) {
    const hostSurface = String(scan.host_surface);
    for (const sourcePlugin of scan.plugins as JsonObject[]) {
      const plugin = { ...sourcePlugin };
      const key = `${plugin.format}\0${plugin.plugin_type_id}`;
      if (!combined.has(key)) {
        combined.set(key, {
          plugin_type_id: plugin.plugin_type_id,
          plugin_id: plugin.plugin_id,
          format: plugin.format,
          vendor: plugin.vendor,
          name: plugin.name,
          version: plugin.version,
          features: plugin.features,
          parameter_count: plugin.parameter_count,
          audio_bus_count: plugin.audio_bus_count,
          summary: plugin.summary,
          interaction_posture: plugin.interaction_posture,
          launch_targets: [],
        });
      }
      combined.get(key)!.launch_targets.push({
        host_surface: hostSurface,
        launch_root: plugin.launch_root,
        plugin_type_id: plugin.plugin_type_id,
        format: plugin.format,
      });
    }
  }
  const inventory = [...combined.values()];
  for (const plugin of inventory) {
    plugin.launch_targets.sort((left: JsonObject, right: JsonObject) => {
      const leftRank = left.host_surface === "local" ? 0 : 1;
      const rightRank = right.host_surface === "local" ? 0 : 1;
      if (leftRank !== rightRank) {
        return leftRank - rightRank;
      }
      return String(left.launch_root).localeCompare(String(right.launch_root));
    });
  }
  return inventory.sort((left, right) =>
    [left.format, left.vendor, left.name, left.plugin_type_id].join("\0")
      .localeCompare([right.format, right.vendor, right.name, right.plugin_type_id].join("\0")));
}

export function choosePrimaryLaunch(inventory: JsonObject[]): JsonObject | null {
  const preferredHosts: Record<string, number> = { local: 0, server: 1 };
  const candidates: Array<[number, number, JsonObject, JsonObject]> = [];
  for (const plugin of inventory) {
    if (!["Clap", "Vst3"].includes(String(plugin.format))) {
      continue;
    }
    for (const target of plugin.launch_targets as JsonObject[]) {
      candidates.push([
        plugin.format === "Clap" ? 0 : 1,
        preferredHosts[String(target.host_surface)] ?? 99,
        plugin,
        target,
      ]);
    }
  }
  if (candidates.length === 0) {
    return null;
  }
  candidates.sort((left, right) => left[0] - right[0] || left[1] - right[1]);
  const [, , plugin, target] = candidates[0]!;
  return { plugin, target };
}

export function executePrimaryLaunch(inventory: JsonObject[]): JsonObject | null {
  const preferredHosts: Record<string, number> = { local: 0, server: 1 };
  const preferredFormats: Record<string, number> = { Vst3: 0, Clap: 1, Au: 2, Lv2: 3 };
  const candidates: Array<[number, number, JsonObject, JsonObject]> = [];
  for (const plugin of inventory) {
    for (const target of plugin.launch_targets as JsonObject[]) {
      candidates.push([
        preferredHosts[String(target.host_surface)] ?? 99,
        preferredFormats[String(plugin.format)] ?? 99,
        plugin,
        target,
      ]);
    }
  }
  if (candidates.length === 0) {
    return null;
  }
  candidates.sort((left, right) => left[0] - right[0] || left[1] - right[1]);
  let firstResult: JsonObject | null = null;
  for (const [, , plugin, target] of candidates.slice(0, 6)) {
    const pkg = "signal-host-local";
    const result = runLaunch(pkg, target);
    const candidateResult = { plugin, target, ...result };
    if (!firstResult) {
      firstResult = candidateResult;
    }
    if (result.status === "passed") {
      return candidateResult;
    }
  }
  return firstResult;
}

export function preferredServerRootsFromInventory(inventory: JsonObject[]): Record<string, string[]> {
  const rootsByFormat: Record<string, string[]> = { clap: [], vst3: [], au: [], lv2: [] };
  for (const plugin of inventory) {
    const format = renderedScanFormat(String(plugin.format));
    if (!(format in rootsByFormat)) {
      continue;
    }
    const root = exactLaunchRoot(plugin);
    if (root && !rootsByFormat[format]!.includes(root)) {
      rootsByFormat[format]!.push(root);
    }
  }
  for (const [format, roots] of Object.entries(rootsByFormat)) {
    rootsByFormat[format] = roots.slice(0, INTERACTIVE_SERVER_FOLLOWUP_LIMITS[format] ?? 4);
  }
  return rootsByFormat;
}
