# Build on the ForkLaunch hardened Node 26 image (Wolfi, glibc), the same
# Node as frontend/.nvmrc; only the static build reaches the nginx stage.
FROM public.ecr.aws/e6h0t2z8/node:26-dev@sha256:c31798fa091a730089b0bd8d6b3995ba43d444a4f114f316e686fa36c47feb74 AS build
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci
COPY . .
RUN npm run build

# Runs as a non-root user and listens on 8080.
FROM nginxinc/nginx-unprivileged:1.27.5-alpine3.21
COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=build /app/dist /usr/share/nginx/html
