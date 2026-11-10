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
  organization:'Example ISP' \
  tenant:Production \
  org_slug:example-isp \
  tenant_slug:production
```

The task reads the password from `EISP_ADMIN_PASSWORD` rather than a command-line argument. Use the silent, one-process prompt shown in [Secure initial administrator provisioning](#secure-initial-administrator-provisioning) below. Choose a unique password of at least 12 characters. After the task succeeds, sign in at `http://localhost:3000/login`.

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

List endpoints currently return at most 500 records. Cursor pagination, update/delete operations, catalog management, and the full permission catalog remain follow-up work. The device page supports tenant selection when a user belongs to multiple tenants; device creation/edit UI is not yet implemented.

## Tenant-aware device and interface inventory API

The inventory endpoints use the same authenticated user and active tenant membership check as region/site/POP operations. Every request must include the tenant public UUID in `X-Tenant-ID`.

| Method | Route | Permission |
|---|---|---|
| GET | `/api/v1/network/devices` | Active tenant membership |
| POST | `/api/v1/network/devices` | owner / admin / network_admin |
| GET | `/api/v1/network/devices/{device_id}` | Active tenant membership |
| GET | `/api/v1/network/devices/{device_id}/interfaces` | Active tenant membership |
| POST | `/api/v1/network/devices/{device_id}/interfaces` | owner / admin / network_admin |

Example device request:

```json
{
  "inventory_key": "core-rtr-01",
  "name": "Core Router 01",
  "hostname": "core-rtr-01.isp.example",
  "management_address": "192.0.2.10",
  "device_role": "router",
  "lifecycle_status": "active",
  "site_id": "SITE_PUBLIC_UUID",
  "pop_id": "POP_PUBLIC_UUID"
}
```

Supported initial roles: `router`, `switch`, `firewall`, `olt`, `server`, `wireless`, `other`. Lifecycle values: `planned`, `active`, `maintenance`, `retired`. Vendor/model references are optional while the catalog is not yet managed through an API. Management addresses are validated as IPv4 or IPv6.

Interfaces require a stable operator-defined `stable_key`. `if_index` is optional observed identity and must not be treated as permanent. New interface admin and operational states are `unknown`; no telemetry is fabricated. Device and interface lists are bounded to 500 rows until cursor pagination is delivered. The smoke-test coverage for these tables is part of the PostgreSQL 17 clean-database CI job.

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

CI runs the Rust formatting/check/lint/test commands and the Next.js lint/typecheck/build commands. CI includes a PostgreSQL 17 clean-database migration integration test covering identity, tenant, network location, and device inventory schema, including key uniqueness constraints.

## Current implementation boundary

Implemented in the current repository:
- Loco.rs 1.3.1 API workspace with PostgreSQL/SeaORM configuration.
- Initial identity, organization, tenant, membership, region, site, and POP migrations/entities.
- Password sign-in, JWT current-user endpoint, and administrator bootstrap task.
- Tenant-context resolution that checks active membership and organization.
- Tenant-scoped region/site/POP list and create APIs.
- Tenant-scoped vendor/model/device/interface inventory schema and device/interface list/create APIs.
- Next.js sign-in flow, HttpOnly session bridge, operational overview shell, and read-only device register page.
- Local Docker Compose stack and CI baseline.

Not yet implemented:
- Full permission catalog/custom RBAC and tenant administration UI.
- Vendor/model catalog management, inventory update/delete, cursor pagination, credential assignment, and discovery.
- Collector registration/heartbeat, polling scheduler, SNMP/ICMP/gNMI/NETCONF/SSH/BGP adapters.
- NATS event transport, TSDB metrics ingestion/query, alert evaluation, topology, incidents.
- Subscriber/RADIUS/DHCP/BNG, FTTH optical/GIS, SLA reporting.
- Configuration capture, approval workflow, controlled network writes, verification, rollback.
- Production backup/restore drills, retention, load testing, HA, and operational runbooks.

Do not present planned or scaffold-only modules as operational features. Use [the master execution board](https://github.com/bjo163/e-isp/issues/110) for the detailed issue-by-issue delivery plan.


## Secure initial administrator provisioning

The bootstrap password is read only from `EISP_ADMIN_PASSWORD`; it is not accepted as a command-line argument. For an interactive shell:

```sh
(
  read -s -p "Initial admin password: " EISP_ADMIN_PASSWORD
  echo
  EISP_ADMIN_PASSWORD="$EISP_ADMIN_PASSWORD" cargo loco task admin:create email:admin@example.com name:Administrator organization:"Example ISP" tenant:Production org_slug:example-isp tenant_slug:production
)
```

Use a unique password of at least 12 characters. Do not paste production credentials into shell command arguments, GitHub issues, logs, or source files.
