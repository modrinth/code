# ==============================================
# OWYX Frontend — Next.js Dockerfile
# Multi-stage build для production
# ==============================================

# ---------- Stage 1: deps ----------
FROM node:22-alpine AS deps
WORKDIR /app

# Копируем package файлы
COPY package.json package-lock.json ./

# Устанавливаем зависимости (только production-нужные для build)
RUN npm ci

# ---------- Stage 2: builder ----------
FROM node:22-alpine AS builder
WORKDIR /app

# Копируем зависимости из предыдущего stage
COPY --from=deps /app/node_modules ./node_modules

# Копируем весь исходник
COPY . .

# Build-time env (prefer same-origin /api on owyx.site — leave public API URL empty)
ARG NEXT_PUBLIC_API_URL=
ARG NEXT_PUBLIC_DIRECTUS_URL=
ARG NEXT_PUBLIC_SOCKET_URL=
ARG NEXT_PUBLIC_TURNSTILE_SITE_KEY=
ARG BACKEND_URL=http://backend:3001

ENV NEXT_PUBLIC_API_URL=$NEXT_PUBLIC_API_URL
ENV NEXT_PUBLIC_DIRECTUS_URL=$NEXT_PUBLIC_DIRECTUS_URL
ENV NEXT_PUBLIC_SOCKET_URL=$NEXT_PUBLIC_SOCKET_URL
ENV NEXT_PUBLIC_TURNSTILE_SITE_KEY=$NEXT_PUBLIC_TURNSTILE_SITE_KEY
ENV BACKEND_URL=$BACKEND_URL

# Отключаем телеметрию Next.js
ENV NEXT_TELEMETRY_DISABLED=1

# Собираем приложение (standalone output)
RUN npm run build

# ---------- Stage 3: runner ----------
FROM node:22-alpine AS runner
WORKDIR /app

ENV NODE_ENV=production
ENV NEXT_TELEMETRY_DISABLED=1
ENV TZ=Europe/Moscow

# Создаём непривилегированного пользователя
RUN addgroup --system --gid 1001 nodejs \
  && adduser --system --uid 1001 nextjs

# Копируем статику и standalone сборку
COPY --from=builder /app/public ./public

# standalone папка содержит всё необходимое для запуска (без node_modules)
COPY --from=builder --chown=nextjs:nodejs /app/.next/standalone ./
COPY --from=builder --chown=nextjs:nodejs /app/.next/static ./.next/static

USER nextjs

EXPOSE 3000

ENV PORT=3000
ENV HOSTNAME=0.0.0.0

# Запускаем через standalone server.js (Next.js создаёт его автоматически)
CMD ["node", "server.js"]
