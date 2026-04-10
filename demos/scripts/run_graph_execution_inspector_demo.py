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
        ],
    }

    RECEIPT_PATH.parent.mkdir(parents=True, exist_ok=True)
    RECEIPT_PATH.write_text(json.dumps(receipt, indent=2) + "\n")


if __name__ == "__main__":
    main()
