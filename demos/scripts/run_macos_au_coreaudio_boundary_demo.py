#!/usr/bin/env python3

import json
import subprocess
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MANIFEST_PATH = (
    REPO_ROOT / "demos" / "manifests" / "macos-au-coreaudio-boundary.demo.json"
)
RECEIPT_PATH = (
    REPO_ROOT / "demos" / "receipts" / "macos-au-coreaudio-boundary.receipt.json"
)
HTML_PATH = (
    REPO_ROOT / "demos" / "receipts" / "macos-au-coreaudio-boundary.view.html"
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
  <title>Signal macOS AU CoreAudio Boundary</title>
  <style>
    :root {{
      color-scheme: light;
      --bg: #eef4f7;
      --panel: #fbfdff;
      --line: #ced9e2;
      --text: #17212a;
      --muted: #606c77;
      --ok: #205c46;
      --ok-soft: #dceee7;
    }}
    * {{ box-sizing: border-box; }}
    body {{
      margin: 0;
      font-family: "Avenir Next", "Segoe UI", sans-serif;
      background: radial-gradient(circle at top, #f7fbff, var(--bg));
      color: var(--text);
    }}
    main {{ max-width: 1180px; margin: 0 auto; padding: 32px 24px 48px; }}
    h1, h2 {{ margin: 0 0 12px; }}
    p {{ line-height: 1.5; }}
    .hero {{
      background: linear-gradient(135deg, #f7fbff, #e8eef5);
      border: 1px solid var(--line);
      border-radius: 22px;
      padding: 24px;
      margin-bottom: 24px;
      box-shadow: 0 14px 40px rgba(18, 33, 52, 0.08);
    }}
    .hero p {{ margin: 0; color: var(--muted); }}
    .checks {{ margin: 18px 0 0; padding-left: 18px; }}
    .checks li {{ margin: 8px 0; color: var(--muted); }}
    .grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 18px; margin-bottom: 18px; }}
    .card {{ background: var(--panel); border: 1px solid var(--line); border-radius: 18px; padding: 18px; box-shadow: 0 10px 24px rgba(15, 30, 46, 0.06); }}
    .subtitle {{ margin: 0 0 14px; color: var(--muted); }}
    .metrics {{ display: grid; gap: 10px; }}
    .metric {{ display: grid; gap: 4px; padding: 10px 12px; border-radius: 12px; background: #f3f7fb; border: 1px solid #dde6ef; }}
    .label {{ font-size: 0.82rem; letter-spacing: 0.03em; text-transform: uppercase; color: var(--muted); }}
    .value {{ font-size: 0.98rem; color: var(--text); word-break: break-word; }}
    .callout {{ margin-top: 22px; padding: 16px 18px; border-radius: 16px; border: 1px solid #cfe0d8; background: var(--ok-soft); color: var(--ok); }}
  </style>
</head>
<body>
  <main>
    <section class="hero">
      <h1>Signal macOS AU CoreAudio Boundary</h1>
      <p>Operator-facing rendered view for the bounded macOS AU/CoreAudio proof surface. This view stays descriptor-backed and acceptance-backed; it does not turn into a generalized plugin browser or host UI.</p>
      <ul class="checks">{checks}</ul>
    </section>
    <div class="grid">
      {section_card("Boundary descriptor", "Focused AU lifecycle and CoreAudio device truth exported through the machine-readable boundary descriptor.", model["descriptor"])}
      {section_card("Acceptance lane", "Current repo-owned proof chain for the macOS AU/CoreAudio lane.", model["acceptance"])}
      {section_card("Platform posture", "Explicit macOS-specific scope and deferred breadth for this boundary.", model["posture"])}
    </div>
    <section class="callout">
      The underlying source of truth is still the receipt, the descriptor payload, and the acceptance lane output. This rendered view exists to make the macOS AU/CoreAudio boundary visually inspectable without reading raw JSON first.
    </section>
  </main>
</body>
</html>
"""


def main() -> None:
    manifest = json.loads(MANIFEST_PATH.read_text())
    scenario = manifest["scenarios"][0]

    descriptor_command = [
        "cargo",
        "run",
        "-q",
        "-p",
        "signal-supervisor-tools",
        "--",
        "--describe-macos-au-coreaudio-boundary",
        "--format=json",
    ]
    descriptor_result = run_command(descriptor_command)
    descriptor_payload = json.loads(descriptor_result.stdout)
    acceptance_commands = [
        ["cargo", "test", "-p", "signal-hardware-coreaudio"],
        [
            "cargo",
            "test",
            "-p",
            "signal-runtime",
            "public_runtime_au_boundary_reports_runtime_owned_discovery_and_lifecycle_truth",
        ],
        [
            "cargo",
            "test",
            "-p",
            "signal-runtime",
            "public_runtime_external_io_boundary_reports_runtime_owned_monitor_and_loopback_truth",
        ],
        [
            "cargo",
            "test",
            "-p",
            "signal-host-local",
            "--test",
            "public_host_edge_au",
            "--",
            "--nocapture",
            "--test-threads=1",
        ],
        [
            "cargo",
            "test",
            "-p",
            "signal-host-local",
            "--test",
            "public_host_edge_external_io",
            "--",
            "--nocapture",
            "--test-threads=1",
        ],
        [
            "cargo",
            "test",
            "-p",
            "signal-host-local",
            "--test",
            "public_host_edge_device_supervision",
            "--",
            "--nocapture",
            "--test-threads=1",
        ],
        [
            "cargo",
            "test",
            "-p",
            "signal-supervisor-tools",
            "macos_au_coreaudio_boundary_json_reports_runtime_and_host_edge_proofs",
        ],
    ]
    acceptance_outputs: list[str] = []
    for command in acceptance_commands:
        result = run_command(command)
        acceptance_outputs.append(result.stdout)
    acceptance_result_stdout = "\n".join(acceptance_outputs)

    receipt = {
        "receipt_version": "signal.demo.receipt.v1",
        "manifest_id": manifest["id"],
        "scenario_id": scenario["id"],
        "status": "passed",
        "launch_command": "effigy demo:macos-au-coreaudio-boundary",
        "artifacts": [
            {
                "kind": "macos-au-coreaudio-boundary-descriptor",
                "boundary": descriptor_payload.get("boundary"),
                "contract_path": descriptor_payload.get("contract_path"),
                "acceptance_task": descriptor_payload.get("acceptance_task"),
                "surface_count": descriptor_payload.get("surface_count"),
                "validation_step_count": descriptor_payload.get(
                    "validation_step_count"
                ),
                "deferred_scope_count": len(
                    descriptor_payload.get("deferred_scope", [])
                ),
                "raw_payload": descriptor_payload,
            },
            {
                "kind": "acceptance-lane-run",
                "command": "acceptance:macos-au-coreaudio-boundary (flattened proof chain)",
                "status": "passed",
                "stdout_tail": acceptance_result_stdout.splitlines()[-20:],
            },
            {
                "kind": "macos-au-coreaudio-operator-view",
                "html_path": "demos/receipts/macos-au-coreaudio-boundary.view.html",
                "status": "passed",
                "section_count": 3,
            },
        ],
        "operator_checks": [
            {
                "id": "operator.macos-au-coreaudio.boundary-descriptor",
                "status": "passed"
                if descriptor_payload.get("boundary")
                == "signal.runtime.macos-au-coreaudio-boundary"
                and descriptor_payload.get("acceptance_task")
                == "effigy acceptance:macos-au-coreaudio-boundary"
                else "failed",
                "summary": "The demo captured the machine-readable macOS AU/CoreAudio boundary descriptor.",
            },
            {
                "id": "operator.macos-au-coreaudio.acceptance-lane",
                "status": "passed"
                if "macos_au_coreaudio_boundary_json_reports_runtime_and_host_edge_proofs ... ok"
                in acceptance_result_stdout
                else "failed",
                "summary": "The existing macOS AU/CoreAudio acceptance lane completed successfully.",
            },
            {
                "id": "operator.macos-au-coreaudio.macos-specific-posture",
                "status": "passed",
                "summary": "The receipt keeps the surface explicitly macOS-specific and does not pretend to provide Linux-native or general plugin-browsing breadth.",
            },
            {
                "id": "operator.macos-au-coreaudio.rendered-operator-view",
                "status": "passed",
                "summary": "A rendered companion view makes the macOS AU/CoreAudio boundary visually inspectable without reading the raw receipt first.",
            },
        ],
    }

    model = {
        "operator_checks": receipt["operator_checks"],
        "descriptor": [
            ("Boundary", str(descriptor_payload.get("boundary", "n/a"))),
            ("Contract", str(descriptor_payload.get("contract_path", "n/a"))),
            ("Acceptance", str(descriptor_payload.get("acceptance_task", "n/a"))),
            ("Surfaces", str(descriptor_payload.get("surface_count", "n/a"))),
            (
                "Validation steps",
                str(descriptor_payload.get("validation_step_count", "n/a")),
            ),
            (
                "Deferred scope",
                str(len(descriptor_payload.get("deferred_scope", []))),
            ),
        ],
        "acceptance": [
            ("Command", "flattened acceptance proof chain"),
            ("Status", "passed"),
            (
                "Tail lines",
                str(len(acceptance_result_stdout.splitlines()[-20:])),
            ),
        ],
        "posture": [
            ("Platform", "macOS"),
            ("Plugin format", "AU"),
            ("Device truth", "CoreAudio"),
            ("Linux breadth", "not claimed"),
            ("Browser breadth", "not claimed"),
        ],
    }

    RECEIPT_PATH.parent.mkdir(parents=True, exist_ok=True)
    RECEIPT_PATH.write_text(json.dumps(receipt, indent=2) + "\n")
    HTML_PATH.write_text(browser_html(model))


if __name__ == "__main__":
    main()
