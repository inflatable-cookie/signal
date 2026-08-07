export function parseArgs(argv: string[]) {
  const args = {
    serve: false,
    noOpen: false,
    port: 8765,
    scanMode: "auto",
  };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index]!;
    if (arg === "--serve") args.serve = true;
    else if (arg === "--no-open") args.noOpen = true;
    else if (arg === "--port") args.port = Number.parseInt(argv[++index]!, 10);
    else if (arg === "--scan-mode") args.scanMode = argv[++index]!;
  }
  return args;
}
