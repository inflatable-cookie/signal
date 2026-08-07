import { spawn } from "node:child_process";
import { rmSync } from "node:fs";
import process from "node:process";
import { parseArgs } from "./plugin_capability_browser/args.ts";
import {
  LOCAL_PROBE_ATTEMPT_LIMIT,
} from "./plugin_capability_browser/constants.ts";
import { createVst3FixtureRoot } from "./plugin_capability_browser/fixtures.ts";
import { browserHtml } from "./plugin_capability_browser/html.ts";
import { readJson, writeJson, writeText } from "./plugin_capability_browser/io.ts";
import {
  choosePrimaryLaunch,
  combineInventory,
  executePrimaryLaunch,
  preferredServerRootsFromInventory,
} from "./plugin_capability_browser/inventory.ts";
import {
  HTML_PATH,
  MANIFEST_PATH,
  RECEIPT_PATH,
  REPO_ROOT,
  type JsonObject,
} from "./plugin_capability_browser/paths.ts";
import { attachContainedLocalTargets, canRunLocalSurface } from "./plugin_capability_browser/probe.ts";
import { receiptFromRun } from "./plugin_capability_browser/receipt.ts";
import { dedupe, systemRootsByFormat } from "./plugin_capability_browser/roots.ts";
import {
  collectScans,
  PROOF_SCAN_TIMEOUT_SECONDS,
  SYSTEM_SCAN_TIMEOUT_SECONDS,
} from "./plugin_capability_browser/scan.ts";
import { bindBrowserServer } from "./plugin_capability_browser/server.ts";

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
    "signal-host-server",
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
      "signal-host-server",
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
    const state = { browserModel };
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
