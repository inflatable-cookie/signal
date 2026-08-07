import { resolve } from "node:path";

export type JsonObject = Record<string, any>;

export const REPO_ROOT = resolve(import.meta.dir, "../../..");
export const MANIFEST_PATH = resolve(REPO_ROOT, "demos/manifests/plugin-capability-browser.demo.json");
export const RECEIPT_PATH = resolve(REPO_ROOT, "demos/receipts/plugin-capability-browser.receipt.json");
export const HTML_PATH = resolve(REPO_ROOT, "demos/receipts/plugin-capability-browser.view.html");
