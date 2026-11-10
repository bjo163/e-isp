# e-ISP

**Network Operations, Service Assurance & Automation Platform**

e-ISP is a vendor-neutral operations platform for Internet Service Providers, fiber operators, telecommunications networks, and managed network operators. It connects network inventory, topology, telemetry, fault management, subscriber/service assurance, and controlled automation in one operational workspace.

> **Know the network. Protect the service. Automate the operation.**

## Product direction

e-ISP is designed around three operating principles:

1. **Network-first** — devices, links, sites, protocols, and observed state are first-class concepts.
2. **Service-aware** — network conditions are connected to circuits, subscribers, service health, and SLA impact.
3. **Automation-driven** — repeatable operations are executed through policy, approval, audit, verification, and rollback.

The product is not a thin dashboard over ping checks. It is an operational system that helps teams discover what exists, understand what is happening, identify who is affected, and take safe action.

## Technology direction

- **Web:** Next.js App Router + TypeScript.
- **Backend:** Loco.rs (Rust) for the API and modular domain application.
- **Collectors and protocol runtimes:** Rust, deployed centrally or at remote POPs.
- **Operational source of truth:** PostgreSQL + SeaORM (through Loco).
- **Telemetry storage:** VictoriaMetrics-compatible time-series storage.
- **Event transport:** NATS JetStream for buffered telemetry/events where required.
- **Geospatial:** PostGIS; MapLibre-based map UI.
- **Deployment:** Docker Compose first; support remote collectors and later orchestrated deployments.

Loco's application workers are for durable business/background jobs. High-volume telemetry transport is a separate pipeline; do not route raw metric samples through ordinary web requests or business-job queues.

## Documentation

- [Product narrative](docs/PRODUCT_NARRATIVE.md)
- [Master blueprint](docs/MASTER_BLUEPRINT.md)
- [Engineering and design rules](docs/ENGINEERING_RULES.md)
- [Issue execution standard](docs/ISSUE_EXECUTION_STANDARD.md)

## Delivery tracking

All implementation work is tracked as small GitHub issues. Each issue must have one bounded deliverable, explicit dependencies, design rules, and a checkbox-based acceptance criteria. The blueprint is the product/architecture source of truth; issues are its executable work breakdown.

## Scope guardrails

- Do not claim protocol or vendor support without a capability definition and tested adapter.
- Do not mix desired configuration with observed device state.
- Do not store high-volume time-series samples in relational business tables.
- Do not run production-changing commands without authorization policy, approval, audit, and post-change verification.
- Do not introduce a new infrastructure dependency without documenting its ownership, failure mode, and operational reason.

## Status

The repository is currently at the documentation and execution-planning foundation. Implementation status must be reported from merged code and automated checks, never inferred from open issues alone.
