---
sidebar_position: 2
title: Server Configuration
description: Configure Orbis server networking and HTTP options
---

## Overview

Server configuration controls how Orbis binds to network interfaces, handles HTTP requests, and manages server-side features.

## Environment Variables

### Basic Server Settings

| Variable | Description | Default |
|----------|-------------|---------|
| `ORBIS_MODE` | Operating mode | `standalone` |
| `ORBIS_HOST` | Bind address | `127.0.0.1` |
| `ORBIS_PORT` | Listen port | `8080` |

### Mode Options

```bash
# Standalone mode (default) - single user, SQLite
ORBIS_MODE=standalone

# Client-server mode - multi-user, PostgreSQL
ORBIS_MODE=client-server
```

## Host Binding

### Local Development

```bash
# Only accessible from localhost
ORBIS_HOST=127.0.0.1
ORBIS_PORT=8080
```

### Network Access

```bash
# Accessible from any interface
ORBIS_HOST=0.0.0.0
ORBIS_PORT=8080
```

### Specific Interface

```bash
# Bind to specific IP
ORBIS_HOST=192.168.1.100
ORBIS_PORT=8080
```

## Port Configuration

```bash
# Default HTTP port
ORBIS_PORT=8080

# Standard HTTP (requires root/admin)
ORBIS_PORT=80

# Custom port
ORBIS_PORT=3000
```

## CORS Configuration

Cross-Origin Resource Sharing settings for API access:

```bash
# Allow specific origins
ORBIS_CORS_ORIGINS=https://app.example.com,https://admin.example.com

# Allow all origins (development only)
ORBIS_CORS_ORIGINS=*

# Allowed methods (default: GET,POST,PUT,DELETE,OPTIONS)
ORBIS_CORS_METHODS=GET,POST,PUT,DELETE

# Allowed headers
ORBIS_CORS_HEADERS=Content-Type,Authorization
```

## Request Limits

```bash
# Maximum request body size (default: 10MB)
ORBIS_MAX_BODY_SIZE=10485760

# Request timeout in seconds
ORBIS_REQUEST_TIMEOUT=30

# Maximum concurrent connections
ORBIS_MAX_CONNECTIONS=1000
```

## Rate Limiting

```bash
# Enable rate limiting
ORBIS_RATE_LIMIT_ENABLED=true

# Requests per window
ORBIS_RATE_LIMIT_REQUESTS=100

# Window duration in seconds
ORBIS_RATE_LIMIT_WINDOW=60

# Rate limit by IP or user
ORBIS_RATE_LIMIT_BY=ip
```

## Configuration File

You can also use a TOML configuration file:

```toml
# orbis.toml
[server]
host = "0.0.0.0"
port = 8080
mode = "client-server"

[server.cors]
origins = ["https://app.example.com"]
methods = ["GET", "POST", "PUT", "DELETE"]
credentials = true

[server.limits]
max_body_size = 10485760
request_timeout = 30
max_connections = 1000

[server.rate_limit]
enabled = true
requests = 100
window = 60
by = "ip"
```

## Production Recommendations

### Security Hardening

```bash
# Production server settings
ORBIS_MODE=client-server
ORBIS_HOST=0.0.0.0
ORBIS_PORT=8080

# Strict CORS
ORBIS_CORS_ORIGINS=https://app.yourdomain.com

# Rate limiting
ORBIS_RATE_LIMIT_ENABLED=true
ORBIS_RATE_LIMIT_REQUESTS=100
ORBIS_RATE_LIMIT_WINDOW=60
```

### Behind Reverse Proxy

When running behind nginx or similar:

```bash
# Trust proxy headers
ORBIS_TRUST_PROXY=true

# Get real IP from header
ORBIS_REAL_IP_HEADER=X-Forwarded-For

# Bind to localhost only (proxy handles external)
ORBIS_HOST=127.0.0.1
```

### nginx Configuration Example

```nginx
upstream orbis {
    server 127.0.0.1:8080;
    keepalive 32;
}

server {
    listen 443 ssl http2;
    server_name app.example.com;

    ssl_certificate /etc/ssl/certs/app.crt;
    ssl_certificate_key /etc/ssl/private/app.key;

    location / {
        proxy_pass http://orbis;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Connection "";
        
        # WebSocket support
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
```

## Logging

Server logging configuration:

```bash
# General log level
ORBIS_LOG_LEVEL=info

# Rust-specific logging
RUST_LOG=orbis=info,orbis_server=debug,tower_http=debug

# Access log format
ORBIS_ACCESS_LOG=true
ORBIS_ACCESS_LOG_FORMAT=combined
```

## Health Checks

Built-in health endpoints for monitoring:

```bash
# Health check endpoint
GET /health

# Readiness check (includes database)
GET /ready

# Liveness check (basic)
GET /live
```

### Response Example

```json
{
  "status": "healthy",
  "version": "1.0.0",
  "uptime": 3600,
  "database": "connected",
  "plugins": {
    "loaded": 5,
    "active": 5
  }
}
```

## See Also

- [Database Configuration](./database) - Database connection settings
- [Authentication](./authentication) - JWT and user authentication
- [TLS Security](./tls-security) - HTTPS and SSL configuration
- [Deployment Overview](../deployment/overview) - Production deployment guide
