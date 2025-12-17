---
sidebar_position: 2
title: Database Configuration
description: Database connection and management
---

## Database Configuration

Orbis supports SQLite (standalone) and PostgreSQL (client-server) databases.

## Connection URL Format

### SQLite

```bash
ORBIS_DATABASE_URL=sqlite://./data/orbis.db
ORBIS_DATABASE_URL=sqlite:///absolute/path/to/orbis.db
ORBIS_DATABASE_URL=sqlite::memory:  # In-memory (testing)
```

### PostgreSQL

```bash
ORBIS_DATABASE_URL=postgres://user:password@host:port/database
ORBIS_DATABASE_URL=postgres://user:password@localhost:5432/orbis
ORBIS_DATABASE_URL=postgres://user:password@host/database?sslmode=require
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `ORBIS_DATABASE_URL` | Connection URL | `sqlite://./data/orbis.db` |
| `ORBIS_DATABASE_RUN_MIGRATIONS` | Auto-run migrations | `true` |
| `ORBIS_DATABASE_MAX_CONNECTIONS` | Max pool connections | `10` |
| `ORBIS_DATABASE_MIN_CONNECTIONS` | Min pool connections | `1` |
| `ORBIS_DATABASE_CONNECT_TIMEOUT` | Connection timeout (s) | `30` |
| `ORBIS_DATABASE_IDLE_TIMEOUT` | Idle connection timeout (s) | `600` |

## Configuration File

```toml
[database]
url = "postgres://user:password@localhost:5432/orbis"
run_migrations = true
max_connections = 20
min_connections = 5
connect_timeout = 30
idle_timeout = 600
```

## SQLite Configuration

### File-Based

```bash
ORBIS_DATABASE_URL=sqlite://./data/orbis.db
```

Creates database file if not exists.

### In-Memory

```bash
ORBIS_DATABASE_URL=sqlite::memory:
```

For testing only - data lost on restart.

### SQLite Options

```bash
# With options
ORBIS_DATABASE_URL=sqlite://./data/orbis.db?mode=rwc
```

| Option | Values | Description |
|--------|--------|-------------|
| `mode` | `rwc`, `ro`, `rw` | Read-write-create, read-only, read-write |

## PostgreSQL Configuration

### Basic Connection

```bash
ORBIS_DATABASE_URL=postgres://user:password@localhost:5432/orbis
```

### With SSL

```bash
ORBIS_DATABASE_URL=postgres://user:password@host:5432/orbis?sslmode=require
```

| SSL Mode | Description |
|----------|-------------|
| `disable` | No SSL |
| `prefer` | Try SSL, fall back to plain |
| `require` | Require SSL |
| `verify-ca` | Verify CA certificate |
| `verify-full` | Verify CA and hostname |

### Connection Pool

```toml
[database]
max_connections = 20   # Maximum connections
min_connections = 5    # Minimum idle connections
connect_timeout = 30   # Connection timeout (seconds)
idle_timeout = 600     # Idle connection timeout (seconds)
```

## Migrations

### Automatic Migrations

Enabled by default:

```bash
ORBIS_DATABASE_RUN_MIGRATIONS=true
```

### Disable Auto-Migration

For production with manual migration management:

```bash
ORBIS_DATABASE_RUN_MIGRATIONS=false
```

### Migration Locations

```
crates/orbis-db/migrations/
├── postgres/
│   ├── 001_initial.sql
│   └── 002_add_users.sql
└── sqlite/
    ├── 001_initial.sql
    └── 002_add_users.sql
```

## Database Selection

### Standalone Mode

Uses SQLite by default:

```bash
ORBIS_MODE=standalone
# SQLite used automatically
```

### Client-Server Mode

Requires PostgreSQL:

```bash
ORBIS_MODE=client-server
ORBIS_DATABASE_URL=postgres://...
```

## Security Best Practices

### PostgreSQL

1. **Use strong passwords** - Avoid simple passwords
2. **Enable SSL** - Use `sslmode=require` or higher
3. **Limit permissions** - Grant minimal required privileges
4. **Network security** - Use private networks or VPN

### SQLite

1. **File permissions** - Restrict access to database file
2. **Backup regularly** - SQLite is a single file
3. **WAL mode** - Better concurrent performance

## Connection Examples

### Local Development

```bash
# SQLite
ORBIS_DATABASE_URL=sqlite://./dev.db

# PostgreSQL (local)
ORBIS_DATABASE_URL=postgres://postgres:postgres@localhost:5432/orbis_dev
```

### Docker Compose

```yaml
services:
  orbis:
    environment:
      - ORBIS_DATABASE_URL=postgres://orbis:password@db:5432/orbis
  db:
    image: postgres:16
    environment:
      - POSTGRES_USER=orbis
      - POSTGRES_PASSWORD=password
      - POSTGRES_DB=orbis
```

### Production

```bash
ORBIS_DATABASE_URL=postgres://orbis_user:${DB_PASSWORD}@db.example.com:5432/orbis_prod?sslmode=require
ORBIS_DATABASE_RUN_MIGRATIONS=false
ORBIS_DATABASE_MAX_CONNECTIONS=50
```

## Troubleshooting

### Connection Refused

```
Error: Connection refused
```

- Check database is running
- Verify host and port
- Check firewall rules

### Authentication Failed

```
Error: Authentication failed
```

- Verify username and password
- Check user permissions
- Verify database exists

### SSL Required

```
Error: SSL required
```

Add SSL mode to connection:

```bash
ORBIS_DATABASE_URL=postgres://...?sslmode=require
```

### Pool Exhausted

```
Error: Pool exhausted
```

Increase max connections:

```bash
ORBIS_DATABASE_MAX_CONNECTIONS=50
```
