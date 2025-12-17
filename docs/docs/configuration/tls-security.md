---
sidebar_position: 4
title: TLS Security
description: HTTPS and TLS configuration
---

## TLS Security

Configure HTTPS for secure communication in client-server deployments.

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `ORBIS_TLS_ENABLED` | Enable TLS | `false` |
| `ORBIS_TLS_CERT_PATH` | Certificate file path | - |
| `ORBIS_TLS_KEY_PATH` | Private key file path | - |
| `ORBIS_TLS_CA_PATH` | CA certificate path | - |

## Configuration File

```toml
[tls]
enabled = true
cert_path = "/etc/orbis/certs/server.crt"
key_path = "/etc/orbis/certs/server.key"
ca_path = "/etc/orbis/certs/ca.crt"      # Optional
min_version = "1.2"
```

## Certificate Setup

### Using Let's Encrypt

1. Install Certbot:

```bash
sudo apt install certbot
```

2. Obtain certificate:

```bash
sudo certbot certonly --standalone -d yourdomain.com
```

3. Configure Orbis:

```bash
ORBIS_TLS_ENABLED=true
ORBIS_TLS_CERT_PATH=/etc/letsencrypt/live/yourdomain.com/fullchain.pem
ORBIS_TLS_KEY_PATH=/etc/letsencrypt/live/yourdomain.com/privkey.pem
```

### Self-Signed Certificates

For development/testing only:

```bash
# Generate private key
openssl genrsa -out server.key 2048

# Generate self-signed certificate
openssl req -new -x509 -sha256 \
  -key server.key \
  -out server.crt \
  -days 365 \
  -subj "/CN=localhost"
```

### Certificate Requirements

- **Format:** PEM
- **Key:** RSA 2048+ or ECDSA P-256+
- **Chain:** Include intermediate certificates

## TLS Versions

```toml
[tls]
min_version = "1.2"  # Minimum TLS version
```

| Version | Recommendation |
|---------|----------------|
| `1.0` | Not recommended (deprecated) |
| `1.1` | Not recommended (deprecated) |
| `1.2` | Recommended minimum |
| `1.3` | Best security |

## Cipher Suites

```toml
[tls]
cipher_suites = [
  "TLS_AES_128_GCM_SHA256",
  "TLS_AES_256_GCM_SHA384",
  "TLS_CHACHA20_POLY1305_SHA256",
  "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256",
  "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384"
]
```

## Security Headers

Enable with TLS for complete security:

```toml
[security.headers]
strict_transport_security = "max-age=31536000; includeSubDomains"
content_security_policy = "default-src 'self'"
x_frame_options = "DENY"
x_content_type_options = "nosniff"
referrer_policy = "strict-origin-when-cross-origin"
```

### HSTS

HTTP Strict Transport Security:

```toml
[security]
hsts_enabled = true
hsts_max_age = 31536000      # 1 year
hsts_include_subdomains = true
hsts_preload = false
```

## Client Certificate Authentication

For mutual TLS (mTLS):

```toml
[tls]
client_auth = "required"     # none, optional, required
ca_path = "/etc/orbis/certs/ca.crt"
```

## Docker Configuration

### With Volume Mounts

```yaml
services:
  orbis:
    environment:
      - ORBIS_TLS_ENABLED=true
      - ORBIS_TLS_CERT_PATH=/certs/server.crt
      - ORBIS_TLS_KEY_PATH=/certs/server.key
    volumes:
      - ./certs:/certs:ro
    ports:
      - "443:8080"
```

### With Secrets

```yaml
services:
  orbis:
    secrets:
      - tls_cert
      - tls_key
    environment:
      - ORBIS_TLS_CERT_PATH=/run/secrets/tls_cert
      - ORBIS_TLS_KEY_PATH=/run/secrets/tls_key

secrets:
  tls_cert:
    file: ./certs/server.crt
  tls_key:
    file: ./certs/server.key
```

## Reverse Proxy Setup

### Nginx

If using a reverse proxy, terminate TLS at the proxy:

```nginx
server {
    listen 443 ssl http2;
    server_name yourdomain.com;

    ssl_certificate /etc/letsencrypt/live/yourdomain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/yourdomain.com/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;

    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

Orbis configuration (TLS disabled, proxy handles it):

```bash
ORBIS_TLS_ENABLED=false
ORBIS_TRUST_PROXY=true
```

### Traefik

```yaml
labels:
  - "traefik.enable=true"
  - "traefik.http.routers.orbis.rule=Host(`yourdomain.com`)"
  - "traefik.http.routers.orbis.tls=true"
  - "traefik.http.routers.orbis.tls.certresolver=letsencrypt"
```

## Certificate Renewal

### Automatic (Let's Encrypt)

```bash
# Add to crontab
0 0 1 * * certbot renew --quiet && systemctl reload orbis
```

### Manual Check

```bash
# Check certificate expiry
openssl x509 -in /path/to/server.crt -noout -enddate
```

## Troubleshooting

### Certificate Not Found

```
Error: Certificate file not found
```

- Verify file paths
- Check file permissions
- Ensure files are readable by Orbis process

### Invalid Certificate

```
Error: Invalid certificate
```

- Check PEM format
- Verify certificate chain
- Ensure certificate matches key

### Certificate Expired

```
Error: Certificate has expired
```

- Renew certificate
- Check system clock

### TLS Handshake Failed

```
Error: TLS handshake failed
```

- Check TLS version compatibility
- Verify cipher suite support
- Check client configuration

## Security Checklist

- [ ] TLS 1.2 minimum version
- [ ] Valid certificate from trusted CA
- [ ] Strong cipher suites only
- [ ] HSTS enabled
- [ ] Certificate renewal automated
- [ ] Private key properly secured
- [ ] Security headers configured
