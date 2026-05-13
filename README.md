# LLM Workbench

Single-user learning workspace prototype with a Rust backend and Vue frontend.

This repository is currently at implementation Batch 1. It includes the
development baseline plus PostgreSQL pool initialization, migration execution,
database-aware health checks, and a small JSON settings store.

## Requirements

- Rust toolchain with Cargo.
- Node.js and npm.
- PostgreSQL for persisted settings and later product data.
- `sqlx-cli` for running migrations outside backend startup:

```cmd
cargo install sqlx-cli --no-default-features --features native-tls,postgres
```

## Configuration

Copy `.env.example` to `.env` and adjust values if needed.

```cmd
copy .env.example .env
```

When `DATABASE_URL` is set, backend startup connects to PostgreSQL and runs
migrations from `migrations/`. When it is unset, the backend still starts and
reports `database.status = "not_configured"` from `/api/health`; database-backed
API routes return the standard `database_error` response.

Set `TEST_DATABASE_URL` to run PostgreSQL-backed tests against a disposable test
database. Tests that need PostgreSQL return early when this variable is missing.

## Development

Run the backend:

```cmd
.\scripts\backend.cmd
```

Run the frontend:

```cmd
.\scripts\frontend.cmd
```

The frontend dev server proxies `/api` to `http://127.0.0.1:3000`.

Run tests:

```cmd
.\scripts\test.cmd
```

Use `npm.cmd` rather than `npm` when running frontend package commands directly.

Run migrations manually:

```cmd
.\scripts\migrate.cmd
```

## API

`GET /api/health`

Returns service status and basic app configuration:

```json
{
  "service": "llm_workbench",
  "version": "0.1.0",
  "status": "ok",
  "app": {
    "host": "127.0.0.1",
    "port": 3000,
    "app_data_dir": "./data"
  },
  "database": {
    "configured": true,
    "status": "ok"
  }
}
```

`GET /api/settings`

Returns all persisted settings:

```json
{
  "settings": {
    "editor": {
      "fontSize": 16
    }
  }
}
```

`PATCH /api/settings`

Upserts multiple arbitrary JSON settings:

```json
{
  "settings": {
    "editor": {
      "fontSize": 16
    },
    "workspace": {
      "density": "compact"
    }
  }
}
```

API errors use this shape:

```json
{
  "error": {
    "code": "not_found",
    "message": "No API route exists for `/api/missing`"
  }
}
```
