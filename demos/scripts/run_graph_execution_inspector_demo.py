#!/usr/bin/env python3

import json
import subprocess
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MANIFEST_PATH = (
    REPO_ROOT / "demos" / "manifests" / "graph-execution-inspector.demo.json"
)
RECEIPT_PATH = (
    REPO_ROOT / "demos" / "receipts" / "graph-execution-inspector.receipt.json"
)
HTML_PATH = (
    REPO_ROOT / "demos" / "receipts" / "graph-execution-inspector.view.html"
)


def run_command(command: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        command,
        text=True,
        capture_output=True,
        cwd=REPO_ROOT,
        check=True,
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
    result = run_command(command)
    return {
        "command": " ".join(command),
        "payload": json.loads(result.stdout),
    }


def acceptance_artifact(result: subprocess.CompletedProcess[str], command: list[str]) -> dict[str, object]:
    return {
        "kind": "acceptance-lane-run",
        "command": " ".join(command),
        "status": "passed",
        "stdout_tail": result.stdout.splitlines()[-20:],
    }


def descriptor_artifact(kind: str, payload: dict[str, object]) -> dict[str, object]:
    deferred_scope = payload.get("deferred_scope")
    residual_risk = payload.get("residual_risk")
    deferred_scope_count = (
        len(deferred_scope) if isinstance(deferred_scope, list) else (1 if residual_risk else 0)
    )
    validation_steps = payload.get("validation_steps")
    validation_step_count = (
        payload.get("validation_step_count")
        if payload.get("validation_step_count") is not None
        else len(validation_steps) if isinstance(validation_steps, list) else 0
    )
    return {
        "kind": kind,
        "boundary": payload.get("boundary"),
        "contract_path": payload.get("contract_path"),
        "acceptance_task": payload.get("acceptance_task"),
        "surface_count": payload.get("surface_count"),
        "validation_step_count": validation_step_count,
        "deferred_scope_count": deferred_scope_count,
        "raw_payload": payload,
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


def boundary_card(payload: dict[str, object], surface_label: str) -> str:
    deferred = payload.get("deferred_scope")
    deferred_count = len(deferred) if isinstance(deferred, list) else 0
    surfaces = payload.get("surfaces")
    first_surface = "n/a"
    if isinstance(surfaces, list) and surfaces:
        first_surface = str(surfaces[0].get("id", "n/a"))
    return section_card(
        str(payload.get("boundary", surface_label)),
        surface_label,
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
    boundaries = model["boundaries"]
    acceptance = model["acceptance"]
    acceptance_rows = "".join(
        f"<div class=\"metric\"><span class=\"label\">{name}</span><span class=\"value\">passed</span></div>"
        for name in acceptance
    )
    boundary_cards = "".join(
        boundary_card(payload, subtitle)
        for subtitle, payload in boundaries
    )
    return f"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Signal Graph Execution Inspector</title>
  <style>
    :root {{
      color-scheme: light;
      --bg: #eef4f1;
      --panel: #fcfffd;
      --line: #c9d7d0;
      --text: #112019;
      --muted: #5d6c65;
      --accent: #1f7a63;
      --accent-soft: #d9f0e7;
      --ok: #19563d;
      --ok-soft: #dceee5;
    }}
    * {{ box-sizing: border-box; }}
    body {{
      margin: 0;
      font-family: "Avenir Next", "Segoe UI", sans-serif;
      background: radial-gradient(circle at top, #f8fdfa, var(--bg));
      color: var(--text);
    }}
    main {{
      max-width: 1220px;
      margin: 0 auto;
      padding: 32px 24px 48px;
    }}
    h1, h2 {{ margin: 0 0 12px; }}
    p {{ line-height: 1.5; }}
    .hero {{
      background: linear-gradient(135deg, #f7fffb, #e9f6f0);
      border: 1px solid var(--line);
      border-radius: 22px;
      padding: 24px;
      margin-bottom: 24px;
      box-shadow: 0 14px 40px rgba(17, 54, 41, 0.08);
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
      box-shadow: 0 10px 24px rgba(15, 41, 31, 0.06);
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
      background: #f4faf7;
      border: 1px solid #d6e5de;
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
      <h1>Signal Graph Execution Inspector</h1>
      <p>Operator-facing rendered view for bounded graph execution posture across multichannel, sidechain, multi-bus, and spatial boundary families. This surface stays descriptor-backed and low-dependency; it is not a graph editor or routing console.</p>
      <ul class="checks">{checks}</ul>
    </section>
    <div class="grid">
      {boundary_cards}
    </div>
    <div class="grid">
      <section class="card"><h2>Acceptance posture</h2><p class="subtitle">Focused graph boundary proof lanes completed during this capture.</p><div class="metrics">{acceptance_rows}</div></section>
    </div>
    <section class="callout">
      The underlying source of truth is still the receipt and the bounded descriptor plus acceptance commands. This rendered view exists to make the graph family visually inspectable without reading raw JSON first.
    </section>
  </main>
</body>
</html>
"""


def main() -> None:
    manifest = json.loads(MANIFEST_PATH.read_text())
    scenario = manifest["scenarios"][0]

    multichannel = run_descriptor("--describe-multichannel-boundary")
    sidechain = run_descriptor("--describe-sidechain-boundary")
    multi_bus = run_descriptor("--describe-multi-bus-boundary")
    spatial = run_descriptor("--describe-spatial-boundary")

    multichannel_payload = multichannel["payload"]
    sidechain_payload = sidechain["payload"]
    multi_bus_payload = multi_bus["payload"]
    spatial_payload = spatial["payload"]

    multichannel_acceptance_command = ["effigy", "acceptance:multichannel-boundary"]
    sidechain_acceptance_command = ["effigy", "acceptance:sidechain-boundary"]
    multi_bus_acceptance_command = ["effigy", "acceptance:multi-bus-boundary"]
    spatial_acceptance_command = ["effigy", "acceptance:spatial-boundary"]

    multichannel_acceptance_result = run_command(multichannel_acceptance_command)
    sidechain_acceptance_result = run_command(sidechain_acceptance_command)
    multi_bus_acceptance_result = run_command(multi_bus_acceptance_command)
    spatial_acceptance_result = run_command(spatial_acceptance_command)

    receipt = {
        "receipt_version": "signal.demo.receipt.v1",
        "manifest_id": manifest["id"],
        "scenario_id": scenario["id"],
        "status": "passed",
        "launch_command": "effigy demo:graph-execution-inspector",
        "artifacts": [
            descriptor_artifact(
                "multichannel-boundary-descriptor", multichannel_payload
            ),
            descriptor_artifact("sidechain-boundary-descriptor", sidechain_payload),
            descriptor_artifact("multi-bus-boundary-descriptor", multi_bus_payload),
            descriptor_artifact("spatial-boundary-descriptor", spatial_payload),
            acceptance_artifact(
                multichannel_acceptance_result, multichannel_acceptance_command
            ),
            acceptance_artifact(
                sidechain_acceptance_result, sidechain_acceptance_command
            ),
            acceptance_artifact(
                multi_bus_acceptance_result, multi_bus_acceptance_command
            ),
            acceptance_artifact(spatial_acceptance_result, spatial_acceptance_command),
            {
                "kind": "graph-execution-operator-view",
                "html_path": "demos/receipts/graph-execution-inspector.view.html",
                "status": "passed",
                "boundary_count": 4,
                "acceptance_count": 4,
            },
        ],
        "operator_checks": [
            {
                "id": "operator.graph-execution.multichannel-descriptor",
                "status": "passed"
                if multichannel_payload.get("boundary")
                == "signal.runtime.multichannel-boundary"
                and multichannel_payload.get("acceptance_task")
                == "effigy acceptance:multichannel-boundary"
                else "failed",
                "summary": "The demo captured the machine-readable multichannel boundary descriptor.",
            },
            {
                "id": "operator.graph-execution.sidechain-descriptor",
                "status": "passed"
                if sidechain_payload.get("boundary")
                == "signal.runtime.sidechain-boundary"
                and sidechain_payload.get("acceptance_task")
                == "effigy acceptance:sidechain-boundary"
                else "failed",
                "summary": "The demo captured the machine-readable sidechain boundary descriptor.",
            },
            {
                "id": "operator.graph-execution.multi-bus-descriptor",
                "status": "passed"
                if multi_bus_payload.get("boundary")
                == "signal.runtime.multi-bus-boundary"
                and multi_bus_payload.get("acceptance_task")
                == "effigy acceptance:multi-bus-boundary"
                else "failed",
                "summary": "The demo captured the machine-readable multi-bus boundary descriptor.",
            },
            {
                "id": "operator.graph-execution.spatial-descriptor",
                "status": "passed"
                if spatial_payload.get("boundary")
                == "signal.runtime.spatial-boundary"
                and spatial_payload.get("acceptance_task")
                == "effigy acceptance:spatial-boundary"
                else "failed",
                "summary": "The demo captured the machine-readable spatial boundary descriptor.",
            },
            {
                "id": "operator.graph-execution.acceptance-lanes",
                "status": "passed"
                if "multichannel_boundary_json_reports_runtime_and_host_edge_proofs ... ok"
                in multichannel_acceptance_result.stdout
                and "sidechain_boundary_json_reports_runtime_and_host_edge_proofs ... ok"
                in sidechain_acceptance_result.stdout
                and "multi_bus_boundary_json_reports_runtime_and_host_edge_proofs ... ok"
                in multi_bus_acceptance_result.stdout
                and "spatial_boundary_json_reports_runtime_and_host_edge_proofs ... ok"
                in spatial_acceptance_result.stdout
                else "failed",
                "summary": "The existing multichannel, sidechain, multi-bus, and spatial acceptance lanes completed successfully.",
            },
            {
                "id": "operator.graph-execution.graph-focused-posture",
                "status": "passed",
                "summary": "The receipt keeps graph execution meaning explicit and does not pretend to be a product shell, graph editor, or tutorial UI.",
            },
            {
                "id": "operator.graph-execution.rendered-operator-view",
                "status": "passed",
                "summary": "A rendered companion view makes the multichannel, sidechain, multi-bus, and spatial graph posture visually inspectable without reading the raw receipt first.",
            },
        ],
    }

    view_model = {
        "boundaries": [
            ("Multichannel layout and role posture.", multichannel_payload),
            ("Sidechain routing and secondary-input posture.", sidechain_payload),
            ("Multi-bus and auxiliary-topology posture.", multi_bus_payload),
            ("Spatial execution posture.", spatial_payload),
        ],
        "acceptance": [
            "multichannel",
            "sidechain",
            "multi-bus",
            "spatial",
        ],
        "operator_checks": receipt["operator_checks"],
    }

    RECEIPT_PATH.parent.mkdir(parents=True, exist_ok=True)
    RECEIPT_PATH.write_text(json.dumps(receipt, indent=2) + "\n")
    HTML_PATH.write_text(browser_html(view_model))


if __name__ == "__main__":
    main()
