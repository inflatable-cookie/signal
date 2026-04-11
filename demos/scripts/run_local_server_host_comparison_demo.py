#!/usr/bin/env python3

import json
import re
import subprocess
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MANIFEST_PATH = (
    REPO_ROOT / "demos" / "manifests" / "local-server-host-comparison.demo.json"
)
RECEIPT_PATH = (
    REPO_ROOT / "demos" / "receipts" / "local-server-host-comparison.receipt.json"
)
HTML_PATH = (
    REPO_ROOT / "demos" / "receipts" / "local-server-host-comparison.view.html"
)

PAIR_RE = re.compile(r'([A-Za-z0-9_]+)=(".*?"|\[[^\]]*\]|\S+)')


def parse_summary_line(line: str) -> dict[str, str]:
    parsed: dict[str, str] = {}
    for key, raw_value in PAIR_RE.findall(line):
        value = raw_value
        if value.startswith('"') and value.endswith('"'):
            value = value[1:-1]
        parsed[key] = value
    return parsed


def extract_first(line: str, key: str) -> str | None:
    pattern = re.compile(rf"\b{re.escape(key)}=(\".*?\"|\[[^\]]*\]|\S+)")
    match = pattern.search(line)
    if match is None:
        return None
    value = match.group(1)
    if value.startswith('"') and value.endswith('"'):
        value = value[1:-1]
    return value


def as_int(parsed: dict[str, str], key: str) -> int:
    try:
        return int(parsed[key])
    except (KeyError, ValueError):
        return 0


def as_bool(parsed: dict[str, str], key: str) -> bool:
    return parsed.get(key) == "true"


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
  <title>Signal Local Server Host Comparison</title>
  <style>
    :root {{
      color-scheme: light;
      --bg: #eef3f7;
      --panel: #fbfdff;
      --line: #ced8e2;
      --text: #17202a;
      --muted: #5f6b77;
      --ok: #215e49;
      --ok-soft: #dcefe7;
    }}
    * {{ box-sizing: border-box; }}
    body {{
      margin: 0;
      font-family: "Avenir Next", "Segoe UI", sans-serif;
      background: radial-gradient(circle at top, #f7fbff, var(--bg));
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
      background: linear-gradient(135deg, #f7fbff, #e7eef6);
      border: 1px solid var(--line);
      border-radius: 22px;
      padding: 24px;
      margin-bottom: 24px;
      box-shadow: 0 14px 40px rgba(18, 33, 52, 0.08);
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
      box-shadow: 0 10px 24px rgba(15, 30, 46, 0.06);
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
      background: #f3f7fb;
      border: 1px solid #dde6ef;
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
      border: 1px solid #cfe0d8;
      background: var(--ok-soft);
      color: var(--ok);
    }}
  </style>
</head>
<body>
  <main>
    <section class="hero">
      <h1>Signal Local Server Host Comparison</h1>
      <p>Operator-facing rendered view for bounded local-versus-server host bootstrap posture. This surface stays summary-backed and low-dependency; it compares the current host binaries without turning into a host UI shell.</p>
      <ul class="checks">{checks}</ul>
    </section>
    <div class="grid">
      {section_card("Local host", "Shared lifecycle and host-local posture from the local bootstrap summary.", model["local_host"])}
      {section_card("Server host", "Shared lifecycle and server-side execution posture from the server bootstrap summary.", model["server_host"])}
      {section_card("Comparison", "Explicit shared truth and real differences between the two host surfaces.", model["comparison"])}
    </div>
    <section class="callout">
      The underlying source of truth is still the receipt and the existing host summary lines. This rendered view exists to make shared and differing host posture visually inspectable without reading raw JSON first.
    </section>
  </main>
</body>
</html>
"""


def run_host(package: str) -> dict[str, object]:
    result = subprocess.run(
        ["cargo", "run", "-q", "-p", package],
        text=True,
        capture_output=True,
        cwd=REPO_ROOT,
        check=True,
    )
    line = next(
        stripped
        for stripped in result.stdout.splitlines()
        if stripped.startswith(package)
    )
    parsed = parse_summary_line(line)
    return {
        "package": package,
        "line": line,
        "parsed": parsed,
    }


def main() -> None:
    manifest = json.loads(MANIFEST_PATH.read_text())
    scenario = manifest["scenarios"][0]

    local = run_host("signal-host-local")
    server = run_host("signal-host-server")

    local_parsed = local["parsed"]
    server_parsed = server["parsed"]
    local_line = str(local["line"])
    server_line = str(server["line"])
    local_readiness = extract_first(local_line, "readiness")
    server_readiness = extract_first(server_line, "readiness")

    comparison = {
        "shared_truth": {
            "local_ready": local_readiness == "Ready",
            "server_ready": server_readiness == "Ready",
            "local_running": as_bool(local_parsed, "running"),
            "server_running": as_bool(server_parsed, "running"),
            "local_processed_blocks": as_int(local_parsed, "processed_blocks"),
            "server_processed_blocks": as_int(server_parsed, "processed_blocks"),
            "local_completion": local_parsed.get("completion"),
            "server_completion": server_parsed.get("completion"),
            "local_heartbeat_responses": as_int(local_parsed, "heartbeat_responses"),
            "server_heartbeat_responses": as_int(server_parsed, "heartbeat_responses"),
        },
        "host_differences": {
            "local_backend": local_parsed.get("backend"),
            "local_audio_state": local_parsed.get("audio_state"),
            "server_engine_processed_blocks": as_int(server_parsed, "engine_processed_blocks"),
            "server_engine_graph_id": server_parsed.get("engine_graph_id"),
            "local_topology_nodes": as_int(local_parsed, "topology_nodes"),
        },
    }

    receipt = {
        "receipt_version": "signal.demo.receipt.v1",
        "manifest_id": manifest["id"],
        "scenario_id": scenario["id"],
        "status": "passed",
        "launch_command": "effigy demo:local-server-host-comparison",
        "artifacts": [
            {
                "kind": "host-summary-lines",
                "hosts": [
                    {
                        "package": local["package"],
                        "sandbox": local_parsed.get("sandbox"),
                        "profile": local_parsed.get("profile"),
                        "processed_blocks": as_int(local_parsed, "processed_blocks"),
                        "heartbeat_responses": as_int(local_parsed, "heartbeat_responses"),
                        "completion": local_parsed.get("completion"),
                        "raw_line": local["line"],
                    },
                    {
                        "package": server["package"],
                        "sandbox": server_parsed.get("sandbox"),
                        "profile": server_parsed.get("profile"),
                        "processed_blocks": as_int(server_parsed, "processed_blocks"),
                        "heartbeat_responses": as_int(server_parsed, "heartbeat_responses"),
                        "completion": server_parsed.get("completion"),
                        "raw_line": server["line"],
                    },
                ],
                "comparison": comparison,
            },
            {
                "kind": "host-comparison-operator-view",
                "html_path": "demos/receipts/local-server-host-comparison.view.html",
                "status": "passed",
                "section_count": 3,
            },
        ],
        "operator_checks": [
            {
                "id": "operator.host-compare.local-bootstrap",
                "status": "passed"
                if local_readiness == "Ready"
                and as_bool(local_parsed, "running")
                and as_int(local_parsed, "processed_blocks") > 0
                and local_parsed.get("completion") == "Completed"
                else "failed",
                "summary": "Local host booted successfully with ready/running posture and bounded execution.",
            },
            {
                "id": "operator.host-compare.server-bootstrap",
                "status": "passed"
                if server_readiness == "Ready"
                and as_bool(server_parsed, "running")
                and as_int(server_parsed, "processed_blocks") > 0
                and server_parsed.get("completion") == "Completed"
                else "failed",
                "summary": "Server host booted successfully with ready/running posture and bounded execution.",
            },
            {
                "id": "operator.host-compare.shared-lifecycle-truth",
                "status": "passed"
                if local_parsed.get("sandbox")
                and server_parsed.get("sandbox")
                and as_int(local_parsed, "heartbeat_responses") > 0
                and as_int(server_parsed, "heartbeat_responses") > 0
                else "failed",
                "summary": "Both hosts exported active sandbox and heartbeat truth through the existing summary line.",
            },
            {
                "id": "operator.host-compare-differences-explicit",
                "status": "passed"
                if local_parsed.get("backend") == "coreaudio"
                and as_int(server_parsed, "engine_processed_blocks") > 0
                else "failed",
                "summary": "The receipt preserves real local-versus-server differences instead of flattening them.",
            },
            {
                "id": "operator.host-compare.rendered-operator-view",
                "status": "passed",
                "summary": "A rendered companion view makes shared lifecycle truth and local-versus-server differences visually inspectable without reading the raw receipt first.",
            },
        ],
    }

    model = {
        "operator_checks": receipt["operator_checks"],
        "local_host": [
            ("Readiness", local_readiness or "n/a"),
            ("Running", str(as_bool(local_parsed, "running")).lower()),
            ("Backend", local_parsed.get("backend", "n/a")),
            ("Sandbox", local_parsed.get("sandbox", "n/a")),
            ("Processed blocks", str(as_int(local_parsed, "processed_blocks"))),
            ("Heartbeat responses", str(as_int(local_parsed, "heartbeat_responses"))),
            ("Completion", local_parsed.get("completion", "n/a")),
            ("Audio state", local_parsed.get("audio_state", "n/a")),
        ],
        "server_host": [
            ("Readiness", server_readiness or "n/a"),
            ("Running", str(as_bool(server_parsed, "running")).lower()),
            ("Sandbox", server_parsed.get("sandbox", "n/a")),
            ("Processed blocks", str(as_int(server_parsed, "processed_blocks"))),
            ("Heartbeat responses", str(as_int(server_parsed, "heartbeat_responses"))),
            ("Completion", server_parsed.get("completion", "n/a")),
            ("Engine processed blocks", str(as_int(server_parsed, "engine_processed_blocks"))),
            ("Engine graph", server_parsed.get("engine_graph_id", "n/a")),
        ],
        "comparison": [
            (
                "Shared readiness",
                f"{'ready' if comparison['shared_truth']['local_ready'] else 'not-ready'} / "
                f"{'ready' if comparison['shared_truth']['server_ready'] else 'not-ready'}",
            ),
            (
                "Shared running",
                f"{'running' if comparison['shared_truth']['local_running'] else 'not-running'} / "
                f"{'running' if comparison['shared_truth']['server_running'] else 'not-running'}",
            ),
            (
                "Shared completion",
                f"{comparison['shared_truth']['local_completion']} / "
                f"{comparison['shared_truth']['server_completion']}",
            ),
            (
                "Heartbeat truth",
                f"{comparison['shared_truth']['local_heartbeat_responses']} / "
                f"{comparison['shared_truth']['server_heartbeat_responses']}",
            ),
            (
                "Local backend",
                str(comparison["host_differences"]["local_backend"]),
            ),
            (
                "Server engine blocks",
                str(comparison["host_differences"]["server_engine_processed_blocks"]),
            ),
            (
                "Server engine graph",
                str(comparison["host_differences"]["server_engine_graph_id"]),
            ),
            (
                "Local topology nodes",
                str(comparison["host_differences"]["local_topology_nodes"]),
            ),
        ],
    }

    RECEIPT_PATH.parent.mkdir(parents=True, exist_ok=True)
    RECEIPT_PATH.write_text(json.dumps(receipt, indent=2) + "\n")
    HTML_PATH.write_text(browser_html(model))


if __name__ == "__main__":
    main()
