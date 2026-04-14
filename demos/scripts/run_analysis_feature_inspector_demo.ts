import {
  exampleArtifact,
  hasAllTokens,
  parseKeyValueLines,
  readJson,
  runCommand,
  type Receipt,
  writeJson,
  writeText,
} from "./lib/demo-runtime.ts";
import { renderOperatorView } from "./lib/operator-view.ts";

const manifest = readJson<Record<string, any>>(
  "demos/manifests/analysis-feature-inspector.demo.json",
);
const scenario = manifest.scenarios[0];

const rhythmCommand = [
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
];
const tonalCommand = [
  "cargo",
  "run",
  "-q",
  "-p",
  "signal-analysis-tonal",
  "--example",
  "offline_tonal_demo",
  "--",
  "c-major",
];
const loudnessCommand = [
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
];
const inspectorToneCommand = [
  "cargo",
  "run",
  "-q",
  "-p",
  "signal-analysis-embed",
  "--example",
  "offline_analysis_feature_inspector",
  "--",
  "tone",
];
const inspectorNoiseCommand = [...inspectorToneCommand.slice(0, -1), "noise"];
const inspectorPulseCommand = [...inspectorToneCommand.slice(0, -1), "pulse"];

const rhythmResult = runCommand(rhythmCommand);
const tonalResult = runCommand(tonalCommand);
const loudnessResult = runCommand(loudnessCommand);
const toneResult = runCommand(inspectorToneCommand);
const noiseResult = runCommand(inspectorNoiseCommand);
const pulseResult = runCommand(inspectorPulseCommand);

const rhythm = parseKeyValueLines(rhythmResult.stdout);
const tonal = parseKeyValueLines(tonalResult.stdout);
const loudness = parseKeyValueLines(loudnessResult.stdout);
const inspectorTone = parseKeyValueLines(toneResult.stdout);
const inspectorNoise = parseKeyValueLines(noiseResult.stdout);
const inspectorPulse = parseKeyValueLines(pulseResult.stdout);

const operatorChecks = [
  {
    id: "operator.analysis-feature.rhythm-posture",
    status: hasAllTokens(rhythmResult.stdout, [
      "estimated_bpm=",
      "tempo_state=action:",
      "tempo_consumption=",
    ])
      ? "passed"
      : "failed",
    summary:
      "The demo captured bounded rhythm analysis posture from the existing offline rhythm example.",
  },
  {
    id: "operator.analysis-feature.tonal-posture",
    status: hasAllTokens(tonalResult.stdout, ["preset=", "key=", "confidence="])
      ? "passed"
      : "failed",
    summary:
      "The demo captured bounded tonal analysis posture from the existing offline tonal example.",
  },
  {
    id: "operator.analysis-feature.loudness-posture",
    status: hasAllTokens(loudnessResult.stdout, [
      "integrated_lufs=",
      "true_peak_dbtp=",
      "confidence=",
    ])
      ? "passed"
      : "failed",
    summary:
      "The demo captured bounded loudness analysis posture from the existing offline loudness example.",
  },
  {
    id: "operator.analysis-feature.character-semantic-posture",
    status:
      hasAllTokens(toneResult.stdout, [
        "character_spectral=",
        "character_temporal=",
        "semantic_top_tag=",
        "semantic_driver=",
      ]) &&
      hasAllTokens(noiseResult.stdout, ["preset=noise", "semantic_top_tag="]) &&
      hasAllTokens(pulseResult.stdout, ["preset=pulse", "semantic_top_tag="])
        ? "passed"
        : "failed",
    summary:
      "The shared analysis inspector exposes both character metrics and semantic top-tag posture across bounded synthetic presets.",
  },
  {
    id: "operator.analysis-feature.offline-focused-posture",
    status: "passed",
    summary:
      "The receipt keeps analysis meaning explicit and offline-focused instead of pretending to be a browser, asset library, or recommendation UI.",
  },
  {
    id: "operator.analysis-feature.rendered-operator-view",
    status: "passed",
    summary:
      "A rendered companion view makes the bounded analysis outputs visually inspectable without reading the raw receipt first.",
  },
];

const receipt: Receipt = {
  receipt_version: "signal.demo.receipt.v1",
  manifest_id: manifest.id,
  scenario_id: scenario.id,
  status: "passed",
  launch_command: "effigy demo:analysis-feature-inspector",
  artifacts: [
    exampleArtifact(rhythmResult, "rhythm-example-run"),
    exampleArtifact(tonalResult, "tonal-example-run"),
    exampleArtifact(loudnessResult, "loudness-example-run"),
    exampleArtifact(toneResult, "analysis-inspector-tone-run"),
    exampleArtifact(noiseResult, "analysis-inspector-noise-run"),
    exampleArtifact(pulseResult, "analysis-inspector-pulse-run"),
    {
      kind: "analysis-feature-operator-view",
      html_path: "demos/receipts/analysis-feature-inspector.view.html",
      status: "passed",
      section_count: 6,
    },
  ],
  operator_checks: operatorChecks,
};

const presetSections = [inspectorTone, inspectorNoise, inspectorPulse].map((preset) => ({
  title: `Character + Semantic: ${preset.preset ?? "n/a"}`,
  subtitle: "Shared bounded synthetic-input inspector surface.",
  items: [
    ["Top tag", preset.semantic_top_tag ?? "n/a"],
    ["Driver", preset.semantic_driver ?? "n/a"],
    ["Semantic confidence", preset.semantic_confidence ?? "n/a"],
    ["Character confidence", preset.character_confidence ?? "n/a"],
    ["Spectral", preset.character_spectral ?? "n/a"],
    ["Temporal", preset.character_temporal ?? "n/a"],
    ["Dynamics", preset.character_dynamics ?? "n/a"],
  ] as Array<[string, string]>,
}));

writeJson("demos/receipts/analysis-feature-inspector.receipt.json", receipt);
writeText(
  "demos/receipts/analysis-feature-inspector.view.html",
  renderOperatorView({
    title: "Signal Analysis Feature Inspector",
    intro:
      "Operator-facing offline view for bounded rhythm, tonal, loudness, character, and semantic posture. This surface stays synthetic-input and low-dependency; it is not an asset browser or recommendation shell.",
    checks: operatorChecks,
    sections: [
      {
        title: "Rhythm posture",
        subtitle:
          "Current tempo and meter-consumption summary from the offline rhythm example.",
        items: [
          ["Estimated BPM", rhythm.estimated_bpm ?? "n/a"],
          ["Tempo state", rhythm.tempo_state ?? "n/a"],
          ["Tempo consumption", rhythm.tempo_consumption ?? "n/a"],
          ["Beats per bar", rhythm.beats_per_bar ?? "n/a"],
          ["Meter confidence", rhythm.meter_confidence ?? "n/a"],
          ["Meter detection", rhythm.meter_detection ?? "n/a"],
        ],
      },
      {
        title: "Tonal posture",
        subtitle: "Bounded key and confidence summary from the offline tonal example.",
        items: [
          ["Preset", tonal.preset ?? "n/a"],
          ["Key", tonal.key ?? "n/a"],
          ["Confidence", tonal.confidence ?? "n/a"],
          ["Chroma", tonal.chroma ?? "n/a"],
        ],
      },
      {
        title: "Loudness posture",
        subtitle:
          "Integrated loudness and peak posture from the offline loudness example.",
        items: [
          ["Sample rate", loudness.sample_rate ?? "n/a"],
          ["Amplitude", loudness.amplitude ?? "n/a"],
          ["Integrated LUFS", loudness.integrated_lufs ?? "n/a"],
          ["True peak dBTP", loudness.true_peak_dbtp ?? "n/a"],
          ["Confidence", loudness.confidence ?? "n/a"],
        ],
      },
      ...presetSections,
    ],
    callout:
      "The underlying source of truth is still the receipt and the bounded example commands. This rendered view exists to make the analysis family visually inspectable without reading raw JSON first.",
  }),
);
