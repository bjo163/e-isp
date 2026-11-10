# Development Guide

## Prerequisites

- Rust 1.94.0 with `rustfmt` and `clippy`.
- Node.js 22.16.0 or newer within Node 22.
- npm (included with Node.js).
- Docker Engine and Docker Compose v2 for the containerized stack.

## Local environment

1. Copy `.env.example` to `.env` and replace all local credentials.
2. Start PostgreSQL: `docker compose up -d postgres`.
3. Apply pending migrations explicitly when starting from a clean database: `cargo loco db migrate` (development config currently also has `auto_migrate: true`).
4. In one terminal, start the Loco API from the repository root: `cargo loco start`.
5. In another terminal, install and start the web app: `npm install --no-package-lock` then `npm run dev`.
6. Open `http://localhost:3000`.

The API info endpoint is `http://localhost:5150/api/v1/system/info`; the process health endpoint is `http://localhost:5150/api/v1/system/health`.

## Create the first administrator

Public self-registration is intentionally disabled. Create the first user, organization, tenant, and owner membership using the Loco task:

```sh
cargo loco task admin:create \
  email:admin@example.com \
  name:Administrator \
  password:'replace-this-with-a-long-unique-password' \
  organization:'Example ISP' \
  tenant:Production \
  org_slug:example-isp \
  tenant_slug:production
```

Use a unique password with at least 12 characters. Avoid sharing terminal history or logs containing the password argument. After the task succeeds, sign in at `http://localhost:3000/login`.

The Next.js application exchanges credentials with the Loco API and stores the returned JWT in an HttpOnly, SameSite=Lax cookie. The browser does not receive the raw token in the login response. The current-user API returns the user's active tenant memberships.

## Tenant-aware network location API

Network location operations require:
- a valid JWT in the `Authorization: Bearer <token>` header,
- an active user account,
- an active membership for the selected tenant,
- an active tenant and organization.

Send the tenant **public UUID** in `X-Tenant-ID`; do not send the internal database ID.

Example:

```sh
curl \
  -H "Authorization: Bearer $EISP_TOKEN" \
  -H "X-Tenant-ID: $EISP_TENANT_PUBLIC_ID" \
  http://localhost:5150/api/v1/network/sites
```

Implemented routes:

| Method | Route | Permission |
|---|---|---|
| GET | `/api/v1/network/regions` | Active tenant membership |
| POST | `/api/v1/network/regions` | owner / admin / network_admin |
| GET | `/api/v1/network/sites` | Active tenant membership |
| POST | `/api/v1/network/sites` | owner / admin / network_admin |
| GET | `/api/v1/network/pops` | Active tenant membership |
| POST | `/api/v1/network/pops` | owner / admin / network_admin |

List endpoints currently return at most 500 records. Cursor pagination, update/delete operations, and the full permission catalog remain follow-up work. The current frontend shows the authenticated operator and tenant count; tenant selection and dedicated location CRUD screens are not yet implemented.

## Docker Compose

- `docker compose up --build` starts PostgreSQL, the Loco API, and Next.js.
- `docker compose --profile collector-bootstrap up --build` also starts the collector bootstrap process.
- `docker compose logs -f api web` follows application logs.
- `docker compose down` stops the stack.
- `docker compose down -v` also destroys local PostgreSQL data.

Compose publishes service ports on loopback only. Do not remove this restriction casually. The sample credentials are for local development, not a shared or production deployment. The production config requires a real `DATABASE_URL`, `JWT_SECRET`, and `HOST`. Production migrations are deliberately not automatic; run `cargo loco -e production db migrate` as a controlled deployment step before rolling out the API version that needs the new schema. Check `cargo loco -e production db status` before and after migration.

The collector is bootstrap-only: it reports that the process is alive but does not yet register with a control plane, poll devices, or send telemetry.

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

CI runs the Rust formatting/check/lint/test commands and the Next.js lint/typecheck/build commands. A PostgreSQL migration integration test is being added to CI; treat schema readiness as unverified until that test passes on the current commit.

## Current implementation boundary

Implemented in the current repository:
- Loco.rs 1.3.1 API workspace with PostgreSQL/SeaORM configuration.
- Initial identity, organization, tenant, membership, region, site, and POP migrations/entities.
- Password sign-in, JWT current-user endpoint, and administrator bootstrap task.
- Tenant-context resolution that checks active membership and organization.
- Tenant-scoped region/site/POP list and create APIs.
- Next.js sign-in flow, HttpOnly session bridge, and operational overview shell.
- Local Docker Compose stack and CI baseline.

Not yet implemented:
- Full permission catalog/custom RBAC and tenant administration UI.
- Inventory beyond regions/sites/POPs (vendors, devices, interfaces, credentials, discovery).
- Collector registration/heartbeat, polling scheduler, SNMP/ICMP/gNMI/NETCONF/SSH/BGP adapters.
- NATS event transport, TSDB metrics ingestion/query, alert evaluation, topology, incidents.
- Subscriber/RADIUS/DHCP/BNG, FTTH optical/GIS, SLA reporting.
- Configuration capture, approval workflow, controlled network writes, verification, rollback.
- Production backup/restore drills, retention, load testing, HA, and operational runbooks.

Do not present planned or scaffold-only modules as operational features. Use [the master execution board](https://github.com/bjo163/e-isp/issues/110) for the detailed issue-by-issue delivery plan.
