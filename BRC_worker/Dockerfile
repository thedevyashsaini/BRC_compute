# syntax=docker/dockerfile:1

ARG NODE_VERSION=20.17.0

################################################################################
# Use node image for base image for all stages.
FROM node:${NODE_VERSION}-alpine as base

# Set working directory for all build stages.
WORKDIR /usr/src/app
RUN mkdir -p /usr/src/app/dist/src && chown -R node:node /usr/src/app

# Install Git, Docker, Docker Daemon, and dependencies.
RUN apk add --no-cache git docker-cli docker-cli-buildx docker-compose \
    docker openrc shadow

# Ensure the docker group exists and matches the host's GID
ARG DOCKER_GID=984
RUN addgroup -g $DOCKER_GID docker || true && adduser node docker

# Start Docker daemon in the background
RUN dockerd &

# Set up new user session so group membership applies
USER node

# Ensure Docker daemon is running
RUN docker version || echo "Docker is not running!"

ENTRYPOINT ["sh", "-c", "exec node dist/src/index.js"]

################################################################################
# Create a stage for installing production dependencies.
FROM base as deps

# Download dependencies as a separate step to take advantage of Docker's caching.
RUN --mount=type=bind,source=package.json,target=package.json \
    --mount=type=bind,source=package-lock.json,target=package-lock.json \
    --mount=type=cache,target=/root/.npm \
    npm ci --omit=dev

################################################################################
# Create a stage for building the application.
FROM deps as build

# Install dev dependencies
RUN --mount=type=bind,source=package.json,target=package.json \
    --mount=type=bind,source=package-lock.json,target=package-lock.json \
    --mount=type=cache,target=/root/.npm \
    npm ci

# Copy source files and build
COPY . .
RUN npm run build

################################################################################
# Final runtime image
FROM base as final

# Use production node environment
ENV NODE_ENV production

# Run as non-root user
USER node

# Copy required files
COPY package.json .
COPY --from=deps /usr/src/app/node_modules ./node_modules
COPY --from=build /usr/src/app/dist ./dist

# Expose port
EXPOSE 5000

# Start the app
CMD ["npm", "start"]
