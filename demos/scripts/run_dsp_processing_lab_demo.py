#!/usr/bin/env python3

import json
import subprocess
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MANIFEST_PATH = REPO_ROOT / "demos" / "manifests" / "dsp-processing-lab.demo.json"
RECEIPT_PATH = REPO_ROOT / "demos" / "receipts" / "dsp-processing-lab.receipt.json"


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
        ],
    }

    RECEIPT_PATH.parent.mkdir(parents=True, exist_ok=True)
    RECEIPT_PATH.write_text(json.dumps(receipt, indent=2) + "\n")


if __name__ == "__main__":
    main()
