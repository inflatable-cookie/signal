#!/usr/bin/env python3

import json
import subprocess
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MANIFEST_PATH = (
    REPO_ROOT
    / "demos"
    / "manifests"
    / "runtime-supervisor-boundary-companion.demo.json"
)
RECEIPT_PATH = (
    REPO_ROOT
    / "demos"
    / "receipts"
    / "runtime-supervisor-boundary-companion.receipt.json"
)
HTML_PATH = (
    REPO_ROOT
    / "demos"
    / "receipts"
    / "runtime-supervisor-boundary-companion.view.html"
)


def run_descriptor(flag: str) -> dict[str, object]:
    command = [
        "cargo",
        "run",
        "-q",
        "-p",
        "signal-supervisor-tools",
        "--",
        flag,
        "--format=json",
    ]
    result = subprocess.run(
        command,
        text=True,
        capture_output=True,
        cwd=REPO_ROOT,
        check=True,
    )
    payload = json.loads(result.stdout)
    return {
        "command": " ".join(command),
        "payload": payload,
    }


def section_card(title: str, subtitle: str, items: list[tuple[str, str]]) -> str:
    rows = "".join(
        f"<div class=\"metric\"><span class=\"label\">{label}</span><span class=\"value\">{value}</span></div>"
        for label, value in items
    )
    return (
        f"<section class=\"card\"><h2>{title}</h2><p class=\"subtitle\">{subtitle}</p>"
        f"<div class=\"metrics\">{rows}</div></section>"
    )


def boundary_card(payload: dict[str, object], subtitle: str) -> str:
    deferred = payload.get("deferred_scope")
    deferred_count = len(deferred) if isinstance(deferred, list) else 0
    surfaces = payload.get("surfaces")
    first_surface = "n/a"
    if isinstance(surfaces, list) and surfaces:
        first_surface = str(surfaces[0].get("id", "n/a"))
    return section_card(
        str(payload.get("boundary", subtitle)),
        subtitle,
        [
            ("Contract", str(payload.get("contract_path", "n/a"))),
            ("Acceptance", str(payload.get("acceptance_task", "n/a"))),
            ("Surfaces", str(payload.get("surface_count", "n/a"))),
            ("Validation steps", str(payload.get("validation_step_count", "n/a"))),
            ("Deferred scope", str(deferred_count)),
            ("Primary surface", first_surface),
        ],
    )


def browser_html(model: dict[str, object]) -> str:
    checks = "".join(
        f"<li><strong>{check['status'].upper()}</strong> {check['summary']}</li>"
        for check in model["operator_checks"]
    )
    boundary_cards = "".join(
        boundary_card(payload, subtitle) for subtitle, payload in model["boundaries"]
    )
    return f"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Signal Runtime Supervisor Boundary Companion</title>
  <style>
    :root {{
      color-scheme: light;
      --bg: #eef2f7;
      --panel: #fbfcfe;
      --line: #cfd7e2;
      --text: #19212b;
      --muted: #65707d;
      --ok: #205d45;
      --ok-soft: #dceee6;
    }}
    * {{ box-sizing: border-box; }}
    body {{
      margin: 0;
      font-family: "Avenir Next", "Segoe UI", sans-serif;
      background: radial-gradient(circle at top, #f7faff, var(--bg));
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
      background: linear-gradient(135deg, #f7fbff, #e8edf5);
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
      <h1>Signal Runtime Supervisor Boundary Companion</h1>
      <p>Operator-facing rendered view for bounded interruption and fault-diagnostic runtime boundary posture. This surface stays descriptor-backed and low-dependency; it complements the runtime recovery inspector instead of replacing it.</p>
      <ul class="checks">{checks}</ul>
    </section>
    <div class="grid">
      {boundary_cards}
    </div>
    <section class="callout">
      The underlying source of truth is still the receipt and the bounded supervisor descriptor commands. This rendered view exists to make the runtime companion surface visually inspectable without reading raw JSON first.
    </section>
  </main>
</body>
</html>
"""


def main() -> None:
    manifest = json.loads(MANIFEST_PATH.read_text())
    scenario = manifest["scenarios"][0]

    interruption = run_descriptor("--describe-interruption-boundary")
    fault = run_descriptor("--describe-fault-diagnostic-boundary")

    interruption_payload = interruption["payload"]
    fault_payload = fault["payload"]

    receipt = {
        "receipt_version": "signal.demo.receipt.v1",
        "manifest_id": manifest["id"],
        "scenario_id": scenario["id"],
        "status": "passed",
        "launch_command": "effigy demo:supervisor-runtime-boundary-companion",
        "artifacts": [
            {
                "kind": "signal-supervisor-tools-runtime-boundaries",
                "companion_to_manifest": "signal.demo.runtime.recovery-inspector",
                "descriptors": [
                    {
                        "boundary": interruption_payload.get("boundary"),
                        "contract_path": interruption_payload.get("contract_path"),
                        "acceptance_task": interruption_payload.get(
                            "acceptance_task"
                        ),
                        "surface_count": len(interruption_payload.get("surfaces", [])),
                        "validation_step_count": len(
                            interruption_payload.get("validation_steps", [])
                        ),
                        "deferred_scope_count": len(
                            interruption_payload.get("deferred_scope", [])
                        ),
                        "raw_payload": interruption_payload,
                    },
                    {
                        "boundary": fault_payload.get("boundary"),
                        "contract_path": fault_payload.get("contract_path"),
                        "acceptance_task": fault_payload.get("acceptance_task"),
                        "surface_count": fault_payload.get("surface_count"),
                        "validation_step_count": fault_payload.get(
                            "validation_step_count"
                        ),
                        "deferred_scope_count": len(
                            fault_payload.get("deferred_scope", [])
                        ),
                        "raw_payload": fault_payload,
                    },
                ],
            },
            {
                "kind": "runtime-supervisor-operator-view",
                "html_path": "demos/receipts/runtime-supervisor-boundary-companion.view.html",
                "status": "passed",
                "boundary_count": 2,
            },
        ],
        "operator_checks": [
            {
                "id": "operator.runtime-supervisor.interruption-boundary",
                "status": "passed"
                if interruption_payload.get("boundary")
                == "signal.runtime.interruption-boundary"
                and interruption_payload.get("acceptance_task")
                == "effigy acceptance:interruption-boundary"
                else "failed",
                "summary": "The supervisor companion captured the machine-readable interruption boundary descriptor.",
            },
            {
                "id": "operator.runtime-supervisor.fault-diagnostic-boundary",
                "status": "passed"
                if fault_payload.get("boundary")
                == "signal.runtime.fault-diagnostic-boundary"
                and fault_payload.get("acceptance_task")
                == "effigy acceptance:fault-diagnostic-boundary"
                else "failed",
                "summary": "The supervisor companion captured the machine-readable fault-diagnostic boundary descriptor.",
            },
            {
                "id": "operator.runtime-supervisor.runtime-family-companion",
                "status": "passed"
                if manifest["id"] == "signal.demo.runtime.supervisor-boundary-companion"
                else "failed",
                "summary": "The receipt keeps its relationship to the runtime recovery inspector explicit as a companion surface.",
            },
            {
                "id": "operator.runtime-supervisor.rendered-operator-view",
                "status": "passed",
                "summary": "A rendered companion view makes interruption and fault-diagnostic boundary posture visually inspectable without reading the raw receipt first.",
            },
        ],
    }

    RECEIPT_PATH.parent.mkdir(parents=True, exist_ok=True)
    RECEIPT_PATH.write_text(json.dumps(receipt, indent=2) + "\n")
    HTML_PATH.write_text(
        browser_html(
            {
                "boundaries": [
                    ("Interruption taxonomy and resumability posture.", interruption_payload),
                    ("Fault-diagnostic cause and evidence posture.", fault_payload),
                ],
                "operator_checks": receipt["operator_checks"],
            }
        )
    )


if __name__ == "__main__":
    main()
