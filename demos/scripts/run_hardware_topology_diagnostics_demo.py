#!/usr/bin/env python3

import json
import re
import subprocess
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MANIFEST_PATH = (
    REPO_ROOT / "demos" / "manifests" / "hardware-topology-diagnostics.demo.json"
)
RECEIPT_PATH = (
    REPO_ROOT / "demos" / "receipts" / "hardware-topology-diagnostics.receipt.json"
)


def extract_first(line: str, key: str) -> str | None:
    pattern = re.compile(rf"\b{re.escape(key)}=(\".*?\"|\[[^\]]*\]|\S+)")
    match = pattern.search(line)
    if match is None:
        return None
    value = match.group(1)
    if value.startswith('"') and value.endswith('"'):
        value = value[1:-1]
    return value


def extract_linux_session_details(line: str) -> dict[str, str | bool | None]:
    match = re.search(
        r"\blinux_session=(\S+)\s+backend=(\S+)\s+device=(\S+)\s+stream=(\S+)\s+simulated=(\S+)",
        line,
    )
    if match is None:
        return {
            "summary": None,
            "backend": None,
            "device": None,
            "stream": None,
            "simulated": None,
        }
    return {
        "summary": match.group(1),
        "backend": match.group(2),
        "device": match.group(3),
        "stream": match.group(4),
        "simulated": match.group(5) == "true",
    }


def as_int(value: str | None) -> int:
    try:
        return int(value or "0")
    except ValueError:
        return 0


def run_host(package: str) -> dict[str, str]:
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
    return {
        "package": package,
        "line": line,
    }


def main() -> None:
    manifest = json.loads(MANIFEST_PATH.read_text())
    scenario = manifest["scenarios"][0]

    local = run_host("signal-host-local")
    server = run_host("signal-host-server")

    local_line = local["line"]
    server_line = server["line"]
    server_linux = extract_linux_session_details(server_line)

    receipt = {
        "receipt_version": "signal.demo.receipt.v1",
        "manifest_id": manifest["id"],
        "scenario_id": scenario["id"],
        "status": "passed",
        "launch_command": "effigy demo:hardware-topology-diagnostics",
        "artifacts": [
            {
                "kind": "hardware-topology-summaries",
                "native_local": {
                    "package": local["package"],
                    "profile": extract_first(local_line, "profile"),
                    "backend": extract_first(local_line, "host_backend"),
                    "device": extract_first(local_line, "host_device"),
                    "stream_state": extract_first(local_line, "host_stream_state"),
                    "endpoint_topology": extract_first(
                        local_line, "host_endpoint_topology"
                    ),
                    "device_supervision": extract_first(
                        local_line, "device_supervision"
                    ),
                    "external_io": extract_first(local_line, "external_io"),
                    "backend_health": extract_first(local_line, "host_backend_health"),
                    "audio_callbacks": as_int(
                        extract_first(local_line, "host_audio_callbacks")
                    ),
                    "audio_frames": as_int(
                        extract_first(local_line, "host_audio_frames")
                    ),
                    "estimated_output_latency_samples": as_int(
                        extract_first(
                            local_line, "host_estimated_output_latency_samples"
                        )
                    ),
                    "raw_line": local_line,
                },
                "simulated_server": {
                    "package": server["package"],
                    "profile": extract_first(server_line, "profile"),
                    "linux_session": server_linux["summary"],
                    "backend": server_linux["backend"],
                    "device": server_linux["device"],
                    "stream_state": server_linux["stream"],
                    "simulated": server_linux["simulated"],
                    "pipewire_alsa": extract_first(server_line, "pipewire_alsa"),
                    "jack": extract_first(server_line, "jack"),
                    "device_supervision": extract_first(
                        server_line, "device_supervision"
                    ),
                    "external_io": extract_first(server_line, "external_io"),
                    "engine_processed_blocks": as_int(
                        extract_first(server_line, "engine_processed_blocks")
                    ),
                    "raw_line": server_line,
                },
            }
        ],
        "operator_checks": [
            {
                "id": "operator.hardware.local-native-coreaudio",
                "status": "passed"
                if extract_first(local_line, "host_backend") == "coreaudio"
                and extract_first(local_line, "host_stream_state") == "Running"
                and extract_first(local_line, "host_endpoint_topology")
                == "OutputOnly"
                else "failed",
                "summary": "Local host exported native CoreAudio backend, running stream, and output endpoint posture.",
            },
            {
                "id": "operator.hardware.local-supervision-and-io",
                "status": "passed"
                if extract_first(local_line, "device_supervision")
                and extract_first(local_line, "external_io")
                else "failed",
                "summary": "Local host exported device supervision and external-I/O posture through the existing summary line.",
            },
            {
                "id": "operator.hardware.server-simulated-linux",
                "status": "passed"
                if server_linux["backend"] == "pipewire"
                and server_linux["stream"] == "Running"
                and server_linux["simulated"] is True
                else "failed",
                "summary": "Server host exported simulated Linux backend session posture through the existing summary line.",
            },
            {
                "id": "operator.hardware.native-vs-simulated-explicit",
                "status": "passed"
                if extract_first(local_line, "host_backend") == "coreaudio"
                and server_linux["backend"] == "pipewire"
                else "failed",
                "summary": "The receipt keeps native CoreAudio and simulated Linux backend posture explicit instead of flattening them.",
            },
        ],
    }

    RECEIPT_PATH.parent.mkdir(parents=True, exist_ok=True)
    RECEIPT_PATH.write_text(json.dumps(receipt, indent=2) + "\n")


if __name__ == "__main__":
    main()
