/**
 * Cloudflare Pages Function — /api/* proxy.
 *
 * The edge equivalent of the Node BFF in `server/routes.ts`. Keeping the API on
 * the same origin as the app is deliberate: `queryClient.ts` fetches relative
 * URLs with `credentials: "include"`, so a cross-origin backend would require
 * CORS plus `SameSite=None` cookies. Proxying here avoids all of that.
 *
 * Configure `API_GATEWAY_URL` in the Pages project (Settings → Environment
 * variables), e.g. https://verdyx-api.onrender.com. Until it is set, every /api
 * call returns 503 with the same JSON shape the Node BFF uses, which the UI
 * already renders as an empty state rather than a crash.
 */

interface Env {
  API_GATEWAY_URL?: string;
}

/** Hop-by-hop headers that must not be forwarded to the origin. */
const STRIPPED_REQUEST_HEADERS = ["host", "connection", "content-length"];

/**
 * Normalize any incoming /api shape to the gateway's canonical /api/v1/… form,
 * matching `normalizeToV1` in server/routes.ts so both proxies agree.
 */
function normalizeToV1(segments: string[]): string {
  const parts = [...segments];
  if (parts[0] === "v1") parts.shift();
  return `/api/v1/${parts.join("/")}`;
}

export const onRequest: PagesFunction<Env> = async (context) => {
  const { request, env, params } = context;

  const gateway = env.API_GATEWAY_URL;
  if (!gateway) {
    return Response.json(
      {
        error: "API Gateway unavailable",
        message:
          "API_GATEWAY_URL is not configured for this deployment. The frontend " +
          "is live but no backend is attached yet.",
      },
      { status: 503 },
    );
  }

  // `params.path` is the [[path]] wildcard: undefined for /api, string for a
  // single segment, string[] for the rest.
  const raw = params.path;
  const segments = raw === undefined ? [] : Array.isArray(raw) ? raw : [raw];

  const incoming = new URL(request.url);
  const target = new URL(gateway);
  target.pathname = normalizeToV1(segments);
  target.search = incoming.search;

  const headers = new Headers(request.headers);
  for (const h of STRIPPED_REQUEST_HEADERS) headers.delete(h);
  // Preserve the browser-facing host so the gateway can build correct absolute
  // URLs (redirects, cookie domains) despite sitting behind this proxy.
  headers.set("X-Forwarded-Host", incoming.host);
  headers.set("X-Forwarded-Proto", incoming.protocol.replace(":", ""));

  // GET/HEAD must not carry a body; everything else streams straight through.
  const hasBody = request.method !== "GET" && request.method !== "HEAD";

  try {
    return await fetch(target.toString(), {
      method: request.method,
      headers,
      body: hasBody ? request.body : undefined,
      redirect: "manual",
    });
  } catch (err) {
    // Log the URL as a separate argument rather than interpolating it, so a
    // user-controlled path cannot inject into the log line.
    console.error("[proxy] error forwarding request:", incoming.pathname, err);
    return Response.json(
      {
        error: "API Gateway unavailable",
        message: "The backend API gateway is not reachable.",
      },
      { status: 502 },
    );
  }
};
