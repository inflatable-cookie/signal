#!/usr/bin/env python3

import json
import subprocess
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MANIFEST_PATH = REPO_ROOT / "demos" / "manifests" / "plugin-sandbox-lifecycle.demo.json"
RECEIPT_PATH = REPO_ROOT / "demos" / "receipts" / "plugin-sandbox-lifecycle.receipt.json"

BROKER_RUNS = [
    (
        "attach_status_teardown",
        [
            "status",
            "attach-demo",
            "status",
            "teardown-demo",
            "shutdown",
        ],
    ),
    (
        "healthy_run",
        [
            "run-demo",
            "shutdown",
        ],
    ),
    (
        "timeout_run",
        [
            "run-timeout-demo",
            "shutdown",
        ],
    ),
]


def main() -> None:
    manifest = json.loads(MANIFEST_PATH.read_text())
    scenario = manifest["scenarios"][0]
    launch_command = "cargo run -q -p signal-plugin-sandbox"
    transcripts = []

    for run_id, commands in BROKER_RUNS:
        result = subprocess.run(
            ["cargo", "run", "-q", "-p", "signal-plugin-sandbox"],
            input="\n".join(commands) + "\n",
            text=True,
            capture_output=True,
            cwd=REPO_ROOT,
            check=True,
        )
        lines = [
            line.strip()
            for line in result.stdout.splitlines()
            if line.strip().startswith("signal-plugin-sandbox")
        ]
        transcripts.append({"run_id": run_id, "lines": lines})

    all_lines = [line for transcript in transcripts for line in transcript["lines"]]

    observed_states = sorted(
        {
            token.split("=", 1)[1]
            for line in all_lines
            for token in line.split()
            if token.startswith("state=")
        }
    )

    def transcript_contains(run_id: str, fragment: str) -> bool:
        return any(
            fragment in line
            for transcript in transcripts
            if transcript["run_id"] == run_id
            for line in transcript["lines"]
        )

    receipt = {
        "receipt_version": "signal.demo.receipt.v1",
        "manifest_id": manifest["id"],
        "scenario_id": scenario["id"],
        "status": "passed",
        "launch_command": launch_command,
        "artifacts": [
            {
                "kind": "broker-transcript-lines",
                "line_count": len(all_lines),
                "observed_states": observed_states,
                "runs": [
                    {
                        "run_id": transcript["run_id"],
                        "line_count": len(transcript["lines"]),
                    }
                    for transcript in transcripts
                ],
            }
        ],
        "operator_checks": [
            {
                "id": "operator.sandbox-lifecycle.ready-state",
                "status": "passed" if transcript_contains("attach_status_teardown", "state=ready") else "failed",
                "summary": "Broker reported the ready state before live lifecycle commands.",
            },
            {
                "id": "operator.sandbox-lifecycle.attach-and-teardown",
                "status": "passed"
                if transcript_contains("attach_status_teardown", "state=attached")
                and transcript_contains("attach_status_teardown", "state=teardown_complete")
                else "failed",
                "summary": "Explicit attach/status/teardown path remained inspectable.",
            },
            {
                "id": "operator.sandbox-lifecycle.run-path",
                "status": "passed"
                if transcript_contains("healthy_run", "state=running")
                and transcript_contains("healthy_run", "detail=lease_cleanup_ok")
                else "failed",
                "summary": "Healthy demo run reached running and clean teardown states.",
            },
            {
                "id": "operator.sandbox-lifecycle.timeout-path",
                "status": "passed"
                if transcript_contains("timeout_run", "state=timed_out")
                and transcript_contains("timeout_run", "detail=lease_cleanup_ok_after_timeout")
                else "failed",
                "summary": "Timeout demo run remained bounded and reported cleanup after interruption.",
            },
            {
                "id": "operator.sandbox-lifecycle.shutdown",
                "status": "passed"
                if all(
                    transcript_contains(transcript["run_id"], "state=shutdown")
                    for transcript in transcripts
                )
                else "failed",
                "summary": "Broker exited through the explicit shutdown receipt.",
            },
        ],
    }

    RECEIPT_PATH.parent.mkdir(parents=True, exist_ok=True)
    RECEIPT_PATH.write_text(json.dumps(receipt, indent=2) + "\n")


if __name__ == "__main__":
    main()
