import { type JsonObject } from "./paths.ts";

export function htmlEscape(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll("\"", "&quot;")
    .replaceAll("'", "&#39;");
}

export function browserHtml(browserModel: JsonObject): string {
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
