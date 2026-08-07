import { createServer, type IncomingMessage, type ServerResponse } from "node:http";
import { type JsonObject } from "./paths.ts";
import { browserHtml } from "./html.ts";
import { runLaunch } from "./launch.ts";

export type BrowserHandlerState = {
  browserModel: JsonObject;
};

export function bindBrowserServer(state: BrowserHandlerState, preferredPort: number) {
  return new Promise<{ server: ReturnType<typeof createServer>; port: number }>((resolvePromise, reject) => {
    const tryPort = (port: number) => {
      const server = createServer(async (req: IncomingMessage, res: ServerResponse) => {
        await handleRequest(state, req, res);
      });
      server.listen(port, "127.0.0.1");
      server.once("listening", () => resolvePromise({ server, port }));
      server.once("error", (error: any) => {
        server.close();
        if (error?.code === "EADDRINUSE" && port < preferredPort + 19) {
          tryPort(port + 1);
          return;
        }
        reject(error);
      });
    };
    tryPort(preferredPort);
  });
}

async function handleRequest(state: BrowserHandlerState, req: IncomingMessage, res: ServerResponse) {
  const url = req.url ?? "/";
  if (req.method === "GET" && (url === "/" || url === "/index.html")) {
    const body = Buffer.from(browserHtml(state.browserModel), "utf8");
    res.writeHead(200, {
      "Content-Type": "text/html; charset=utf-8",
      "Content-Length": body.length,
    });
    res.end(body);
    return;
  }
  if (req.method === "POST" && url === "/launch") {
    const chunks: Buffer[] = [];
    for await (const chunk of req) {
      chunks.push(Buffer.from(chunk));
    }
    const payload = JSON.parse(Buffer.concat(chunks).toString("utf8") || "{}") as JsonObject;
    const pkg = "signal-host-local";
    const result = runLaunch(pkg, payload);
    const body = Buffer.from(JSON.stringify(result, null, 2), "utf8");
    res.writeHead(200, {
      "Content-Type": "application/json; charset=utf-8",
      "Content-Length": body.length,
    });
    res.end(body);
    return;
  }
  res.writeHead(404);
  res.end();
}
