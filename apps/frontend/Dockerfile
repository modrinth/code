# syntax=docker/dockerfile:1

FROM node:24-slim

LABEL org.opencontainers.image.source=https://github.com/modrinth/code
LABEL org.opencontainers.image.title=frontend
LABEL org.opencontainers.image.description="Modrinth website"
LABEL org.opencontainers.image.licenses=AGPL-3.0-only

RUN apt-get update \
	&& apt-get install -y --no-install-recommends dumb-init \
	&& rm -rf /var/lib/apt/lists/*

ENV NODE_ENV=production
ENV HOST=0.0.0.0
ENV PORT=3000

WORKDIR /app

# Nitro bundles every runtime dependency into .output, so no node_modules is needed.
COPY --chown=node:node .output ./.output

USER node
EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --retries=3 \
	CMD node -e "fetch('http://127.0.0.1:'+process.env.PORT+'/robots.txt').then(r=>process.exit(r.ok?0:1),()=>process.exit(1))"

ENTRYPOINT ["dumb-init", "--"]
CMD ["node", "/app/.output/server/index.mjs"]
