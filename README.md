# e-ISP

**Network Operations, Service Assurance & Automation Platform**

> **Know the network. Protect the service. Automate the operation.**

A vendor-neutral operations platform for Internet Service Providers, fiber operators, telecommunications networks, and managed network operators.

## Technology

- **Web:** Next.js App Router 16.4.6 + TypeScript.
- **Backend:** Loco.rs 1.3.1 + Rust 1.94.
- **Collectors:** Rust processes for central and remote POP deployment.
- **Operational data:** PostgreSQL + SeaORM.
- **Telemetry:** VictoriaMetrics-compatible time-series storage (planned integration).
- **Events:** NATS JetStream (planned integration).
- **Geospatial:** PostGIS and MapLibre (planned integration).

## Get started

See [Development Guide](docs/DEVELOPMENT.md). For a local Compose stack, copy `.env.example` to `.env`, then run:

```sh
docker compose up --build
```

Open `http://localhost:3000`. The Loco system endpoint is `http://localhost:5150/api/v1/system/info`.

Create the initial administrator and tenant (after the API is running and migrations have completed):

```sh
(
  read -s -p "Initial admin password: " EISP_ADMIN_PASSWORD
  echo
  EISP_ADMIN_PASSWORD="$EISP_ADMIN_PASSWORD" cargo loco task admin:create email:admin@example.com name:Administrator organization:'Example ISP' tenant:Production org_slug:example-isp tenant_slug:production
)
```

Then sign in at `http://localhost:3000/login`. Public self-registration is deliberately disabled.

## Product and architecture documents

- [Product Narrative](docs/PRODUCT_NARRATIVE.md)
- [Master Blueprint](docs/MASTER_BLUEPRINT.md)
- [Engineering Rules](docs/ENGINEERING_RULES.md)
- [Issue Execution Standard](docs/ISSUE_EXECUTION_STANDARD.md)
- [Execution Backlog](docs/EXECUTION_BACKLOG.md)
- [Development Guide](docs/DEVELOPMENT.md)

## Delivery tracking

- [Master GitHub execution board — 127 granular implementation issues](https://github.com/bjo163/e-isp/issues/110)
- [Repository execution backlog index](docs/EXECUTION_BACKLOG.md)

The blueprint owns architecture decisions; child issues own bounded implementation deliverables and acceptance evidence.

## Network inventory API

Network location operations require a signed-in user and an active tenant membership. Send the tenant public UUID in the `X-Tenant-ID` header; the API verifies that the authenticated user is an active member before querying or writing tenant-owned rows.

- `GET /api/v1/network/regions`
- `POST /api/v1/network/regions`
- `GET /api/v1/network/sites`
- `POST /api/v1/network/sites`
- `GET /api/v1/network/pops`
- `POST /api/v1/network/pops`

List endpoints currently return at most 500 records. Writes require owner/admin/network_admin membership roles. Cursor pagination and the complete permission catalog remain follow-up work.

## Current status

Implemented: Loco API foundation, PostgreSQL identity/organization/tenant/membership schema, password login and current-user APIs, administrator bootstrap task, Next.js HttpOnly session bridge, and sign-in screen.

Still pending: full RBAC permissions, tenant-scoped network inventory CRUD, collectors, protocol polling, telemetry ingestion, topology, incidents, ISP/FTTH integrations, controlled network automation, and production hardening. Do not treat planned modules as operational features.
