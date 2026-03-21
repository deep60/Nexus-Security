import type { Express } from "express";
import { createServer, type Server } from "http";
import { createProxyMiddleware } from "http-proxy-middleware";
import { config } from "./config";

/**
 * Register routes for the frontend server.
 *
 * In production the Node server is a thin "Backend-for-Frontend" (BFF):
 *   - All /api/* requests are proxied to the Rust api-gateway
 *   - Static files are served by Express (configured in index.ts)
 *
 * In development Vite's built-in proxy handles forwarding, so the
 * middleware below is only active in production builds.
 */
export async function registerRoutes(app: Express): Promise<Server> {
  const API_GATEWAY_URL =
    process.env.API_GATEWAY_URL || config.apiGatewayUrl || "http://localhost:8080";

  // Proxy all /api/* requests to the Rust api-gateway, rewriting
  // /api/… → /api/v1/… to match the gateway's versioned routes.
  app.use(
    "/api",
    createProxyMiddleware({
      target: API_GATEWAY_URL,
      changeOrigin: true,
      pathRewrite: { "^/api": "/api/v1" },
      // Forward WebSocket upgrade requests
      ws: true,
      // Log proxy events for debugging
      on: {
        proxyReq: (proxyReq, req) => {
          console.log(
            `[proxy] ${req.method} ${req.url} → ${API_GATEWAY_URL}/api/v1${req.url}`
          );
        },
        error: (err, req, res) => {
          console.error(`[proxy] Error proxying ${req.url}:`, err.message);
          if ("writeHead" in res && typeof res.writeHead === "function") {
            (res as any).writeHead(502, { "Content-Type": "application/json" });
            (res as any).end(
              JSON.stringify({
                error: "API Gateway unavailable",
                message:
                  "The backend API gateway is not reachable. Please ensure it is running.",
              })
            );
          }
        },
      },
    })
  );

  const httpServer = createServer(app);
  return httpServer;
}
