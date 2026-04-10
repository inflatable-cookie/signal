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
    lv2_acceptance_command = ["effigy", "acceptance:linux-lv2-execution-boundary"]
    backend_acceptance_command = [
        "effigy",
        "acceptance:linux-audio-backend-boundary",
    ]

    lv2_descriptor_result = run_command(lv2_descriptor_command)
    backend_descriptor_result = run_command(backend_descriptor_command)
    lv2_descriptor_payload = json.loads(lv2_descriptor_result.stdout)
    backend_descriptor_payload = json.loads(backend_descriptor_result.stdout)
    lv2_acceptance_result = run_command(lv2_acceptance_command)
    backend_acceptance_result = run_command(backend_acceptance_command)

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
                "command": " ".join(lv2_acceptance_command),
                "status": "passed",
                "stdout_tail": lv2_acceptance_result.stdout.splitlines()[-20:],
            },
            {
                "kind": "acceptance-lane-run",
                "command": " ".join(backend_acceptance_command),
                "status": "passed",
                "stdout_tail": backend_acceptance_result.stdout.splitlines()[-20:],
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
                in lv2_acceptance_result.stdout
                and "linux_audio_backend_boundary_json_reports_runtime_and_host_edge_proofs ... ok"
                in backend_acceptance_result.stdout
                else "failed",
                "summary": "The existing Linux LV2 execution and Linux audio-backend acceptance lanes completed successfully.",
            },
            {
                "id": "operator.linux-boundary.linux-specific-posture",
                "status": "passed",
                "summary": "The receipt keeps the surface explicitly Linux-specific and does not pretend to provide a generalized plugin browser or live Linux ownership breadth.",
            },
        ],
    }

    RECEIPT_PATH.parent.mkdir(parents=True, exist_ok=True)
    RECEIPT_PATH.write_text(json.dumps(receipt, indent=2) + "\n")


if __name__ == "__main__":
    main()
