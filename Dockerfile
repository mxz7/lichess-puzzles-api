# syntax=docker/dockerfile:1.7

FROM node:24-slim AS deps
WORKDIR /app
RUN corepack enable

COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
RUN pnpm install --frozen-lockfile

FROM deps AS build
COPY tsconfig.json prisma.config.ts ./
COPY prisma ./prisma
COPY src ./src
RUN npx prisma generate
RUN pnpm run build
RUN pnpm prune --prod

FROM node:24-slim AS runtime
WORKDIR /app
RUN corepack enable && mkdir -p /app/data

ENV NODE_ENV=production
ENV PORT=3000

COPY --from=build /app/package.json ./package.json
COPY --from=build /app/node_modules ./node_modules
COPY --from=build /app/dist ./dist
COPY --from=build /app/prisma ./prisma

EXPOSE 3000
VOLUME ["/app/data"]

CMD ["node", "dist/src/index.js"]
