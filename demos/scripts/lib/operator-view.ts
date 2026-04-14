import type { OperatorCheck } from "./demo-runtime.ts";

export type MetricSection = {
  title: string;
  subtitle: string;
  items: Array<[string, string]>;
};

export function renderOperatorView(input: {
  title: string;
  intro: string;
  checks: OperatorCheck[];
  sections: MetricSection[];
  callout: string;
}): string {
  const checks = input.checks
    .map(
      (check) =>
        `<li><strong>${escapeHtml(check.status.toUpperCase())}</strong> ${escapeHtml(check.summary)}</li>`,
    )
    .join("");
  const sections = input.sections
    .map((section) => {
      const rows = section.items
        .map(
          ([label, value]) =>
            `<div class="metric"><span class="label">${escapeHtml(label)}</span><span class="value">${escapeHtml(value)}</span></div>`,
        )
        .join("");
      return `<section class="card"><h2>${escapeHtml(section.title)}</h2><p class="subtitle">${escapeHtml(section.subtitle)}</p><div class="metrics">${rows}</div></section>`;
    })
    .join("");

  return `<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>${escapeHtml(input.title)}</title>
  <style>
    :root {
      color-scheme: light;
      --bg: #eef3f7;
      --panel: #fbfdff;
      --line: #ced8e2;
      --text: #17202a;
      --muted: #5f6b77;
      --ok: #215e49;
      --ok-soft: #dcefe7;
    }
    * { box-sizing: border-box; }
    body {
      margin: 0;
      font-family: "Avenir Next", "Segoe UI", sans-serif;
      background: radial-gradient(circle at top, #f7fbff, var(--bg));
      color: var(--text);
    }
    main {
      max-width: 1220px;
      margin: 0 auto;
      padding: 32px 24px 48px;
    }
    h1, h2 { margin: 0 0 12px; }
    p { line-height: 1.5; }
    .hero {
      background: linear-gradient(135deg, #f7fbff, #e7eef6);
      border: 1px solid var(--line);
      border-radius: 22px;
      padding: 24px;
      margin-bottom: 24px;
      box-shadow: 0 14px 40px rgba(18, 33, 52, 0.08);
    }
    .hero p { margin: 0; color: var(--muted); }
    .checks {
      margin: 18px 0 0;
      padding-left: 18px;
    }
    .checks li {
      margin: 8px 0;
      color: var(--muted);
    }
    .grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
      gap: 18px;
      margin-bottom: 18px;
    }
    .card {
      background: var(--panel);
      border: 1px solid var(--line);
      border-radius: 18px;
      padding: 18px;
      box-shadow: 0 10px 24px rgba(15, 30, 46, 0.06);
    }
    .subtitle {
      margin: 0 0 14px;
      color: var(--muted);
    }
    .metrics {
      display: grid;
      gap: 10px;
    }
    .metric {
      display: grid;
      gap: 4px;
      padding: 10px 12px;
      border-radius: 12px;
      background: #f3f7fb;
      border: 1px solid #dde6ef;
    }
    .label {
      font-size: 0.82rem;
      letter-spacing: 0.03em;
      text-transform: uppercase;
      color: var(--muted);
    }
    .value {
      font-size: 0.98rem;
      color: var(--text);
      word-break: break-word;
    }
    .callout {
      margin-top: 22px;
      padding: 16px 18px;
      border-radius: 16px;
      border: 1px solid #cfe0d8;
      background: var(--ok-soft);
      color: var(--ok);
    }
  </style>
</head>
<body>
  <main>
    <section class="hero">
      <h1>${escapeHtml(input.title)}</h1>
      <p>${escapeHtml(input.intro)}</p>
      <ul class="checks">${checks}</ul>
    </section>
    <div class="grid">${sections}</div>
    <section class="callout">${escapeHtml(input.callout)}</section>
  </main>
</body>
</html>`;
}

function escapeHtml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}
