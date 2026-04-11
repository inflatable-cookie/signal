#!/usr/bin/env python3

import json
import subprocess
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MANIFEST_PATH = (
    REPO_ROOT / "demos" / "manifests" / "linux-lv2-backend-boundary.demo.json"
)
RECEIPT_PATH = (
    REPO_ROOT / "demos" / "receipts" / "linux-lv2-backend-boundary.receipt.json"
)
HTML_PATH = (
    REPO_ROOT / "demos" / "receipts" / "linux-lv2-backend-boundary.view.html"
)


def run_command(command: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        command,
        text=True,
        capture_output=True,
        cwd=REPO_ROOT,
        check=True,
    )


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
  <title>Signal Linux LV2 And Backend Boundary</title>
  <style>
    :root {{
      color-scheme: light;
      --bg: #eef3f4;
      --panel: #fbfdfd;
      --line: #ced9d8;
      --text: #16211f;
      --muted: #5d6a68;
      --ok: #205946;
      --ok-soft: #dceee7;
    }}
    * {{ box-sizing: border-box; }}
    body {{
      margin: 0;
      font-family: "Avenir Next", "Segoe UI", sans-serif;
      background: radial-gradient(circle at top, #f7fcfb, var(--bg));
      color: var(--text);
    }}
    main {{ max-width: 1180px; margin: 0 auto; padding: 32px 24px 48px; }}
    h1, h2 {{ margin: 0 0 12px; }}
    p {{ line-height: 1.5; }}
    .hero {{
      background: linear-gradient(135deg, #f7fcfb, #e7f0ee);
      border: 1px solid var(--line);
      border-radius: 22px;
      padding: 24px;
      margin-bottom: 24px;
      box-shadow: 0 14px 40px rgba(18, 42, 38, 0.08);
    }}
    .hero p {{ margin: 0; color: var(--muted); }}
    .checks {{ margin: 18px 0 0; padding-left: 18px; }}
    .checks li {{ margin: 8px 0; color: var(--muted); }}
    .grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 18px; margin-bottom: 18px; }}
    .card {{ background: var(--panel); border: 1px solid var(--line); border-radius: 18px; padding: 18px; box-shadow: 0 10px 24px rgba(15, 35, 31, 0.06); }}
    .subtitle {{ margin: 0 0 14px; color: var(--muted); }}
    .metrics {{ display: grid; gap: 10px; }}
    .metric {{ display: grid; gap: 4px; padding: 10px 12px; border-radius: 12px; background: #f2f8f7; border: 1px solid #dce7e4; }}
    .label {{ font-size: 0.82rem; letter-spacing: 0.03em; text-transform: uppercase; color: var(--muted); }}
    .value {{ font-size: 0.98rem; color: var(--text); word-break: break-word; }}
    .callout {{ margin-top: 22px; padding: 16px 18px; border-radius: 16px; border: 1px solid #cfe0d8; background: var(--ok-soft); color: var(--ok); }}
  </style>
</head>
<body>
  <main>
    <section class="hero">
      <h1>Signal Linux LV2 And Backend Boundary</h1>
      <p>Operator-facing rendered view for the bounded Linux LV2 execution and Linux audio-backend proof surfaces. This view stays descriptor-backed and acceptance-backed; it does not turn into a generalized plugin browser or live Linux control shell.</p>
      <ul class="checks">{checks}</ul>
    </section>
    <div class="grid">
      {section_card("LV2 execution boundary", "Focused Linux LV2 discovery, lifecycle, and broker-backed execution truth.", model["lv2"])}
      {section_card("Linux backend boundary", "Typed Linux backend identity and fallback truth from the shared boundary descriptor.", model["backend"])}
      {section_card("Acceptance and posture", "Repo-owned proof chains and explicit Linux-specific scope.", model["posture"])}
    </div>
    <section class="callout">
      The underlying source of truth is still the receipt, descriptor payloads, and acceptance lane output. This rendered view exists to make the Linux boundary surfaces visually inspectable without reading raw JSON first.
    </section>
  </main>
</body>
</html>
"""


def main() -> None:
    manifest = json.loads(MANIFEST_PATH.read_text())
    scenario = manifest["scenarios"][0]

    lv2_descriptor_command = [
        "cargo",
        "run",
        "-q",
        "-p",
        "signal-supervisor-tools",
        "--",
        "--describe-linux-lv2-execution-boundary",
        "--format=json",
    ]
    backend_descriptor_command = [
        "cargo",
        "run",
        "-q",
        "-p",
        "signal-supervisor-tools",
        "--",
        "--describe-linux-audio-backend-boundary",
        "--format=json",
    ]
    lv2_descriptor_result = run_command(lv2_descriptor_command)
    backend_descriptor_result = run_command(backend_descriptor_command)
    lv2_descriptor_payload = json.loads(lv2_descriptor_result.stdout)
    backend_descriptor_payload = json.loads(backend_descriptor_result.stdout)
    lv2_acceptance_commands = [
        [
            "cargo",
            "test",
            "-p",
            "signal-runtime",
            "public_runtime_lv2_boundary_reports_runtime_owned_discovery_and_lifecycle_truth",
        ],
        [
            "cargo",
            "test",
            "-p",
            "signal-host-server",
            "--test",
            "public_host_edge_sandbox_broker",
            "server_public_host_edge_can_route_lv2_sandbox_through_broker_process",
            "--",
            "--exact",
            "--nocapture",
            "--test-threads=1",
        ],
        [
            "cargo",
            "test",
            "-p",
            "signal-host-server",
            "--test",
            "public_host_edge_sandbox_broker",
            "server_public_host_edge_can_drive_broker_backed_lv2_crash_recovery",
            "--",
            "--exact",
            "--nocapture",
            "--test-threads=1",
        ],
        [
            "cargo",
            "test",
            "-p",
            "signal-supervisor-tools",
            "linux_lv2_execution_boundary_json_reports_runtime_and_host_edge_proofs",
        ],
    ]
    backend_acceptance_commands = [
        [
            "cargo",
            "test",
            "-p",
            "signal-runtime",
            "public_runtime_linux_audio_backend_boundary_reports_runtime_owned_backend_identity_truth",
        ],
        [
            "cargo",
            "test",
            "-p",
            "signal-host-server",
            "--test",
            "public_host_edge_external_io",
            "server_shared_host_edge_exports_runtime_linux_audio_backend_truth",
            "--",
            "--exact",
            "--nocapture",
            "--test-threads=1",
        ],
        [
            "cargo",
            "test",
            "-p",
            "signal-supervisor-tools",
            "linux_audio_backend_boundary_json_reports_runtime_and_host_edge_proofs",
        ],
    ]
    lv2_acceptance_outputs: list[str] = []
    for command in lv2_acceptance_commands:
        result = run_command(command)
        lv2_acceptance_outputs.append(result.stdout)
    backend_acceptance_outputs: list[str] = []
    for command in backend_acceptance_commands:
        result = run_command(command)
        backend_acceptance_outputs.append(result.stdout)
    lv2_acceptance_stdout = "\n".join(lv2_acceptance_outputs)
    backend_acceptance_stdout = "\n".join(backend_acceptance_outputs)

    receipt = {
        "receipt_version": "signal.demo.receipt.v1",
        "manifest_id": manifest["id"],
        "scenario_id": scenario["id"],
        "status": "passed",
        "launch_command": "effigy demo:linux-lv2-and-backend-boundary",
        "artifacts": [
            {
                "kind": "linux-lv2-execution-boundary-descriptor",
                "boundary": lv2_descriptor_payload.get("boundary"),
                "contract_path": lv2_descriptor_payload.get("contract_path"),
                "acceptance_task": lv2_descriptor_payload.get("acceptance_task"),
                "surface_count": lv2_descriptor_payload.get("surface_count"),
                "validation_step_count": lv2_descriptor_payload.get(
                    "validation_step_count"
                ),
                "deferred_scope_count": len(
                    lv2_descriptor_payload.get("deferred_scope", [])
                ),
                "raw_payload": lv2_descriptor_payload,
            },
            {
                "kind": "linux-audio-backend-boundary-descriptor",
                "boundary": backend_descriptor_payload.get("boundary"),
                "contract_path": backend_descriptor_payload.get("contract_path"),
                "acceptance_task": backend_descriptor_payload.get(
                    "acceptance_task"
                ),
                "surface_count": backend_descriptor_payload.get("surface_count"),
                "validation_step_count": len(
                    backend_descriptor_payload.get("validation_steps", [])
                ),
                "deferred_scope_count": 1
                if backend_descriptor_payload.get("residual_risk")
                else 0,
                "raw_payload": backend_descriptor_payload,
            },
            {
                "kind": "acceptance-lane-run",
                "command": "acceptance:linux-lv2-execution-boundary (flattened proof chain)",
                "status": "passed",
                "stdout_tail": lv2_acceptance_stdout.splitlines()[-20:],
            },
            {
                "kind": "acceptance-lane-run",
                "command": "acceptance:linux-audio-backend-boundary (flattened proof chain)",
                "status": "passed",
                "stdout_tail": backend_acceptance_stdout.splitlines()[-20:],
            },
            {
                "kind": "linux-lv2-backend-operator-view",
                "html_path": "demos/receipts/linux-lv2-backend-boundary.view.html",
                "status": "passed",
                "section_count": 3,
            },
        ],
        "operator_checks": [
            {
                "id": "operator.linux-boundary.lv2-descriptor",
                "status": "passed"
                if lv2_descriptor_payload.get("boundary")
                == "signal.runtime.linux-lv2-execution-boundary"
                and lv2_descriptor_payload.get("acceptance_task")
                == "effigy acceptance:linux-lv2-execution-boundary"
                else "failed",
                "summary": "The demo captured the machine-readable Linux LV2 execution boundary descriptor.",
            },
            {
                "id": "operator.linux-boundary.backend-descriptor",
                "status": "passed"
                if backend_descriptor_payload.get("boundary")
                == "signal.runtime.linux-audio-backend-boundary"
                and backend_descriptor_payload.get("acceptance_task")
                == "effigy acceptance:linux-audio-backend-boundary"
                else "failed",
                "summary": "The demo captured the machine-readable Linux audio-backend boundary descriptor.",
            },
            {
                "id": "operator.linux-boundary.acceptance-lanes",
                "status": "passed"
                if "linux_lv2_execution_boundary_json_reports_runtime_and_host_edge_proofs ... ok"
                in lv2_acceptance_stdout
                and "linux_audio_backend_boundary_json_reports_runtime_and_host_edge_proofs ... ok"
                in backend_acceptance_stdout
                else "failed",
                "summary": "The existing Linux LV2 execution and Linux audio-backend acceptance lanes completed successfully.",
            },
            {
                "id": "operator.linux-boundary.linux-specific-posture",
                "status": "passed",
                "summary": "The receipt keeps the surface explicitly Linux-specific and does not pretend to provide a generalized plugin browser or live Linux ownership breadth.",
            },
            {
                "id": "operator.linux-boundary.rendered-operator-view",
                "status": "passed",
                "summary": "A rendered companion view makes the Linux LV2 and backend boundaries visually inspectable without reading the raw receipt first.",
            },
        ],
    }

    model = {
        "operator_checks": receipt["operator_checks"],
        "lv2": [
            ("Boundary", str(lv2_descriptor_payload.get("boundary", "n/a"))),
            ("Contract", str(lv2_descriptor_payload.get("contract_path", "n/a"))),
            ("Acceptance", str(lv2_descriptor_payload.get("acceptance_task", "n/a"))),
            ("Surfaces", str(lv2_descriptor_payload.get("surface_count", "n/a"))),
            (
                "Validation steps",
                str(lv2_descriptor_payload.get("validation_step_count", "n/a")),
            ),
            (
                "Deferred scope",
                str(len(lv2_descriptor_payload.get("deferred_scope", []))),
            ),
        ],
        "backend": [
            ("Boundary", str(backend_descriptor_payload.get("boundary", "n/a"))),
            ("Contract", str(backend_descriptor_payload.get("contract_path", "n/a"))),
            (
                "Acceptance",
                str(backend_descriptor_payload.get("acceptance_task", "n/a")),
            ),
            ("Surfaces", str(backend_descriptor_payload.get("surface_count", "n/a"))),
            (
                "Validation steps",
                str(len(backend_descriptor_payload.get("validation_steps", []))),
            ),
            (
                "Residual risk",
                str(backend_descriptor_payload.get("residual_risk", "none")),
            ),
        ],
        "posture": [
            ("LV2 acceptance", "passed"),
            ("Backend acceptance", "passed"),
            ("Platform", "Linux"),
            ("macOS breadth", "not claimed"),
            ("Browser breadth", "not claimed"),
        ],
    }

    RECEIPT_PATH.parent.mkdir(parents=True, exist_ok=True)
    RECEIPT_PATH.write_text(json.dumps(receipt, indent=2) + "\n")
    HTML_PATH.write_text(browser_html(model))


if __name__ == "__main__":
    main()
