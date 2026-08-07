import { spawnSync } from "node:child_process";
import process from "node:process";
import { REPO_ROOT, type JsonObject } from "./paths.ts";

export function launchCommand(pkg: string): string[] {
  return ["cargo", "run", "-q", "-p", pkg];
}

export function launchEnv(plugin: JsonObject): Record<string, string> {
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

export function runLaunch(pkg: string, plugin: JsonObject, timeoutSeconds = 15): JsonObject {
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
