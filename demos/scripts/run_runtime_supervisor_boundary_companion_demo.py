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
            }
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
        ],
    }

    RECEIPT_PATH.parent.mkdir(parents=True, exist_ok=True)
    RECEIPT_PATH.write_text(json.dumps(receipt, indent=2) + "\n")


if __name__ == "__main__":
    main()
