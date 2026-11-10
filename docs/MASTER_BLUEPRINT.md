# Master Blueprint — e-ISP

**Status:** Approved product/architecture baseline for implementation planning  
**Product:** e-ISP Network Operations, Service Assurance & Automation Platform  
**Backend:** Loco.rs (Rust)  
**Frontend:** Next.js App Router (TypeScript)  
**Architecture posture:** modular monolith for control-plane domain logic + separately deployed Rust collectors/agents + purpose-built telemetry/event infrastructure.

## 1. Architecture decisions

| Decision | Baseline | Rationale / guardrail |
|---|---|---|
| Backend framework | Loco.rs 1.3.1 | Rust application framework for controllers, models, auth, workers, scheduler, and tasks |
| Frontend | Next.js 16.4.6 App Router + TypeScript | Operator cockpit and administration experience |
| Rust toolchain | Rust 1.94.0 | Matches Loco/SeaORM MSRV |
| Node.js | Node 22.16.0+ | Supported frontend build/runtime baseline |
| Backend shape | Modular monolith first | Keep domain behavior cohesive; split deployables only for justified operational boundaries |
| Database | PostgreSQL + SeaORM through Loco | Operational source of truth and transactional workflows |
| Telemetry | VictoriaMetrics-compatible TSDB | High-volume numeric samples must not live in business tables |
| Event transport | NATS JetStream | Durable buffering/fan-out for normalized telemetry and operational events |
| Collector | Rust processes/agents | Vendor-neutral protocol adapters; central or remote POP deployment |
| Geospatial | PostGIS | Fiber, POP, site, route, and field-location queries |
| Topology | Relational source model + graph projection/query layer | Avoid introducing a graph database before measured need |
| Object storage | S3-compatible storage abstraction | Configuration snapshots, exports, and diagnostic artifacts |
| Observability | OpenTelemetry instrumentation + Prometheus-compatible metrics | Trace requests/jobs/collector pipelines and expose system health |
| Deployment | Docker Compose baseline | Repeatable single-node/self-hosted deployment; remote collectors are separate processes |

Loco's application workers are for durable business/background jobs. High-volume telemetry transport is a separate pipeline; do not route raw metric samples through ordinary web requests or business-job queues.

### 1.1 Framework boundary

Loco owns the application/control plane: HTTP API, domain services, transactional persistence, authentication/authorization, scheduled tasks, and finite background jobs. Next.js owns presentation and client interaction; it must not become a second domain backend.

Use Loco's supported worker mechanism for finite business jobs such as reports, imports, notifications, and controlled orchestration steps. Use the dedicated telemetry/event pipeline for continuous/high-volume device samples. Do not push raw polling samples through ordinary request handlers or the business-job queue.

Next.js uses the App Router for filesystem routing, layouts, and React Server/Client Components. Keep browser-only code in client components and server-only integration URLs/credentials out of public bundles.

### 1.2 Architecture rules

1. Domain logic belongs in Rust domain/application modules, not Next.js Server Actions or route handlers.
2. HTTP controllers translate transport input/output; they do not contain domain workflows.
3. SeaORM entities are persistence models, not public API contracts.
4. Every integration is behind a typed port/interface and an adapter implementation.
5. Collector protocol implementations must not write arbitrary business records directly.
6. Normalized observations/events cross an explicit versioned boundary before being consumed by domain services.
7. Desired configuration, observed device state, and historical telemetry are different concepts and must not be merged into one mutable record.
8. No service split, database split, or new infrastructure dependency without a measured operational reason and migration plan.

## 2. Context and system boundaries

### 2.1 External systems

Network devices include routers, switches, firewalls, wireless controllers, OLTs, ONTs/ONUs, servers, and probes. Expected protocols/integrations include SNMP, ICMP, syslog, NetFlow/IPFIX/sFlow, BGP/BMP, gNMI, NETCONF/RESTCONF, SSH, HTTP APIs, RADIUS, DHCP, BNG/BRAS, CRM, billing, and vendor APIs.

Integrations are opt-in, versioned, capability-declared, and independently testable. Support for a protocol family does not imply support for every vendor model or firmware.

### 2.2 Logical context

```text
[Operator / Admin / Engineer / Field Team]
                 |
             [Next.js]
                 |
        HTTPS / WebSocket / SSE
                 |
            [Loco.rs API]
                 |
   +-------------+-------------------------------+
   |             |             |                 |
[Domain]      [Auth/RBAC]   [Loco Jobs]       [Integration Ports]
   |             |             |                 |
   +-------------+-------------+-----------------+
                 |
            [PostgreSQL]
                 |
     +-----------+-------------------------+
     |                                     |
[Topology / Service graph]          [Outbox / Audit]
     |
     +----------------------+
                            |
[Remote/Central Rust Collectors] -> [NATS JetStream] -> [Normalizer/Consumers]
          |                                             |
      Network devices                              [TSDB / Postgres]
          |                                             |
 SNMP/ICMP/flows/syslog/etc.                      [Query/API views]
```

The diagram is logical, not a requirement to deploy every box as an independent microservice. The first implementation deploys the Loco API and workers from the same application artifact while collectors are separately deployable.

## 3. Deployable units

### 3.1 `apps/web` — Next.js operator interface

Responsibilities: command center, role-based navigation, search, dashboards, topology, maps, inventory, incidents, service assurance, automation review, and administration. It calls the Loco API through a typed client and handles loading, empty, stale, partial, permission-denied, and error states.

Must not connect directly to PostgreSQL, NATS, TSDB, or device networks, implement authoritative business rules, store device credentials in browser state, or execute network commands.

### 3.2 `apps/api` — Loco.rs application

Responsibilities: versioned HTTP API, authentication/authorization enforcement, domain/application services, validation, persistence through SeaORM, audit/outbox writes, read models, Loco workers, and scheduled finite business tasks.

Keep it a modular monolith. Controllers remain thin. API DTOs are separate from SeaORM entities.

### 3.3 `apps/collector` — Rust collection runtime

Responsibilities: protocol collection, bounded polling, normalization, buffering, secure forwarding, and collector health. It must run centrally or at remote POPs. Remote agents use outbound authenticated connections by default.

The initial repository scaffold is bootstrap-only: it does not yet poll devices or register with the control plane. Protocol collection is implemented only through child issues and tested capability profiles.

### 3.4 Pipeline consumers

Start as Rust worker/binary processes in the same workspace where practical. Separate scaling units only when workload profiles demand it: ingest/validation, event normalization, alert evaluation, correlation, retention/rollup, and analytics/forecast jobs.

## 4. Repository and workspace layout

```text
e-isp/
├── apps/
│   ├── web/                       # Next.js App Router
│   ├── api/                       # Loco.rs application
│   └── collector/                 # Rust collector executable
├── crates/
│   ├── domain-core/               # shared domain IDs and invariants
│   ├── event-envelope/            # versioned event transport contract
│   ├── domain-organization/       # tenants, orgs, sites, POPs (add when implemented)
│   ├── domain-inventory/          # devices, interfaces, hardware
│   ├── domain-topology/           # links, adjacency, dependency graph
│   ├── domain-telemetry/          # metric/event contracts and query abstractions
│   ├── domain-alerting/           # alert rules, state transitions, dedupe
│   ├── domain-incidents/          # incidents, impact, timeline, escalation
│   ├── domain-services/           # subscribers, circuits, services, SLA
│   ├── domain-fiber/              # OLT, PON, ONT, splitter, fiber assets
│   ├── domain-config/             # snapshots, diffs, changes, approvals
│   ├── domain-automation/         # typed workflows and policy execution
│   ├── integration-contracts/     # ports and capability declarations
│   ├── adapter-*/                 # isolated vendor/protocol adapters
│   └── test-support/              # fixtures, simulators, contract helpers
├── packages/
│   ├── api-client/                # typed frontend client
│   ├── ui/                        # reusable product UI primitives
│   └── design-tokens/             # semantic tokens
├── docs/
├── deploy/
├── scripts/
└── .github/
```

Do not create placeholder crates for every future domain; add a crate when its API boundary has a real consumer and tests. The checked-in workspace currently includes only the foundational crates with implemented responsibilities.

## 5. Domain model and ownership

### 5.1 Tenancy and organization

Target entities: Organization, Tenant, User, Team, Role, Permission, Membership, Site, Region, and POP. Every tenant-owned record carries tenant context or is reachable through an explicitly scoped parent. Tenant scoping is mandatory at repository/query boundaries; background jobs and collectors must carry explicit scope.

### 5.2 Inventory

Target entities: Device, DeviceModel, Vendor, DeviceRole, DeviceInterface, CredentialReference, DeviceObservation, NetworkLink, Circuit, and InterfaceAddress. Separate desired inventory, observed discovery facts, telemetry samples, and operational status projections. Observations retain source, timestamp, collector identity, and provenance. Discovery must not silently overwrite operator-managed inventory.

### 5.3 IPAM

Target entities: IpNamespace, Prefix, IpAddress, Vlan, Vrf, Asn, BgpPeer, and DhcpPool. Allocation history is auditable. Validate overlaps, parent containment, address family, and allocation state.

### 5.4 Topology and dependencies

Represent declared topology and discovered topology separately. Topology edges retain type, source/evidence, and validity timestamps. Initial graph traversal uses PostgreSQL recursive queries and indexed adjacency tables; do not add a graph database before measured need.

### 5.5 Telemetry and events

A metric sample is not an incident. An event is not automatically an alert. An alert is not automatically an incident.

- Metric: numeric time-series observation with controlled labels.
- Event: discrete normalized occurrence/state change.
- Alert: rule evaluation result with lifecycle/deduplication state.
- Incident: operator-managed case grouping related alerts/events and affected entities/services.

### 5.6 ISP services and fiber

Service entities include Subscriber, AccessSession, Circuit, ServiceInstance, ServiceProfile, and SlaPolicy. Fiber entities include Olt, PonPort, OnuOnt, Splitter, FiberCable, FiberSegment, Odf, Odc, Odp, and OpticalObservation. Keep logical PON hierarchy distinct from geographic route geometry.

### 5.7 Change and automation

ConfigurationSnapshot and ConfigurationDiff are immutable evidence. ChangeRequest captures scope, risk, author, window, and approvals. WorkflowDefinition is a versioned typed graph; WorkflowRun captures immutable execution history. The initial action catalog is restricted and typed, not arbitrary code.

## 6. API and contract standards

- Prefix public endpoints with `/api/v1`.
- Use stable JSON error envelopes with machine-readable codes and request IDs.
- Validate at API boundaries and enforce domain invariants in services.
- Use bounded cursor pagination for large lists.
- Require idempotency keys for harmful retryable operations.
- Keep API DTOs separate from SeaORM entities.
- Publish OpenAPI as contract artifact and generate frontend types/client from it.
- Event envelopes carry schema version, event ID, tenant, source, occurrence time, ingestion time, and typed payload.
- Consumers are idempotent and reject unsupported versions safely.

## 7. Telemetry and event processing

Collectors select due checks, execute adapters with bounded controls, normalize results, send metric samples to TSDB, and publish discrete events to JetStream. Consumers validate schema/tenant/source, deduplicate, and update operational projections. Local spooling is disk/age bounded and replay preserves event IDs.

PostgreSQL stores operational records and policy; VictoriaMetrics stores high-volume numeric samples; NATS JetStream transports normalized events but is not the source of truth. Metric labels are bounded and must not contain arbitrary free-form payloads or unbounded subscriber identifiers.

## 8. Monitoring and alert model

Initial metric families: reachability, latency, loss, device health, interface counters, routing, optical values, service session state, and collector health. Status semantics distinguish healthy, degraded, critical, unknown, stale, and maintenance. Missing data is not zero or healthy.

Alert lifecycle includes firing, acknowledged, resolved, suppressed, and silenced with explicit semantics. Fingerprints are deterministic and deduplication idempotent. Maintenance annotates/suppresses alerting by policy but does not delete telemetry/history.

## 9. Fault correlation and incident management

Correlate alerts using topology/dependency evidence, time windows, and rule-specific relationships. Correlation is explainable, confidence-aware, reversible, and never asserts certainty without evidence. Customer impact is computed only when service/subscriber mappings exist and are fresh. Incident transitions are policy-checked and audited.

## 10. ISP and telecom scope

Initial RADIUS and DHCP integrations are read-only. BNG/BRAS, PPPoE/IPoE, OLT/ONT, BGP, and OSS/BSS integrations are adapter/capability-scoped. No integration is declared supported without tested source/version/device evidence. SLA calculation requires explicit calendar, timezone, exclusions, thresholds, evidence coverage, and calculation version.

## 11. Configuration and automation safety

Actions are typed, target-scoped, capability-checked, permissioned, and risk-classified. Service-affecting actions require approval policy. Execution records actor, request, policy, approval, adapter/version, redacted input, result, and verification. Secrets resolve just-in-time and are never logged or sent to the browser. Verify postconditions; rollback is a separate authorized action. No arbitrary shell/script or AI-generated command execution in the initial workflow engine.

AI may summarize evidence, explain topology, draft diagnostics, and recommend next steps. It cannot bypass RBAC, policy, approval, command allowlists, or operator confirmation.

## 12. Security, privacy, and tenancy

Deny by default. Enforce tenant scope on requests, background jobs, events, subscriptions, exports, and collector identities. Store credentials encrypted through a replaceable secret provider, use rotation/revocation, and keep ordinary APIs write-only for secret material. Remote collector identity is authenticated and revocable. Audit permission changes, exports, credential access metadata, configuration changes, approvals, and execution. Redact credentials and PII from logs/traces.

## 13. Frontend experience and design system

### 13.1 Information architecture

Command; Network; Observability; Services; Fiber; IPAM; Incidents; Automation; Engineering; Intelligence; Workforce; Administration. A nav item is shown only when its first usable workflow exists.

### 13.2 UX rules

Desktop-first for NOC workstations, tablet-usable triage, global search, explicit data freshness/source, loading/empty/stale/partial/denied/failure states, accessible severity semantics, and impact preview/confirmation for risky actions. No fake charts or nonfunctional buttons.

### 13.3 Visual direction

**Industrial Operations Cockpit:** information density with intentional hierarchy, restrained high-contrast accents, precise typography, compact panels, topology/map focus, clear status semantics, and accessible light/dark modes. Visual effects must not reduce operator readability.

## 14. Reliability and non-functional requirements

Bound memory, concurrency, polling, message size, query range, and spool. Support graceful shutdown/restart, idempotent consumers, retries/timeouts, backpressure, structured logs, metrics, traces, backup/restore, and representative workload testing. Do not claim carrier-grade HA until failover, recovery, and upgrade behavior are tested.

## 15. Deployment model

### 15.1 Initial deployment

Docker Compose baseline: Next.js web, Loco API/worker, Rust collector(s), PostgreSQL, VictoriaMetrics, NATS JetStream, and optional S3-compatible object storage. Keep internal data services private. Production secrets are injected securely, not committed.

### 15.2 Remote POP collector

Outbound-only secure communication by default; collector identity includes tenant/site, version, and capabilities. Local spool and replay are bounded. Releases are signed/versioned and upgrades are controlled with rollback.

### 15.3 Backup and recovery

Back up PostgreSQL and object artifacts/config snapshots; TSDB and event-stream policies are separate. Test restore into a clean environment. Backup completion alone is not acceptance evidence.

## 16. Test strategy

Unit tests for domain invariants/state machines/parsers; integration tests for Loco/PostgreSQL/jobs/NATS/TSDB; adapter contract and simulator tests; hardware-in-the-loop for declared vendor/model/firmware support; frontend critical-flow/accessibility tests; e2e fault-to-incident and approved-change verification; security tests for tenant and privilege boundaries; and realistic performance/backpressure profiles.

## 17. Delivery roadmap and dependency gates

1. Foundation: runnable Loco + Next.js, API contract, PostgreSQL, CI, local Compose.
2. Tenant and inventory: tenant isolation, RBAC, site/device/IPAM.
3. Collector and telemetry: authenticated collector identity, initial adapters, normalized evidence, TSDB, freshness.
4. Fault operations: topology, alerts, incident lifecycle, explainable impact.
5. ISP/FTTH: subscriber/session/service and optical/fiber context.
6. Automation: snapshots, changes, approvals, typed execution, verification, rollback.
7. Intelligence/production: capacity, anomaly insight, recovery, security, performance, runbooks.

## 18. Explicit non-goals for initial product

Billing/payment collection, full CRM/ERP, universal vendor coverage, arbitrary remote terminal, full digital twin simulation, autonomous AI changes, premature microservices/graph database/Kubernetes-only deployment, and fixed performance guarantees before load tests.

## 19. Definition of Done

- [ ] Scope matches this blueprint and has one bounded deliverable.
- [ ] Domain/API/UI ownership is clear and no duplicate source of truth was introduced.
- [ ] Authorization and tenant scoping are enforced server-side.
- [ ] Validation, errors, loading/stale/empty states are handled.
- [ ] Tests cover happy path, invalid input, failure path, and relevant security boundary.
- [ ] Logs/metrics/traces are adequate for operational diagnosis.
- [ ] Documentation and examples match actual behavior.
- [ ] No credentials or real customer data are committed.
- [ ] CI checks pass; migrations and rollback implications are reviewed.
- [ ] Acceptance criteria are checked with evidence.

## 20. Change control

A change to product domains, framework choices, event/data ownership, tenant model, or safety boundaries requires evidence, impact analysis, blueprint update, issue/dependency updates, and compatibility/test/rollout notes. Implementation issues may refine details but must not silently contradict this document.
