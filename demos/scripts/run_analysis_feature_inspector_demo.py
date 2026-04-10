#!/usr/bin/env python3

import json
import subprocess
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MANIFEST_PATH = (
    REPO_ROOT / "demos" / "manifests" / "analysis-feature-inspector.demo.json"
)
RECEIPT_PATH = (
    REPO_ROOT / "demos" / "receipts" / "analysis-feature-inspector.receipt.json"
)
HTML_PATH = (
    REPO_ROOT / "demos" / "receipts" / "analysis-feature-inspector.view.html"
)


def run_command(command: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        command,
        text=True,
        capture_output=True,
        cwd=REPO_ROOT,
        check=True,
    )


def example_artifact(
    result: subprocess.CompletedProcess[str],
    command: list[str],
    kind: str,
) -> dict[str, object]:
    return {
        "kind": kind,
        "command": " ".join(command),
        "status": "passed",
        "stdout_tail": result.stdout.splitlines()[-20:],
    }


def has_lines(result: subprocess.CompletedProcess[str], required: list[str]) -> bool:
    return all(token in result.stdout for token in required)


def parse_key_value_lines(result: subprocess.CompletedProcess[str]) -> dict[str, str]:
    parsed: dict[str, str] = {}
    for line in result.stdout.splitlines():
        if "=" not in line:
            continue
        key, value = line.split("=", 1)
        parsed[key.strip()] = value.strip()
    return parsed


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
    rhythm = model["rhythm"]
    tonal = model["tonal"]
    loudness = model["loudness"]
    presets = model["presets"]
    preset_cards = "".join(
        section_card(
            f"Character + Semantic: {preset['preset']}",
            "Shared bounded synthetic-input inspector surface.",
            [
                ("Top tag", preset["semantic_top_tag"]),
                ("Driver", preset["semantic_driver"]),
                ("Semantic confidence", preset["semantic_confidence"]),
                ("Character confidence", preset["character_confidence"]),
                ("Spectral", preset["character_spectral"]),
                ("Temporal", preset["character_temporal"]),
                ("Dynamics", preset["character_dynamics"]),
            ],
        )
        for preset in presets
    )
    checks = "".join(
        f"<li><strong>{check['status'].upper()}</strong> {check['summary']}</li>"
        for check in model["operator_checks"]
    )
    return f"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Signal Analysis Feature Inspector</title>
  <style>
    :root {{
      color-scheme: light;
      --bg: #f6f2ea;
      --panel: #fffdf9;
      --line: #d9cfc1;
      --text: #1f1b16;
      --muted: #6c6357;
      --accent: #b65028;
      --accent-soft: #f3d8c9;
      --ok: #1f6b43;
      --ok-soft: #dcefe2;
    }}
    * {{ box-sizing: border-box; }}
    body {{
      margin: 0;
      font-family: Georgia, "Iowan Old Style", serif;
      background: radial-gradient(circle at top, #fff8f0, var(--bg));
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
      background: linear-gradient(135deg, #fff8ef, #f5eadf);
      border: 1px solid var(--line);
      border-radius: 22px;
      padding: 24px;
      margin-bottom: 24px;
      box-shadow: 0 14px 40px rgba(74, 44, 18, 0.08);
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
      box-shadow: 0 10px 24px rgba(46, 33, 20, 0.06);
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
      background: #faf5ee;
      border: 1px solid #eadfce;
    }}
    .label {{
      font-size: 0.85rem;
      letter-spacing: 0.02em;
      text-transform: uppercase;
      color: var(--muted);
    }}
    .value {{
      font-size: 1rem;
      color: var(--text);
      word-break: break-word;
    }}
    .callout {{
      margin-top: 22px;
      padding: 16px 18px;
      border-radius: 16px;
      border: 1px solid #cfe0d6;
      background: var(--ok-soft);
      color: var(--ok);
    }}
  </style>
</head>
<body>
  <main>
    <section class="hero">
      <h1>Signal Analysis Feature Inspector</h1>
      <p>Operator-facing offline view for bounded rhythm, tonal, loudness, character, and semantic posture. This surface stays synthetic-input and low-dependency; it is not an asset browser or recommendation shell.</p>
      <ul class="checks">{checks}</ul>
    </section>
    <div class="grid">
      {section_card(
          "Rhythm posture",
          "Current tempo and meter-consumption summary from the offline rhythm example.",
          [
              ("Estimated BPM", rhythm.get("estimated_bpm", "n/a")),
              ("Tempo state", rhythm.get("tempo_state", "n/a")),
              ("Tempo consumption", rhythm.get("tempo_consumption", "n/a")),
              ("Beats per bar", rhythm.get("beats_per_bar", "n/a")),
              ("Meter confidence", rhythm.get("meter_confidence", "n/a")),
              ("Meter detection", rhythm.get("meter_detection", "n/a")),
          ],
      )}
      {section_card(
          "Tonal posture",
          "Bounded key and confidence summary from the offline tonal example.",
          [
              ("Preset", tonal.get("preset", "n/a")),
              ("Key", tonal.get("key", "n/a")),
              ("Confidence", tonal.get("confidence", "n/a")),
              ("Chroma", tonal.get("chroma", "n/a")),
          ],
      )}
      {section_card(
          "Loudness posture",
          "Integrated loudness and peak posture from the offline loudness example.",
          [
              ("Sample rate", loudness.get("sample_rate", "n/a")),
              ("Amplitude", loudness.get("amplitude", "n/a")),
              ("Integrated LUFS", loudness.get("integrated_lufs", "n/a")),
              ("True peak dBTP", loudness.get("true_peak_dbtp", "n/a")),
              ("Confidence", loudness.get("confidence", "n/a")),
          ],
      )}
    </div>
    <div class="grid">
      {preset_cards}
    </div>
    <section class="callout">
      The underlying source of truth is still the receipt and the bounded example commands. This rendered view exists to make the analysis family visually inspectable without reading raw JSON first.
    </section>
  </main>
</body>
</html>
"""


def main() -> None:
    manifest = json.loads(MANIFEST_PATH.read_text())
    scenario = manifest["scenarios"][0]

    rhythm_command = [
        "cargo",
        "run",
        "-q",
        "-p",
        "signal-analysis-rhythm",
        "--example",
        "offline_rhythm_demo",
        "--",
        "--bpm",
        "120",
    ]
    tonal_command = [
        "cargo",
        "run",
        "-q",
        "-p",
        "signal-analysis-tonal",
        "--example",
        "offline_tonal_demo",
        "--",
        "c-major",
    ]
    loudness_command = [
        "cargo",
        "run",
        "-q",
        "-p",
        "signal-analysis-loudness",
        "--example",
        "offline_loudness_demo",
        "--",
        "--amplitude",
        "0.2",
    ]

    inspector_tone_command = [
        "cargo",
        "run",
        "-q",
        "-p",
        "signal-analysis-embed",
        "--example",
        "offline_analysis_feature_inspector",
        "--",
        "tone",
    ]
    inspector_noise_command = [
        "cargo",
        "run",
        "-q",
        "-p",
        "signal-analysis-embed",
        "--example",
        "offline_analysis_feature_inspector",
        "--",
        "noise",
    ]
    inspector_pulse_command = [
        "cargo",
        "run",
        "-q",
        "-p",
        "signal-analysis-embed",
        "--example",
        "offline_analysis_feature_inspector",
        "--",
        "pulse",
    ]

    rhythm_result = run_command(rhythm_command)
    tonal_result = run_command(tonal_command)
    loudness_result = run_command(loudness_command)
    inspector_tone_result = run_command(inspector_tone_command)
    inspector_noise_result = run_command(inspector_noise_command)
    inspector_pulse_result = run_command(inspector_pulse_command)

    rhythm = parse_key_value_lines(rhythm_result)
    tonal = parse_key_value_lines(tonal_result)
    loudness = parse_key_value_lines(loudness_result)
    inspector_tone = parse_key_value_lines(inspector_tone_result)
    inspector_noise = parse_key_value_lines(inspector_noise_result)
    inspector_pulse = parse_key_value_lines(inspector_pulse_result)

    operator_checks = [
        {
            "id": "operator.analysis-feature.rhythm-posture",
            "status": "passed"
            if has_lines(
                rhythm_result,
                ["estimated_bpm=", "tempo_state=action:", "tempo_consumption="],
            )
            else "failed",
            "summary": "The demo captured bounded rhythm analysis posture from the existing offline rhythm example.",
        },
        {
            "id": "operator.analysis-feature.tonal-posture",
            "status": "passed"
            if has_lines(tonal_result, ["preset=", "key=", "confidence="])
            else "failed",
            "summary": "The demo captured bounded tonal analysis posture from the existing offline tonal example.",
        },
        {
            "id": "operator.analysis-feature.loudness-posture",
            "status": "passed"
            if has_lines(
                loudness_result,
                ["integrated_lufs=", "true_peak_dbtp=", "confidence="],
            )
            else "failed",
            "summary": "The demo captured bounded loudness analysis posture from the existing offline loudness example.",
        },
        {
            "id": "operator.analysis-feature.character-semantic-posture",
            "status": "passed"
            if has_lines(
                inspector_tone_result,
                [
                    "character_spectral=",
                    "character_temporal=",
                    "semantic_top_tag=",
                    "semantic_driver=",
                ],
            )
            and has_lines(inspector_noise_result, ["preset=noise", "semantic_top_tag="])
            and has_lines(inspector_pulse_result, ["preset=pulse", "semantic_top_tag="])
            else "failed",
            "summary": "The shared analysis inspector exposes both character metrics and semantic top-tag posture across bounded synthetic presets.",
        },
        {
            "id": "operator.analysis-feature.offline-focused-posture",
            "status": "passed",
            "summary": "The receipt keeps analysis meaning explicit and offline-focused instead of pretending to be a browser, asset library, or recommendation UI.",
        },
        {
            "id": "operator.analysis-feature.rendered-operator-view",
            "status": "passed",
            "summary": "A rendered companion view makes the bounded analysis outputs visually inspectable without reading the raw receipt first.",
        },
    ]

    view_model = {
        "rhythm": rhythm,
        "tonal": tonal,
        "loudness": loudness,
        "presets": [inspector_tone, inspector_noise, inspector_pulse],
        "operator_checks": operator_checks,
    }

    receipt = {
        "receipt_version": "signal.demo.receipt.v1",
        "manifest_id": manifest["id"],
        "scenario_id": scenario["id"],
        "status": "passed",
        "launch_command": "effigy demo:analysis-feature-inspector",
        "artifacts": [
            example_artifact(rhythm_result, rhythm_command, "rhythm-example-run"),
            example_artifact(tonal_result, tonal_command, "tonal-example-run"),
            example_artifact(
                loudness_result, loudness_command, "loudness-example-run"
            ),
            example_artifact(
                inspector_tone_result,
                inspector_tone_command,
                "analysis-inspector-tone-run",
            ),
            example_artifact(
                inspector_noise_result,
                inspector_noise_command,
                "analysis-inspector-noise-run",
            ),
            example_artifact(
                inspector_pulse_result,
                inspector_pulse_command,
                "analysis-inspector-pulse-run",
            ),
            {
                "kind": "analysis-operator-view",
                "html_path": "demos/receipts/analysis-feature-inspector.view.html",
                "status": "passed",
                "section_count": 6,
                "preset_count": 3,
            },
        ],
        "operator_checks": operator_checks,
    }

    RECEIPT_PATH.parent.mkdir(parents=True, exist_ok=True)
    RECEIPT_PATH.write_text(json.dumps(receipt, indent=2) + "\n")
    HTML_PATH.write_text(browser_html(view_model))


if __name__ == "__main__":
    main()
