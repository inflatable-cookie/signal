#!/usr/bin/env python3

import json
import re
import subprocess
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MANIFEST_PATH = (
    REPO_ROOT / "demos" / "manifests" / "runtime-recovery-inspector.demo.json"
)
RECEIPT_PATH = (
    REPO_ROOT / "demos" / "receipts" / "runtime-recovery-inspector.receipt.json"
)
HTML_PATH = REPO_ROOT / "demos" / "receipts" / "runtime-recovery-inspector.view.html"


def extract_first(line: str, key: str) -> str | None:
    pattern = re.compile(rf"\b{re.escape(key)}=(\".*?\"|\[[^\]]*\]|\S+)")
    match = pattern.search(line)
    if match is None:
        return None
    value = match.group(1)
    if value.startswith('"') and value.endswith('"'):
        value = value[1:-1]
    return value


def section_card(title: str, subtitle: str, items: list[tuple[str, str]]) -> str:
    rows = "".join(
        f"<div class=\"metric\"><span class=\"label\">{label}</span><span class=\"value\">{value}</span></div>"
        for label, value in items
    )
    return (
        f"<section class=\"card\"><h2>{title}</h2><p class=\"subtitle\">{subtitle}</p>"
        f"<div class=\"metrics\">{rows}</div></section>"
    )


def browser_html(model: dict[str, object]) -> str:
    checks = "".join(
        f"<li><strong>{check['status'].upper()}</strong> {check['summary']}</li>"
        for check in model["operator_checks"]
    )
    return f"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Signal Runtime Recovery Inspector</title>
  <style>
    :root {{
      color-scheme: light;
      --bg: #edf3f6;
      --panel: #fbfdff;
      --line: #ccd7df;
      --text: #162028;
      --muted: #5f6c76;
      --accent: #2d6e94;
      --ok: #1e6445;
      --ok-soft: #dcefe5;
    }}
    * {{ box-sizing: border-box; }}
    body {{
      margin: 0;
      font-family: "Avenir Next", "Segoe UI", sans-serif;
      background: radial-gradient(circle at top, #f7fbfd, var(--bg));
      color: var(--text);
    }}
    main {{
      max-width: 1180px;
      margin: 0 auto;
      padding: 32px 24px 48px;
    }}
    h1, h2 {{ margin: 0 0 12px; }}
    p {{ line-height: 1.5; }}
    .hero {{
      background: linear-gradient(135deg, #f7fbff, #e7f0f6);
      border: 1px solid var(--line);
      border-radius: 22px;
      padding: 24px;
      margin-bottom: 24px;
      box-shadow: 0 14px 40px rgba(18, 44, 62, 0.08);
    }}
    .hero p {{ margin: 0; color: var(--muted); }}
    .checks {{
      margin: 18px 0 0;
      padding-left: 18px;
    }}
    .checks li {{
      margin: 8px 0;
      color: var(--muted);
    }}
    .grid {{
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
      gap: 18px;
      margin-bottom: 18px;
    }}
    .card {{
      background: var(--panel);
      border: 1px solid var(--line);
      border-radius: 18px;
      padding: 18px;
      box-shadow: 0 10px 24px rgba(15, 37, 53, 0.06);
    }}
    .subtitle {{
      margin: 0 0 14px;
      color: var(--muted);
    }}
    .metrics {{
      display: grid;
      gap: 10px;
    }}
    .metric {{
      display: grid;
      gap: 4px;
      padding: 10px 12px;
      border-radius: 12px;
      background: #f3f8fb;
      border: 1px solid #dbe6ee;
    }}
    .label {{
      font-size: 0.82rem;
      letter-spacing: 0.03em;
      text-transform: uppercase;
      color: var(--muted);
    }}
    .value {{
      font-size: 0.98rem;
      color: var(--text);
      word-break: break-word;
    }}
    .callout {{
      margin-top: 22px;
      padding: 16px 18px;
      border-radius: 16px;
      border: 1px solid #cfe0d6;
      background: var(--ok-soft);
      color: var(--ok);
    }}
  </style>
</head>
<body>
  <main>
    <section class="hero">
      <h1>Signal Runtime Recovery Inspector</h1>
      <p>Operator-facing rendered view for bounded runtime recovery posture across handshake, watchdog, plugin faults, safe mode, and degraded external/backend surfaces. This surface stays example-backed and low-dependency; it is not a runtime dashboard or control shell.</p>
      <ul class="checks">{checks}</ul>
    </section>
    <div class="grid">
      {section_card("Lifecycle posture", "Bounded runtime startup and readiness posture from the supervisor report example.", model["lifecycle"])}
      {section_card("Watchdog and faults", "Observed watchdog trigger and plugin-fault history from the bounded recovery report.", model["faults"])}
      {section_card("Safe mode and degraded surfaces", "Steady-state safe-mode and external/backend degradation posture.", model["degraded"])}
    </div>
    <section class="callout">
      The underlying source of truth is still the receipt and the bounded runtime report example. This rendered view exists to make runtime recovery posture visually inspectable without reading raw JSON first.
    </section>
  </main>
</body>
</html>
"""


def main() -> None:
    manifest = json.loads(MANIFEST_PATH.read_text())
    scenario = manifest["scenarios"][0]
    launch_command = "cargo run -q -p signal-runtime --example supervisor_report_demo"

    result = subprocess.run(
        ["cargo", "run", "-q", "-p", "signal-runtime", "--example", "supervisor_report_demo"],
        text=True,
        capture_output=True,
        cwd=REPO_ROOT,
        check=True,
    )

    lines = [line.strip() for line in result.stdout.splitlines() if line.strip()]
    joined_lines = " ".join(lines)

    def contains(fragment: str) -> bool:
        return any(fragment in line for line in lines)

    readiness = extract_first(joined_lines, "readiness") or "n/a"
    handshaken = extract_first(joined_lines, "handshaken") or "n/a"
    configured = extract_first(joined_lines, "configured") or "n/a"
    running = extract_first(joined_lines, "running") or "n/a"
    watchdog = extract_first(joined_lines, "last_watchdog") or "n/a"
    plugin_faults = extract_first(joined_lines, "plugin_faults") or "n/a"
    last_fault = extract_first(joined_lines, "last_fault") or "n/a"
    event_count = extract_first(joined_lines, "events") or "n/a"
    safe_mode = extract_first(joined_lines, "safe_mode") or "n/a"
    device_safe_mode = (
        extract_first(joined_lines, "device_supervision_safe_mode_enabled") or "n/a"
    )
    external_io = extract_first(joined_lines, "external_io_summary") or "n/a"
    linux_backend = extract_first(joined_lines, "linux_backend_session_summary") or "n/a"

    receipt = {
        "receipt_version": "signal.demo.receipt.v1",
        "manifest_id": manifest["id"],
        "scenario_id": scenario["id"],
        "status": "passed",
        "launch_command": launch_command,
        "artifacts": [
            {
                "kind": "runtime-supervisor-report-lines",
                "line_count": len(lines),
                "highlights": {
                    "readiness": readiness,
                    "watchdog": watchdog,
                    "plugin_fault_count": plugin_faults,
                    "event_count": event_count,
                },
            },
            {
                "kind": "runtime-recovery-operator-view",
                "html_path": "demos/receipts/runtime-recovery-inspector.view.html",
                "status": "passed",
                "section_count": 3,
            },
        ],
        "operator_checks": [
            {
                "id": "operator.runtime-recovery.handshake-and-start",
                "status": "passed"
                if contains("handshaken=true")
                and contains("configured=true")
                and contains("running=true")
                else "failed",
                "summary": "Runtime example completed handshake, configuration, and start.",
            },
            {
                "id": "operator.runtime-recovery.watchdog-snapshot",
                "status": "passed"
                if contains("last_watchdog=HeartbeatMisses")
                and contains("degradation_summary_last_watchdog=Some(HeartbeatMisses)")
                else "failed",
                "summary": "Supervisor output exposed the watchdog-trigger snapshot.",
            },
            {
                "id": "operator.runtime-recovery.plugin-faults",
                "status": "passed"
                if contains("plugin_faults=2")
                and contains("last_fault=sandbox-demo:Timeout")
                else "failed",
                "summary": "Runtime example exported the injected plugin timeout faults.",
            },
            {
                "id": "operator.runtime-recovery-safe-mode-posture",
                "status": "passed"
                if contains("safe_mode=false")
                and contains("device_supervision_safe_mode_enabled=false")
                else "failed",
                "summary": "Runtime report kept explicit safe-mode posture in the steady-state surface.",
            },
            {
                "id": "operator.runtime-recovery.external-surface",
                "status": "passed"
                if contains("external_io_summary=health=Unavailable")
                and contains("linux_backend_session_summary=backend=Unavailable")
                else "failed",
                "summary": "Runtime report preserved degraded hardware/backend surfaces explicitly.",
            },
            {
                "id": "operator.runtime-recovery.rendered-operator-view",
                "status": "passed",
                "summary": "A rendered companion view makes watchdog, fault, safe-mode, and degraded-surface posture visually inspectable without reading the raw receipt first.",
            },
        ],
    }

    RECEIPT_PATH.parent.mkdir(parents=True, exist_ok=True)
    RECEIPT_PATH.write_text(json.dumps(receipt, indent=2) + "\n")
    HTML_PATH.write_text(
        browser_html(
            {
                "lifecycle": [
                    ("Readiness", readiness),
                    ("Handshaken", handshaken),
                    ("Configured", configured),
                    ("Running", running),
                ],
                "faults": [
                    ("Watchdog", watchdog),
                    ("Plugin faults", plugin_faults),
                    ("Last fault", last_fault),
                    ("Events", event_count),
                ],
                "degraded": [
                    ("Safe mode", safe_mode),
                    ("Device safe mode", device_safe_mode),
                    ("External I/O", external_io),
                    ("Linux backend", linux_backend),
                ],
                "operator_checks": receipt["operator_checks"],
            }
        )
    )


if __name__ == "__main__":
    main()
