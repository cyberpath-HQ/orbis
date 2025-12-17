---
sidebar_position: 1
title: Deployment Overview
description: Deploying Orbis applications
---

## Deployment Overview

Orbis supports two deployment modes: Standalone and Client-Server.

## Deployment Modes

### Standalone Mode

Single-user desktop application:

- **Database:** SQLite (local file)
- **Users:** Single user
- **Platform:** Windows, macOS, Linux
- **Distribution:** Desktop installer

```bash
ORBIS_MODE=standalone
```

### Client-Server Mode

Multi-user web deployment:

- **Database:** PostgreSQL
- **Users:** Multiple concurrent users
- **Platform:** Server or container
- **Access:** Web browser

```bash
ORBIS_MODE=client-server
```

## Deployment Options

| Option | Best For | Complexity |
|--------|----------|------------|
| [Standalone](./standalone) | Individual users | Low |
| [Client-Server](./client-server) | Teams, organizations | Medium |
| [Docker](./docker) | Container environments | Medium |

## Quick Start

### Standalone

```bash
# Download release for your platform
# Run installer or executable
./orbis
```

### Client-Server

```bash
# Set environment
export ORBIS_MODE=client-server
export ORBIS_DATABASE_URL=postgres://user:pass@host:5432/orbis
export ORBIS_JWT_SECRET=your-secure-secret

# Run server
./orbis-server
```

### Docker

```bash
docker run -d \
  -e ORBIS_MODE=client-server \
  -e ORBIS_DATABASE_URL=postgres://user:pass@host:5432/orbis \
  -e ORBIS_JWT_SECRET=your-secure-secret \
  -p 8080:8080 \
  orbis/orbis:latest
```

## System Requirements

### Standalone

| Resource | Minimum | Recommended |
|----------|---------|-------------|
| CPU | 2 cores | 4 cores |
| RAM | 2 GB | 4 GB |
| Disk | 500 MB | 1 GB |
| OS | Windows 10+, macOS 12+, Linux (glibc 2.31+) | - |

### Client-Server

| Resource | Minimum | Recommended |
|----------|---------|-------------|
| CPU | 2 cores | 4+ cores |
| RAM | 2 GB | 8 GB |
| Disk | 1 GB | 10 GB+ |
| Database | PostgreSQL 14+ | PostgreSQL 16 |

## Plugin Deployment

### Plugin Directory

```bash
# Default location
./plugins/

# Custom location
ORBIS_PLUGINS_DIR=/path/to/plugins
```

### Plugin Structure

```
plugins/
├── my-plugin/
│   ├── manifest.json
│   └── plugin.wasm
└── another-plugin/
    ├── manifest.json
    └── plugin.wasm
```

## Production Checklist

### Security

- [ ] Set strong `ORBIS_JWT_SECRET`
- [ ] Enable TLS/HTTPS
- [ ] Configure security headers
- [ ] Enable rate limiting
- [ ] Set proper file permissions

### Database

- [ ] Use PostgreSQL for client-server
- [ ] Configure connection pooling
- [ ] Set up backups
- [ ] Disable auto-migrations in production

### Monitoring

- [ ] Configure logging
- [ ] Set up health checks
- [ ] Monitor resource usage
- [ ] Set up alerts

### High Availability

- [ ] Use load balancer
- [ ] Configure replicas
- [ ] Set up database failover
- [ ] Plan disaster recovery

## Environment Configuration

### Development

```bash
ORBIS_MODE=standalone
ORBIS_LOG_LEVEL=debug
RUST_LOG=orbis=debug
```

### Staging

```bash
ORBIS_MODE=client-server
ORBIS_LOG_LEVEL=info
ORBIS_DATABASE_URL=postgres://staging-db/orbis
```

### Production

```bash
ORBIS_MODE=client-server
ORBIS_LOG_LEVEL=warn
ORBIS_DATABASE_URL=postgres://prod-db/orbis
ORBIS_DATABASE_RUN_MIGRATIONS=false
ORBIS_TLS_ENABLED=true
```

## Upgrade Process

### 1. Backup

```bash
# Backup database
pg_dump orbis > backup.sql

# Backup plugins
cp -r plugins/ plugins-backup/
```

### 2. Deploy New Version

```bash
# Stop current version
systemctl stop orbis

# Deploy new binary
cp orbis-new /usr/local/bin/orbis

# Start new version
systemctl start orbis
```

### 3. Verify

```bash
# Check health
curl http://localhost:8080/health

# Check logs
journalctl -u orbis -f
```

### 4. Rollback (if needed)

```bash
# Stop new version
systemctl stop orbis

# Restore old binary
cp orbis-old /usr/local/bin/orbis

# Restore database if needed
psql orbis < backup.sql

# Start old version
systemctl start orbis
```

## See Also

- [Standalone Deployment](./standalone) - Desktop application deployment
- [Client-Server Deployment](./client-server) - Server deployment guide
- [Docker Deployment](./docker) - Container deployment
