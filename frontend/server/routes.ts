import type { Express, Request, Response } from "express";
import { createServer, type Server, request as httpRequest } from "http";
import { URL } from "url";
import { config } from "./config";

/**
 * Register routes for the frontend server.
 *
 * The Node server is a thin "Backend-for-Frontend" (BFF):
 *   - All /api/* requests are proxied to the Rust api-gateway
 *   - Static files are served by Express (configured in index.ts)
 *
 * We implement the proxy manually (piping the raw request stream) rather than
 * using http-proxy-middleware. The library's v3 release stalls proxied POST/PUT
 * bodies in this setup; a direct stream pipe is simpler and reliable.
 */
export async function registerRoutes(app: Express): Promise<Server> {
  const API_GATEWAY_URL =
    process.env.API_GATEWAY_URL || config.apiGatewayUrl || "http://localhost:8080";
  const target = new URL(API_GATEWAY_URL);

  // Normalize any incoming /api shape to the gateway's canonical /api/v1/… form.
  // Express mounts this at "/api" and strips that prefix, so `req.url` here is
  // already relative (e.g. "/analysis/stats" or "/v1/auth/login").
  const normalizeToV1 = (url: string): string => {
    let p = url;
    if (p.startsWith("/api")) p = p.slice(4);
    if (p.startsWith("/v1")) p = p.slice(3);
    if (!p.startsWith("/")) p = `/${p}`;
    return `/api/v1${p}`;
  };

  app.use("/api", (req: Request, res: Response) => {
    const path = normalizeToV1(req.url);
    console.log(`[proxy] ${req.method} ${req.originalUrl} → ${API_GATEWAY_URL}${path}`);

    // Forward original headers; fix Host so changeOrigin semantics hold.
    const headers = { ...req.headers, host: target.host };

    const proxyReq = httpRequest(
      {
        protocol: target.protocol,
        hostname: target.hostname,
        port: target.port || 80,
        method: req.method,
        path,
        headers,
      },
      (proxyRes) => {
        res.writeHead(proxyRes.statusCode || 502, proxyRes.headers);
        proxyRes.pipe(res);
      }
    );

    proxyReq.on("error", (err) => {
      console.error(`[proxy] Error proxying ${req.originalUrl}:`, err.message);
      if (!res.headersSent) {
        res.writeHead(502, { "Content-Type": "application/json" });
      }
      res.end(
        JSON.stringify({
          error: "API Gateway unavailable",
          message:
            "The backend API gateway is not reachable. Please ensure it is running.",
        })
      );
    });

    // Pipe the raw request body straight through (works for GET and POST/PUT).
    req.pipe(proxyReq);
  });

  const httpServer = createServer(app);
  return httpServer;
}
