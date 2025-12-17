---
sidebar_position: 3
title: Client-Server Deployment
description: Multi-user server deployment
---

## Client-Server Deployment

Deploy Orbis as a multi-user web application.

## Overview

Client-server mode provides:

- PostgreSQL database
- Multi-user access
- Web-based interface
- Centralized management
- Scalable architecture

## Prerequisites

- PostgreSQL 14 or higher
- Linux server (recommended) or Windows Server
- Domain name (for HTTPS)
- SSL certificate

## Quick Start

### 1. Database Setup

```bash
# Create database
sudo -u postgres createdb orbis
sudo -u postgres createuser orbis_user -P

# Grant permissions
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE orbis TO orbis_user;"
```

### 2. Environment Configuration

```bash
export ORBIS_MODE=client-server
export ORBIS_HOST=0.0.0.0
export ORBIS_PORT=8080
export ORBIS_DATABASE_URL=postgres://orbis_user:password@localhost:5432/orbis
export ORBIS_JWT_SECRET=$(openssl rand -base64 32)
```

### 3. Run Server

```bash
./orbis-server
```

## Systemd Service

### Service File

Create `/etc/systemd/system/orbis.service`:

```ini
[Unit]
Description=Orbis Server
After=network.target postgresql.service

[Service]
Type=simple
User=orbis
Group=orbis
WorkingDirectory=/opt/orbis
ExecStart=/opt/orbis/orbis-server
Restart=always
RestartSec=10

Environment=ORBIS_MODE=client-server
Environment=ORBIS_HOST=0.0.0.0
Environment=ORBIS_PORT=8080
EnvironmentFile=/etc/orbis/orbis.env

[Install]
WantedBy=multi-user.target
```

### Environment File

Create `/etc/orbis/orbis.env`:

```bash
ORBIS_DATABASE_URL=postgres://orbis_user:password@localhost:5432/orbis
ORBIS_JWT_SECRET=your-secure-secret
ORBIS_LOG_LEVEL=info
```

### Enable Service

```bash
sudo systemctl daemon-reload
sudo systemctl enable orbis
sudo systemctl start orbis
```

## Nginx Reverse Proxy

### Configuration

Create `/etc/nginx/sites-available/orbis`:

```nginx
server {
    listen 80;
    server_name orbis.example.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name orbis.example.com;

    ssl_certificate /etc/letsencrypt/live/orbis.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/orbis.example.com/privkey.pem;
    
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256;
    ssl_prefer_server_ciphers off;

    # Security headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Frame-Options DENY always;
    add_header X-Content-Type-Options nosniff always;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    # Static files caching
    location /assets/ {
        proxy_pass http://127.0.0.1:8080;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
}
```

### Enable Site

```bash
sudo ln -s /etc/nginx/sites-available/orbis /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

## SSL Certificate

### Let's Encrypt

```bash
sudo apt install certbot python3-certbot-nginx
sudo certbot --nginx -d orbis.example.com
```

### Auto-Renewal

```bash
sudo systemctl enable certbot.timer
```

## Database Configuration

### Connection Pool

```bash
ORBIS_DATABASE_MAX_CONNECTIONS=50
ORBIS_DATABASE_MIN_CONNECTIONS=5
ORBIS_DATABASE_CONNECT_TIMEOUT=30
ORBIS_DATABASE_IDLE_TIMEOUT=600
```

### Migrations

Disable auto-migrations in production:

```bash
ORBIS_DATABASE_RUN_MIGRATIONS=false
```

Run migrations manually:

```bash
./orbis-server migrate
```

## Scaling

### Horizontal Scaling

Run multiple instances behind a load balancer:

```nginx
upstream orbis_backend {
    least_conn;
    server 127.0.0.1:8080;
    server 127.0.0.1:8081;
    server 127.0.0.1:8082;
}

server {
    location / {
        proxy_pass http://orbis_backend;
    }
}
```

### Database Read Replicas

```bash
# Primary database
ORBIS_DATABASE_URL=postgres://user:pass@primary:5432/orbis

# Read replica (for read-heavy operations)
ORBIS_DATABASE_READ_URL=postgres://user:pass@replica:5432/orbis
```

## Monitoring

### Health Check

```bash
curl http://localhost:8080/health
```

Response:

```json
{
  "status": "healthy",
  "version": "1.0.0",
  "database": "connected",
  "uptime": 3600
}
```

### Logs

```bash
# View logs
journalctl -u orbis -f

# Last 100 lines
journalctl -u orbis -n 100
```

### Metrics

Enable Prometheus metrics:

```bash
ORBIS_METRICS_ENABLED=true
ORBIS_METRICS_PORT=9090
```

Scrape endpoint: `http://localhost:9090/metrics`

## Backup

### Database Backup

```bash
# Full backup
pg_dump -U orbis_user orbis > /backups/orbis_$(date +%Y%m%d).sql

# Compressed
pg_dump -U orbis_user orbis | gzip > /backups/orbis_$(date +%Y%m%d).sql.gz
```

### Automated Backups

```bash
# Crontab entry
0 2 * * * /usr/local/bin/backup-orbis.sh
```

### Backup Script

```bash
#!/bin/bash
BACKUP_DIR=/backups
RETENTION_DAYS=30

# Create backup
pg_dump -U orbis_user orbis | gzip > $BACKUP_DIR/orbis_$(date +%Y%m%d).sql.gz

# Remove old backups
find $BACKUP_DIR -name "orbis_*.sql.gz" -mtime +$RETENTION_DAYS -delete
```

## Security Hardening

### Firewall

```bash
# Allow only necessary ports
sudo ufw allow 22/tcp    # SSH
sudo ufw allow 80/tcp    # HTTP
sudo ufw allow 443/tcp   # HTTPS
sudo ufw enable
```

### File Permissions

```bash
chmod 600 /etc/orbis/orbis.env
chown orbis:orbis /opt/orbis -R
chmod 755 /opt/orbis/orbis-server
```

### Database Security

```sql
-- Limit permissions
REVOKE ALL ON DATABASE orbis FROM PUBLIC;
GRANT CONNECT ON DATABASE orbis TO orbis_user;
GRANT USAGE ON SCHEMA public TO orbis_user;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO orbis_user;
```

## Troubleshooting

### Connection Refused

```bash
# Check service status
systemctl status orbis

# Check port binding
ss -tlnp | grep 8080

# Check logs
journalctl -u orbis -n 50
```

### Database Connection Failed

```bash
# Test connection
psql -U orbis_user -h localhost orbis

# Check PostgreSQL status
systemctl status postgresql
```

### High Memory Usage

```bash
# Check process
ps aux | grep orbis

# Adjust connection pool
ORBIS_DATABASE_MAX_CONNECTIONS=20
```
