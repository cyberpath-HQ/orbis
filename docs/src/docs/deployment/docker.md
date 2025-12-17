---
sidebar_position: 4
title: Docker Deployment
description: Container-based deployment
---

## Docker Deployment

Deploy Orbis using Docker containers.

## Quick Start

### Single Container

```bash
docker run -d \
  --name orbis \
  -e ORBIS_MODE=client-server \
  -e ORBIS_DATABASE_URL=postgres://user:pass@host:5432/orbis \
  -e ORBIS_JWT_SECRET=your-secure-secret \
  -p 8080:8080 \
  orbis/orbis:latest
```

### With Docker Compose

```yaml
version: '3.8'

services:
  orbis:
    image: orbis/orbis:latest
    ports:
      - "8080:8080"
    environment:
      - ORBIS_MODE=client-server
      - ORBIS_DATABASE_URL=postgres://orbis:password@db:5432/orbis
      - ORBIS_JWT_SECRET=${JWT_SECRET}
    depends_on:
      db:
        condition: service_healthy
    volumes:
      - ./plugins:/app/plugins
    restart: unless-stopped

  db:
    image: postgres:16-alpine
    environment:
      - POSTGRES_USER=orbis
      - POSTGRES_PASSWORD=password
      - POSTGRES_DB=orbis
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U orbis"]
      interval: 5s
      timeout: 5s
      retries: 5

volumes:
  postgres_data:
```

## Building Docker Image

### Dockerfile

```dockerfile
# Build stage
FROM rust:1.75-bookworm AS builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build dependencies (cached layer)
RUN cargo build --release --package orbis-server

# Copy source
COPY . .

# Build application
RUN cargo build --release --package orbis-server

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary
COPY --from=builder /app/target/release/orbis-server /app/

# Copy static files
COPY --from=builder /app/orbis/dist /app/static

# Create non-root user
RUN useradd -r -s /bin/false orbis
USER orbis

EXPOSE 8080

CMD ["./orbis-server"]
```

### Build Command

```bash
docker build -t orbis/orbis:latest .
```

## Docker Compose Configurations

### Development

```yaml
version: '3.8'

services:
  orbis:
    build: .
    ports:
      - "8080:8080"
    environment:
      - ORBIS_MODE=client-server
      - ORBIS_DATABASE_URL=postgres://orbis:password@db:5432/orbis
      - ORBIS_JWT_SECRET=dev-secret
      - ORBIS_LOG_LEVEL=debug
      - RUST_LOG=orbis=debug
    volumes:
      - ./plugins:/app/plugins
      - ./orbis/dist:/app/static
    depends_on:
      - db

  db:
    image: postgres:16-alpine
    ports:
      - "5432:5432"
    environment:
      - POSTGRES_USER=orbis
      - POSTGRES_PASSWORD=password
      - POSTGRES_DB=orbis
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  postgres_data:
```

### Production

```yaml
version: '3.8'

services:
  orbis:
    image: orbis/orbis:${VERSION:-latest}
    deploy:
      replicas: 3
      resources:
        limits:
          cpus: '2'
          memory: 2G
        reservations:
          cpus: '0.5'
          memory: 512M
    environment:
      - ORBIS_MODE=client-server
      - ORBIS_DATABASE_URL=postgres://orbis:${DB_PASSWORD}@db:5432/orbis
      - ORBIS_JWT_SECRET=${JWT_SECRET}
      - ORBIS_LOG_LEVEL=warn
      - ORBIS_DATABASE_RUN_MIGRATIONS=false
    volumes:
      - plugins:/app/plugins:ro
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
    restart: unless-stopped

  db:
    image: postgres:16-alpine
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 4G
    environment:
      - POSTGRES_USER=orbis
      - POSTGRES_PASSWORD=${DB_PASSWORD}
      - POSTGRES_DB=orbis
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U orbis"]
      interval: 10s
      timeout: 5s
      retries: 5
    restart: unless-stopped

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./certs:/etc/nginx/certs:ro
    depends_on:
      - orbis
    restart: unless-stopped

volumes:
  postgres_data:
  plugins:
```

### With Traefik

```yaml
version: '3.8'

services:
  traefik:
    image: traefik:v3.0
    command:
      - "--api.insecure=true"
      - "--providers.docker=true"
      - "--entrypoints.web.address=:80"
      - "--entrypoints.websecure.address=:443"
      - "--certificatesresolvers.letsencrypt.acme.httpchallenge=true"
      - "--certificatesresolvers.letsencrypt.acme.email=admin@example.com"
      - "--certificatesresolvers.letsencrypt.acme.storage=/letsencrypt/acme.json"
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro
      - letsencrypt:/letsencrypt

  orbis:
    image: orbis/orbis:latest
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.orbis.rule=Host(`orbis.example.com`)"
      - "traefik.http.routers.orbis.entrypoints=websecure"
      - "traefik.http.routers.orbis.tls.certresolver=letsencrypt"
    environment:
      - ORBIS_MODE=client-server
      - ORBIS_DATABASE_URL=postgres://orbis:${DB_PASSWORD}@db:5432/orbis
      - ORBIS_JWT_SECRET=${JWT_SECRET}

  db:
    image: postgres:16-alpine
    environment:
      - POSTGRES_USER=orbis
      - POSTGRES_PASSWORD=${DB_PASSWORD}
      - POSTGRES_DB=orbis
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  postgres_data:
  letsencrypt:
```

## Kubernetes Deployment

### Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: orbis
spec:
  replicas: 3
  selector:
    matchLabels:
      app: orbis
  template:
    metadata:
      labels:
        app: orbis
    spec:
      containers:
      - name: orbis
        image: orbis/orbis:latest
        ports:
        - containerPort: 8080
        env:
        - name: ORBIS_MODE
          value: "client-server"
        - name: ORBIS_DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: orbis-secrets
              key: database-url
        - name: ORBIS_JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: orbis-secrets
              key: jwt-secret
        resources:
          limits:
            cpu: "2"
            memory: "2Gi"
          requests:
            cpu: "500m"
            memory: "512Mi"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
```

### Service

```yaml
apiVersion: v1
kind: Service
metadata:
  name: orbis
spec:
  selector:
    app: orbis
  ports:
  - port: 80
    targetPort: 8080
  type: ClusterIP
```

### Ingress

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: orbis
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
spec:
  tls:
  - hosts:
    - orbis.example.com
    secretName: orbis-tls
  rules:
  - host: orbis.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: orbis
            port:
              number: 80
```

## Environment Variables

### Required

| Variable | Description |
|----------|-------------|
| `ORBIS_MODE` | `client-server` |
| `ORBIS_DATABASE_URL` | PostgreSQL connection |
| `ORBIS_JWT_SECRET` | JWT signing secret |

### Optional

| Variable | Default | Description |
|----------|---------|-------------|
| `ORBIS_HOST` | `0.0.0.0` | Bind address |
| `ORBIS_PORT` | `8080` | Server port |
| `ORBIS_LOG_LEVEL` | `info` | Log level |

## Health Checks

```yaml
healthcheck:
  test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
  interval: 30s
  timeout: 10s
  retries: 3
  start_period: 40s
```

## Volumes

### Plugins

```yaml
volumes:
  - ./plugins:/app/plugins:ro
```

### Database

```yaml
volumes:
  - postgres_data:/var/lib/postgresql/data
```

## Networking

### Internal Network

```yaml
networks:
  orbis-net:
    driver: bridge

services:
  orbis:
    networks:
      - orbis-net
  db:
    networks:
      - orbis-net
```

## Secrets Management

### Docker Secrets

```yaml
secrets:
  jwt_secret:
    external: true
  db_password:
    external: true

services:
  orbis:
    secrets:
      - jwt_secret
      - db_password
    environment:
      - ORBIS_JWT_SECRET_FILE=/run/secrets/jwt_secret
```

### Environment File

```bash
# .env (not committed)
DB_PASSWORD=secure-password
JWT_SECRET=your-jwt-secret
```

```yaml
services:
  orbis:
    env_file:
      - .env
```

## Logging

### Docker Logs

```bash
docker logs orbis -f
docker-compose logs -f orbis
```

### Structured Logging

```yaml
services:
  orbis:
    logging:
      driver: json-file
      options:
        max-size: "10m"
        max-file: "3"
```

## Troubleshooting

### Container Won't Start

```bash
# Check logs
docker logs orbis

# Check environment
docker exec orbis env

# Interactive shell
docker exec -it orbis /bin/sh
```

### Database Connection

```bash
# Test from container
docker exec orbis pg_isready -h db -U orbis

# Check network
docker network inspect orbis_default
```

### Memory Issues

```yaml
deploy:
  resources:
    limits:
      memory: 4G
```
