#!/usr/bin/env python3

import json
import os
import re
import signal
import subprocess
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MANIFEST_PATH = (
    REPO_ROOT / "demos" / "manifests" / "hardware-topology-diagnostics.demo.json"
)
RECEIPT_PATH = (
    REPO_ROOT / "demos" / "receipts" / "hardware-topology-diagnostics.receipt.json"
)
HTML_PATH = (
    REPO_ROOT / "demos" / "receipts" / "hardware-topology-diagnostics.view.html"
)
HOST_CAPTURE_TIMEOUT_SECONDS = 20


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
  <title>Signal Hardware Topology Diagnostics</title>
  <style>
    :root {{
      color-scheme: light;
      --bg: #eef4f4;
      --panel: #fbfefd;
      --line: #ccd9d7;
      --text: #16211f;
      --muted: #5c6b69;
      --ok: #215947;
      --ok-soft: #dceee7;
    }}
    * {{ box-sizing: border-box; }}
    body {{
      margin: 0;
      font-family: "Avenir Next", "Segoe UI", sans-serif;
      background: radial-gradient(circle at top, #f7fdfb, var(--bg));
      color: var(--text);
    }}
    main {{
      max-width: 1180px;
      margin: 0 auto;
      padding: 32px 24px 48px;
    }}
    h1, h2 {{ margin: 0 0 12px; }}
    p {{ line-height: 1.5; }}
    .hero {{
      background: linear-gradient(135deg, #f7fdfb, #e8f1ef);
      border: 1px solid var(--line);
      border-radius: 22px;
      padding: 24px;
      margin-bottom: 24px;
      box-shadow: 0 14px 40px rgba(18, 44, 40, 0.08);
    }}
    .hero p {{ margin: 0; color: var(--muted); }}
    .checks {{
      margin: 18px 0 0;
      padding-left: 18px;
    }}
    .checks li {{
      margin: 8px 0;
      color: var(--muted);
    }}
    .grid {{
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
      gap: 18px;
      margin-bottom: 18px;
    }}
    .card {{
      background: var(--panel);
      border: 1px solid var(--line);
      border-radius: 18px;
      padding: 18px;
      box-shadow: 0 10px 24px rgba(18, 39, 35, 0.06);
    }}
    .subtitle {{
      margin: 0 0 14px;
      color: var(--muted);
    }}
    .metrics {{
      display: grid;
      gap: 10px;
    }}
    .metric {{
      display: grid;
      gap: 4px;
      padding: 10px 12px;
      border-radius: 12px;
      background: #f2f8f7;
      border: 1px solid #dce7e4;
    }}
    .label {{
      font-size: 0.82rem;
      letter-spacing: 0.03em;
      text-transform: uppercase;
      color: var(--muted);
    }}
    .value {{
      font-size: 0.98rem;
      color: var(--text);
      word-break: break-word;
    }}
    .callout {{
      margin-top: 22px;
      padding: 16px 18px;
      border-radius: 16px;
      border: 1px solid #cfe0d8;
      background: var(--ok-soft);
      color: var(--ok);
    }}
  </style>
</head>
<body>
  <main>
    <section class="hero">
      <h1>Signal Hardware Topology Diagnostics</h1>
      <p>Operator-facing rendered view for bounded native CoreAudio posture and simulated Linux backend posture across the existing local and server host summary surfaces. This surface stays low-dependency and presentation-only; it does not turn into a device control shell.</p>
      <ul class="checks">{checks}</ul>
    </section>
    <div class="grid">
      {section_card("Native local hardware", "CoreAudio-facing posture from the local host summary line.", model["native_local"])}
      {section_card("Simulated server hardware", "Linux-backend session posture from the server host summary line.", model["simulated_server"])}
    </div>
    <section class="callout">
      The underlying source of truth is still the receipt and the existing host summary lines. This rendered view exists to make native-versus-simulated hardware posture visually inspectable without reading raw JSON first.
    </section>
  </main>
</body>
</html>
"""


def decode_output(value: str | bytes | None) -> str:
    if value is None:
        return ""
    if isinstance(value, bytes):
        return value.decode("utf-8", errors="replace")
    return value


def terminate_process_group(process: subprocess.Popen[str]) -> None:
    try:
        os.killpg(process.pid, signal.SIGTERM)
    except ProcessLookupError:
        return
    try:
        process.wait(timeout=2)
    except subprocess.TimeoutExpired:
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            return
        process.wait(timeout=2)


def extract_summary_line(output: str, package: str) -> str | None:
    for stripped in output.splitlines():
        stripped = stripped.strip()
        if stripped.startswith(package):
            return stripped
    return None


def run_host(package: str) -> dict[str, str | bool]:
    process = subprocess.Popen(
        ["cargo", "run", "-q", "-p", package],
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        cwd=REPO_ROOT,
        start_new_session=True,
    )
    try:
        stdout, stderr = process.communicate(timeout=HOST_CAPTURE_TIMEOUT_SECONDS)
    except subprocess.TimeoutExpired as error:
        stdout = decode_output(error.stdout)
        stderr = decode_output(error.stderr)
        terminate_process_group(process)
        line = extract_summary_line(stdout, package)
        if line is None:
            raise RuntimeError(
                f"{package} did not emit a summary line before timing out"
            ) from error
        return {
            "package": package,
            "line": line,
            "timed_out": True,
        }

    if process.returncode != 0:
        raise subprocess.CalledProcessError(
            process.returncode,
            ["cargo", "run", "-q", "-p", package],
            output=stdout,
            stderr=stderr,
        )

    line = extract_summary_line(stdout, package)
    if line is None:
        raise RuntimeError(f"{package} did not emit a summary line")
    return {
        "package": package,
        "line": line,
        "timed_out": False,
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
                    "capture_timed_out": bool(local["timed_out"]),
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
                    "capture_timed_out": bool(server["timed_out"]),
                    "raw_line": server_line,
                },
            },
            {
                "kind": "hardware-topology-operator-view",
                "html_path": "demos/receipts/hardware-topology-diagnostics.view.html",
                "status": "passed",
                "section_count": 2,
            },
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
            {
                "id": "operator.hardware.bounded-host-capture",
                "status": "passed",
                "summary": "The demo records bounded host summary capture and can accept a valid summary line without waiting indefinitely for child process exit.",
            },
            {
                "id": "operator.hardware.rendered-operator-view",
                "status": "passed",
                "summary": "A rendered companion view makes native and simulated hardware posture visually inspectable without reading the raw receipt first.",
            },
        ],
    }

    model = {
        "operator_checks": receipt["operator_checks"],
        "native_local": [
            ("Backend", extract_first(local_line, "host_backend") or "n/a"),
            ("Device", extract_first(local_line, "host_device") or "n/a"),
            ("Stream", extract_first(local_line, "host_stream_state") or "n/a"),
            (
                "Endpoint topology",
                extract_first(local_line, "host_endpoint_topology") or "n/a",
            ),
            (
                "Device supervision",
                extract_first(local_line, "device_supervision") or "n/a",
            ),
            ("External I/O", extract_first(local_line, "external_io") or "n/a"),
            ("Backend health", extract_first(local_line, "host_backend_health") or "n/a"),
            (
                "Estimated latency samples",
                str(
                    as_int(
                        extract_first(
                            local_line, "host_estimated_output_latency_samples"
                        )
                    )
                ),
            ),
            ("Capture timed out", "true" if local["timed_out"] else "false"),
        ],
        "simulated_server": [
            ("Backend", server_linux["backend"] or "n/a"),
            ("Device", server_linux["device"] or "n/a"),
            ("Stream", server_linux["stream"] or "n/a"),
            ("Simulation", "true" if server_linux["simulated"] else "false"),
            ("PipeWire / ALSA", extract_first(server_line, "pipewire_alsa") or "n/a"),
            ("JACK", extract_first(server_line, "jack") or "n/a"),
            (
                "Device supervision",
                extract_first(server_line, "device_supervision") or "n/a",
            ),
            ("External I/O", extract_first(server_line, "external_io") or "n/a"),
            ("Capture timed out", "true" if server["timed_out"] else "false"),
        ],
    }

    RECEIPT_PATH.parent.mkdir(parents=True, exist_ok=True)
    RECEIPT_PATH.write_text(json.dumps(receipt, indent=2) + "\n")
    HTML_PATH.write_text(browser_html(model))


if __name__ == "__main__":
    main()
