# Product Narrative — e-ISP

## 1. Product identity

**Product:** e-ISP  
**Category:** Network Operations, Service Assurance & Automation Platform  
**Audience:** Internet Service Providers, FTTH/fiber operators, telecom operators, wholesale bandwidth providers, MSPs, and enterprise network teams.  
**Tagline:** **Know the network. Protect the service. Automate the operation.**

e-ISP is a unified operational platform for understanding and operating the networks that deliver connectivity. It brings network assets, physical and logical relationships, live telemetry, incidents, subscriber/service context, and controlled automation into one coherent system.

It replaces fragmented operational visibility—separate device lists, graphs, spreadsheets, ticket queues, and ad-hoc scripts—with an evidence-led operational model.

## 2. The problem

Network teams commonly work across disconnected tools and incomplete records:

- Device monitoring says an interface is down, but does not explain the affected services.
- Inventory records do not match what is actually deployed.
- Topology is incomplete or maintained manually.
- Hundreds of downstream alarms obscure one upstream failure.
- Subscriber complaints are handled without a shared view of access, aggregation, and core network health.
- Configuration changes are performed through scripts or terminal sessions without consistent approval and verification.
- Capacity decisions are made from snapshots rather than historical trends and forecasts.

The result is slow diagnosis, repeated work, uncertain customer impact, and avoidable operational risk.

## 3. The promise

e-ISP gives an operator a connected answer to five questions:

1. **What do we have?** — devices, interfaces, sites, POPs, links, circuits, addresses, subscribers, and services.
2. **What is happening?** — health, telemetry, events, alarms, routing changes, and degradation.
3. **Why is it happening?** — topology-aware correlation, evidence, and probable root cause.
4. **Who or what is affected?** — service dependency, subscriber/circuit mapping, geography, and SLA exposure.
5. **What can we safely do?** — diagnostics, approved changes, remediation, verification, and rollback.

## 4. Product principles

### Network-first
The network is the primary operational model. Devices, interfaces, links, sites, POPs, protocols, and observed state are explicit domain objects.

### Service-aware
Network state must be traceable to service delivery. A device alert alone is not a complete operational outcome; the platform should identify the affected circuit, subscriber, customer, or service when the data model supports that relationship.

### Automation-driven, not automation-at-any-cost
Automation is a controlled operational capability. Actions must be scoped, authorized, approved where required, recorded, verified, and recoverable.

### Evidence before assertion
A diagnosis must show source observations, timestamps, affected dependencies, and confidence/limitations. The platform must distinguish confirmed facts from inferred causes.

### Vendor-neutral by contract
Vendor adapters implement capability contracts. A protocol or vendor is not considered supported merely because a generic connectivity method exists.

### Edge-capable by design
Remote POP collectors must continue useful local collection during temporary central connectivity loss, buffer safely, and forward data after recovery.

### Operational simplicity
Start with a modular Loco.rs application and separately deployable Rust collector processes. Split services only when reliability, scaling, security, or ownership requires it.

## 5. Product domains

1. **Command Center** — role-oriented operations overview, network status, active incidents, and service impact.
2. **Inventory & Source of Truth** — organizations, sites, POPs, devices, interfaces, hardware, credentials references, and observed-vs-desired inventory.
3. **Topology & Maps** — physical/logical links, discovered neighbors, service dependencies, and geospatial assets.
4. **Observability** — polling, telemetry ingestion, time-series queries, events, logs, and performance views.
5. **Fault & Incident Management** — alert lifecycle, deduplication, correlation, incidents, escalation, maintenance suppression, and post-incident records.
6. **ISP Service Assurance** — subscribers, access sessions, circuits, service health, SLA, and customer-impact mapping.
7. **Fiber / FTTH Operations** — OLT, PON, ONT/ONU, splitter hierarchy, optical metrics, routes, and field-work context.
8. **IPAM & Resource Management** — prefixes, addresses, VLANs, VRFs, ASNs, pools, and allocation history.
9. **Configuration & Change Management** — snapshots, diffs, templates, approvals, change windows, execution evidence, and rollback records.
10. **Automation & Orchestration** — typed workflows, policy checks, approvals, execution, retries, and verification.
11. **Traffic & Capacity Intelligence** — utilization, peaks, upstream/IX analysis, saturation risk, and capacity forecasts.
12. **Workforce & Operations** — work orders, maintenance, shift handover, dispatch, and operational reporting.
13. **Administration & Integrations** — tenant/organization boundaries, identity, RBAC, audit, API credentials, and integration lifecycle.

## 6. Primary users and outcomes

| Persona | Primary outcome |
|---|---|
| NOC operator | Detect, triage, correlate, and track incidents quickly |
| Network engineer | Inspect topology, routing, configuration, and device evidence |
| ISP operations lead | Understand service availability, customer impact, and SLA |
| Field technician | Receive accurate location, asset, and work-order context |
| Capacity planner | Identify growth, saturation risk, and investment needs |
| Platform administrator | Manage tenants, users, collectors, integrations, and policy |
| Executive / service owner | See trustworthy service and operational health summaries |

## 7. Operating experience

The interface is an operational cockpit, not a generic admin template:

- Information-dense but readable layouts for large screens.
- Persistent severity and freshness indicators.
- Clear distinction between live state, last-known state, stale data, and unknown state.
- Topology and map interactions that lead to relevant device/service detail.
- Search-first access to devices, sites, subscribers, incidents, and circuits.
- Role-aware dashboards and navigation.
- Light and dark themes with accessible contrast.
- Keyboard-friendly workflows and explicit confirmation for risky operations.

Visual direction: **industrial operations cockpit** — functional hierarchy, crisp typography, restrained high-contrast accents, compact data panels, map/topology focus, and no decorative effects that weaken readability.

## 8. What e-ISP is not

- Not a replacement for every vendor's element-management system (EMS).
- Not a billing/CRM/ERP suite in its first product scope.
- Not a generic log analytics platform.
- Not an unrestricted remote shell or arbitrary command runner.
- Not an AI system that autonomously changes production networks without controls.
- Not a promise of universal support for every device, firmware, or telecom protocol.

External OSS/BSS, billing, CRM, GIS, RADIUS, DHCP, and vendor EMS systems are integrated through explicit adapters and contracts where needed.

## 9. Success measures

Product success must be measurable. Initial measures include:

- Time from fault observation to actionable incident.
- Percentage of critical assets with verified inventory and topology.
- Percentage of incidents with identified affected services/subscribers.
- Alert deduplication/correlation effectiveness.
- Telemetry freshness and collector delivery reliability.
- Mean time to acknowledge and restore.
- Change success, rollback, and verification rates.
- Capacity forecast error over defined horizons.
- Tenant isolation and authorization test coverage.

Targets must be established from a measured baseline; do not invent universal SLO numbers before workload and deployment profiles are known.

## 10. Product narrative

**e-ISP turns network operations into a connected discipline: observe the infrastructure, understand its dependencies, protect the services running on it, and automate repeatable work under clear operational control.**

The network is the foundation. Service continuity is the outcome. Intelligence and safe automation are the force multipliers.
