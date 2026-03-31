# Multi-stage Dockerfile for Nexus Security Frontend
# Stage 1: Build the application
FROM node:20-alpine AS builder

WORKDIR /app

# Copy package files
COPY package*.json ./

# Install dependencies
RUN npm ci

# Copy source code
COPY . .

# Build the client application (vite build -> dist/)
RUN npm run build

# Bundle the server TypeScript into a single JS file
# tsconfig has noEmit:true so tsc won't produce .js files — use esbuild instead
RUN npx esbuild server/index.ts --bundle --platform=node --outfile=dist/server.js \
    --external:express --external:dotenv --external:drizzle-orm --external:drizzle-kit \
    --external:pg --external:connect-pg-simple --external:express-session \
    --external:memorystore --external:ws --external:cors

# Stage 2: Production image
FROM node:20-alpine

WORKDIR /app

# Install production dependencies only
COPY package*.json ./
RUN npm ci --production

# Copy built assets and bundled server from builder
COPY --from=builder /app/dist ./dist
COPY --from=builder /app/drizzle ./drizzle
COPY --from=builder /app/drizzle.config.ts ./

# Create non-root user
RUN addgroup -g 1001 -S nodejs && \
    adduser -S nodejs -u 1001

# Change ownership
RUN chown -R nodejs:nodejs /app

# Switch to non-root user
USER nodejs

# Expose port
EXPOSE 5000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=40s --retries=3 \
  CMD node -e "require('http').get('http://localhost:5000/api/engines', (r) => {process.exit(r.statusCode === 200 ? 0 : 1)})"

# Start the application
CMD ["node", "dist/server.js"]