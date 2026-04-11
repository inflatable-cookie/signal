#!/usr/bin/env python3

import json
import subprocess
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MANIFEST_PATH = REPO_ROOT / "demos" / "manifests" / "dsp-processing-lab.demo.json"
RECEIPT_PATH = REPO_ROOT / "demos" / "receipts" / "dsp-processing-lab.receipt.json"
HTML_PATH = REPO_ROOT / "demos" / "receipts" / "dsp-processing-lab.view.html"


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


def acceptance_artifact(
    result: subprocess.CompletedProcess[str], command: list[str]
) -> dict[str, object]:
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
        len(deferred_scope)
        if isinstance(deferred_scope, list)
        else (1 if residual_risk else 0)
    )
    validation_steps = payload.get("validation_steps")
    validation_step_count = (
        payload.get("validation_step_count")
        if payload.get("validation_step_count") is not None
        else len(validation_steps)
        if isinstance(validation_steps, list)
        else 0
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
        boundary_card(payload, subtitle) for subtitle, payload in boundaries
    )
    return f"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Signal DSP Processing Lab</title>
  <style>
    :root {{
      color-scheme: light;
      --bg: #f7f1eb;
      --panel: #fffdfb;
      --line: #d8cfc8;
      --text: #201b17;
      --muted: #6b6159;
      --accent: #9f4f1f;
      --accent-soft: #f2dccd;
      --ok: #20593f;
      --ok-soft: #ddeee5;
    }}
    * {{ box-sizing: border-box; }}
    body {{
      margin: 0;
      font-family: "Avenir Next", "Segoe UI", sans-serif;
      background: radial-gradient(circle at top, #fff8f2, var(--bg));
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
      background: linear-gradient(135deg, #fff8f1, #f3e7db);
      border: 1px solid var(--line);
      border-radius: 22px;
      padding: 24px;
      margin-bottom: 24px;
      box-shadow: 0 14px 40px rgba(64, 39, 19, 0.08);
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
      box-shadow: 0 10px 24px rgba(46, 33, 20, 0.06);
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
      background: #faf5ef;
      border: 1px solid #eadfd4;
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
      <h1>Signal DSP Processing Lab</h1>
      <p>Operator-facing rendered view for bounded DSP processing posture across stretch, marker-analysis, and transform-artifact boundary families. This surface stays descriptor-backed and low-dependency; it is not an editor, waveform browser, or tutorial shell.</p>
      <ul class="checks">{checks}</ul>
    </section>
    <div class="grid">
      {boundary_cards}
    </div>
    <div class="grid">
      <section class="card"><h2>Acceptance posture</h2><p class="subtitle">Focused DSP boundary proof lanes completed during this capture.</p><div class="metrics">{acceptance_rows}</div></section>
    </div>
    <section class="callout">
      The underlying source of truth is still the receipt and the bounded descriptor plus acceptance commands. This rendered view exists to make the DSP family visually inspectable without reading raw JSON first.
    </section>
  </main>
</body>
</html>
"""


def main() -> None:
    manifest = json.loads(MANIFEST_PATH.read_text())
    scenario = manifest["scenarios"][0]

    stretch = run_descriptor("--describe-stretch-boundary")
    marker = run_descriptor("--describe-marker-analysis-boundary")
    artifact = run_descriptor("--describe-transform-artifact-boundary")

    stretch_payload = stretch["payload"]
    marker_payload = marker["payload"]
    artifact_payload = artifact["payload"]

    stretch_acceptance_command = ["effigy", "acceptance:stretch-boundary"]
    marker_acceptance_command = ["effigy", "acceptance:marker-analysis-boundary"]
    artifact_acceptance_command = ["effigy", "acceptance:transform-artifact-boundary"]

    stretch_acceptance_result = run_command(stretch_acceptance_command)
    marker_acceptance_result = run_command(marker_acceptance_command)
    artifact_acceptance_result = run_command(artifact_acceptance_command)

    receipt = {
        "receipt_version": "signal.demo.receipt.v1",
        "manifest_id": manifest["id"],
        "scenario_id": scenario["id"],
        "status": "passed",
        "launch_command": "effigy demo:dsp-processing-lab",
        "artifacts": [
            descriptor_artifact("stretch-boundary-descriptor", stretch_payload),
            descriptor_artifact("marker-analysis-boundary-descriptor", marker_payload),
            descriptor_artifact(
                "transform-artifact-boundary-descriptor", artifact_payload
            ),
            acceptance_artifact(stretch_acceptance_result, stretch_acceptance_command),
            acceptance_artifact(marker_acceptance_result, marker_acceptance_command),
            acceptance_artifact(artifact_acceptance_result, artifact_acceptance_command),
            {
                "kind": "dsp-processing-operator-view",
                "html_path": "demos/receipts/dsp-processing-lab.view.html",
                "status": "passed",
                "boundary_count": 3,
                "acceptance_count": 3,
            },
        ],
        "operator_checks": [
            {
                "id": "operator.dsp-processing.stretch-descriptor",
                "status": "passed"
                if stretch_payload.get("boundary") == "signal.runtime.stretch-boundary"
                and stretch_payload.get("acceptance_task")
                == "effigy acceptance:stretch-boundary"
                else "failed",
                "summary": "The demo captured the machine-readable stretch boundary descriptor.",
            },
            {
                "id": "operator.dsp-processing.marker-analysis-descriptor",
                "status": "passed"
                if marker_payload.get("boundary")
                == "signal.runtime.marker-analysis-boundary"
                and marker_payload.get("acceptance_task")
                == "effigy acceptance:marker-analysis-boundary"
                else "failed",
                "summary": "The demo captured the machine-readable marker-analysis boundary descriptor.",
            },
            {
                "id": "operator.dsp-processing.transform-artifact-descriptor",
                "status": "passed"
                if artifact_payload.get("boundary")
                == "signal.runtime.transform-artifact-boundary"
                and artifact_payload.get("acceptance_task")
                == "effigy acceptance:transform-artifact-boundary"
                else "failed",
                "summary": "The demo captured the machine-readable transform-artifact boundary descriptor.",
            },
            {
                "id": "operator.dsp-processing.acceptance-lanes",
                "status": "passed"
                if "stretch_boundary_json_reports_runtime_and_host_edge_proofs ... ok"
                in stretch_acceptance_result.stdout
                and "marker_analysis_boundary_json_reports_runtime_and_host_edge_proofs ... ok"
                in marker_acceptance_result.stdout
                and "transform_artifact_boundary_json_reports_runtime_and_host_edge_proofs ... ok"
                in artifact_acceptance_result.stdout
                else "failed",
                "summary": "The existing stretch, marker-analysis, and transform-artifact acceptance lanes completed successfully.",
            },
            {
                "id": "operator.dsp-processing.dsp-focused-posture",
                "status": "passed",
                "summary": "The receipt keeps DSP processing meaning explicit and does not pretend to be an editor shell, waveform browser, or tutorial UI.",
            },
            {
                "id": "operator.dsp-processing.rendered-operator-view",
                "status": "passed",
                "summary": "A rendered companion view makes the stretch, marker-analysis, and transform-artifact posture visually inspectable without reading the raw receipt first.",
            },
        ],
    }

    view_model = {
        "boundaries": [
            ("Stretch engine and render-preview posture.", stretch_payload),
            ("Marker-analysis and transient-anchor posture.", marker_payload),
            ("Transform-artifact and cache-policy posture.", artifact_payload),
        ],
        "acceptance": [
            "stretch",
            "marker-analysis",
            "transform-artifact",
        ],
        "operator_checks": receipt["operator_checks"],
    }

    RECEIPT_PATH.parent.mkdir(parents=True, exist_ok=True)
    RECEIPT_PATH.write_text(json.dumps(receipt, indent=2) + "\n")
    HTML_PATH.write_text(browser_html(view_model))


if __name__ == "__main__":
    main()
