# Master Blueprint — e-ISP

**Status:** Approved product/architecture baseline for implementation planning  
**Product:** e-ISP Network Operations, Service Assurance & Automation Platform  
**Backend:** Loco.rs (Rust)  
**Frontend:** Next.js App Router (TypeScript)  
**Architecture posture:** modular monolith for control-plane domain logic + separately deployed Rust collectors/agents + purpose-built telemetry/event infrastructure.

This document is the architectural source of truth. GitHub issues break this blueprint into executable tasks; issues must not redefine product scope or create a competing architecture.

---

## 1. Architecture decisions

| Decision | Baseline | Rationale / guardrail |
|---|---|---|
| Backend framework | Loco.rs | Rust application framework for controllers, models, auth, workers, scheduler, and tasks |
| Frontend | Next.js App Router + TypeScript | Operator cockpit and administration experience |
| Backend shape | Modular monolith first | Keep domain behavior cohesive; split deployables only for justified operational boundaries |
| Database | PostgreSQL + SeaORM through Loco | Operational source of truth and transactional workflows |
| Telemetry | VictoriaMetrics-compatible TSDB | High-volume numeric samples must not live in business tables |
| Event transport | NATS JetStream | Durable buffering/fan-out for normalized telemetry and operational events |
| Collector | Rust processes/agents | Vendor-neutral protocol adapters; central or remote POP deployment |
| Geospatial | PostGIS | Fiber, POP, site, route, and field-location queries |
| Topology | Relational source model + graph projection/query layer | Avoid introducing a graph database before measured need |
| Object storage | S3-compatible storage abstraction | Configuration snapshots, exports, and diagnostic artifacts |
| Observability | OpenTelemetry instrumentation + Prometheus-compatible metrics | Trace requests/jobs/collector pipelines and expose system health |
| Cache | No mandatory cache initially | Add Redis/Valkey only for a measured need; do not duplicate queue/event responsibilities |
| Deployment | Docker Compose baseline | Repeatable single-node/self-hosted deployment; remote collectors are separate processes |

### 1.1 Framework boundary

Loco.rs owns the application/control plane: HTTP API, domain services, transactional persistence, authentication/authorization integration, scheduled tasks, and finite background jobs. Next.js owns presentation and client interaction; it must not become a second domain backend.

Use Loco's supported worker mechanism for finite business jobs such as reports, imports, notifications, and controlled orchestration steps. Use the dedicated telemetry/event pipeline for continuous/high-volume device samples. Do not push raw polling samples through ordinary request handlers or the business-job queue.

The official Loco documentation describes its controllers, models, background jobs, scheduler, and worker modes. Confirm API/configuration details against the version pinned by the project before implementation:
- https://loco.rs/docs/
- https://loco.rs/docs/how-to/add-worker/
- https://loco.rs/docs/explanation/background-processing-model/
- https://loco.rs/docs/how-to/choose-queue-backend/

Next.js uses the App Router for filesystem routing, layouts, and React Server/Client Components. Use its official documentation and pin a tested version:
- https://nextjs.org/docs/app

### 1.2 Architecture rules

1. Domain logic belongs in Rust domain/application modules, not Next.js Server Actions or route handlers.
2. HTTP controllers translate transport input/output; they do not contain domain workflows.
3. SeaORM entities are persistence models, not public API contracts.
4. Every integration is behind a typed port/interface and an adapter implementation.
5. Collector protocol implementations must not write arbitrary business records directly.
6. Normalized observations/events cross an explicit versioned boundary before being consumed by domain services.
7. Desired configuration, observed device state, and historical telemetry are different concepts and must not be merged into one mutable record.
8. No service split, database split, or new infrastructure dependency without a measured operational reason and migration plan.

---

## 2. Context and system boundaries

### 2.1 External systems

e-ISP may integrate with:
- Network devices: routers, switches, firewalls, wireless controllers, OLTs, ONTs/ONUs, servers, and probes.
- Protocols: SNMP, ICMP, syslog, NetFlow/IPFIX/sFlow, BGP/BMP (where supported), gNMI, NETCONF/RESTCONF, SSH, HTTP APIs, and vendor APIs.
- ISP service systems: RADIUS, DHCP/IPAM, BNG/BRAS, PPPoE/IPoE session systems, CRM, billing, and ticketing.
- External identity providers and notification channels.
- S3-compatible object storage and observability infrastructure.

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

The diagram is logical, not a requirement to deploy every box as an independent microservice. The first implementation can deploy the Loco API and worker from the same application artifact, while collectors run as separate processes.

---

## 3. Deployable units

### 3.1 `apps/web` — Next.js operator interface

Responsibilities:
- Operator command center, role-based navigation, search, dashboards, topology, maps, inventory, incidents, service assurance, automation review, and administration.
- Call the Loco API through a typed client.
- Render server components for stable/read-heavy layouts; use client components only for interaction, realtime views, and graph/map controls.
- Maintain URL-addressable filters and views where appropriate.
- Handle loading, empty, stale, partial, permission-denied, and error states explicitly.

Must not:
- Connect directly to PostgreSQL, NATS, TSDB, or device networks.
- Implement business rules that can be bypassed by another API client.
- Store device credentials in browser state.
- execute network commands directly.

### 3.2 `apps/api` — Loco.rs application

Responsibilities:
- Versioned HTTP API.
- Authentication and authorization enforcement.
- Domain/application services and transactional use cases.
- Input validation, pagination, filtering, idempotency, and structured errors.
- Persistence through SeaORM.
- Outbox/audit writes for business actions.
- Read models/queries for UI.
- Loco workers and scheduled tasks for finite business jobs.

Keep it a modular monolith. A domain module may expose controllers, DTOs, services, repositories/queries, policies, and tests without being a deployable microservice.

### 3.3 `apps/collector` — Rust collection runtime

Responsibilities:
- Poll or subscribe to supported network sources.
- Run health checks and protocol-specific collection loops.
- Apply per-target timeout, concurrency, interval, jitter, retry, and rate-limit controls.
- Normalize observations into versioned envelopes.
- Buffer locally during temporary upstream disconnection.
- Forward events/metrics securely to the central pipeline.
- Report collector identity, version, capabilities, health, and last successful contact.

Collector runtime must be safe to run centrally or at remote POPs. Remote agents must use outbound authenticated connections; do not require a public inbound management port by default.

### 3.4 Pipeline consumers

Start as Rust worker/binary processes in the same workspace where practical. Separate scaling units only when workload profiles demand it:
- ingest/validation,
- event normalization and routing,
- alert evaluation,
- correlation,
- retention/rollup,
- analytics/forecast jobs.

Do not create a separate service for every domain.

---

## 4. Repository and workspace layout

The structure below is the target contract; adapt only when repository evidence proves a better layout.

```text
e-isp/
├── apps/
│   ├── web/                       # Next.js App Router
│   ├── api/                       # Loco.rs application
│   └── collector/                 # Rust collector executable
├── crates/
│   ├── domain-core/               # IDs, time, errors, shared domain primitives
│   ├── domain-organization/       # tenants, orgs, sites, POPs
│   ├── domain-inventory/          # devices, interfaces, hardware
│   ├── domain-ipam/               # prefixes, IPs, VLANs, VRFs, ASNs
│   ├── domain-topology/           # links, adjacency, dependency graph
│   ├── domain-telemetry/          # metric/event contracts and query abstractions
│   ├── domain-alerting/           # alert rules, state transitions, dedupe
│   ├── domain-incidents/          # incidents, impact, timeline, escalation
│   ├── domain-services/           # subscribers, circuits, services, SLA
│   ├── domain-fiber/              # OLT, PON, ONT, splitter, fiber assets
│   ├── domain-config/             # snapshots, diffs, changes, approvals
│   ├── domain-automation/         # typed workflows and policy execution
│   ├── integration-contracts/     # ports, capabilities, normalized envelopes
│   ├── adapter-snmp/              # SNMP adapter
│   ├── adapter-icmp/              # ICMP probe adapter
│   ├── adapter-syslog/            # syslog receiver/parser
│   ├── adapter-flow/              # NetFlow/IPFIX/sFlow adapters
│   ├── adapter-gnmi/              # gNMI adapter
│   ├── adapter-netconf/           # NETCONF/RESTCONF adapter
│   ├── adapter-ssh/               # restricted SSH adapter
│   ├── adapter-radius/            # RADIUS integration
│   ├── adapter-dhcp/              # DHCP integration
│   ├── event-envelope/            # versioned transport envelope
│   ├── event-pipeline/            # publish/consume abstractions
│   ├── telemetry-store/           # TSDB port and implementation
│   ├── object-store/              # artifact/config storage abstraction
│   ├── security-policy/           # authorization and execution policy primitives
│   └── test-support/              # fixtures, simulators, contract test helpers
├── packages/
│   ├── api-client/                # typed frontend client
│   ├── contracts/                 # OpenAPI-derived/shared transport contracts
│   ├── ui/                        # product UI primitives
│   ├── design-tokens/             # semantic tokens
│   ├── config/                    # shared frontend/tooling config
│   └── typescript-config/
├── docs/
├── deploy/
│   ├── compose/
│   ├── collector/
│   └── examples/
├── scripts/
├── .github/
│   ├── ISSUE_TEMPLATE/
│   └── workflows/
├── Cargo.toml
├── pnpm-workspace.yaml
└── README.md
```

Rules:
- Do not create placeholder crates for every future domain immediately; add a crate when its API boundary has at least one real consumer and tests.
- All Rust packages use workspace-managed dependency versions and consistent lint/test policy.
- The Next.js app uses the package manager and workspace policy established by the repository (default proposal: pnpm).
- Generated API types are generated from the contract source; do not hand-maintain duplicate DTO definitions in multiple places.

---

## 5. Domain model and ownership

### 5.1 Tenancy and organization

Entities:
- `Organization`: contractual/operator boundary.
- `Tenant`: data and policy isolation boundary (may map 1:1 with an organization initially).
- `User`, `Team`, `Role`, `Permission`, `Membership`.
- `Site`, `Region`, `Pop`: operator network geography and operational hierarchy.

Every tenant-owned record carries tenant context or is reachable through an explicitly scoped parent. Tenant scoping is mandatory at repository/query boundaries; background jobs and collectors must carry an explicit tenant/organization scope.

### 5.2 Inventory

Core entities:
- `Device`, `DeviceModel`, `Vendor`, `DeviceRole`, `DeviceInterface`, `CredentialReference`, `DeviceObservation`.
- `Site`, `Pop`, `Rack` (only when rack-level inventory is required), `PowerDomain` (later).
- `NetworkLink`, `Circuit`, `InterfaceAddress`.

Separate:
- desired inventory/configuration,
- observed inventory/discovery facts,
- telemetry samples,
- operational status projections.

An observation must retain source, timestamp, collector identity, and confidence/provenance. Do not silently overwrite operator-managed inventory with uncertain discovery data.

### 5.3 IPAM

Core entities:
- `IpNamespace`, `Prefix`, `IpAddress`, `Vlan`, `Vrf`, `Asn`, `BgpPeer`, `DhcpPool`.
- Allocation/reservation history must be auditable.
- Validate overlaps, parent-child containment, address family, and allocation state.
- Keep vendor-specific runtime configuration outside the IPAM source-of-truth model.

### 5.4 Topology and dependencies

Core entities:
- `TopologyNode` references typed entities (device, interface, site, service, OLT, etc.).
- `TopologyEdge` describes a typed relationship with source, evidence, and validity timestamps.
- `NetworkLink` represents an operator-managed physical/logical connection where appropriate.
- `ServiceDependency` links service delivery to network components.

Support two separate concepts:
1. **Declared topology** — operator/source-of-truth relationship.
2. **Discovered topology** — observed neighbor/route/link evidence.

A reconciled view may compare them, but must not erase their provenance. Initial graph traversal can use PostgreSQL recursive queries and indexed adjacency tables. Do not add Neo4j/graph DB until real query/scale evidence justifies it.

### 5.5 Telemetry and events

A metric sample is not an incident. An event is not automatically an alert. An alert is not automatically an incident.

- **Metric:** numeric time-series observation (value, time, series identity, controlled labels).
- **Event:** discrete normalized occurrence or state change.
- **Alert:** rule evaluation result with lifecycle/deduplication state.
- **Incident:** operator-managed case grouping one or more related alerts/events and affected entities/services.

Metric labels must be bounded and controlled. Never attach unbounded subscriber identifiers, free-form messages, or arbitrary payloads as time-series labels.

### 5.6 ISP services and fiber

Service entities:
- `CustomerReference` (external-system reference first; do not build full CRM by default).
- `Subscriber`, `AccessSession`, `Circuit`, `ServiceInstance`, `ServiceProfile`, `SlaPolicy`.
- `ServiceDependency` links the service to access/aggregation/core components.

Fiber entities:
- `Olt`, `PonPort`, `OnuOnt`, `Splitter`, `FiberCable`, `FiberSegment`, `Odf`, `Odc`, `Odp`, `OpticalObservation`.
- Keep the logical PON hierarchy distinct from geographic route geometry.
- Validate vendor-specific optical metrics and alarm interpretations per device capability.

### 5.7 Change and automation

- `ConfigurationSnapshot`: immutable captured configuration artifact + metadata/hash.
- `ConfigurationDiff`: comparison between immutable snapshots.
- `ChangeRequest`: proposed change, scope, impact, author, risk, approval state, window.
- `ExecutionRecord`: command/action reference, actor, policy decision, timestamps, result, verification.
- `RollbackPlan`: typed recovery action and preconditions.
- `WorkflowDefinition`: versioned typed graph/steps, inputs, policy requirements.
- `WorkflowRun`: immutable execution and step history.

A workflow definition is not executable arbitrary code. Start with a restricted action catalog and typed parameters.

---

## 6. API and contract standards

### 6.1 HTTP API

- Prefix all public API endpoints with `/api/v1`.
- Use resource-oriented routes and explicit action endpoints for state transitions.
- Use JSON and a stable error envelope with machine-readable code, safe message, request ID, and optional field errors.
- Validate input at the API boundary and again enforce domain invariants in application/domain services.
- Use cursor pagination for large/high-churn lists; document sort stability.
- Require idempotency keys for retryable create/execute operations where duplicate action would be harmful.
- Use optimistic concurrency/version fields for mutable configuration and workflow definitions.
- Keep API DTOs separate from SeaORM entity structs.
- Publish OpenAPI as the contract artifact and generate the frontend client/types from it.

### 6.2 Realtime

Use WebSocket or SSE for live operational views only when needed. Realtime messages carry identifiers and state deltas, not unbounded metric history. Clients must tolerate reconnect, duplicate messages, and stale cursors; they must refresh canonical state after reconnect.

### 6.3 Contract versioning

- Event envelopes include `schema_version`, `event_id`, `tenant_id`, `source_id`, `occurred_at`, `ingested_at`, and typed payload.
- Consumers must be idempotent and reject unsupported versions safely.
- Breaking transport changes require a new version and a compatibility plan.
- Do not use untyped `serde_json::Value` as the permanent domain contract.

---

## 7. Telemetry and event processing

### 7.1 Collection pipeline

1. Scheduler/collector selects targets and due checks.
2. Adapter executes with bounded timeout, concurrency, interval, and vendor capability.
3. Result is normalized with origin/provenance and freshness metadata.
4. Metric samples are sent to TSDB.
5. Discrete state changes/events are published to JetStream.
6. Consumers validate schema, enforce tenant/source scope, deduplicate, and update operational projections.
7. Alert evaluation and topology-aware correlation operate on normalized data, not raw vendor payloads.

### 7.2 Collector reliability

- Each collector has stable identity, tenant/org scope, deployment/site metadata, version, and capability report.
- Each target has a configured polling profile and per-protocol safety limits.
- Use jitter to avoid synchronized polling.
- Retry transient failures with bounded exponential backoff and jitter.
- Do not retry permanent authentication/unsupported-operation failures indefinitely.
- Local spool is bounded by disk quota, age, and event priority.
- On recovery, replay with idempotent event IDs and controlled throughput.
- Report buffer age, dropped count, backlog, last successful contact, and collector health.
- Central control can revoke a collector identity/credential.

### 7.3 Telemetry storage

- Store operational metric samples in VictoriaMetrics or compatible TSDB.
- Store metadata, policies, inventory, and query definitions in PostgreSQL.
- Query API must constrain time range, resolution, and series count.
- Apply retention by metric class and tenant plan.
- Use rollups/downsampling for longer history.
- Record collection gaps and stale status; absence of a sample is not a zero value.
- Do not promise a fixed telemetry resolution/retention until load tests establish sizing.

### 7.4 Event transport

NATS JetStream carries normalized discrete events and pipeline messages where durability/replay are needed. Define stream subjects, retention, max message size, consumer ack, redelivery, dead-letter/parking behavior, and schema compatibility. Avoid a subject per device/subscriber. Do not make NATS the system of record.

---

## 8. Monitoring and alert model

### 8.1 Initial metric families

- Reachability: availability, latency, packet loss.
- Device health: CPU, memory, temperature, power where supported.
- Interface: operational state, throughput, utilization, errors, discards.
- Routing: BGP peer state, prefix/route-change evidence where supported.
- Optical: RX/TX power, temperature, laser/LOS state where supported.
- Services: session count/state, authentication failures, synthetic service checks.
- Collector: scrape success, collection duration, queue/spool depth, freshness.

### 8.2 Freshness and state model

Every monitored target/metric view must distinguish:
- `healthy`: recent valid evidence within configured threshold.
- `degraded`: evidence exists and violates a configured quality threshold.
- `critical`: evidence violates a critical threshold or confirmed failure condition.
- `unknown`: no valid evidence or not enough data.
- `stale`: previous evidence exists but is older than its configured freshness window.
- `maintenance`: collection/alerting suppression is explicitly active.

Do not label missing data as healthy or down without a rule that supports that conclusion.

### 8.3 Alert lifecycle

States: `firing`, `acknowledged`, `resolved`, `suppressed`, `silenced` (with explicit semantics).

Alert fingerprint must be deterministic from rule + target + bounded dimension set. Deduplication must be idempotent. Preserve first seen, last seen, transitions, evidence, and rule version.

Maintenance windows suppress/annotate alert generation according to explicit policy; they do not delete telemetry or hide incident history.

---

## 9. Fault correlation and incident management

- Correlate alerts using topology and dependency evidence, shared time windows, and rule-specific relationships.
- Correlation produces a candidate root event with explanation; it must not assert certainty without evidence.
- Grouping is explainable and reversible by an operator.
- Incident records include severity, status, owner/team, linked alerts, affected devices/services/subscribers, timeline, notes, work items, and resolution.
- Customer impact is computed only when service/subscriber dependency data exists and is sufficiently fresh.
- Incident transitions are policy-checked and audited.
- Post-incident review captures cause, evidence, impact, restoration, and follow-up tasks.

A single failing upstream device may explain many downstream symptoms, but the UI must show which evidence supports that relationship and which downstream targets remain independently uncertain.

---

## 10. ISP and telecom scope

### 10.1 Initial integration boundaries

- RADIUS: session/authentication/accounting visibility via a read-only integration first.
- DHCP: lease/pool visibility via supported API/log integration.
- BNG/BRAS: sessions and service state through vendor/system-specific adapters.
- PPPoE/IPoE: session views where source system exposes supported data.
- OLT/ONT: discovery, PON state, optical metrics, alarms, and inventory based on vendor capability.
- BGP: peer and routing health through supported device API/telemetry or a dedicated routing collector.
- OSS/BSS/CRM: external identifiers and service/customer references; no full billing/CRM rebuild in this scope.

### 10.2 Fiber model

Represent physical/geospatial fiber route and logical optical topology separately. A splitter hierarchy is not a map line. OLT/port/ONT relationships must be validated against real device models. Optical thresholds must be configurable by vendor/model/technology and must not be hardcoded globally.

### 10.3 SLA and service assurance

SLA calculation needs explicit policy: service calendar, timezone, exclusion windows, outage threshold, evidence source, aggregation rule, and calculation version. Start with transparent measured availability; advanced contractual SLA reporting requires validated service mappings and policy sign-off.

---

## 11. Configuration and automation safety

### 11.1 Risk classes

- **Read-only:** diagnostics, inventory reads, telemetry, config capture.
- **Low-risk write:** reversible metadata or non-service-affecting changes.
- **Service-affecting:** interface, routing, VLAN, subscriber, or device changes.
- **High-risk/destructive:** broad config replacement, reboot, factory reset, route policy changes, or actions with uncertain rollback.

### 11.2 Execution controls

- Every action has a typed schema, explicit target scope, required permission, and risk class.
- Validate preconditions and current device capability before execution.
- Service-affecting actions require approval policy; high-risk actions require stronger approval and/or maintenance window.
- Enforce timeouts, rate limits, concurrency caps, and target allowlists.
- Record actor/automation identity, request ID, policy decision, approval, adapter/version, redacted input, result, and verification.
- Secrets are resolved just-in-time by a secret provider and never copied into logs, UI payloads, or issue artifacts.
- Verify postcondition and device state after action.
- Rollback must be explicit, tested where possible, and reported as successful/failed/unknown.
- Emergency break-glass access is auditable, time-bound, and separately permissioned.
- No arbitrary shell, raw script, or AI-generated command execution in the initial workflow engine.

### 11.3 AI assistance boundary

AI may summarize evidence, explain topology, draft diagnostics, recommend next steps, and prepare a change proposal. It cannot bypass RBAC, policy, approval, command allowlists, or operator confirmation. AI outputs are recommendations until executed through the normal action pipeline.

---

## 12. Security, privacy, and tenancy

- Tenant/org scope is required for every tenant-owned request, background job, event, and collector identity.
- Enforce authorization server-side for every read and write; UI hiding is not security.
- Add PostgreSQL Row-Level Security only as defense-in-depth after application scoping is correct and migration/test behavior is proven.
- Store credentials encrypted using envelope encryption or a dedicated secret manager; use key rotation and scoped decryption.
- Credential values are write-only in normal APIs and are never rendered back after creation.
- Remote collector identity uses mTLS or equivalent short-lived identity; support revocation and rotation.
- Network management access must be least privilege, target-scoped, allowlisted, and outbound from remote sites by default.
- Audit authentication events, permission changes, data exports, secret access metadata, configuration changes, and automation execution.
- Redact credentials, tokens, customer personal data, and sensitive network configuration from logs.
- Rate-limit public/authenticated endpoints and expensive queries.
- Define data retention and deletion behavior per data class and tenant contract.
- Security tests must cover horizontal privilege escalation and cross-tenant access.

---

## 13. Frontend experience and design system

### 13.1 Information architecture

- **Command:** overview, live operations, shift handover.
- **Network:** sites/POPs, devices, interfaces, links, discovery, topology.
- **Observability:** metrics, traffic, events, logs, probes.
- **Services:** subscribers, sessions, circuits, service health, SLA.
- **Fiber:** OLT, PON, ONT/ONU, splitters, routes, optical health.
- **IPAM:** prefixes, IPs, VLANs, VRFs, ASNs, pools.
- **Incidents:** alerts, incidents, maintenance, escalation, postmortems.
- **Automation:** workflows, approvals, executions, policies.
- **Engineering:** configuration snapshots, diffs, change requests.
- **Intelligence:** traffic, capacity, trends, forecasts.
- **Workforce:** work orders, dispatch, maintenance, handover.
- **Administration:** organizations, users, roles, integrations, collectors, audit.

Only expose a navigation item when its first usable workflow exists; do not ship empty menu shells.

### 13.2 UX rules

- Design desktop-first for NOC workstations, but keep essential triage views usable on tablet.
- Provide a clear global search for devices, sites, POPs, incidents, subscribers, and circuits.
- Use stable severity semantics and status colors with labels/icons; color alone must not encode meaning.
- Always expose data freshness and collection source on live metrics.
- Show explicit skeleton, empty, stale, partial, denied, and failure states.
- Avoid fake live activity, random charts, sample values presented as production truth, and nonfunctional buttons.
- Tables require stable sorting, filters, pagination, and clear empty states.
- Graphs/maps must support keyboard/fallback list navigation for accessibility.
- Destructive or service-affecting actions require a clear impact preview and confirmation.
- Prefer reusable UI primitives, semantic design tokens, and domain-specific components over duplicated page styling.

### 13.3 Visual direction

**Industrial Operations Cockpit:** high information density with intentional hierarchy, restrained high-contrast accent colors, precise typography, compact panels, strong topology/map visualization, clear status semantics, and accessible light/dark modes. Visual effects must never reduce legibility, contrast, or operator speed.

---

## 14. Reliability and non-functional requirements

Targets are established after baseline workload tests. Initial engineering requirements:

- Bounded memory, concurrency, polling, message size, query range, and local spool.
- Graceful shutdown and restart recovery for API, workers, and collectors.
- Idempotent event consumers and retry-safe API operations.
- Backpressure and explicit overload behavior.
- Timeouts and bounded retries for all external protocol calls.
- Health/readiness/liveness endpoints separated by responsibility.
- Structured logs with correlation/request IDs.
- Metrics for API, DB, queue, event pipeline, TSDB, collectors, and automation.
- Distributed traces across request/job/event boundaries where supported.
- Backup/restore runbooks and tested restore procedure.
- Database migrations are reviewed, backward-aware, and tested on a representative dataset.
- Load tests must use realistic device counts, poll intervals, series cardinality, event burst, and tenant distribution.
- Do not claim carrier-grade HA until deployment, failover, data recovery, and upgrade behavior are tested.

---

## 15. Deployment model

### 15.1 First production shape

- Next.js web container.
- Loco API container.
- Loco worker process (same artifact, worker mode).
- Rust collector container(s), centrally or at POPs.
- PostgreSQL.
- VictoriaMetrics.
- NATS JetStream.
- S3-compatible object storage (external or optional local deployment).
- Reverse proxy with TLS.

Use Docker Compose for reproducible development and initial self-hosted deployment. Provide separate profiles for local development and production-like deployment. Secrets must be injected securely, not committed in Compose files.

### 15.2 Remote POP collector

- Outbound-only secure connection to central ingestion/control endpoint by default.
- Tenant, site/POP, collector identity, version, and capability registration.
- Local bounded spool and replay.
- Configurable collection policy and update mechanism.
- Signed/versioned release artifacts and controlled upgrade/rollback.
- Collector cannot widen its scope or permissions from commands received from the center.

### 15.3 Backup and disaster recovery

Back up PostgreSQL, object-store artifacts/config snapshots, and service configuration. TSDB backup/retention policy is documented separately. Test restore into a clean environment; backup completion without restore verification is not accepted.

---

## 16. Test strategy

- Unit tests: domain invariants, state machines, rule evaluation, parsers, authorization policies.
- Integration tests: Loco routes/services + PostgreSQL; worker idempotency; NATS consumer redelivery; TSDB query boundaries.
- Contract tests: each protocol adapter and integration port; schema compatibility.
- Simulation tests: virtual devices, malformed responses, timeouts, partial failures, disconnect/reconnect, event bursts.
- Hardware-in-the-loop: selected real vendor/model/firmware capability matrix before declaring production support.
- Frontend tests: critical workflows, permission visibility, freshness and error states, keyboard/accessibility checks.
- End-to-end: discover → monitor → alert → correlate → incident → approve action → execute → verify.
- Security tests: tenant isolation, role boundaries, credential redaction, collector identity, and action authorization.
- Performance tests: API query profiles, polling fan-out, event ingestion, graph traversal, TSDB cardinality, and backpressure.

No feature is complete based only on a successful local build.

---

## 17. Delivery roadmap and dependency gates

### Phase 0 — Product and engineering foundation
- Narrative, blueprint, issue workflow, repository standards, architecture decisions, acceptance policy.
- Gate: one documented source of truth and repeatable local setup plan.

### Phase 1 — Application foundation
- Rust workspace, Loco API, Next.js app, shared contracts, PostgreSQL, migrations, auth/RBAC baseline, CI, Compose.
- Gate: a developer can start the platform locally, authenticate, and pass smoke tests.

### Phase 2 — Organization and inventory
- Tenancy, users/roles, sites/POPs, device and interface inventory, credential references, audit.
- Gate: scoped CRUD and authorization tests pass.

### Phase 3 — Collector and observability foundation
- Collector identity, ICMP/SNMP initial adapters, normalized envelopes, NATS pipeline, TSDB writes/queries, freshness.
- Gate: simulated devices generate trustworthy history and stale/unknown state.

### Phase 4 — Topology and fault operations
- Declared/discovered topology, graph projection, event rules, alerts, maintenance, correlation, incidents.
- Gate: simulated upstream fault yields explainable affected-dependency view and one correlated incident.

### Phase 5 — ISP service and fiber
- RADIUS/DHCP integration, session/subscriber reference, service dependency, FTTH inventory, optical metrics, SLA baseline.
- Gate: supported integration can show evidence-backed service/customer impact.

### Phase 6 — Configuration and automation
- Configuration snapshots/diff, action catalog, policy, approval, workflow runtime, verification, rollback.
- Gate: sandbox/simulator change completes with full audit and demonstrated rollback path.

### Phase 7 — Intelligence and production hardening
- Capacity analytics, anomaly insights, AI-assisted summaries, HA profiles, restore drills, performance/security validation, production docs.
- Gate: measurable operational readiness with documented workload envelope and recovery behavior.

Dependencies are strict: service-impact analytics depends on validated inventory/topology/service relations; automated remediation depends on action policy and audit; AI summaries depend on trustworthy evidence APIs.

---

## 18. Explicit non-goals for the initial product

- Billing, payment collection, full CRM, sales, and ERP.
- Replacing all vendor EMS/NMS systems.
- Arbitrary command runner or general-purpose remote terminal.
- Universal adapter coverage for every hardware/firmware combination.
- Full digital-twin simulation of every routing/protocol behavior.
- Autonomous AI change execution.
- Premature microservices, graph database, Kubernetes-only deployment, or mandatory Redis cache.
- Fixed performance/SLA promises before workload benchmarks.

These may be considered later as separately scoped integrations or product expansions.

---

## 19. Definition of Done

A task is complete only when all applicable checks are satisfied:

- [ ] Scope matches this blueprint and has one bounded deliverable.
- [ ] Domain/API/UI ownership is clear and no duplicate source of truth was introduced.
- [ ] Authorization and tenant scoping are enforced server-side.
- [ ] Validation, errors, loading/stale/empty states are handled.
- [ ] Tests cover happy path, invalid input, failure path, and relevant security boundary.
- [ ] Logs/metrics/traces are adequate for operational diagnosis.
- [ ] Documentation and examples match actual behavior.
- [ ] No credentials, real customer data, or production network secrets are committed.
- [ ] CI checks pass; migrations and rollback implications are reviewed.
- [ ] Acceptance criteria in the linked issue are checked with evidence.

## 20. Change control for this blueprint

A change to product domains, framework choices, event/data ownership, tenant model, or safety boundaries requires:
1. Explain the evidence and problem.
2. Identify impacted modules, issues, contracts, and migrations.
3. Update this blueprint first (or in the same change).
4. Update affected issues and dependencies.
5. Add compatibility, test, and rollout notes.

Implementation issues may refine details but must not silently contradict this document.
