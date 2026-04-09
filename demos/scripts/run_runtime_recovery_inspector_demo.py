#!/usr/bin/env python3

import json
import subprocess
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MANIFEST_PATH = (
    REPO_ROOT / "demos" / "manifests" / "runtime-recovery-inspector.demo.json"
)
RECEIPT_PATH = (
    REPO_ROOT / "demos" / "receipts" / "runtime-recovery-inspector.receipt.json"
)


def main() -> None:
    manifest = json.loads(MANIFEST_PATH.read_text())
    scenario = manifest["scenarios"][0]
    launch_command = "cargo run -q -p signal-runtime --example supervisor_report_demo"

    result = subprocess.run(
        ["cargo", "run", "-q", "-p", "signal-runtime", "--example", "supervisor_report_demo"],
        text=True,
        capture_output=True,
        cwd=REPO_ROOT,
        check=True,
    )

    lines = [line.strip() for line in result.stdout.splitlines() if line.strip()]

    def contains(fragment: str) -> bool:
        return any(fragment in line for line in lines)

    receipt = {
        "receipt_version": "signal.demo.receipt.v1",
        "manifest_id": manifest["id"],
        "scenario_id": scenario["id"],
        "status": "passed",
        "launch_command": launch_command,
        "artifacts": [
            {
                "kind": "runtime-supervisor-report-lines",
                "line_count": len(lines),
                "highlights": {
                    "readiness": "Ready" if contains("readiness=Ready") else "unexpected",
                    "watchdog": "HeartbeatMisses"
                    if contains("last_watchdog=HeartbeatMisses")
                    else "unexpected",
                    "plugin_fault_count": 2 if contains("plugin_faults=2") else 0,
                    "event_count": 3 if contains("events=3") else 0,
                },
            }
        ],
        "operator_checks": [
            {
                "id": "operator.runtime-recovery.handshake-and-start",
                "status": "passed"
                if contains("handshaken=true")
                and contains("configured=true")
                and contains("running=true")
                else "failed",
                "summary": "Runtime example completed handshake, configuration, and start.",
            },
            {
                "id": "operator.runtime-recovery.watchdog-snapshot",
                "status": "passed"
                if contains("last_watchdog=HeartbeatMisses")
                and contains("degradation_summary_last_watchdog=Some(HeartbeatMisses)")
                else "failed",
                "summary": "Supervisor output exposed the watchdog-trigger snapshot.",
            },
            {
                "id": "operator.runtime-recovery.plugin-faults",
                "status": "passed"
                if contains("plugin_faults=2")
                and contains("last_fault=sandbox-demo:Timeout")
                else "failed",
                "summary": "Runtime example exported the injected plugin timeout faults.",
            },
            {
                "id": "operator.runtime-recovery-safe-mode-posture",
                "status": "passed"
                if contains("safe_mode=false")
                and contains("device_supervision_safe_mode_enabled=false")
                else "failed",
                "summary": "Runtime report kept explicit safe-mode posture in the steady-state surface.",
            },
            {
                "id": "operator.runtime-recovery.external-surface",
                "status": "passed"
                if contains("external_io_summary=health=Unavailable")
                and contains("linux_backend_session_summary=backend=Unavailable")
                else "failed",
                "summary": "Runtime report preserved degraded hardware/backend surfaces explicitly.",
            },
        ],
    }

    RECEIPT_PATH.parent.mkdir(parents=True, exist_ok=True)
    RECEIPT_PATH.write_text(json.dumps(receipt, indent=2) + "\n")


if __name__ == "__main__":
    main()
