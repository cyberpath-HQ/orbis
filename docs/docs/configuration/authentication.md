---
sidebar_position: 3
title: Authentication
description: Authentication and security configuration
---

## Authentication

Orbis uses JWT (JSON Web Tokens) for authentication and Argon2 for password hashing.

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `ORBIS_JWT_SECRET` | JWT signing secret | Generated |
| `ORBIS_JWT_EXPIRY` | Token expiry time | `24h` |
| `ORBIS_SESSION_DURATION` | Session duration | `24h` |
| `ORBIS_REFRESH_TOKEN_EXPIRY` | Refresh token expiry | `7d` |
| `ORBIS_PASSWORD_MIN_LENGTH` | Minimum password length | `8` |

## Configuration File

```toml
[auth]
jwt_secret = "your-secure-secret-key-at-least-32-characters"
jwt_expiry = "24h"
session_duration = "24h"
refresh_token_expiry = "7d"

[auth.password]
min_length = 8
require_uppercase = true
require_lowercase = true
require_number = true
require_special = false
```

## JWT Configuration

### Secret Key

**Required in production** - Set a secure random secret:

```bash
# Generate a secure secret
openssl rand -base64 32

# Set in environment
ORBIS_JWT_SECRET=your-generated-secret-here
```

**Important:** Use at least 32 characters for the secret.

### Token Expiry

```bash
ORBIS_JWT_EXPIRY=24h    # 24 hours
ORBIS_JWT_EXPIRY=1d     # 1 day
ORBIS_JWT_EXPIRY=7d     # 7 days
ORBIS_JWT_EXPIRY=30m    # 30 minutes
```

### Token Structure

JWT tokens contain:

```json
{
  "sub": "user_id",
  "name": "username",
  "role": "user",
  "exp": 1234567890,
  "iat": 1234567890
}
```

## Session Management

### Session Duration

```bash
ORBIS_SESSION_DURATION=24h
```

### Refresh Tokens

```bash
ORBIS_REFRESH_TOKEN_EXPIRY=7d
```

Refresh tokens allow obtaining new access tokens without re-authentication.

## Password Configuration

### Minimum Length

```bash
ORBIS_PASSWORD_MIN_LENGTH=8
```

### Password Requirements

```toml
[auth.password]
min_length = 8
require_uppercase = true
require_lowercase = true
require_number = true
require_special = false
```

### Password Hashing

Orbis uses Argon2id for password hashing:

- Memory cost: 65536 KB
- Time cost: 3 iterations
- Parallelism: 4 threads

## User Roles

### Built-in Roles

| Role | Description |
|------|-------------|
| `admin` | Full access |
| `user` | Standard access |
| `guest` | Limited access |

### Role Permissions

```toml
[auth.roles.admin]
permissions = ["*"]

[auth.roles.user]
permissions = ["read", "write"]

[auth.roles.guest]
permissions = ["read"]
```

## Login Flow

### 1. Login Request

```json
POST /api/auth/login
{
  "username": "user@example.com",
  "password": "password123"
}
```

### 2. Success Response

```json
{
  "access_token": "eyJhbGc...",
  "refresh_token": "eyJhbGc...",
  "expires_in": 86400,
  "user": {
    "id": "uuid",
    "username": "user@example.com",
    "role": "user"
  }
}
```

### 3. Using Access Token

```bash
Authorization: Bearer eyJhbGc...
```

### 4. Refresh Token

```json
POST /api/auth/refresh
{
  "refresh_token": "eyJhbGc..."
}
```

## Tauri Integration

In Tauri desktop mode, authentication uses commands:

```typescript
import { invoke } from '@tauri-apps/api/core';

// Login
const session = await invoke('login', { 
  username: 'user@example.com', 
  password: 'password123' 
});

// Check auth status
const isAuth = await invoke('is_authenticated');

// Logout
await invoke('logout');
```

## Security Headers

Enable in production:

```toml
[security]
strict_transport_security = true
content_security_policy = true
x_frame_options = "DENY"
x_content_type_options = "nosniff"
x_xss_protection = true
```

## Session Storage

### Standalone Mode

Sessions stored in SQLite:

```sql
CREATE TABLE sessions (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  token TEXT NOT NULL,
  expires_at DATETIME NOT NULL,
  created_at DATETIME NOT NULL
);
```

### Client-Server Mode

Sessions stored in PostgreSQL with same schema.

## Rate Limiting

Protect authentication endpoints:

```toml
[rate_limit]
enabled = true
login_attempts = 5           # Max attempts
login_window = "15m"         # Time window
lockout_duration = "30m"     # Lockout time
```

## Security Best Practices

### JWT Secret

1. **Never commit to version control**
2. **Use environment variables**
3. **Rotate periodically**
4. **Use at least 256 bits (32 characters)**

### Passwords

1. **Enforce minimum length**
2. **Use complexity requirements**
3. **Never store plaintext**
4. **Use secure hashing (Argon2)**

### Sessions

1. **Set reasonable expiry times**
2. **Invalidate on logout**
3. **Invalidate on password change**
4. **Use secure cookies (HTTPS only)**

### General

1. **Enable HTTPS in production**
2. **Use security headers**
3. **Enable rate limiting**
4. **Log authentication events**

## Troubleshooting

### Invalid Token

```
Error: Invalid token
```

- Check token expiry
- Verify JWT secret matches
- Ensure token not tampered

### Token Expired

```
Error: Token expired
```

- Use refresh token to get new access token
- Re-authenticate if refresh token also expired

### Authentication Required

```
Error: Authentication required
```

- Include Authorization header
- Check token format: `Bearer <token>`
