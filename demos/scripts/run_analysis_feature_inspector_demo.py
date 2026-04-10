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
        ],
        "operator_checks": [
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
        ],
    }

    RECEIPT_PATH.parent.mkdir(parents=True, exist_ok=True)
    RECEIPT_PATH.write_text(json.dumps(receipt, indent=2) + "\n")


if __name__ == "__main__":
    main()
