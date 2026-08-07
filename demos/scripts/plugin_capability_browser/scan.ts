import { spawn } from "node:child_process";
import process from "node:process";
import { REPO_ROOT, type JsonObject } from "./paths.ts";
import {
  chunked,
  interactiveCandidateRoots,
  INTERACTIVE_SCAN_BATCH_SIZE,
} from "./roots.ts";

export const SYSTEM_SCAN_TIMEOUT_SECONDS = 10;
export const PROOF_SCAN_TIMEOUT_SECONDS = 120;

export function decodeJsonPayload(rawOutput: string): JsonObject {
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

export async function runScanExample(
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

export async function safeRunScanExample(
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

export async function collectScans(
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

export function renderedScanFormat(pluginFormat: string): string {
  return pluginFormat.toLowerCase();
}
