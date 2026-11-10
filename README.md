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

## Product and architecture documents

- [Product Narrative](docs/PRODUCT_NARRATIVE.md)
- [Master Blueprint](docs/MASTER_BLUEPRINT.md)
- [Engineering Rules](docs/ENGINEERING_RULES.md)
- [Issue Execution Standard](docs/ISSUE_EXECUTION_STANDARD.md)
- [Execution Backlog](docs/EXECUTION_BACKLOG.md)

## Delivery tracking

- [Master GitHub execution board — 127 granular implementation issues](https://github.com/bjo163/e-isp/issues/110)
- [Repository execution backlog index](docs/EXECUTION_BACKLOG.md)

The blueprint owns architecture decisions; child issues own bounded implementation deliverables and acceptance evidence.

## Current status

The Loco API and Next.js shell are scaffolded. The collector is explicitly bootstrap-only. Network discovery, protocol polling, tenant/auth flows, inventory, telemetry ingestion, topology, incidents, ISP integrations, automation, and production hardening are not yet implemented. Do not treat issue creation or a successful scaffold build as proof those capabilities exist.
