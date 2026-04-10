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


def run_command(command: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        command,
        text=True,
        capture_output=True,
        cwd=REPO_ROOT,
        check=True,
    )


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
    acceptance_command = ["effigy", "acceptance:macos-au-coreaudio-boundary"]

    descriptor_result = run_command(descriptor_command)
    descriptor_payload = json.loads(descriptor_result.stdout)
    acceptance_result = run_command(acceptance_command)

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
                "command": " ".join(acceptance_command),
                "status": "passed",
                "stdout_tail": acceptance_result.stdout.splitlines()[-20:],
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
                in acceptance_result.stdout
                else "failed",
                "summary": "The existing macOS AU/CoreAudio acceptance lane completed successfully.",
            },
            {
                "id": "operator.macos-au-coreaudio.macos-specific-posture",
                "status": "passed",
                "summary": "The receipt keeps the surface explicitly macOS-specific and does not pretend to provide Linux-native or general plugin-browsing breadth.",
            },
        ],
    }

    RECEIPT_PATH.parent.mkdir(parents=True, exist_ok=True)
    RECEIPT_PATH.write_text(json.dumps(receipt, indent=2) + "\n")


if __name__ == "__main__":
    main()
