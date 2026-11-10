# Engineering & Design Rules

This document is mandatory for human and AI contributors. It operationalizes `docs/MASTER_BLUEPRINT.md`; it does not create an alternative architecture.

## 1. Execution discipline

1. Read README, product narrative, master blueprint, and the linked issue before editing.
2. Work only within the issue's declared scope. If an adjacent need is discovered, create a separate issue and link it; do not silently expand scope.
3. Inspect existing code and tests before adding files. Reuse an existing abstraction when it matches the contract.
4. Confirm dependencies are complete before implementation. Do not bypass an unmet dependency with temporary duplicate logic.
5. Make the smallest coherent change that satisfies acceptance criteria.
6. Run focused tests first, then relevant workspace checks.
7. Report commands run, results, files changed, known gaps, and evidence. Never claim an unchecked acceptance item is complete.
8. Do not close an issue until its acceptance checklist is satisfied and evidence is recorded in the PR/issue.

## 2. Ownership and boundaries

- Next.js is presentation/client experience. Rust/Loco owns authoritative business rules and authorization.
- Loco controllers handle transport; application services coordinate use cases; domain modules own invariants; adapters own external protocol details.
- SeaORM entities are persistence implementation, not cross-layer contracts.
- Frontend consumes generated/typed API contracts. Do not duplicate backend domain rules in TypeScript.
- Collectors produce normalized observations; they do not mutate business inventory or incident state directly.
- Telemetry samples, discrete events, alerts, and incidents remain separate concepts.
- Declared inventory and discovered/observed facts must retain provenance and must not overwrite one another silently.
- Every tenant-owned operation, background job, event, and collector identity carries tenant/org scope.

## 3. Rust rules

- Use workspace-managed dependency versions and shared lint configuration.
- Prefer explicit types, typed errors, and small modules with one responsibility.
- Avoid `unwrap()`, `expect()`, and panic paths in runtime code; justify any exception.
- Avoid unbounded task spawning, unbounded channels, unbounded retries, and unbounded local queues.
- External I/O must have timeout, cancellation/shutdown behavior, and bounded retry policy.
- Do not block Tokio executor threads with synchronous I/O or CPU-heavy work.
- Do not leak secrets or raw customer/device payloads into logs.
- Use stable IDs and UTC timestamps; preserve source timestamp separately from ingestion timestamp.
- Public event/API contracts require explicit versioning and compatibility tests.
- Domain logic must be testable without live hardware.

## 4. Loco.rs rules

- Use Loco conventions and supported extension points; pin and verify the actual framework version.
- Controllers are thin: authenticate, parse/validate transport input, call use case, map result to response.
- Put authorization at the application/use-case boundary, not only in route middleware or UI.
- Keep workers idempotent and arguments small, serializable, and version-aware.
- Use Loco workers for finite business jobs; do not use them as a raw telemetry sample bus.
- Scheduled jobs must be safe to run more than once and must record their scope/result.
- Migrations must be reproducible, reviewed, and tested on a clean database.
- Never expose credential fields in generated model responses or generic serialization.

## 5. Next.js rules

- Use App Router and TypeScript strict mode.
- Server components are the default; client components are only for browser interactivity/realtime/visualization.
- No direct DB, network-device, NATS, or TSDB access from the web app.
- All mutations go through the Loco API and are authorized there.
- Use a typed API client generated from the authoritative contract.
- Use reusable UI primitives, semantic design tokens, consistent forms/tables/filters, and accessible status semantics.
- Every data screen handles loading, empty, stale, partial, error, and forbidden states.
- Never present mock/sample values as live production data.
- Destructive or service-impacting actions require a preview, explicit confirmation, and clear result state.
- Preserve URL state for meaningful filters and detail navigation.

## 6. Network protocol and collector rules

- Each adapter declares protocol, supported operations, device/vendor/model/firmware capability, authentication mode, and known limitations.
- Prefer read-only discovery/monitoring in initial adapters.
- Use per-target timeout, interval, jitter, concurrency cap, retry/backoff, and rate limit.
- SNMP implementations must support SNMPv3 where device capability supports it; credentials are secret references, not inline configuration strings.
- Treat device responses as untrusted input. Bound packet size, parser work, and response cardinality.
- Normalize vendor payloads at the adapter boundary; retain raw payload only when policy and retention explicitly allow it.
- A collector must report freshness, collection errors, buffer depth, dropped events, and last contact.
- Store-and-forward spool is disk bounded, age bounded, and replay-safe.
- Remote collector communication is authenticated and outbound-only by default.
- No arbitrary shell execution. SSH adapter operations are allowlisted and typed.

## 7. Security and privacy rules

- Deny by default; grant least privilege.
- Enforce tenant scoping server-side on reads, writes, exports, subscriptions, and background work.
- Do not trust client-supplied tenant IDs without membership authorization.
- Secrets are write-only via ordinary API, encrypted at rest, rotated, and never included in logs, traces, errors, or UI state.
- Audit permission changes, exports, credential access metadata, configuration changes, approvals, and execution.
- Sanitize untrusted device labels, syslog text, hostnames, and imported records before rendering.
- Apply query limits and rate limits to expensive graph, metric, export, and search endpoints.
- Security tests must include cross-tenant access and horizontal/vertical privilege escalation.

## 8. Data and event rules

- PostgreSQL is the operational source of truth.
- TSDB stores numeric time-series data; PostgreSQL stores metadata, policy, workflow, inventory, and incident state.
- NATS transports durable events; it is not the source of truth.
- Event envelopes require event ID, schema version, tenant scope, source, occurrence time, ingestion time, and typed payload.
- Consumers must be idempotent and handle duplicate delivery and replay.
- Time-series labels must have bounded cardinality; no free-form strings or unbounded subscriber IDs as labels.
- Missing data is not zero. Staleness and unknown are explicit.
- Retention is policy-driven by data class and tenant plan.

## 9. UX and visual rules

- Design for fast triage and safe operation, not decorative novelty.
- Keep severity, freshness, ownership, and action state visually distinct.
- Do not rely on color alone; include text/icon/shape semantics.
- Provide useful keyboard focus, accessible labels, contrast, and graph/map alternatives.
- Dense screens still need alignment, grouping, whitespace, and predictable interactions.
- No fake charts, random activity, dead buttons, or placeholders presented as completed features.
- Light/dark themes must preserve semantic meaning and contrast.

## 10. Testing and review

- Unit test domain invariants and state transitions.
- Integration test Loco + PostgreSQL and worker idempotency.
- Contract test adapters and event schemas.
- Simulation test device timeout, malformed response, disconnection, duplicate event, backlog, and recovery.
- Test frontend loading/error/stale/forbidden states for critical flows.
- Test authorization with both allowed and denied cases.
- For changes affecting migrations, event contracts, or action execution, include upgrade/rollback/compatibility notes.
- For vendor protocol claims, provide simulator or hardware evidence and a capability record.
- CI must pass; no ignored failing test can be counted as acceptance evidence.

## 11. Pull request and issue etiquette

- One issue should map to one focused PR where practical.
- Reference issue numbers in PR title/body and use closing keywords only when complete.
- No unrelated formatting churn or drive-by refactors.
- Explain architecture changes and update blueprint/contract/docs when needed.
- Include screenshots for UI changes and sanitized logs/fixtures for protocol changes.
- Keep main branch releasable; do not force-push shared branches.
- Avoid duplicated TODO files or parallel project plans; GitHub issues are the execution backlog.

## 12. Done means verified

A completed task has:
- code/docs matching the approved scope,
- tests for the acceptance path and relevant failure/security paths,
- successful relevant CI,
- updated documentation/contracts where needed,
- no unresolved security or data-integrity issue,
- a concise evidence report linked to the issue.

If an acceptance item cannot be tested yet, mark it blocked and create/link the specific prerequisite issue. Do not check it merely because code compiles.
