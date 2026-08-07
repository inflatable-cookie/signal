import { existsSync } from "node:fs";
import { extname, resolve } from "node:path";
import process from "node:process";

export const INTERACTIVE_SCAN_BATCH_SIZE = 4;
export const INTERACTIVE_SCAN_CANDIDATE_LIMITS: Record<string, number> = {
  clap: 8,
  vst3: 12,
  au: 8,
  lv2: 12,
};

export function splitPaths(value: string | undefined): string[] {
  if (!value) {
    return [];
  }
  return value.split(process.platform === "win32" ? ";" : ":").filter(Boolean);
}

export function existingPaths(paths: string[]): string[] {
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

export function dedupe(values: string[]): string[] {
  return [...new Set(values)];
}

export function systemRootsByFormat(): Record<string, string[]> {
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

export function isExactPluginRoot(format: string, path: string): boolean {
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

export function chunked(values: string[], size: number): string[][] {
  const chunks: string[][] = [];
  for (let index = 0; index < values.length; index += size) {
    chunks.push(values.slice(index, index + size));
  }
  return chunks;
}

export function interactiveCandidateRoots(format: string, roots: string[]): string[] {
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
