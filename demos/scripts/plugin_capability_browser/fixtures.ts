import { mkdtempSync, mkdirSync } from "node:fs";
import { resolve } from "node:path";
import { tmpdir } from "node:os";
import { writeJson, writeText } from "./io.ts";

export function createVst3FixtureRoot(): { tempdir: string; pluginTypeId: string } {
  const tempdir = mkdtempSync(resolve(tmpdir(), "signal-plugin-browser-vst3-"));
  const bundleRoot = resolve(tempdir, "Signal Browser Instrument.vst3");
  const resourcesRoot = resolve(bundleRoot, "Contents/Resources");
  mkdirSync(resourcesRoot, { recursive: true });
  const pluginTypeId = "plugin:vst3:browser-fixture";
  const infoPlist = `<?xml version="1.0" encoding="UTF-8"?>
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
    <string>${pluginTypeId}</string>
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
`;
  const moduleinfo = {
    Classes: [
      {
        CID: "7E1D8F8A4D874D56A2C44DE250199901",
        Category: "Audio Module Class",
        Name: "Signal Browser Instrument",
        Vendor: "Signal",
        Version: "0.1.0",
        SubCategories: ["Instrument", "Analyzer"],
        ClassFlags: 1,
        Snapshots: [],
      },
      {
        CID: "7E1D8F8A4D874D56A2C44DE250199902",
        Category: "Component Controller Class",
        Name: "Signal Browser Instrument Controller",
        Vendor: "Signal",
        Version: "0.1.0",
        SubCategories: [],
        ClassFlags: 1,
        Snapshots: [],
      },
    ],
  };
  writeText(resolve(bundleRoot, "Contents/Info.plist"), infoPlist);
  writeJson(resolve(resourcesRoot, "moduleinfo.json"), moduleinfo);
  return { tempdir, pluginTypeId };
}
