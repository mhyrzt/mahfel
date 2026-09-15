FROM oven/bun:1.4.2-alpine AS dependencies
WORKDIR /app
COPY frontend/package.json frontend/bun.lock ./
RUN bun install --frozen-lockfile

FROM dependencies AS builder
WORKDIR /app
COPY frontend/ ./
ARG NEXT_PUBLIC_MAHFEL_API_URL=https://api.mahfel.localhost
ENV NEXT_PUBLIC_MAHFEL_API_URL=$NEXT_PUBLIC_MAHFEL_API_URL
RUN bun run build

FROM oven/bun:1.4.2-alpine AS runtime
WORKDIR /app
ENV NODE_ENV=production \
    HOSTNAME=0.0.0.0 \
    PORT=3000
RUN addgroup -S mahfel && adduser -S -G mahfel -u 10001 mahfel
COPY --from=builder --chown=mahfel:mahfel /app/public ./public
COPY --from=builder --chown=mahfel:mahfel /app/.next/standalone ./
COPY --from=builder --chown=mahfel:mahfel /app/.next/static ./.next/static
USER mahfel
EXPOSE 3000
HEALTHCHECK --interval=10s --timeout=3s --start-period=20s --retries=5 \
  CMD wget -qO- http://127.0.0.1:3000/api/health >/dev/null || exit 1
CMD ["bun", "server.js"]
