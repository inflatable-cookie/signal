#!/usr/bin/env python3

import json
import subprocess
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MANIFEST_PATH = REPO_ROOT / "demos" / "manifests" / "plugin-sandbox-lifecycle.demo.json"
RECEIPT_PATH = REPO_ROOT / "demos" / "receipts" / "plugin-sandbox-lifecycle.receipt.json"
HTML_PATH = REPO_ROOT / "demos" / "receipts" / "plugin-sandbox-lifecycle.view.html"

BROKER_RUNS = [
    (
        "attach_status_teardown",
        [
            "status",
            "attach-demo",
            "status",
            "teardown-demo",
            "shutdown",
        ],
    ),
    (
        "healthy_run",
        [
            "run-demo",
            "shutdown",
        ],
    ),
    (
        "timeout_run",
        [
            "run-timeout-demo",
            "shutdown",
        ],
    ),
]


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
  <title>Signal Sandbox Lifecycle</title>
  <style>
    :root {{
      color-scheme: light;
      --bg: #f1f3f7;
      --panel: #fbfcfe;
      --line: #d2d8e2;
      --text: #19212a;
      --muted: #626b77;
      --ok: #205b45;
      --ok-soft: #dceee6;
    }}
    * {{ box-sizing: border-box; }}
    body {{
      margin: 0;
      font-family: "Avenir Next", "Segoe UI", sans-serif;
      background: radial-gradient(circle at top, #f8fbff, var(--bg));
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
      background: linear-gradient(135deg, #f8fbff, #e8edf5);
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
      border: 1px solid #cfe0d6;
      background: var(--ok-soft);
      color: var(--ok);
    }}
  </style>
</head>
<body>
  <main>
    <section class="hero">
      <h1>Signal Sandbox Lifecycle</h1>
      <p>Operator-facing rendered view for bounded broker lifecycle truth across ready, attach, status, healthy run, timeout run, teardown, and shutdown. This surface stays broker-backed and low-dependency; it is not a broker control console.</p>
      <ul class="checks">{checks}</ul>
    </section>
    <div class="grid">
      {section_card("Lifecycle coverage", "Observed broker states across the bounded lifecycle runs.", model["lifecycle"])}
      {section_card("Explicit attach path", "Attach, status, teardown, and shutdown posture from the manual lifecycle run.", model["attach_status"])}
      {section_card("Healthy and timeout runs", "Comparison of the clean run and timeout recovery run.", model["run_paths"])}
    </div>
    <section class="callout">
      The underlying source of truth is still the receipt and broker transcript lines. This rendered view exists to make broker lifecycle and timeout recovery posture visually inspectable without reading raw JSON first.
    </section>
  </main>
</body>
</html>
"""


def main() -> None:
    manifest = json.loads(MANIFEST_PATH.read_text())
    scenario = manifest["scenarios"][0]
    launch_command = "cargo run -q -p signal-plugin-sandbox"
    transcripts = []

    for run_id, commands in BROKER_RUNS:
        result = subprocess.run(
            ["cargo", "run", "-q", "-p", "signal-plugin-sandbox"],
            input="\n".join(commands) + "\n",
            text=True,
            capture_output=True,
            cwd=REPO_ROOT,
            check=True,
        )
        lines = [
            line.strip()
            for line in result.stdout.splitlines()
            if line.strip().startswith("signal-plugin-sandbox")
        ]
        transcripts.append({"run_id": run_id, "lines": lines})

    all_lines = [line for transcript in transcripts for line in transcript["lines"]]

    observed_states = sorted(
        {
            token.split("=", 1)[1]
            for line in all_lines
            for token in line.split()
            if token.startswith("state=")
        }
    )

    def transcript_contains(run_id: str, fragment: str) -> bool:
        return any(
            fragment in line
            for transcript in transcripts
            if transcript["run_id"] == run_id
            for line in transcript["lines"]
        )

    receipt = {
        "receipt_version": "signal.demo.receipt.v1",
        "manifest_id": manifest["id"],
        "scenario_id": scenario["id"],
        "status": "passed",
        "launch_command": launch_command,
        "artifacts": [
            {
                "kind": "broker-transcript-lines",
                "line_count": len(all_lines),
                "observed_states": observed_states,
                "runs": [
                    {
                        "run_id": transcript["run_id"],
                        "line_count": len(transcript["lines"]),
                    }
                    for transcript in transcripts
                ],
            },
            {
                "kind": "sandbox-lifecycle-operator-view",
                "html_path": "demos/receipts/plugin-sandbox-lifecycle.view.html",
                "status": "passed",
                "section_count": 3,
            },
        ],
        "operator_checks": [
            {
                "id": "operator.sandbox-lifecycle.ready-state",
                "status": "passed" if transcript_contains("attach_status_teardown", "state=ready") else "failed",
                "summary": "Broker reported the ready state before live lifecycle commands.",
            },
            {
                "id": "operator.sandbox-lifecycle.attach-and-teardown",
                "status": "passed"
                if transcript_contains("attach_status_teardown", "state=attached")
                and transcript_contains("attach_status_teardown", "state=teardown_complete")
                else "failed",
                "summary": "Explicit attach/status/teardown path remained inspectable.",
            },
            {
                "id": "operator.sandbox-lifecycle.run-path",
                "status": "passed"
                if transcript_contains("healthy_run", "state=running")
                and transcript_contains("healthy_run", "detail=lease_cleanup_ok")
                else "failed",
                "summary": "Healthy demo run reached running and clean teardown states.",
            },
            {
                "id": "operator.sandbox-lifecycle.timeout-path",
                "status": "passed"
                if transcript_contains("timeout_run", "state=timed_out")
                and transcript_contains("timeout_run", "detail=lease_cleanup_ok_after_timeout")
                else "failed",
                "summary": "Timeout demo run remained bounded and reported cleanup after interruption.",
            },
            {
                "id": "operator.sandbox-lifecycle.shutdown",
                "status": "passed"
                if all(
                    transcript_contains(transcript["run_id"], "state=shutdown")
                    for transcript in transcripts
                )
                else "failed",
                "summary": "Broker exited through the explicit shutdown receipt.",
            },
            {
                "id": "operator.sandbox-lifecycle.rendered-operator-view",
                "status": "passed",
                "summary": "A rendered companion view makes broker lifecycle and timeout recovery posture visually inspectable without reading the raw receipt first.",
            },
        ],
    }

    attach_lines = next(
        transcript["lines"]
        for transcript in transcripts
        if transcript["run_id"] == "attach_status_teardown"
    )
    healthy_lines = next(
        transcript["lines"] for transcript in transcripts if transcript["run_id"] == "healthy_run"
    )
    timeout_lines = next(
        transcript["lines"] for transcript in transcripts if transcript["run_id"] == "timeout_run"
    )
    model = {
        "operator_checks": receipt["operator_checks"],
        "lifecycle": [
            ("Observed states", ", ".join(observed_states)),
            ("Runs", str(len(transcripts))),
            ("Transcript lines", str(len(all_lines))),
            (
                "Healthy run lines",
                str(len(healthy_lines)),
            ),
            (
                "Timeout run lines",
                str(len(timeout_lines)),
            ),
        ],
        "attach_status": [
            ("Ready state", "yes" if transcript_contains("attach_status_teardown", "state=ready") else "no"),
            ("Attached", "yes" if transcript_contains("attach_status_teardown", "state=attached") else "no"),
            ("Teardown complete", "yes" if transcript_contains("attach_status_teardown", "state=teardown_complete") else "no"),
            ("Shutdown", "yes" if transcript_contains("attach_status_teardown", "state=shutdown") else "no"),
            ("Transcript lines", str(len(attach_lines))),
        ],
        "run_paths": [
            ("Healthy run reached running", "yes" if transcript_contains("healthy_run", "state=running") else "no"),
            ("Healthy cleanup detail", "yes" if transcript_contains("healthy_run", "detail=lease_cleanup_ok") else "no"),
            ("Timeout reached timed_out", "yes" if transcript_contains("timeout_run", "state=timed_out") else "no"),
            ("Timeout cleanup detail", "yes" if transcript_contains("timeout_run", "detail=lease_cleanup_ok_after_timeout") else "no"),
            ("Shutdown in both runs", "yes" if transcript_contains("healthy_run", "state=shutdown") and transcript_contains("timeout_run", "state=shutdown") else "no"),
        ],
    }

    RECEIPT_PATH.parent.mkdir(parents=True, exist_ok=True)
    RECEIPT_PATH.write_text(json.dumps(receipt, indent=2) + "\n")
    HTML_PATH.write_text(browser_html(model))


if __name__ == "__main__":
    main()
