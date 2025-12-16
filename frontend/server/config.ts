/**
 * Environment Configuration
 * Centralizes all environment variables with type safety and defaults
 */

export const config = {
  // Application
  nodeEnv: process.env.NODE_ENV || 'development',
  port: parseInt(process.env.PORT || '5000', 10),
  frontendUrl: process.env.FRONTEND_URL || 'http://localhost:5173',

  // Database
  databaseUrl: process.env.DATABASE_URL || '',

  // Redis
  redisUrl: process.env.REDIS_URL || '',

  // Authentication
  jwtSecret: process.env.JWT_SECRET || 'dev-secret-key-do-not-use-in-production',
  sessionExpiry: parseInt(process.env.SESSION_EXPIRY || '604800000', 10), // 7 days

  // Security
  bcryptSaltRounds: parseInt(process.env.BCRYPT_SALT_ROUNDS || '10', 10),
  rateLimitWindowMs: parseInt(process.env.RATE_LIMIT_WINDOW_MS || '900000', 10), // 15 minutes
  rateLimitMaxRequests: parseInt(process.env.RATE_LIMIT_MAX_REQUESTS || '100', 10),

  // Blockchain
  ethProviderUrl: process.env.ETH_PROVIDER_URL || '',
  paymentContractAddress: process.env.PAYMENT_CONTRACT_ADDRESS || '',
  stakingContractAddress: process.env.STAKING_CONTRACT_ADDRESS || '',
  rewardContractAddress: process.env.REWARD_CONTRACT_ADDRESS || '',

  // IPFS
  ipfsGateway: process.env.IPFS_GATEWAY || '',
  ipfsProjectId: process.env.IPFS_PROJECT_ID || '',
  ipfsProjectSecret: process.env.IPFS_PROJECT_SECRET || '',

  // Email
  smtpHost: process.env.SMTP_HOST || '',
  smtpPort: parseInt(process.env.SMTP_PORT || '587', 10),
  smtpUser: process.env.SMTP_USER || '',
  smtpPassword: process.env.SMTP_PASSWORD || '',
  emailFrom: process.env.EMAIL_FROM || 'noreply@nexus-security.com',

  // Monitoring
  sentryDsn: process.env.SENTRY_DSN || '',

  // Feature Flags
  enableBlockchain: process.env.ENABLE_BLOCKCHAIN === 'true',
  enableIpfs: process.env.ENABLE_IPFS === 'true',
  enableEmail: process.env.ENABLE_EMAIL === 'true',

  // Computed flags
  isProduction: process.env.NODE_ENV === 'production',
  isDevelopment: process.env.NODE_ENV === 'development',
  isTest: process.env.NODE_ENV === 'test',
} as const;

// Validate critical configuration in production
if (config.isProduction) {
  const requiredVars = [
    'JWT_SECRET',
    'DATABASE_URL',
    'FRONTEND_URL',
  ];

  const missing = requiredVars.filter(
    (varName) => !process.env[varName] || process.env[varName] === 'dev-secret-key-do-not-use-in-production'
  );

  if (missing.length > 0) {
    console.error('❌ Missing required environment variables for production:');
    missing.forEach((varName) => console.error(`   - ${varName}`));
    console.error('\nPlease configure these in your .env file.');
    process.exit(1);
  }
}

// Log configuration (excluding secrets)
if (config.isDevelopment) {
  console.log('🔧 Configuration loaded:');
  console.log(`   Environment: ${config.nodeEnv}`);
  console.log(`   Port: ${config.port}`);
  console.log(`   Frontend URL: ${config.frontendUrl}`);
  console.log(`   Database: ${config.databaseUrl ? 'Configured' : 'In-Memory'}`);
  console.log(`   Redis: ${config.redisUrl ? 'Configured' : 'Disabled'}`);
  console.log(`   Blockchain: ${config.enableBlockchain ? 'Enabled' : 'Disabled'}`);
  console.log(`   IPFS: ${config.enableIpfs ? 'Enabled' : 'Disabled'}`);
  console.log(`   Email: ${config.enableEmail ? 'Enabled' : 'Disabled'}`);
}
