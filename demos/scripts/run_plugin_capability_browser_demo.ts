import { createServer, type IncomingMessage, type ServerResponse } from "node:http";
import { spawn, spawnSync } from "node:child_process";
import { existsSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, extname, resolve } from "node:path";
import process from "node:process";

type JsonObject = Record<string, any>;

const REPO_ROOT = resolve(import.meta.dir, "../..");
const MANIFEST_PATH = resolve(REPO_ROOT, "demos/manifests/plugin-capability-browser.demo.json");
const RECEIPT_PATH = resolve(REPO_ROOT, "demos/receipts/plugin-capability-browser.receipt.json");
const HTML_PATH = resolve(REPO_ROOT, "demos/receipts/plugin-capability-browser.view.html");

const LOCAL_PROBE_TIMEOUT_SECONDS = 8;
const LOCAL_PROBE_SUCCESS_LIMIT = 6;
const LOCAL_PROBE_ATTEMPT_LIMIT = 18;
const SYSTEM_SCAN_TIMEOUT_SECONDS = 10;
const PROOF_SCAN_TIMEOUT_SECONDS = 120;
const INTERACTIVE_SCAN_BATCH_SIZE = 4;
const INTERACTIVE_SERVER_FOLLOWUP_LIMITS: Record<string, number> = {
  clap: 4,
  vst3: 6,
  au: 4,
  lv2: 4,
};
const INTERACTIVE_SCAN_CANDIDATE_LIMITS: Record<string, number> = {
  clap: 8,
  vst3: 12,
  au: 8,
  lv2: 12,
};

function readJson<T>(path: string): T {
  return JSON.parse(readFileSync(path, "utf8")) as T;
}

function writeJson(path: string, value: unknown): void {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, `${JSON.stringify(value, null, 2)}\n`);
}

function writeText(path: string, value: string): void {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, value);
}

function htmlEscape(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll("\"", "&quot;")
    .replaceAll("'", "&#39;");
}

function splitPaths(value: string | undefined): string[] {
  if (!value) {
    return [];
  }
  return value.split(process.platform === "win32" ? ";" : ":").filter(Boolean);
}

function existingPaths(paths: string[]): string[] {
  const discovered: string[] = [];
  for (const rawPath of paths) {
    const expanded = rawPath.startsWith("~")
      ? resolve(process.env.HOME ?? "", rawPath.slice(2))
      : rawPath;
    if (existsSync(expanded) && !discovered.includes(expanded)) {
      discovered.push(expanded);
    }
  }
  return discovered;
}

function dedupe(values: string[]): string[] {
  return [...new Set(values)];
}

function systemRootsByFormat(): Record<string, string[]> {
  const roots = {
    clap: splitPaths(process.env.SIGNAL_DEMO_CLAP_ROOTS),
    vst3: splitPaths(process.env.SIGNAL_DEMO_VST3_ROOTS),
    au: splitPaths(process.env.SIGNAL_DEMO_AU_ROOTS),
    lv2: splitPaths(process.env.SIGNAL_DEMO_LV2_ROOTS),
  };
  if (Object.values(roots).some((value) => value.length > 0)) {
    return Object.fromEntries(
      Object.entries(roots).map(([key, value]) => [key, dedupe(value)]),
    );
  }

  if (process.platform === "darwin") {
    roots.clap = existingPaths([
      "~/Library/Audio/Plug-Ins/CLAP",
      "/Library/Audio/Plug-Ins/CLAP",
    ]);
    roots.vst3 = existingPaths([
      "~/Library/Audio/Plug-Ins/VST3",
      "/Library/Audio/Plug-Ins/VST3",
    ]);
    roots.au = existingPaths([
      "~/Library/Audio/Plug-Ins/Components",
      "/Library/Audio/Plug-Ins/Components",
    ]);
  } else {
    roots.clap = existingPaths([
      "~/.clap",
      "~/.local/lib/clap",
      "/usr/local/lib/clap",
      "/usr/lib/clap",
    ]);
    roots.vst3 = existingPaths([
      "~/.vst3",
      "~/.local/share/vst3",
      "/usr/local/lib/vst3",
      "/usr/lib/vst3",
    ]);
    roots.lv2 = existingPaths([
      "~/.lv2",
      "~/.local/lib/lv2",
      "/usr/local/lib/lv2",
      "/usr/lib/lv2",
    ]);
  }
  return Object.fromEntries(
    Object.entries(roots).map(([key, value]) => [key, dedupe(value)]),
  );
}

function isExactPluginRoot(format: string, path: string): boolean {
  const suffix = extname(path).toLowerCase();
  if (format === "clap") {
    return suffix === ".clap";
  }
  if (format === "vst3") {
    return suffix === ".vst3";
  }
  if (format === "au") {
    return suffix === ".component";
  }
  if (format === "lv2") {
    return suffix === ".lv2";
  }
  return false;
}

function chunked(values: string[], size: number): string[][] {
  const chunks: string[][] = [];
  for (let index = 0; index < values.length; index += size) {
    chunks.push(values.slice(index, index + size));
  }
  return chunks;
}

function interactiveCandidateRoots(format: string, roots: string[]): string[] {
  const discovered: string[] = [];
  const limit = INTERACTIVE_SCAN_CANDIDATE_LIMITS[format] ?? 12;
  for (const root of roots) {
    if (!existsSync(root)) {
      continue;
    }
    if (isExactPluginRoot(format, root)) {
      if (!discovered.includes(root)) {
        discovered.push(root);
      }
      if (discovered.length >= limit) {
        break;
      }
      continue;
    }
    const entries = Array.from(new Bun.Glob("*").scanSync({ cwd: root }))
      .map((entry) => resolve(root, entry))
      .sort((left, right) => left.localeCompare(right));
    for (const entry of entries) {
      if (!isExactPluginRoot(format, entry)) {
        continue;
      }
      if (!discovered.includes(entry)) {
        discovered.push(entry);
      }
      if (discovered.length >= limit) {
        break;
      }
    }
    if (discovered.length >= limit) {
      break;
    }
  }
  return discovered;
}

function createVst3FixtureRoot(): { tempdir: string; pluginTypeId: string } {
  const tempdir = mkdtempSync(resolve(tmpdir(), "signal-plugin-browser-vst3-"));
  const bundleRoot = resolve(tempdir, "Signal Browser Instrument.vst3");
  const resourcesRoot = resolve(bundleRoot, "Contents/Resources");
  mkdirSync(resourcesRoot, { recursive: true });
  const pluginTypeId = "plugin:vst3:browser-fixture";
  const infoPlist = `<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
  <dict>
    <key>CFBundleName</key>
    <string>Signal Browser Instrument</string>
    <key>CFBundleIdentifier</key>
    <string>dev.signal.plugin.browser.fixture</string>
    <key>CFBundleVersion</key>
    <string>0.1.0</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundlePackageType</key>
    <string>BNDL</string>
    <key>CFBundleExecutable</key>
    <string>Signal Browser Instrument</string>
    <key>SignalPluginTypeId</key>
    <string>${pluginTypeId}</string>
    <key>SignalAudioInputs</key>
    <integer>0</integer>
    <key>SignalAudioOutputs</key>
    <integer>2</integer>
    <key>SignalMidiInputs</key>
    <integer>1</integer>
    <key>SignalMidiOutputs</key>
    <integer>0</integer>
    <key>SignalFeatures</key>
    <array>
      <string>Instrument</string>
      <string>Analyzer</string>
    </array>
  </dict>
</plist>
`;
  const moduleinfo = {
    Classes: [
      {
        CID: "7E1D8F8A4D874D56A2C44DE250199901",
        Category: "Audio Module Class",
        Name: "Signal Browser Instrument",
        Vendor: "Signal",
        Version: "0.1.0",
        SubCategories: ["Instrument", "Analyzer"],
        ClassFlags: 1,
        Snapshots: [],
      },
      {
        CID: "7E1D8F8A4D874D56A2C44DE250199902",
        Category: "Component Controller Class",
        Name: "Signal Browser Instrument Controller",
        Vendor: "Signal",
        Version: "0.1.0",
        SubCategories: [],
        ClassFlags: 1,
        Snapshots: [],
      },
    ],
  };
  writeText(resolve(bundleRoot, "Contents/Info.plist"), infoPlist);
  writeJson(resolve(resourcesRoot, "moduleinfo.json"), moduleinfo);
  return { tempdir, pluginTypeId };
}

function decodeJsonPayload(rawOutput: string): JsonObject {
  const stripped = rawOutput.trim();
  if (!stripped) {
    throw new Error("scan example produced no stdout");
  }
  try {
    return JSON.parse(stripped) as JsonObject;
  } catch {}
  const starts = [...stripped].flatMap((char, index) => (char === "{" || char === "[" ? [index] : []));
  if (starts.length === 0) {
    throw new Error(`scan example did not emit JSON: ${stripped.slice(0, 200)}`);
  }
  let lastError: unknown;
  for (const start of starts) {
    try {
      const payload = JSON.parse(stripped.slice(start)) as JsonObject;
      if (payload && typeof payload === "object" && !Array.isArray(payload)) {
        return payload;
      }
    } catch (error) {
      lastError = error;
    }
  }
  throw new Error(lastError instanceof Error ? lastError.message : "scan example JSON payload was not an object");
}

async function runScanExample(
  pkg: string,
  formats: string[],
  roots: string[],
  timeoutSeconds = 120,
): Promise<JsonObject> {
  const command = [
    "cargo",
    "run",
    "-q",
    "-p",
    pkg,
    "--example",
    `${pkg.replaceAll("-", "_")}_plugin_capability_scan`,
    "--",
  ];
  for (const format of formats) {
    command.push("--format", format);
  }
  for (const root of roots) {
    command.push("--root", root);
  }
  const child = spawn(command[0], command.slice(1), {
    cwd: REPO_ROOT,
    detached: true,
    stdio: ["ignore", "pipe", "pipe"],
    env: { ...process.env, CARGO_TARGET_DIR: "/tmp/signal-demo-target" },
  });
  let stdout = "";
  let stderr = "";
  child.stdout.setEncoding("utf8");
  child.stderr.setEncoding("utf8");
  child.stdout.on("data", (chunk) => {
    stdout += chunk;
  });
  child.stderr.on("data", (chunk) => {
    stderr += chunk;
  });
  const close = new Promise<number | null>((resolvePromise, reject) => {
    child.on("error", reject);
    child.on("close", resolvePromise);
  });
  const timer = setTimeout(() => {
    try {
      process.kill(-child.pid!, "SIGTERM");
    } catch {}
    setTimeout(() => {
      try {
        process.kill(-child.pid!, "SIGKILL");
      } catch {}
    }, 5000);
  }, timeoutSeconds * 1000);
  const code = await close;
  clearTimeout(timer);
  if (code !== 0) {
    throw new Error(`${command.join(" ")} failed with exit code ${code ?? "unknown"}\n${stderr}`);
  }
  try {
    return decodeJsonPayload(stdout);
  } catch (error) {
    throw new Error(
      `failed to decode ${pkg} scan inventory: ${error instanceof Error ? error.message : String(error)}\nstdout tail:\n${stdout.split(/\r?\n/).slice(-20).join("\n")}\nstderr tail:\n${stderr.split(/\r?\n/).slice(-20).join("\n")}`,
    );
  }
}

async function safeRunScanExample(
  pkg: string,
  formats: string[],
  roots: string[],
  timeoutSeconds = 120,
): Promise<[JsonObject | null, string | null]> {
  try {
    return [await runScanExample(pkg, formats, roots, timeoutSeconds), null];
  } catch (error) {
    return [null, error instanceof Error ? error.message : String(error)];
  }
}

async function collectScans(
  pkg: string,
  rootsByFormat: Record<string, string[]>,
  allowedFormats: string[],
  timeoutSeconds = 120,
  exactBatchMode = false,
): Promise<[JsonObject[], string[]]> {
  const scans: JsonObject[] = [];
  const failures: string[] = [];
  for (const format of allowedFormats) {
    const roots = rootsByFormat[format] ?? [];
    let rootGroups = roots.map((root) => [root]);
    if (exactBatchMode) {
      const candidateRoots = interactiveCandidateRoots(format, roots);
      if (candidateRoots.length > 0) {
        rootGroups = chunked(candidateRoots, INTERACTIVE_SCAN_BATCH_SIZE);
      }
    }
    for (const rootGroup of rootGroups) {
      const [scan, error] = await safeRunScanExample(pkg, [format], rootGroup, timeoutSeconds);
      if (scan) {
        scans.push(scan);
      } else if (error) {
        failures.push(`${pkg} ${format} scan failed for ${rootGroup.join(", ")}: ${error}`);
      }
    }
  }
  return [scans, failures];
}

function renderedScanFormat(pluginFormat: string): string {
  return pluginFormat.toLowerCase();
}

function exactLaunchRoot(plugin: JsonObject): string | null {
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

function localProbeSortKey(plugin: JsonObject): [number, string, string, string] {
  const ranks: Record<string, number> = { Clap: 0, Vst3: 1, Au: 2 };
  return [
    ranks[String(plugin.format)] ?? 99,
    String(plugin.vendor),
    String(plugin.name),
    String(plugin.plugin_type_id),
  ];
}

async function attachContainedLocalTargets(inventory: JsonObject[]): Promise<JsonObject> {
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

function canRunLocalSurface(): boolean {
  return process.platform === "darwin";
}

function launchCommand(pkg: string): string[] {
  return ["cargo", "run", "-q", "-p", pkg];
}

function launchEnv(plugin: JsonObject): Record<string, string> {
  return {
    SIGNAL_HOST_DEMO_PLUGIN_FORMAT: String(plugin.format).toLowerCase(),
    SIGNAL_HOST_DEMO_PLUGIN_ROOT: String(plugin.launch_root),
    SIGNAL_HOST_DEMO_PLUGIN_TYPE_ID: String(plugin.plugin_type_id),
    SIGNAL_HOST_DEMO_INTERACTION_MODE: "parameter-step",
    SIGNAL_HOST_DEMO_INTERACTION_VALUE: "0.73",
    CARGO_TARGET_DIR: "/tmp/signal-demo-target",
  };
}

function summaryToken(summaryLine: string, key: string): string | null {
  const marker = `${key}=`;
  const start = summaryLine.indexOf(marker);
  if (start < 0) {
    return null;
  }
  let value = summaryLine.slice(start + marker.length);
  if (value.includes(" ")) {
    value = value.split(" ", 1)[0]!;
  }
  return value;
}

function parseInteractionSummary(summaryLine: string): JsonObject {
  const interactionMode = summaryToken(summaryLine, "interaction_mode");
  const automationValue = summaryToken(summaryLine, "automation_value");
  const parameterEvents = summaryToken(summaryLine, "parameter_events");
  const generatedEventBytes = summaryToken(summaryLine, "generated_event_bytes");
  return {
    interaction_mode: interactionMode,
    interaction_value: automationValue,
    parameter_event_count: parameterEvents,
    generated_event_bytes: generatedEventBytes,
    interaction_proved:
      interactionMode !== null &&
      interactionMode !== "none" &&
      automationValue !== null &&
      automationValue !== "None" &&
      parameterEvents !== null &&
      parameterEvents !== "0",
  };
}

function decodeSubprocessStream(value: string | Buffer | null | undefined): string {
  if (value == null) {
    return "";
  }
  return typeof value === "string" ? value : value.toString("utf8");
}

function runLaunch(pkg: string, plugin: JsonObject, timeoutSeconds = 15): JsonObject {
  const command = launchCommand(pkg);
  const result = spawnSync(command[0], command.slice(1), {
    cwd: REPO_ROOT,
    encoding: "utf8",
    env: { ...process.env, ...launchEnv(plugin) },
    timeout: timeoutSeconds * 1000,
  });
  if (result.error && (result.error as any).code === "ETIMEDOUT") {
    const stdoutText = decodeSubprocessStream(result.stdout);
    const stderrText = decodeSubprocessStream(result.stderr);
    return {
      package: pkg,
      plugin_type_id: plugin.plugin_type_id,
      format: plugin.format,
      launch_root: plugin.launch_root,
      status: "failed",
      exit_code: null,
      command: command.join(" "),
      summary_line: "",
      stdout_tail: stdoutText.split(/\r?\n/).slice(-20),
      stderr_tail: stderrText.split(/\r?\n/).slice(-20),
      failure_kind: "timeout",
    };
  }
  let summaryLine = "";
  for (const line of String(result.stdout ?? "").split(/\r?\n/)) {
    if (line.startsWith(pkg)) {
      summaryLine = line;
      break;
    }
  }
  return {
    package: pkg,
    plugin_type_id: plugin.plugin_type_id,
    format: plugin.format,
    launch_root: plugin.launch_root,
    status: result.status === 0 ? "passed" : "failed",
    exit_code: result.status,
    command: command.join(" "),
    summary_line: summaryLine,
    stdout_tail: String(result.stdout ?? "").split(/\r?\n/).slice(-20),
    stderr_tail: String(result.stderr ?? "").split(/\r?\n/).slice(-20),
    ...parseInteractionSummary(summaryLine),
  };
}

function combineInventory(scans: JsonObject[]): JsonObject[] {
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

function choosePrimaryLaunch(inventory: JsonObject[]): JsonObject | null {
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

function executePrimaryLaunch(inventory: JsonObject[]): JsonObject | null {
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

function preferredServerRootsFromInventory(inventory: JsonObject[]): Record<string, string[]> {
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

function browserHtml(browserModel: JsonObject): string {
  const rows = (browserModel.inventory as JsonObject[]).map((plugin) => {
    const featureList = (plugin.features as string[]).join(", ") || "none";
    const localAvailable = (plugin.launch_targets as JsonObject[]).some((target) => target.host_surface === "local");
    const serverAvailable = (plugin.launch_targets as JsonObject[]).some((target) => target.host_surface === "server");
    const availability: string[] = [];
    if (localAvailable) availability.push('<span class="pill pill-local">Local</span>');
    if (serverAvailable) availability.push('<span class="pill pill-server">Server</span>');
    if (availability.length === 0) availability.push('<span class="pill pill-none">No launch</span>');
    let posture = "no bounded launch";
    if (localAvailable && serverAvailable) posture = "bounded local + server";
    else if (localAvailable) posture = "bounded local only";
    else if (serverAvailable) posture = "bounded server only";
    const launchCells = (plugin.launch_targets as JsonObject[]).map((target) => {
      const payload = htmlEscape(JSON.stringify(target));
      return `<button class="launch" data-launch='${payload}'>Launch ${htmlEscape(String(target.host_surface))}</button>`;
    });
    if (launchCells.length === 0) {
      launchCells.push("<span class=\"muted\">No bounded launch target</span>");
    }
    return "<tr>"
      + `<td>${htmlEscape(String(plugin.format))}</td>`
      + `<td>${htmlEscape(String(plugin.name))}</td>`
      + `<td>${htmlEscape(String(plugin.vendor))}</td>`
      + `<td><code>${htmlEscape(String(plugin.plugin_type_id))}</code></td>`
      + `<td>${htmlEscape(featureList)}</td>`
      + `<td>${availability.join("")}</td>`
      + `<td>${htmlEscape(posture)}<div class="muted">${htmlEscape(String(plugin.interaction_posture))}</div></td>`
      + `<td>${launchCells.join("")}</td>`
      + "</tr>";
  }).join("");
  const exclusionList = (browserModel.known_exclusions as string[])
    .map((note) => `<li>${htmlEscape(note)}</li>`)
    .join("");
  const stats = browserModel.stats as JsonObject;
  const embedded = htmlEscape(JSON.stringify(browserModel));
  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Signal Plugin Capability Browser</title>
  <style>
    :root {
      color-scheme: light;
      --ink: #161616;
      --muted: #6d6a63;
      --paper: #f3efe5;
      --panel: #fffaf1;
      --line: #d8cfbe;
      --accent: #0f5a52;
      --accent-soft: #d4ebe8;
      --warn: #8a4f00;
    }
    body { margin: 0; font-family: "Iowan Old Style", "Palatino Linotype", serif; background: radial-gradient(circle at top left, #fffdf7, var(--paper) 56%); color: var(--ink); }
    main { max-width: 1120px; margin: 0 auto; padding: 32px 24px 64px; }
    h1, h2 { font-family: "Avenir Next Condensed", "Helvetica Neue", sans-serif; letter-spacing: 0.02em; }
    .hero { display: grid; gap: 16px; padding: 24px; border: 1px solid var(--line); background: linear-gradient(135deg, #fffdf7, var(--panel)); box-shadow: 0 16px 32px rgba(22, 22, 22, 0.06); }
    .stats { display: flex; flex-wrap: wrap; gap: 12px; }
    .stat { padding: 10px 12px; border: 1px solid var(--line); background: var(--accent-soft); font-family: "Avenir Next", sans-serif; }
    .muted { color: var(--muted); }
    .pill { display: inline-block; margin-right: 8px; margin-bottom: 6px; padding: 4px 8px; border-radius: 999px; font-family: "Avenir Next", sans-serif; font-size: 0.78rem; border: 1px solid var(--line); background: white; }
    .pill-local { border-color: var(--accent); background: var(--accent-soft); color: var(--accent); }
    .pill-server { border-color: #5f6f8c; background: #edf2fb; color: #31415c; }
    .pill-none { border-color: #b7ac98; background: #f5efe4; color: #6d6a63; }
    table { width: 100%; border-collapse: collapse; margin-top: 24px; background: var(--panel); }
    th, td { padding: 12px 10px; border-bottom: 1px solid var(--line); vertical-align: top; text-align: left; }
    th { font-family: "Avenir Next", sans-serif; font-size: 0.85rem; text-transform: uppercase; letter-spacing: 0.04em; }
    button.launch { margin-right: 8px; margin-bottom: 8px; border: 1px solid var(--accent); background: white; color: var(--accent); padding: 8px 10px; cursor: pointer; font-family: "Avenir Next", sans-serif; }
    pre { white-space: pre-wrap; background: #111; color: #f5f0e4; padding: 16px; border-radius: 8px; min-height: 72px; }
    code { font-family: "SFMono-Regular", "Menlo", monospace; font-size: 0.9em; }
    .launch-status { margin-bottom: 10px; padding: 10px 12px; border: 1px solid var(--line); font-family: "Avenir Next", sans-serif; background: #f8f4ea; }
    .launch-status.passed { border-color: var(--accent); background: var(--accent-soft); color: var(--accent); }
    .launch-status.failed { border-color: #8c3c28; background: #fbeae5; color: #7b2c18; }
    .launch-summary { margin-bottom: 14px; font-family: "Avenir Next", sans-serif; }
    details.launch-detail { margin-top: 10px; }
    .callout { margin-top: 24px; padding: 16px 18px; border-left: 4px solid var(--warn); background: #fff5e8; }
  </style>
</head>
<body>
  <main>
    <section class="hero">
      <div class="muted">signal.demo.plugin.capability-browser</div>
      <h1>Signal Plugin Capability Browser</h1>
      <p>Browse real discovered plugin inventory and launch one bounded supported host path without pulling a heavyweight UI stack into the repo.</p>
      <div class="stats">
        <div class="stat">plugins: ${stats.plugin_count}</div>
        <div class="stat">formats: ${stats.format_count}</div>
        <div class="stat">launch targets: ${stats.launch_target_count}</div>
        <div class="stat">fixture fallback: ${String(stats.fixture_fallback_used).toLowerCase()}</div>
      </div>
      <div class="muted">HTML artifact: <code>${htmlEscape("demos/receipts/plugin-capability-browser.view.html")}</code></div>
    </section>
    <section><h2>Known exclusions</h2><ul>${exclusionList}</ul></section>
    <section>
      <h2>Discovered plugins</h2>
      <table>
        <thead><tr><th>Format</th><th>Name</th><th>Vendor</th><th>Plugin Type</th><th>Features</th><th>Availability</th><th>Interaction</th><th>Launch</th></tr></thead>
        <tbody>${rows}</tbody>
      </table>
    </section>
    <section>
      <h2>Launch output</h2>
      <div id="launch-status" class="launch-status muted">No launch run yet.</div>
      <div id="launch-summary" class="launch-summary muted">Launch a plugin from the browser to capture a bounded host result.</div>
      <details class="launch-detail"><summary>Bounded host detail</summary><pre id="launch-output">Launch a plugin from the browser to capture the bounded host summary here.</pre></details>
    </section>
    <section class="callout">
      <strong>Serving note:</strong> launch buttons only work while the browser is served through the repo-owned wrapper. The static HTML artifact remains useful for visual inspection and audit capture.
    </section>
  </main>
  <script type="application/json" id="browser-model">${embedded}</script>
  <script>
    const launchStatus = document.getElementById("launch-status");
    const launchSummary = document.getElementById("launch-summary");
    const output = document.getElementById("launch-output");
    for (const button of document.querySelectorAll("button.launch")) {
      button.addEventListener("click", async () => {
        const payload = JSON.parse(button.dataset.launch);
        launchStatus.className = "launch-status muted";
        launchStatus.textContent = \`Launching \${payload.host_surface} \${payload.format} \${payload.plugin_type_id}...\`;
        launchSummary.textContent = \`Root: \${payload.launch_root}\`;
        output.textContent = \`Launching \${payload.host_surface} \${payload.format} \${payload.plugin_type_id}...\`;
        try {
          const response = await fetch("/launch", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(payload),
          });
          const result = await response.json();
          const failureKind = result.failure_kind ? \` (\${result.failure_kind})\` : "";
          const interactionBits = [];
          if (result.interaction_mode && result.interaction_mode !== "none") interactionBits.push(\`interaction=\${result.interaction_mode}\`);
          if (result.interaction_value && result.interaction_value !== "None") interactionBits.push(\`value=\${result.interaction_value}\`);
          if (result.parameter_event_count && result.parameter_event_count !== "0") interactionBits.push(\`parameter_events=\${result.parameter_event_count}\`);
          launchStatus.className = \`launch-status \${result.status === "passed" ? "passed" : "failed"}\`;
          launchStatus.textContent = \`\${result.status.toUpperCase()}\${failureKind}: \${result.package} -> \${result.plugin_type_id}\`;
          launchSummary.textContent = interactionBits.length > 0 ? interactionBits.join(" | ") : (result.summary_line || \`Launch root: \${result.launch_root}\`);
          output.textContent = JSON.stringify(result, null, 2);
        } catch (error) {
          launchStatus.className = "launch-status failed";
          launchStatus.textContent = "FAILED: browser launch request";
          launchSummary.textContent = String(error);
          output.textContent = String(error);
        }
      });
    }
  </script>
</body>
</html>`;
}

function receiptFromRun(manifest: JsonObject, browserModel: JsonObject, primaryLaunch: JsonObject | null): JsonObject {
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

type BrowserHandlerState = {
  browserModel: JsonObject;
};

function bindBrowserServer(state: BrowserHandlerState, preferredPort: number) {
  return new Promise<{ server: ReturnType<typeof createServer>; port: number }>((resolvePromise, reject) => {
    const tryPort = (port: number) => {
      const server = createServer(async (req: IncomingMessage, res: ServerResponse) => {
        await handleRequest(state, req, res);
      });
      server.listen(port, "127.0.0.1");
      server.once("listening", () => resolvePromise({ server, port }));
      server.once("error", (error: any) => {
        server.close();
        if (error?.code === "EADDRINUSE" && port < preferredPort + 19) {
          tryPort(port + 1);
          return;
        }
        reject(error);
      });
    };
    tryPort(preferredPort);
  });
}

async function handleRequest(state: BrowserHandlerState, req: IncomingMessage, res: ServerResponse) {
  const url = req.url ?? "/";
  if (req.method === "GET" && (url === "/" || url === "/index.html")) {
    const body = Buffer.from(browserHtml(state.browserModel), "utf8");
    res.writeHead(200, {
      "Content-Type": "text/html; charset=utf-8",
      "Content-Length": body.length,
    });
    res.end(body);
    return;
  }
  if (req.method === "POST" && url === "/launch") {
    const chunks: Buffer[] = [];
    for await (const chunk of req) {
      chunks.push(Buffer.from(chunk));
    }
    const payload = JSON.parse(Buffer.concat(chunks).toString("utf8") || "{}") as JsonObject;
    const pkg = "signal-host-local";
    const result = runLaunch(pkg, payload);
    const body = Buffer.from(JSON.stringify(result, null, 2), "utf8");
    res.writeHead(200, {
      "Content-Type": "application/json; charset=utf-8",
      "Content-Length": body.length,
    });
    res.end(body);
    return;
  }
  res.writeHead(404);
  res.end();
}

function parseArgs(argv: string[]) {
  const args = {
    serve: false,
    noOpen: false,
    port: 8765,
    scanMode: "auto",
  };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index]!;
    if (arg === "--serve") args.serve = true;
    else if (arg === "--no-open") args.noOpen = true;
    else if (arg === "--port") args.port = Number.parseInt(argv[++index]!, 10);
    else if (arg === "--scan-mode") args.scanMode = argv[++index]!;
  }
  return args;
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  const manifest = readJson<JsonObject>(MANIFEST_PATH);
  const interactiveMode = args.serve || process.stdout.isTTY;
  let scanMode = args.scanMode;
  if (scanMode === "auto") {
    scanMode = interactiveMode ? "system" : "fixture";
  }
  const rootsByFormat = scanMode === "system" || scanMode === "hybrid"
    ? systemRootsByFormat()
    : { clap: [], vst3: [], au: [], lv2: [] } as Record<string, string[]>;
  let fixtureTempdir: string | null = null;
  let fixtureFallbackUsed = false;
  if (scanMode === "fixture" || scanMode === "hybrid") {
    const fixture = createVst3FixtureRoot();
    fixtureTempdir = fixture.tempdir;
    rootsByFormat.vst3 = dedupe([...rootsByFormat.vst3, fixture.tempdir]);
    fixtureFallbackUsed = true;
  }
  const scanTimeoutSeconds = scanMode === "system" ? SYSTEM_SCAN_TIMEOUT_SECONDS : PROOF_SCAN_TIMEOUT_SECONDS;
  const exactBatchMode = scanMode === "system";
  const scans: JsonObject[] = [];
  const scanFailures: string[] = [];
  let localFirstInventory: JsonObject[] = [];
  if (scanMode === "system" && canRunLocalSurface()) {
    const [localScans, localFailures] = await collectScans(
      "signal-host-local",
      rootsByFormat,
      ["clap", "vst3", "au"],
      scanTimeoutSeconds,
      exactBatchMode,
    );
    scans.push(...localScans);
    scanFailures.push(...localFailures);
    localFirstInventory = combineInventory(localScans);
  }
  let serverRootsByFormat = rootsByFormat;
  if (scanMode === "system" && localFirstInventory.length > 0) {
    serverRootsByFormat = preferredServerRootsFromInventory(localFirstInventory);
  }
  const [serverScans, serverFailures] = await collectScans(
    serverRootsByFormat,
    ["clap", "vst3", "au", "lv2"],
    scanTimeoutSeconds,
    exactBatchMode,
  );
  scans.push(...serverScans);
  scanFailures.push(...serverFailures);
  let inventory = combineInventory(scans);
  let localProbeSummary = await attachContainedLocalTargets(inventory);
  if (scanMode === "system" && !choosePrimaryLaunch(inventory)) {
    const fixture = createVst3FixtureRoot();
    fixtureTempdir = fixture.tempdir;
    rootsByFormat.vst3 = dedupe([...rootsByFormat.vst3, fixture.tempdir]);
    scans.length = 0;
    if (canRunLocalSurface()) {
      const [localScans, localFailures] = await collectScans(
        "signal-host-local",
        rootsByFormat,
        ["clap", "vst3", "au"],
        scanTimeoutSeconds,
        exactBatchMode,
      );
      scans.push(...localScans);
      scanFailures.push(...localFailures.map((failure) =>
        failure.replace("signal-host-local ", "signal-host-local after fixture fallback ")));
      localFirstInventory = combineInventory(localScans);
    }
    serverRootsByFormat = localFirstInventory.length > 0
      ? preferredServerRootsFromInventory(localFirstInventory)
      : rootsByFormat;
    const [fallbackScans, fallbackFailures] = await collectScans(
        serverRootsByFormat,
      ["clap", "vst3", "au", "lv2"],
      scanTimeoutSeconds,
      exactBatchMode,
    );
    scans.push(...fallbackScans);
    scanFailures.push(...fallbackFailures.map((failure) =>
      failure));
    inventory = combineInventory(scans);
    localProbeSummary = await attachContainedLocalTargets(inventory);
    fixtureFallbackUsed = true;
  }

  const knownExclusions = [
    "The browser launches bounded host bootstrap paths, not embedded vendor plugin editors.",
    "Local host launch is macOS-only; server host launch remains explicit when local host is unavailable.",
    "LV2 launch targets remain server-host only.",
    "If no suitable installed CLAP or VST3 plugin is found, the official proof task falls back to one bounded temporary VST3 fixture root so the browser shell itself stays testable.",
  ];
  if (exactBatchMode) {
    knownExclusions.push(
      "Interactive system scans now use bounded exact-root batches with per-format candidate caps so one problematic plugin directory does not blank the whole browser.",
    );
    if (canRunLocalSurface()) {
      knownExclusions.push(
        "Interactive macOS runs prefer bounded local inventory first and only widen server scans across locally confirmed plugin roots when available.",
      );
    }
  }
  if (localProbeSummary.attempted === 0) {
    knownExclusions.push(
      "Local launch buttons are only shown for plugins confirmed by bounded exact-root local probes.",
    );
  } else if (localProbeSummary.failed > 0) {
    knownExclusions.push(
      `Some local plugin probes failed or timed out during bounded exact-root validation, so local buttons are shown only for ${localProbeSummary.succeeded} confirmed plugins out of ${localProbeSummary.attempted} attempts.`,
    );
    for (const failure of localProbeSummary.failures as string[]) {
      knownExclusions.push(`Local probe note: ${failure}`);
    }
    if (localProbeSummary.limit_hit) {
      knownExclusions.push(
        `Local probe containment stopped after ${LOCAL_PROBE_ATTEMPT_LIMIT} attempts to keep the interactive browser responsive.`,
      );
    }
  }
  for (const failure of scanFailures) {
    knownExclusions.push(`Scan containment note: ${failure.slice(0, 500)}`);
  }

  const browserModel = {
    platform: process.platform,
    scan_roots: rootsByFormat,
    inventory,
    known_exclusions: knownExclusions,
    stats: {
      plugin_count: inventory.length,
      format_count: new Set(inventory.map((plugin) => plugin.format)).size,
      launch_target_count: inventory.reduce((sum, plugin) => sum + plugin.launch_targets.length, 0),
      fixture_fallback_used: fixtureFallbackUsed,
      local_probe_attempted: localProbeSummary.attempted,
      local_probe_succeeded: localProbeSummary.succeeded,
    },
  };

  writeText(HTML_PATH, browserHtml(browserModel));
  const primaryLaunch = executePrimaryLaunch(inventory);
  const receipt = receiptFromRun(manifest, browserModel, primaryLaunch);
  writeJson(RECEIPT_PATH, receipt);

  if (interactiveMode) {
    const state: BrowserHandlerState = { browserModel };
    const { server, port } = await bindBrowserServer(state, args.port);
    const shutdown = () => server.close();
    process.on("SIGINT", shutdown);
    process.on("SIGTERM", shutdown);
    if (!args.noOpen) {
      spawn("open", [`http://127.0.0.1:${port}/`], {
        cwd: REPO_ROOT,
        stdio: "ignore",
        detached: true,
      }).unref();
    }
    console.log(`signal plugin capability browser serving on http://127.0.0.1:${port}/`);
    await new Promise<void>((resolvePromise) => server.on("close", () => resolvePromise()));
  }

  if (fixtureTempdir) {
    rmSync(fixtureTempdir, { recursive: true, force: true });
  }
}

await main();
