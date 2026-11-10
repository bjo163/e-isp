# Development Guide

## Prerequisites

- Rust 1.94.0 (rustfmt and clippy components enabled).
- Node.js 22.16.0 or newer within Node 22.
- npm (included with Node.js).
- Docker Engine and Docker Compose v2 for the containerized stack.

## Local environment

1. Copy `.env.example` to `.env` and replace local credentials.
2. Start PostgreSQL: `docker compose up -d postgres`.
3. In one terminal, start the API from the repository root: `cargo loco start`.
4. In another terminal, install and start the web app: `npm install --no-package-lock` then `npm run dev`.
5. Open `http://localhost:3000`. The API info endpoint is `http://localhost:5150/api/v1/system/info`.

The API process health endpoint reports process health only. It is not a dependency-readiness check and must not be interpreted as proof that database migrations or network collection are healthy.

## Docker Compose

- `docker compose up --build` starts PostgreSQL, the Loco API, and Next.js.
- `docker compose --profile collector-bootstrap up --build` also starts the collector bootstrap process. It does not poll devices or register to a control plane yet.
- `docker compose logs -f api web` follows application logs.
- `docker compose down` stops the stack; `docker compose down -v` also destroys local PostgreSQL data.

Compose publishes application and database ports on loopback only. Do not remove this restriction casually. The supplied credentials are for local development, not shared or production deployment.

## Checks

```sh
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm install --no-package-lock
npm run lint --workspace @e-isp/web
npm run typecheck --workspace @e-isp/web
npm run build --workspace @e-isp/web
```

## Current implementation boundary

The repository contains the application workspace, a basic Loco API, a Next.js operational shell, PostgreSQL Compose setup, and CI. The collector is bootstrap-only. There is no device polling, user authentication flow, tenant persistence, inventory schema, real telemetry pipeline, or vendor protocol adapter in this baseline. Those capabilities remain tracked by the linked child issues and must not be presented as active features.

## Tenant-scoped network location API

After creating an administrator with the `admin:create` task and signing in, use the JWT returned by the API (or held in the Next.js HttpOnly session) together with the tenant public UUID:

```sh
curl -H "Authorization: Bearer $EISP_TOKEN" \
  -H "X-Tenant-ID: $EISP_TENANT_PUBLIC_ID" \
  http://localhost:5150/api/v1/network/sites
```

The API resolves the authenticated user, verifies active membership and organization state, then scopes every site/region/POP query by the internal tenant ID. Do not send an internal database ID as the tenant header. Writes additionally require an owner, admin, or network_admin membership role. The current API list responses are capped at 500 rows; pagination is a remaining task.
