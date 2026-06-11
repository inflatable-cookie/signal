import { spawn, spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

export type OperatorCheck = {
  id: string;
  status: string;
  summary: string;
};

export type Receipt = {
  receipt_version: string;
  manifest_id: string;
  scenario_id: string;
  status: string;
  launch_command: string;
  artifacts: Array<Record<string, unknown>>;
  operator_checks: OperatorCheck[];
};

export type CompletedCommand = {
  command: string[];
  stdout: string;
  stderr: string;
};

const LIB_DIR = dirname(fileURLToPath(import.meta.url));
export const REPO_ROOT = resolve(LIB_DIR, "../../..");
const DEMO_TARGET_DIR = "/tmp/signal-demo-target";
const DEMO_ENV = { ...process.env, CARGO_TARGET_DIR: DEMO_TARGET_DIR };

export function readJson<T>(path: string): T {
  return JSON.parse(readFileSync(resolve(REPO_ROOT, path), "utf8")) as T;
}

export function writeJson(path: string, value: unknown): void {
  const fullPath = resolve(REPO_ROOT, path);
  mkdirSync(dirname(fullPath), { recursive: true });
  writeFileSync(fullPath, `${JSON.stringify(value, null, 2)}\n`);
}

export function writeText(path: string, value: string): void {
  const fullPath = resolve(REPO_ROOT, path);
  mkdirSync(dirname(fullPath), { recursive: true });
  writeFileSync(fullPath, value);
}

export function runCommand(command: string[], input?: string): CompletedCommand {
  const result = spawnSync(command[0], command.slice(1), {
    cwd: REPO_ROOT,
    encoding: "utf8",
    input,
    env: DEMO_ENV,
  });
  if (result.status !== 0) {
    throw new Error(
      `${command.join(" ")} failed with exit code ${result.status ?? "unknown"}\n${result.stderr}`,
    );
  }
  return {
    command,
    stdout: result.stdout,
    stderr: result.stderr,
  };
}

export async function runCommandWithTimeout(
  command: string[],
  timeoutMs: number,
): Promise<CompletedCommand> {
  return await new Promise<CompletedCommand>((resolvePromise, reject) => {
    const child = spawn(command[0], command.slice(1), {
      cwd: REPO_ROOT,
      detached: true,
      stdio: ["ignore", "pipe", "pipe"],
      env: DEMO_ENV,
    });

    let stdout = "";
    let stderr = "";
    let settled = false;

    const finish = (error?: Error) => {
      if (settled) {
        return;
      }
      settled = true;
      clearTimeout(timer);
      if (error) {
        reject(error);
        return;
      }
      resolvePromise({ command, stdout, stderr });
    };

    child.stdout.setEncoding("utf8");
    child.stderr.setEncoding("utf8");
    child.stdout.on("data", (chunk: string) => {
      stdout += chunk;
    });
    child.stderr.on("data", (chunk: string) => {
      stderr += chunk;
    });
    child.on("error", (error) => finish(error));
    child.on("close", (code) => {
      if (code !== 0) {
        finish(
          new Error(
            `${command.join(" ")} failed with exit code ${code ?? "unknown"}\n${stderr}`,
          ),
        );
        return;
      }
      finish();
    });

    const timer = setTimeout(() => {
      try {
        process.kill(-child.pid, "SIGTERM");
      } catch {}
      setTimeout(() => {
        try {
          process.kill(-child.pid, "SIGKILL");
        } catch {}
      }, 2000);
    }, timeoutMs);
  });
}

export function stdoutTail(result: CompletedCommand, lineCount = 20): string[] {
  return result.stdout.split(/\r?\n/).filter(Boolean).slice(-lineCount);
}

export function parseKeyValueLines(output: string): Record<string, string> {
  const parsed: Record<string, string> = {};
  for (const line of output.split(/\r?\n/)) {
    const index = line.indexOf("=");
    if (index === -1) {
      continue;
    }
    const key = line.slice(0, index).trim();
    const value = line.slice(index + 1).trim();
    parsed[key] = value;
  }
  return parsed;
}

export function extractFirst(line: string, key: string): string | undefined {
  const match = new RegExp(`\\b${escapeRegex(key)}=(".*?"|\\[[^\\]]*\\]|\\S+)`).exec(
    line,
  );
  if (!match) {
    return undefined;
  }
  const value = match[1];
  return value.startsWith("\"") && value.endsWith("\"")
    ? value.slice(1, -1)
    : value;
}

export function parseSummaryLine(line: string): Record<string, string> {
  const parsed: Record<string, string> = {};
  const pairRegex = /([A-Za-z0-9_]+)=(".*?"|\[[^\]]*\]|\S+)/g;
  for (const match of line.matchAll(pairRegex)) {
    const [, key, rawValue] = match;
    parsed[key] =
      rawValue.startsWith("\"") && rawValue.endsWith("\"")
        ? rawValue.slice(1, -1)
        : rawValue;
  }
  return parsed;
}

export function asInt(value: string | undefined): number {
  const parsed = Number.parseInt(value ?? "0", 10);
  return Number.isNaN(parsed) ? 0 : parsed;
}

export function asBool(value: string | undefined): boolean {
  return value === "true";
}

export function exampleArtifact(
  result: CompletedCommand,
  kind: string,
): Record<string, unknown> {
  return {
    kind,
    command: result.command.join(" "),
    status: "passed",
    stdout_tail: stdoutTail(result),
  };
}

export function acceptanceArtifact(
  result: CompletedCommand,
  commandLabel?: string,
): Record<string, unknown> {
  return {
    kind: "acceptance-lane-run",
    command: commandLabel ?? result.command.join(" "),
    status: "passed",
    stdout_tail: stdoutTail(result),
  };
}

export function descriptorArtifact(
  kind: string,
  payload: Record<string, unknown>,
): Record<string, unknown> {
  const deferredScope = Array.isArray(payload.deferred_scope)
    ? payload.deferred_scope
    : [];
  const validationSteps = Array.isArray(payload.validation_steps)
    ? payload.validation_steps
    : [];
  return {
    kind,
    boundary: payload.boundary,
    contract_path: payload.contract_path,
    acceptance_task: payload.acceptance_task,
    surface_count: payload.surface_count,
    validation_step_count:
      payload.validation_step_count ?? validationSteps.length,
    deferred_scope_count:
      deferredScope.length || (payload.residual_risk ? 1 : 0),
    raw_payload: payload,
  };
}

export function runDescriptor(flag: string): {
  command: string;
  payload: Record<string, unknown>;
} {
  const result = runCommand([
    "cargo",
    "run",
    "-q",
    "-p",
    "--",
    flag,
    "--format=json",
  ]);
  return {
    command: result.command.join(" "),
    payload: JSON.parse(result.stdout) as Record<string, unknown>,
  };
}

export function findSummaryLine(output: string, prefix: string): string {
  const line = output
    .split(/\r?\n/)
    .map((value) => value.trim())
    .find((value) => value.startsWith(prefix));
  if (!line) {
    throw new Error(`${prefix} did not emit a summary line`);
  }
  return line;
}

export async function runHostSummary(
  packageName: string,
  timeoutMs: number,
): Promise<{ package: string; line: string; timed_out: boolean }> {
  const command = ["cargo", "run", "-q", "-p", packageName];
  const child = spawn(command[0], command.slice(1), {
    cwd: REPO_ROOT,
    detached: true,
    stdio: ["ignore", "pipe", "pipe"],
    env: command[0] === "cargo"
      ? { ...process.env, CARGO_TARGET_DIR: DEMO_TARGET_DIR }
      : process.env,
  });
  let stdout = "";
  let stderr = "";
  let timedOut = false;

  child.stdout.setEncoding("utf8");
  child.stderr.setEncoding("utf8");
  child.stdout.on("data", (chunk: string) => {
    stdout += chunk;
  });
  child.stderr.on("data", (chunk: string) => {
    stderr += chunk;
  });

  const closePromise = new Promise<number | null>((resolvePromise, reject) => {
    child.on("error", reject);
    child.on("close", resolvePromise);
  });

  const timer = setTimeout(() => {
    timedOut = true;
    try {
      process.kill(-child.pid, "SIGTERM");
    } catch {}
    setTimeout(() => {
      try {
        process.kill(-child.pid, "SIGKILL");
      } catch {}
    }, 2000);
  }, timeoutMs);

  const code = await closePromise;
  clearTimeout(timer);

  if (!timedOut && code !== 0) {
    throw new Error(`${command.join(" ")} failed with exit code ${code ?? "unknown"}\n${stderr}`);
  }

  const line = stdout
    .split(/\r?\n/)
    .map((value) => value.trim())
    .find((value) => value.startsWith(packageName));
  if (!line) {
    throw new Error(`${packageName} did not emit a summary line before exit`);
  }

  return {
    package: packageName,
    line,
    timed_out: timedOut,
  };
}

export function hasAllTokens(output: string, required: string[]): boolean {
  return required.every((token) => output.includes(token));
}

export function ensureExists(path: string): boolean {
  return existsSync(resolve(REPO_ROOT, path));
}

function escapeRegex(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
