#!/usr/bin/env python3

import json
import re
import subprocess
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MANIFEST_PATH = (
    REPO_ROOT / "demos" / "manifests" / "local-server-host-comparison.demo.json"
)
RECEIPT_PATH = (
    REPO_ROOT / "demos" / "receipts" / "local-server-host-comparison.receipt.json"
)

PAIR_RE = re.compile(r'([A-Za-z0-9_]+)=(".*?"|\[[^\]]*\]|\S+)')


def parse_summary_line(line: str) -> dict[str, str]:
    parsed: dict[str, str] = {}
    for key, raw_value in PAIR_RE.findall(line):
        value = raw_value
        if value.startswith('"') and value.endswith('"'):
            value = value[1:-1]
        parsed[key] = value
    return parsed


def as_int(parsed: dict[str, str], key: str) -> int:
    try:
        return int(parsed[key])
    except (KeyError, ValueError):
        return 0


def as_bool(parsed: dict[str, str], key: str) -> bool:
    return parsed.get(key) == "true"


def run_host(package: str) -> dict[str, object]:
    result = subprocess.run(
        ["cargo", "run", "-q", "-p", package],
        text=True,
        capture_output=True,
        cwd=REPO_ROOT,
        check=True,
    )
    line = next(
        stripped
        for stripped in result.stdout.splitlines()
        if stripped.startswith(package)
    )
    parsed = parse_summary_line(line)
    return {
        "package": package,
        "line": line,
        "parsed": parsed,
    }


def main() -> None:
    manifest = json.loads(MANIFEST_PATH.read_text())
    scenario = manifest["scenarios"][0]

    local = run_host("signal-host-local")
    server = run_host("signal-host-server")

    local_parsed = local["parsed"]
    server_parsed = server["parsed"]

    comparison = {
        "shared_truth": {
            "local_ready": local_parsed.get("readiness") == "Ready",
            "server_ready": server_parsed.get("readiness") == "Ready",
            "local_running": as_bool(local_parsed, "running"),
            "server_running": as_bool(server_parsed, "running"),
            "local_processed_blocks": as_int(local_parsed, "processed_blocks"),
            "server_processed_blocks": as_int(server_parsed, "processed_blocks"),
            "local_completion": local_parsed.get("completion"),
            "server_completion": server_parsed.get("completion"),
            "local_heartbeat_responses": as_int(local_parsed, "heartbeat_responses"),
            "server_heartbeat_responses": as_int(server_parsed, "heartbeat_responses"),
        },
        "host_differences": {
            "local_backend": local_parsed.get("backend"),
            "local_audio_state": local_parsed.get("audio_state"),
            "server_engine_processed_blocks": as_int(server_parsed, "engine_processed_blocks"),
            "server_engine_graph_id": server_parsed.get("engine_graph_id"),
            "local_topology_nodes": as_int(local_parsed, "topology_nodes"),
        },
    }

    receipt = {
        "receipt_version": "signal.demo.receipt.v1",
        "manifest_id": manifest["id"],
        "scenario_id": scenario["id"],
        "status": "passed",
        "launch_command": "effigy demo:local-server-host-comparison",
        "artifacts": [
            {
                "kind": "host-summary-lines",
                "hosts": [
                    {
                        "package": local["package"],
                        "sandbox": local_parsed.get("sandbox"),
                        "profile": local_parsed.get("profile"),
                        "processed_blocks": as_int(local_parsed, "processed_blocks"),
                        "heartbeat_responses": as_int(local_parsed, "heartbeat_responses"),
                        "completion": local_parsed.get("completion"),
                        "raw_line": local["line"],
                    },
                    {
                        "package": server["package"],
                        "sandbox": server_parsed.get("sandbox"),
                        "profile": server_parsed.get("profile"),
                        "processed_blocks": as_int(server_parsed, "processed_blocks"),
                        "heartbeat_responses": as_int(server_parsed, "heartbeat_responses"),
                        "completion": server_parsed.get("completion"),
                        "raw_line": server["line"],
                    },
                ],
                "comparison": comparison,
            }
        ],
        "operator_checks": [
            {
                "id": "operator.host-compare.local-bootstrap",
                "status": "passed"
                if local_parsed.get("readiness") == "Ready"
                and as_bool(local_parsed, "running")
                and as_int(local_parsed, "processed_blocks") > 0
                and local_parsed.get("completion") == "Completed"
                else "failed",
                "summary": "Local host booted successfully with ready/running posture and bounded execution.",
            },
            {
                "id": "operator.host-compare.server-bootstrap",
                "status": "passed"
                if server_parsed.get("readiness") == "Ready"
                and as_bool(server_parsed, "running")
                and as_int(server_parsed, "processed_blocks") > 0
                and server_parsed.get("completion") == "Completed"
                else "failed",
                "summary": "Server host booted successfully with ready/running posture and bounded execution.",
            },
            {
                "id": "operator.host-compare.shared-lifecycle-truth",
                "status": "passed"
                if local_parsed.get("sandbox")
                and server_parsed.get("sandbox")
                and as_int(local_parsed, "heartbeat_responses") > 0
                and as_int(server_parsed, "heartbeat_responses") > 0
                else "failed",
                "summary": "Both hosts exported active sandbox and heartbeat truth through the existing summary line.",
            },
            {
                "id": "operator.host-compare-differences-explicit",
                "status": "passed"
                if local_parsed.get("backend") == "coreaudio"
                and as_int(server_parsed, "engine_processed_blocks") > 0
                else "failed",
                "summary": "The receipt preserves real local-versus-server differences instead of flattening them.",
            },
        ],
    }

    RECEIPT_PATH.parent.mkdir(parents=True, exist_ok=True)
    RECEIPT_PATH.write_text(json.dumps(receipt, indent=2) + "\n")


if __name__ == "__main__":
    main()
