/**
 * Environment Configuration
 *
 * The Node server is a thin Backend-for-Frontend (BFF): it proxies /api/*
 * to the Rust api-gateway and serves the built client. It owns no database,
 * cache, auth, or blockchain state — those live in the backend services — so
 * this config only carries what the proxy/static server actually needs.
 */

export const config = {
  nodeEnv: process.env.NODE_ENV || 'development',
  port: parseInt(process.env.PORT || '5000', 10),

  // Target for the /api/* reverse proxy (see server/routes.ts).
  apiGatewayUrl: process.env.API_GATEWAY_URL || 'http://localhost:8080',

  // Computed flags
  isProduction: process.env.NODE_ENV === 'production',
  isDevelopment: process.env.NODE_ENV === 'development',
  isTest: process.env.NODE_ENV === 'test',
} as const;

// Log configuration in development (no secrets involved).
if (config.isDevelopment) {
  console.log('🔧 Configuration loaded:');
  console.log(`   Environment: ${config.nodeEnv}`);
  console.log(`   Port: ${config.port}`);
  console.log(`   API Gateway: ${config.apiGatewayUrl}`);
}
