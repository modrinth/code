# ==============================================
# OWYX Backend API — Express + Socket.io
# ==============================================

FROM node:22-alpine AS deps
WORKDIR /app
RUN apk add --no-cache python3 make g++ vips-dev
COPY package.json package-lock.json* ./
RUN npm ci

FROM node:22-alpine AS builder
WORKDIR /app
RUN apk add --no-cache python3 make g++ vips-dev
COPY --from=deps /app/node_modules ./node_modules
COPY . .
RUN npm run build

FROM node:22-alpine AS runner
WORKDIR /app
ENV NODE_ENV=production
ENV TZ=Europe/Moscow
RUN apk add --no-cache vips \
  && addgroup --system --gid 1001 nodejs \
  && adduser --system --uid 1001 owyx

COPY --from=builder /app/package.json ./
COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/dist ./dist
COPY --from=builder /app/src/config ./dist/config
COPY --from=builder /app/fixtures ./fixtures
RUN mkdir -p uploads/avatars uploads/packs && chown -R owyx:nodejs uploads fixtures

USER owyx
EXPOSE 3001
ENV API_PORT=3001
ENV PORT=3001
HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://127.0.0.1:3001/health || exit 1
CMD ["node", "dist/server.js"]
