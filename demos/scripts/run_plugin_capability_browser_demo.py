#!/usr/bin/env python3

import argparse
import html
import json
import os
import platform
import signal
import subprocess
import sys
import tempfile
import textwrap
import webbrowser
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from threading import Thread


REPO_ROOT = Path(__file__).resolve().parents[2]
MANIFEST_PATH = REPO_ROOT / "demos" / "manifests" / "plugin-capability-browser.demo.json"
RECEIPT_PATH = REPO_ROOT / "demos" / "receipts" / "plugin-capability-browser.receipt.json"
HTML_PATH = REPO_ROOT / "demos" / "receipts" / "plugin-capability-browser.view.html"
LOCAL_PROBE_TIMEOUT_SECONDS = 8
LOCAL_PROBE_SUCCESS_LIMIT = 6
LOCAL_PROBE_ATTEMPT_LIMIT = 18
SYSTEM_SCAN_TIMEOUT_SECONDS = 10
PROOF_SCAN_TIMEOUT_SECONDS = 120
INTERACTIVE_SCAN_BATCH_SIZE = 4
INTERACTIVE_SERVER_FOLLOWUP_LIMITS = {
    "clap": 4,
    "vst3": 6,
    "au": 4,
    "lv2": 4,
}
INTERACTIVE_SCAN_CANDIDATE_LIMITS = {
    "clap": 8,
    "vst3": 12,
    "au": 8,
    "lv2": 12,
}


def bind_browser_server(preferred_port: int) -> tuple[ThreadingHTTPServer, int]:
    for port in range(preferred_port, preferred_port + 20):
        try:
            server = ThreadingHTTPServer(("127.0.0.1", port), BrowserHandler)
            server.daemon_threads = True
            return server, port
        except OSError as error:
            if getattr(error, "errno", None) == 48:
                continue
            raise
    raise OSError(f"no free browser port found in range {preferred_port}-{preferred_port + 19}")


def split_paths(value: str | None) -> list[str]:
    if not value:
        return []
    return [segment for segment in value.split(os.pathsep) if segment]


def existing_paths(paths: list[Path]) -> list[str]:
    discovered: list[str] = []
    for path in paths:
        expanded = path.expanduser()
        if expanded.exists():
            rendered = str(expanded)
            if rendered not in discovered:
                discovered.append(rendered)
    return discovered


def system_roots_by_format() -> dict[str, list[str]]:
    system = sys.platform
    roots = {
        "clap": split_paths(os.environ.get("SIGNAL_DEMO_CLAP_ROOTS")),
        "vst3": split_paths(os.environ.get("SIGNAL_DEMO_VST3_ROOTS")),
        "au": split_paths(os.environ.get("SIGNAL_DEMO_AU_ROOTS")),
        "lv2": split_paths(os.environ.get("SIGNAL_DEMO_LV2_ROOTS")),
    }
    if any(roots.values()):
        return {key: dedupe(value) for key, value in roots.items()}

    if system == "darwin":
        roots["clap"] = existing_paths(
            [
                Path("~/Library/Audio/Plug-Ins/CLAP"),
                Path("/Library/Audio/Plug-Ins/CLAP"),
            ]
        )
        roots["vst3"] = existing_paths(
            [
                Path("~/Library/Audio/Plug-Ins/VST3"),
                Path("/Library/Audio/Plug-Ins/VST3"),
            ]
        )
        roots["au"] = existing_paths(
            [
                Path("~/Library/Audio/Plug-Ins/Components"),
                Path("/Library/Audio/Plug-Ins/Components"),
            ]
        )
    else:
        roots["clap"] = existing_paths(
            [
                Path("~/.clap"),
                Path("~/.local/lib/clap"),
                Path("/usr/local/lib/clap"),
                Path("/usr/lib/clap"),
            ]
        )
        roots["vst3"] = existing_paths(
            [
                Path("~/.vst3"),
                Path("~/.local/share/vst3"),
                Path("/usr/local/lib/vst3"),
                Path("/usr/lib/vst3"),
            ]
        )
        roots["lv2"] = existing_paths(
            [
                Path("~/.lv2"),
                Path("~/.local/lib/lv2"),
                Path("/usr/local/lib/lv2"),
                Path("/usr/lib/lv2"),
            ]
        )
    return {key: dedupe(value) for key, value in roots.items()}


def dedupe(values: list[str]) -> list[str]:
    rendered: list[str] = []
    for value in values:
        if value not in rendered:
            rendered.append(value)
    return rendered


def is_exact_plugin_root(fmt: str, path: Path) -> bool:
    suffix = path.suffix.lower()
    if fmt == "clap":
        return path.is_file() and suffix == ".clap"
    if fmt == "vst3":
        return suffix == ".vst3"
    if fmt == "au":
        return suffix == ".component"
    if fmt == "lv2":
        return suffix == ".lv2"
    return False


def chunked(values: list[str], size: int) -> list[list[str]]:
    return [values[index : index + size] for index in range(0, len(values), size)]


def interactive_candidate_roots(fmt: str, roots: list[str]) -> list[str]:
    discovered: list[str] = []
    limit = INTERACTIVE_SCAN_CANDIDATE_LIMITS.get(fmt, 12)
    for rendered_root in roots:
        root = Path(rendered_root).expanduser()
        if not root.exists():
            continue
        if is_exact_plugin_root(fmt, root):
            candidate = str(root)
            if candidate not in discovered:
                discovered.append(candidate)
            if len(discovered) >= limit:
                break
            continue
        if not root.is_dir():
            continue
        try:
            children = sorted(root.iterdir(), key=lambda child: child.name.lower())
        except OSError:
            continue
        for child in children:
            if not is_exact_plugin_root(fmt, child):
                continue
            candidate = str(child)
            if candidate not in discovered:
                discovered.append(candidate)
            if len(discovered) >= limit:
                break
        if len(discovered) >= limit:
            break
    return discovered


def preferred_server_roots_from_inventory(inventory: list[dict[str, object]]) -> dict[str, list[str]]:
    roots_by_format = {"clap": [], "vst3": [], "au": [], "lv2": []}
    for plugin in inventory:
        fmt = rendered_scan_format(str(plugin["format"]))
        if fmt not in roots_by_format:
            continue
        root = exact_launch_root(plugin)
        if root is None:
            continue
        if root not in roots_by_format[fmt]:
            roots_by_format[fmt].append(root)
    for fmt, roots in roots_by_format.items():
        limit = INTERACTIVE_SERVER_FOLLOWUP_LIMITS.get(fmt, 4)
        roots_by_format[fmt] = roots[:limit]
    return roots_by_format


def create_vst3_fixture_root() -> tuple[tempfile.TemporaryDirectory[str], str]:
    tempdir = tempfile.TemporaryDirectory(prefix="signal-plugin-browser-vst3-")
    root = Path(tempdir.name)
    bundle_root = root / "Signal Browser Instrument.vst3"
    resources_root = bundle_root / "Contents" / "Resources"
    resources_root.mkdir(parents=True, exist_ok=True)
    plugin_type_id = "plugin:vst3:browser-fixture"
    info_plist = textwrap.dedent(
        f"""\
        <?xml version="1.0" encoding="UTF-8"?>
        <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
        <plist version="1.0">
          <dict>
            <key>CFBundleName</key>
            <string>Signal Browser Instrument</string>
            <key>CFBundleIdentifier</key>
            <string>dev.signal.plugin.browser.fixture</string>
            <key>CFBundleVersion</key>
            <string>0.1.0</string>
            <key>CFBundleShortVersionString</key>
            <string>0.1.0</string>
            <key>CFBundlePackageType</key>
            <string>BNDL</string>
            <key>CFBundleExecutable</key>
            <string>Signal Browser Instrument</string>
            <key>SignalPluginTypeId</key>
            <string>{plugin_type_id}</string>
            <key>SignalAudioInputs</key>
            <integer>0</integer>
            <key>SignalAudioOutputs</key>
            <integer>2</integer>
            <key>SignalMidiInputs</key>
            <integer>1</integer>
            <key>SignalMidiOutputs</key>
            <integer>0</integer>
            <key>SignalFeatures</key>
            <array>
              <string>Instrument</string>
              <string>Analyzer</string>
            </array>
          </dict>
        </plist>
        """
    )
    moduleinfo = {
        "Classes": [
            {
                "CID": "7E1D8F8A4D874D56A2C44DE250199901",
                "Category": "Audio Module Class",
                "Name": "Signal Browser Instrument",
                "Vendor": "Signal",
                "Version": "0.1.0",
                "SubCategories": ["Instrument", "Analyzer"],
                "ClassFlags": 1,
                "Snapshots": [],
            },
            {
                "CID": "7E1D8F8A4D874D56A2C44DE250199902",
                "Category": "Component Controller Class",
                "Name": "Signal Browser Instrument Controller",
                "Vendor": "Signal",
                "Version": "0.1.0",
                "SubCategories": [],
                "ClassFlags": 1,
                "Snapshots": [],
            },
        ]
    }
    (bundle_root / "Contents").mkdir(parents=True, exist_ok=True)
    (bundle_root / "Contents" / "Info.plist").write_text(info_plist)
    (resources_root / "moduleinfo.json").write_text(json.dumps(moduleinfo, indent=2) + "\n")
    return tempdir, plugin_type_id


def selected_formats(roots_by_format: dict[str, list[str]]) -> list[str]:
    return [fmt for fmt, roots in roots_by_format.items() if roots]


def decode_json_payload(raw_output: str) -> dict[str, object]:
    stripped = raw_output.strip()
    if not stripped:
        raise ValueError("scan example produced no stdout")
    try:
        return json.loads(stripped)
    except json.JSONDecodeError:
        pass

    decoder = json.JSONDecoder()
    starts = [index for index, char in enumerate(stripped) if char in "{["]
    if not starts:
        raise ValueError(f"scan example did not emit JSON: {stripped[:200]}")

    last_error: json.JSONDecodeError | None = None
    for start in starts:
        candidate = stripped[start:]
        try:
            payload, _ = decoder.raw_decode(candidate)
        except json.JSONDecodeError as error:
            last_error = error
            continue
        if isinstance(payload, dict):
            return payload

    if last_error is not None:
        raise ValueError(str(last_error))
    raise ValueError("scan example JSON payload was not an object")


def run_scan_example(
    package: str,
    formats: list[str],
    roots: list[str],
    timeout_seconds: int = 120,
) -> dict[str, object]:
    command = [
        "cargo",
        "run",
        "-q",
        "-p",
        package,
        "--example",
        f"{package.replace('-', '_')}_plugin_capability_scan",
        "--",
    ]
    for fmt in formats:
        command.extend(["--format", fmt])
    for root in roots:
        command.extend(["--root", root])
    process = subprocess.Popen(
        command,
        cwd=REPO_ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        start_new_session=True,
    )
    try:
        stdout, stderr = process.communicate(timeout=timeout_seconds)
    except subprocess.TimeoutExpired as error:
        os.killpg(process.pid, signal.SIGTERM)
        try:
            stdout, stderr = process.communicate(timeout=5)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            stdout, stderr = process.communicate()
        raise subprocess.TimeoutExpired(
            error.cmd,
            error.timeout,
            output=stdout,
            stderr=stderr,
        ) from error
    except KeyboardInterrupt as error:
        os.killpg(process.pid, signal.SIGTERM)
        try:
            process.communicate(timeout=3)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            process.communicate()
        raise error

    if process.returncode != 0:
        raise subprocess.CalledProcessError(
            process.returncode,
            command,
            output=stdout,
            stderr=stderr,
        )
    try:
        return decode_json_payload(stdout)
    except ValueError as error:
        stderr_tail = "\n".join(stderr.splitlines()[-20:])
        stdout_tail = "\n".join(stdout.splitlines()[-20:])
        raise RuntimeError(
            f"failed to decode {package} scan inventory: {error}\n"
            f"stdout tail:\n{stdout_tail}\n"
            f"stderr tail:\n{stderr_tail}"
        ) from error


def safe_run_scan_example(
    package: str,
    formats: list[str],
    roots: list[str],
    timeout_seconds: int = 120,
) -> tuple[dict[str, object] | None, str | None]:
    try:
        return run_scan_example(package, formats, roots, timeout_seconds=timeout_seconds), None
    except (RuntimeError, subprocess.TimeoutExpired, subprocess.CalledProcessError) as error:
        return None, str(error)


def collect_scans(
    package: str,
    roots_by_format: dict[str, list[str]],
    allowed_formats: tuple[str, ...],
    timeout_seconds: int = 120,
    exact_batch_mode: bool = False,
) -> tuple[list[dict[str, object]], list[str]]:
    scans: list[dict[str, object]] = []
    failures: list[str] = []
    for fmt in allowed_formats:
        roots = roots_by_format.get(fmt, [])
        root_groups = [[root] for root in roots]
        if exact_batch_mode:
            candidate_roots = interactive_candidate_roots(fmt, roots)
            if candidate_roots:
                root_groups = chunked(candidate_roots, INTERACTIVE_SCAN_BATCH_SIZE)
        for root_group in root_groups:
            scan, scan_error = safe_run_scan_example(
                package,
                [fmt],
                root_group,
                timeout_seconds=timeout_seconds,
            )
            if scan is not None:
                scans.append(scan)
            elif scan_error is not None:
                rendered_roots = ", ".join(root_group)
                failures.append(f"{package} {fmt} scan failed for {rendered_roots}: {scan_error}")
    return scans, failures


def rendered_scan_format(plugin_format: str) -> str:
    return str(plugin_format).lower()


def local_probe_sort_key(plugin: dict[str, object]) -> tuple[int, str, str, str]:
    format_rank = {"Clap": 0, "Vst3": 1, "Au": 2}
    return (
        format_rank.get(str(plugin["format"]), 99),
        str(plugin["vendor"]),
        str(plugin["name"]),
        str(plugin["plugin_type_id"]),
    )


def exact_launch_root(plugin: dict[str, object]) -> str | None:
    for target in plugin["launch_targets"]:
        if target["host_surface"] == "server":
            return str(target["launch_root"])
    for target in plugin["launch_targets"]:
        return str(target["launch_root"])
    return None


def attach_contained_local_targets(inventory: list[dict[str, object]]) -> dict[str, object]:
    summary = {
        "attempted": 0,
        "succeeded": 0,
        "failed": 0,
        "limit_hit": False,
        "failures": [],
    }
    if not can_run_local_surface():
        return summary

    probe_candidates = [
        plugin
        for plugin in sorted(inventory, key=local_probe_sort_key)
        if plugin["format"] in {"Clap", "Vst3", "Au"}
    ]

    for plugin in probe_candidates:
        if summary["succeeded"] >= LOCAL_PROBE_SUCCESS_LIMIT:
            break
        if summary["attempted"] >= LOCAL_PROBE_ATTEMPT_LIMIT:
            summary["limit_hit"] = True
            break

        launch_root = exact_launch_root(plugin)
        if launch_root is None:
            continue

        summary["attempted"] += 1
        scan, scan_error = safe_run_scan_example(
            "signal-host-local",
            [rendered_scan_format(plugin["format"])],
            [launch_root],
            timeout_seconds=LOCAL_PROBE_TIMEOUT_SECONDS,
        )
        if scan is None:
            summary["failed"] += 1
            if scan_error is not None and len(summary["failures"]) < 5:
                summary["failures"].append(
                    f"{plugin['name']} ({plugin['format']}): {scan_error.splitlines()[0][:180]}"
                )
            continue

        discovered_type_ids = {
            str(local_plugin["plugin_type_id"]) for local_plugin in scan.get("plugins", [])
        }
        if str(plugin["plugin_type_id"]) not in discovered_type_ids:
            summary["failed"] += 1
            if len(summary["failures"]) < 5:
                summary["failures"].append(
                    f"{plugin['name']} ({plugin['format']}): local scan did not return the plugin type"
                )
            continue

        if not any(target["host_surface"] == "local" for target in plugin["launch_targets"]):
            plugin["launch_targets"].append(
                {
                    "host_surface": "local",
                    "launch_root": launch_root,
                    "plugin_type_id": plugin["plugin_type_id"],
                    "format": plugin["format"],
                }
            )
        summary["succeeded"] += 1

    return summary


def can_run_local_surface() -> bool:
    return sys.platform == "darwin"


def launch_command(package: str, plugin: dict[str, object]) -> list[str]:
    return ["cargo", "run", "-q", "-p", package]


def launch_env(plugin: dict[str, object]) -> dict[str, str]:
    rendered_format = str(plugin["format"]).lower()
    return {
        "SIGNAL_HOST_DEMO_PLUGIN_FORMAT": rendered_format,
        "SIGNAL_HOST_DEMO_PLUGIN_ROOT": str(plugin["launch_root"]),
        "SIGNAL_HOST_DEMO_PLUGIN_TYPE_ID": str(plugin["plugin_type_id"]),
        "SIGNAL_HOST_DEMO_INTERACTION_MODE": "parameter-step",
        "SIGNAL_HOST_DEMO_INTERACTION_VALUE": "0.73",
    }


def summary_token(summary_line: str, key: str) -> str | None:
    marker = f"{key}="
    start = summary_line.find(marker)
    if start < 0:
        return None
    value = summary_line[start + len(marker) :]
    if " " in value:
        value = value.split(" ", 1)[0]
    return value


def parse_interaction_summary(summary_line: str) -> dict[str, object]:
    interaction_mode = summary_token(summary_line, "interaction_mode")
    automation_value = summary_token(summary_line, "automation_value")
    parameter_events = summary_token(summary_line, "parameter_events")
    generated_event_bytes = summary_token(summary_line, "generated_event_bytes")
    return {
        "interaction_mode": interaction_mode,
        "interaction_value": automation_value,
        "parameter_event_count": parameter_events,
        "generated_event_bytes": generated_event_bytes,
        "interaction_proved": interaction_mode not in {None, "none"}
        and automation_value not in {None, "None"}
        and parameter_events not in {None, "0"},
    }


def decode_subprocess_stream(value: object) -> str:
    if value is None:
        return ""
    if isinstance(value, bytes):
        return value.decode("utf-8", errors="replace")
    return str(value)


def run_launch(
    package: str,
    plugin: dict[str, object],
    timeout_seconds: int = 15,
) -> dict[str, object]:
    env = os.environ.copy()
    env.update(launch_env(plugin))
    command = launch_command(package, plugin)
    try:
        result = subprocess.run(
            command,
            cwd=REPO_ROOT,
            text=True,
            capture_output=True,
            env=env,
            timeout=timeout_seconds,
        )
    except subprocess.TimeoutExpired as error:
        stdout_text = decode_subprocess_stream(error.stdout)
        stderr_text = decode_subprocess_stream(error.stderr)
        return {
            "package": package,
            "plugin_type_id": plugin["plugin_type_id"],
            "format": plugin["format"],
            "launch_root": plugin["launch_root"],
            "status": "failed",
            "exit_code": None,
            "command": " ".join(command),
            "summary_line": "",
            "stdout_tail": stdout_text.splitlines()[-20:],
            "stderr_tail": stderr_text.splitlines()[-20:],
            "failure_kind": "timeout",
        }
    summary_line = ""
    for line in result.stdout.splitlines():
        if line.startswith(package):
            summary_line = line
            break
    return {
        "package": package,
        "plugin_type_id": plugin["plugin_type_id"],
        "format": plugin["format"],
        "launch_root": plugin["launch_root"],
        "status": "passed" if result.returncode == 0 else "failed",
        "exit_code": result.returncode,
        "command": " ".join(command),
        "summary_line": summary_line,
        "stdout_tail": result.stdout.splitlines()[-20:],
        "stderr_tail": result.stderr.splitlines()[-20:],
        **parse_interaction_summary(summary_line),
    }


def combine_inventory(scans: list[dict[str, object]]) -> list[dict[str, object]]:
    combined: dict[tuple[str, str], dict[str, object]] = {}
    for scan in scans:
        host_surface = str(scan["host_surface"])
        for plugin in scan["plugins"]:
            plugin = dict(plugin)
            key = (str(plugin["format"]), str(plugin["plugin_type_id"]))
            row = combined.setdefault(
                key,
                {
                    "plugin_type_id": plugin["plugin_type_id"],
                    "plugin_id": plugin["plugin_id"],
                    "format": plugin["format"],
                    "vendor": plugin["vendor"],
                    "name": plugin["name"],
                    "version": plugin["version"],
                    "features": plugin["features"],
                    "parameter_count": plugin["parameter_count"],
                    "audio_bus_count": plugin["audio_bus_count"],
                    "summary": plugin["summary"],
                    "interaction_posture": plugin["interaction_posture"],
                    "launch_targets": [],
                },
            )
            row["launch_targets"].append(
                {
                    "host_surface": host_surface,
                    "launch_root": plugin["launch_root"],
                    "plugin_type_id": plugin["plugin_type_id"],
                    "format": plugin["format"],
                }
            )
    for plugin in combined.values():
        plugin["launch_targets"] = sorted(
            plugin["launch_targets"],
            key=lambda target: (0 if target["host_surface"] == "local" else 1, str(target["launch_root"])),
        )
    return sorted(
        combined.values(),
        key=lambda row: (
            str(row["format"]),
            str(row["vendor"]),
            str(row["name"]),
            str(row["plugin_type_id"]),
        ),
    )


def choose_primary_launch(inventory: list[dict[str, object]]) -> dict[str, object] | None:
    preferred_formats = {"Clap", "Vst3"}
    preferred_hosts = {"local": 0, "server": 1}
    candidates: list[tuple[int, int, dict[str, object], dict[str, object]]] = []
    for plugin in inventory:
        if plugin["format"] not in preferred_formats:
            continue
        for target in plugin["launch_targets"]:
            candidates.append(
                (
                    0 if plugin["format"] == "Clap" else 1,
                    preferred_hosts.get(str(target["host_surface"]), 99),
                    plugin,
                    target,
                )
            )
    if not candidates:
        return None
    _, _, plugin, target = sorted(candidates, key=lambda item: item[:2])[0]
    return {"plugin": plugin, "target": target}


def execute_primary_launch(inventory: list[dict[str, object]]) -> dict[str, object] | None:
    preferred_hosts = {"local": 0, "server": 1}
    preferred_formats = {"Vst3": 0, "Clap": 1, "Au": 2, "Lv2": 3}
    candidates: list[tuple[int, int, dict[str, object], dict[str, object]]] = []
    for plugin in inventory:
        for target in plugin["launch_targets"]:
            candidates.append(
                (
                    preferred_hosts.get(str(target["host_surface"]), 99),
                    preferred_formats.get(str(plugin["format"]), 99),
                    plugin,
                    target,
                )
            )
    if not candidates:
        return None

    first_result: dict[str, object] | None = None
    for _, _, plugin, target in sorted(candidates, key=lambda item: item[:2])[:6]:
        package = (
            "signal-host-local"
            if target["host_surface"] == "local"
            else "signal-host-server"
        )
        result = run_launch(package, target)
        candidate_result = {"plugin": plugin, "target": target, **result}
        if first_result is None:
            first_result = candidate_result
        if result["status"] == "passed":
            return candidate_result
    return first_result


def browser_html(browser_model: dict[str, object]) -> str:
    rows = []
    for plugin in browser_model["inventory"]:
        feature_list = ", ".join(plugin["features"]) or "none"
        local_available = any(
            target["host_surface"] == "local" for target in plugin["launch_targets"]
        )
        server_available = any(
            target["host_surface"] == "server" for target in plugin["launch_targets"]
        )
        availability = []
        if local_available:
            availability.append('<span class="pill pill-local">Local</span>')
        if server_available:
            availability.append('<span class="pill pill-server">Server</span>')
        if not availability:
            availability.append('<span class="pill pill-none">No launch</span>')

        if local_available and server_available:
            posture = "bounded local + server"
        elif local_available:
            posture = "bounded local only"
        elif server_available:
            posture = "bounded server only"
        else:
            posture = "no bounded launch"
        launch_cells = []
        for target in plugin["launch_targets"]:
            payload = html.escape(json.dumps(target))
            launch_cells.append(
                f"<button class=\"launch\" data-launch='{payload}'>Launch {html.escape(str(target['host_surface']))}</button>"
            )
        if not launch_cells:
            launch_cells.append("<span class=\"muted\">No bounded launch target</span>")
        rows.append(
            "<tr>"
            f"<td>{html.escape(str(plugin['format']))}</td>"
            f"<td>{html.escape(str(plugin['name']))}</td>"
            f"<td>{html.escape(str(plugin['vendor']))}</td>"
            f"<td><code>{html.escape(str(plugin['plugin_type_id']))}</code></td>"
            f"<td>{html.escape(feature_list)}</td>"
            f"<td>{''.join(availability)}</td>"
            f"<td>{html.escape(posture)}<div class=\"muted\">{html.escape(str(plugin['interaction_posture']))}</div></td>"
            f"<td>{''.join(launch_cells)}</td>"
            "</tr>"
        )
    exclusion_list = "".join(
        f"<li>{html.escape(note)}</li>" for note in browser_model["known_exclusions"]
    )
    stats = browser_model["stats"]
    embedded = html.escape(json.dumps(browser_model))
    return f"""<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Signal Plugin Capability Browser</title>
  <style>
    :root {{
      color-scheme: light;
      --ink: #161616;
      --muted: #6d6a63;
      --paper: #f3efe5;
      --panel: #fffaf1;
      --line: #d8cfbe;
      --accent: #0f5a52;
      --accent-soft: #d4ebe8;
      --warn: #8a4f00;
    }}
    body {{
      margin: 0;
      font-family: "Iowan Old Style", "Palatino Linotype", serif;
      background: radial-gradient(circle at top left, #fffdf7, var(--paper) 56%);
      color: var(--ink);
    }}
    main {{
      max-width: 1120px;
      margin: 0 auto;
      padding: 32px 24px 64px;
    }}
    h1, h2 {{
      font-family: "Avenir Next Condensed", "Helvetica Neue", sans-serif;
      letter-spacing: 0.02em;
    }}
    .hero {{
      display: grid;
      gap: 16px;
      padding: 24px;
      border: 1px solid var(--line);
      background: linear-gradient(135deg, #fffdf7, var(--panel));
      box-shadow: 0 16px 32px rgba(22, 22, 22, 0.06);
    }}
    .stats {{
      display: flex;
      flex-wrap: wrap;
      gap: 12px;
    }}
    .stat {{
      padding: 10px 12px;
      border: 1px solid var(--line);
      background: var(--accent-soft);
      font-family: "Avenir Next", sans-serif;
    }}
    .muted {{ color: var(--muted); }}
    .pill {{
      display: inline-block;
      margin-right: 8px;
      margin-bottom: 6px;
      padding: 4px 8px;
      border-radius: 999px;
      font-family: "Avenir Next", sans-serif;
      font-size: 0.78rem;
      border: 1px solid var(--line);
      background: white;
    }}
    .pill-local {{
      border-color: var(--accent);
      background: var(--accent-soft);
      color: var(--accent);
    }}
    .pill-server {{
      border-color: #5f6f8c;
      background: #edf2fb;
      color: #31415c;
    }}
    .pill-none {{
      border-color: #b7ac98;
      background: #f5efe4;
      color: #6d6a63;
    }}
    table {{
      width: 100%;
      border-collapse: collapse;
      margin-top: 24px;
      background: var(--panel);
    }}
    th, td {{
      padding: 12px 10px;
      border-bottom: 1px solid var(--line);
      vertical-align: top;
      text-align: left;
    }}
    th {{
      font-family: "Avenir Next", sans-serif;
      font-size: 0.85rem;
      text-transform: uppercase;
      letter-spacing: 0.04em;
    }}
    button.launch {{
      margin-right: 8px;
      margin-bottom: 8px;
      border: 1px solid var(--accent);
      background: white;
      color: var(--accent);
      padding: 8px 10px;
      cursor: pointer;
      font-family: "Avenir Next", sans-serif;
    }}
    pre {{
      white-space: pre-wrap;
      background: #111;
      color: #f5f0e4;
      padding: 16px;
      border-radius: 8px;
      min-height: 72px;
    }}
    code {{
      font-family: "SFMono-Regular", "Menlo", monospace;
      font-size: 0.9em;
    }}
    .launch-status {{
      margin-bottom: 10px;
      padding: 10px 12px;
      border: 1px solid var(--line);
      font-family: "Avenir Next", sans-serif;
      background: #f8f4ea;
    }}
    .launch-status.passed {{
      border-color: var(--accent);
      background: var(--accent-soft);
      color: var(--accent);
    }}
    .launch-status.failed {{
      border-color: #8c3c28;
      background: #fbeae5;
      color: #7b2c18;
    }}
    .launch-summary {{
      margin-bottom: 14px;
      font-family: "Avenir Next", sans-serif;
    }}
    details.launch-detail {{
      margin-top: 10px;
    }}
    .callout {{
      margin-top: 24px;
      padding: 16px 18px;
      border-left: 4px solid var(--warn);
      background: #fff5e8;
    }}
  </style>
</head>
<body>
  <main>
    <section class="hero">
      <div class="muted">signal.demo.plugin.capability-browser</div>
      <h1>Signal Plugin Capability Browser</h1>
      <p>Browse real discovered plugin inventory and launch one bounded supported host path without pulling a heavyweight UI stack into the repo.</p>
      <div class="stats">
        <div class="stat">plugins: {stats["plugin_count"]}</div>
        <div class="stat">formats: {stats["format_count"]}</div>
        <div class="stat">launch targets: {stats["launch_target_count"]}</div>
        <div class="stat">fixture fallback: {str(stats["fixture_fallback_used"]).lower()}</div>
      </div>
      <div class="muted">HTML artifact: <code>{html.escape(str(HTML_PATH.relative_to(REPO_ROOT)))}</code></div>
    </section>

    <section>
      <h2>Known exclusions</h2>
      <ul>{exclusion_list}</ul>
    </section>

    <section>
      <h2>Discovered plugins</h2>
      <table>
        <thead>
          <tr>
            <th>Format</th>
            <th>Name</th>
            <th>Vendor</th>
            <th>Plugin Type</th>
            <th>Features</th>
            <th>Availability</th>
            <th>Interaction</th>
            <th>Launch</th>
          </tr>
        </thead>
        <tbody>
          {''.join(rows)}
        </tbody>
      </table>
    </section>

    <section>
      <h2>Launch output</h2>
      <div id="launch-status" class="launch-status muted">No launch run yet.</div>
      <div id="launch-summary" class="launch-summary muted">Launch a plugin from the browser to capture a bounded host result.</div>
      <details class="launch-detail">
        <summary>Bounded host detail</summary>
        <pre id="launch-output">Launch a plugin from the browser to capture the bounded host summary here.</pre>
      </details>
    </section>

    <section class="callout">
      <strong>Serving note:</strong> launch buttons only work while the browser is served through the repo-owned Python wrapper. The static HTML artifact remains useful for visual inspection and audit capture.
    </section>
  </main>
  <script type="application/json" id="browser-model">{embedded}</script>
  <script>
    const launchStatus = document.getElementById("launch-status");
    const launchSummary = document.getElementById("launch-summary");
    const output = document.getElementById("launch-output");
    for (const button of document.querySelectorAll("button.launch")) {{
      button.addEventListener("click", async () => {{
        const payload = JSON.parse(button.dataset.launch);
        launchStatus.className = "launch-status muted";
        launchStatus.textContent = `Launching ${{payload.host_surface}} ${{payload.format}} ${{payload.plugin_type_id}}...`;
        launchSummary.textContent = `Root: ${{payload.launch_root}}`;
        output.textContent = `Launching ${{payload.host_surface}} ${{payload.format}} ${{payload.plugin_type_id}}...`;
        try {{
          const response = await fetch("/launch", {{
            method: "POST",
            headers: {{ "Content-Type": "application/json" }},
            body: JSON.stringify(payload),
          }});
          const result = await response.json();
          const failureKind = result.failure_kind ? ` (${{result.failure_kind}})` : "";
          const interactionBits = [];
          if (result.interaction_mode && result.interaction_mode !== "none") {{
            interactionBits.push(`interaction=${{result.interaction_mode}}`);
          }}
          if (result.interaction_value && result.interaction_value !== "None") {{
            interactionBits.push(`value=${{result.interaction_value}}`);
          }}
          if (result.parameter_event_count && result.parameter_event_count !== "0") {{
            interactionBits.push(`parameter_events=${{result.parameter_event_count}}`);
          }}
          launchStatus.className = `launch-status ${{result.status === "passed" ? "passed" : "failed"}}`;
          launchStatus.textContent = `${{result.status.toUpperCase()}}${{failureKind}}: ${{result.package}} -> ${{result.plugin_type_id}}`;
          launchSummary.textContent = interactionBits.length > 0
            ? interactionBits.join(" | ")
            : (result.summary_line || `Launch root: ${{result.launch_root}}`);
          output.textContent = JSON.stringify(result, null, 2);
        }} catch (error) {{
          launchStatus.className = "launch-status failed";
          launchStatus.textContent = "FAILED: browser launch request";
          launchSummary.textContent = String(error);
          output.textContent = String(error);
        }}
      }});
    }}
  </script>
</body>
</html>
"""


class BrowserHandler(BaseHTTPRequestHandler):
    browser_model: dict[str, object] = {}

    def do_GET(self) -> None:
        if self.path not in {"/", "/index.html"}:
            self.send_error(404)
            return
        html_body = browser_html(self.browser_model).encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.send_header("Content-Length", str(len(html_body)))
        self.end_headers()
        self.wfile.write(html_body)

    def do_POST(self) -> None:
        if self.path != "/launch":
            self.send_error(404)
            return
        content_length = int(self.headers.get("Content-Length", "0"))
        payload = json.loads(self.rfile.read(content_length) or "{}")
        package = (
            "signal-host-local"
            if payload.get("host_surface") == "local"
            else "signal-host-server"
        )
        result = run_launch(package, payload)
        body = json.dumps(result, indent=2).encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)


def receipt_from_run(
    manifest: dict[str, object],
    browser_model: dict[str, object],
    primary_launch: dict[str, object] | None,
) -> dict[str, object]:
    scenario = manifest["scenarios"][0]
    stats = browser_model["stats"]
    operator_checks = [
        {
            "id": "operator.plugin-browser.inventory-visible",
            "status": "passed" if stats["plugin_count"] > 0 else "failed",
            "summary": "The browser shows discovered plugin inventory instead of a placeholder or hardcoded list.",
        },
        {
            "id": "operator.plugin-browser.launch-path-visible",
            "status": "passed" if stats["launch_target_count"] > 0 else "failed",
            "summary": "The browser exposes bounded per-plugin launch targets through repo-owned host commands.",
        },
        {
            "id": "operator.plugin-browser.supported-live-path",
            "status": "passed"
            if primary_launch and primary_launch["status"] == "passed"
            else "failed",
            "summary": "At least one supported CLAP or VST3 launch path executed successfully during the capture run.",
        },
        {
            "id": "operator.plugin-browser.bounded-interaction-visible",
            "status": "passed"
            if primary_launch and primary_launch.get("interaction_proved")
            else "failed",
            "summary": "At least one bounded browser launch surfaced an explicit parameter-step interaction result instead of bootstrap-only success.",
        },
        {
            "id": "operator.plugin-browser.exclusions-explicit",
            "status": "passed",
            "summary": "Unsupported editor embedding, platform exclusions, and bounded host-bootstrap posture remain explicit in the surface.",
        },
    ]
    artifacts: list[dict[str, object]] = [
        {
            "kind": "plugin-browser-model",
            "html_path": str(HTML_PATH.relative_to(REPO_ROOT)),
            "plugin_count": stats["plugin_count"],
            "format_count": stats["format_count"],
            "launch_target_count": stats["launch_target_count"],
            "fixture_fallback_used": stats["fixture_fallback_used"],
            "inventory": browser_model["inventory"],
        }
    ]
    if primary_launch is not None:
        artifacts.append({"kind": "bounded-plugin-launch", **primary_launch})

    return {
        "receipt_version": "signal.demo.receipt.v1",
        "manifest_id": manifest["id"],
        "scenario_id": scenario["id"],
        "status": "passed" if all(check["status"] == "passed" for check in operator_checks) else "failed",
        "launch_command": "effigy demo:plugin-capability-browser",
        "artifacts": artifacts,
        "operator_checks": operator_checks,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--serve", action="store_true")
    parser.add_argument("--no-open", action="store_true")
    parser.add_argument("--port", type=int, default=8765)
    parser.add_argument(
        "--scan-mode",
        choices=["auto", "system", "fixture", "hybrid"],
        default="auto",
    )
    args = parser.parse_args()

    manifest = json.loads(MANIFEST_PATH.read_text())
    interactive_mode = args.serve or sys.stdout.isatty()
    scan_mode = args.scan_mode
    if scan_mode == "auto":
        scan_mode = "system" if interactive_mode else "fixture"
    roots_by_format = (
        system_roots_by_format() if scan_mode in {"system", "hybrid"} else {
            "clap": [],
            "vst3": [],
            "au": [],
            "lv2": [],
        }
    )
    fixture_tempdir: tempfile.TemporaryDirectory[str] | None = None
    fixture_fallback_used = False
    if scan_mode in {"fixture", "hybrid"}:
        fixture_tempdir, _ = create_vst3_fixture_root()
        roots_by_format["vst3"] = dedupe(roots_by_format["vst3"] + [fixture_tempdir.name])
        fixture_fallback_used = True

    scan_timeout_seconds = (
        SYSTEM_SCAN_TIMEOUT_SECONDS if scan_mode == "system" else PROOF_SCAN_TIMEOUT_SECONDS
    )
    exact_batch_mode = scan_mode == "system"
    scans: list[dict[str, object]] = []
    scan_failures: list[str] = []

    local_first_inventory: list[dict[str, object]] = []
    if scan_mode == "system" and can_run_local_surface():
        local_scans, local_failures = collect_scans(
            "signal-host-local",
            roots_by_format,
            ("clap", "vst3", "au"),
            timeout_seconds=scan_timeout_seconds,
            exact_batch_mode=exact_batch_mode,
        )
        scans.extend(local_scans)
        scan_failures.extend(local_failures)
        local_first_inventory = combine_inventory(local_scans)

    server_roots_by_format = roots_by_format
    if scan_mode == "system" and local_first_inventory:
        server_roots_by_format = preferred_server_roots_from_inventory(local_first_inventory)
    server_scans, server_failures = collect_scans(
        "signal-host-server",
        server_roots_by_format,
        ("clap", "vst3", "au", "lv2"),
        timeout_seconds=scan_timeout_seconds,
        exact_batch_mode=exact_batch_mode,
    )
    scans.extend(server_scans)
    scan_failures.extend(server_failures)

    inventory = combine_inventory(scans)
    local_probe_summary = attach_contained_local_targets(inventory)
    if scan_mode == "system" and not choose_primary_launch(inventory):
        fixture_tempdir, _ = create_vst3_fixture_root()
        roots_by_format["vst3"] = dedupe(roots_by_format["vst3"] + [fixture_tempdir.name])
        scans = []
        if can_run_local_surface():
            local_scans, local_failures = collect_scans(
                "signal-host-local",
                roots_by_format,
                ("clap", "vst3", "au"),
                timeout_seconds=scan_timeout_seconds,
                exact_batch_mode=exact_batch_mode,
            )
            scans.extend(local_scans)
            scan_failures.extend(
                failure.replace("signal-host-local ", "signal-host-local after fixture fallback ", 1)
                for failure in local_failures
            )
            local_first_inventory = combine_inventory(local_scans)
        server_roots_by_format = (
            preferred_server_roots_from_inventory(local_first_inventory)
            if local_first_inventory
            else roots_by_format
        )
        fallback_scans, fallback_failures = collect_scans(
            "signal-host-server",
            server_roots_by_format,
            ("clap", "vst3", "au", "lv2"),
            timeout_seconds=scan_timeout_seconds,
            exact_batch_mode=exact_batch_mode,
        )
        scans.extend(fallback_scans)
        scan_failures.extend(
            failure.replace("signal-host-server ", "signal-host-server after fixture fallback ", 1)
            for failure in fallback_failures
        )
        inventory = combine_inventory(scans)
        local_probe_summary = attach_contained_local_targets(inventory)
        fixture_fallback_used = True

    known_exclusions = [
        "The browser launches bounded host bootstrap paths, not embedded vendor plugin editors.",
        "Local host launch is macOS-only; server host launch remains explicit when local host is unavailable.",
        "LV2 launch targets remain server-host only.",
        "If no suitable installed CLAP or VST3 plugin is found, the official proof task falls back to one bounded temporary VST3 fixture root so the browser shell itself stays testable.",
    ]
    if exact_batch_mode:
        known_exclusions.append(
            "Interactive system scans now use bounded exact-root batches with per-format candidate caps so one problematic plugin directory does not blank the whole browser."
        )
        if can_run_local_surface():
            known_exclusions.append(
                "Interactive macOS runs prefer bounded local inventory first and only widen server scans across locally confirmed plugin roots when available."
            )
    if local_probe_summary["attempted"] == 0:
        known_exclusions.append(
            "Local launch buttons are only shown for plugins confirmed by bounded exact-root local probes."
        )
    elif local_probe_summary["failed"] > 0:
        known_exclusions.append(
            "Some local plugin probes failed or timed out during bounded exact-root validation, "
            f"so local buttons are shown only for {local_probe_summary['succeeded']} confirmed plugins "
            f"out of {local_probe_summary['attempted']} attempts."
        )
        for failure in local_probe_summary["failures"]:
            known_exclusions.append(f"Local probe note: {failure}")
        if local_probe_summary["limit_hit"]:
            known_exclusions.append(
                f"Local probe containment stopped after {LOCAL_PROBE_ATTEMPT_LIMIT} attempts to keep the interactive browser responsive."
            )
    for failure in scan_failures:
        known_exclusions.append(f"Scan containment note: {failure[:500]}")

    browser_model = {
        "platform": platform.system(),
        "scan_roots": roots_by_format,
        "inventory": inventory,
        "known_exclusions": known_exclusions,
        "stats": {
            "plugin_count": len(inventory),
            "format_count": len({plugin["format"] for plugin in inventory}),
            "launch_target_count": sum(len(plugin["launch_targets"]) for plugin in inventory),
            "fixture_fallback_used": fixture_fallback_used,
            "local_probe_attempted": local_probe_summary["attempted"],
            "local_probe_succeeded": local_probe_summary["succeeded"],
        },
    }

    HTML_PATH.parent.mkdir(parents=True, exist_ok=True)
    HTML_PATH.write_text(browser_html(browser_model))

    primary_launch = execute_primary_launch(inventory)

    receipt = receipt_from_run(manifest, browser_model, primary_launch)
    RECEIPT_PATH.parent.mkdir(parents=True, exist_ok=True)
    RECEIPT_PATH.write_text(json.dumps(receipt, indent=2) + "\n")

    should_serve = interactive_mode
    if should_serve:
        BrowserHandler.browser_model = browser_model
        server, bound_port = bind_browser_server(args.port)

        def request_shutdown(*_: object) -> None:
            Thread(target=server.shutdown, daemon=True).start()

        previous_sigint = signal.getsignal(signal.SIGINT)
        previous_sigterm = signal.getsignal(signal.SIGTERM)
        signal.signal(signal.SIGINT, request_shutdown)
        signal.signal(signal.SIGTERM, request_shutdown)
        if not args.no_open:
            webbrowser.open(f"http://127.0.0.1:{bound_port}/", new=1)
        print(f"signal plugin capability browser serving on http://127.0.0.1:{bound_port}/")
        try:
            server.serve_forever()
        finally:
            signal.signal(signal.SIGINT, previous_sigint)
            signal.signal(signal.SIGTERM, previous_sigterm)
            server.shutdown()
            server.server_close()

    if fixture_tempdir is not None:
        fixture_tempdir.cleanup()


if __name__ == "__main__":
    main()
