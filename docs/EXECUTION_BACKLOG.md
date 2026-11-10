# Execution Backlog

**Master tracker:** [#110 — e-ISP execution backlog: 128 granular tasks](https://github.com/bjo163/e-isp/issues/110)

The master tracker contains the complete checklist and links to every implementation issue. This document is a durable repository-side entry point; it does not duplicate the detailed issue specifications.

## Delivery sequence

| Phase | Workstream | Issue range / IDs | Gate |
|---|---|---|---|
| 1 | Application foundation and UI system | [FND-01–13](https://github.com/bjo163/e-isp/issues/1) · [UI-01–05](https://github.com/bjo163/e-isp/issues/21) · [UI-11](https://github.com/bjo163/e-isp/issues/116) | Loco + Next.js boot, API contract, PostgreSQL, CI, Compose, shared UI |
| 2 | Tenant, security, inventory, IPAM, discovery, search | [SEC-01–07](https://github.com/bjo163/e-isp/issues/14) · [INV-01–07](https://github.com/bjo163/e-isp/issues/26) · [IPAM-01–05](https://github.com/bjo163/e-isp/issues/33) · [INV-05A](https://github.com/bjo163/e-isp/issues/142) · [DISC-01–02](https://github.com/bjo163/e-isp/issues/84) · [SRCH-01](https://github.com/bjo163/e-isp/issues/86) · [ADM-01–02](https://github.com/bjo163/e-isp/issues/117) · [INT-01](https://github.com/bjo163/e-isp/issues/119) | Tenant-safe operational source of truth |
| 3 | Collectors, protocol adapters, telemetry | [COL-01–13](https://github.com/bjo163/e-isp/issues/37) · [TEL-01–04](https://github.com/bjo163/e-isp/issues/47) · [BGP-01](https://github.com/bjo163/e-isp/issues/85) | Evidence flows from tested adapters into queryable history |
| 4 | Topology, alerts, incidents, live operations | [TOP-01–04](https://github.com/bjo163/e-isp/issues/51) · [ALR-01–03](https://github.com/bjo163/e-isp/issues/55) · [INC-01–05](https://github.com/bjo163/e-isp/issues/58) · [ALR-04](https://github.com/bjo163/e-isp/issues/127) · [UI-08](https://github.com/bjo163/e-isp/issues/90) · [UI-12](https://github.com/bjo163/e-isp/issues/128) | Explainable fault-to-incident and impact workflow |
| 5 | ISP service assurance, FTTH, workforce | [ISP-01–08](https://github.com/bjo163/e-isp/issues/59) · [FIB-01–06](https://github.com/bjo163/e-isp/issues/65) · [WORK-01–03](https://github.com/bjo163/e-isp/issues/97) | Subscriber/service and fiber context backed by verified integrations |
| 6 | Configuration, change management, automation | [CFG-01–06](https://github.com/bjo163/e-isp/issues/74) · [AUT-01–07](https://github.com/bjo163/e-isp/issues/78) | Governed action lifecycle with approval, verification, and rollback |
| 7 | Command center, intelligence, capacity, production readiness | [CMD-01](https://github.com/bjo163/e-isp/issues/87) · [UI-06–10](https://github.com/bjo163/e-isp/issues/88) · [ADM-03](https://github.com/bjo163/e-isp/issues/120) · [CAP-01–04](https://github.com/bjo163/e-isp/issues/93) · [AI-01](https://github.com/bjo163/e-isp/issues/100) · [OPS-01–09](https://github.com/bjo163/e-isp/issues/101) | Evidence-backed intelligence and verified production operations |

> The issue ranges above are convenient entry links; the master tracker is the authoritative complete checklist. Some task IDs are intentionally created in dependency order rather than grouped by numeric issue order.

## How to execute

1. Select an open, unblocked child issue from [the master tracker](https://github.com/bjo163/e-isp/issues/110).
2. Read [Product Narrative](PRODUCT_NARRATIVE.md), [Master Blueprint](MASTER_BLUEPRINT.md), [Engineering Rules](ENGINEERING_RULES.md), and [Issue Execution Standard](ISSUE_EXECUTION_STANDARD.md).
3. Inspect the actual repository state before editing; never assume a dependency is implemented because its issue is closed.
4. Implement only the selected issue's bounded scope and run its verification.
5. Link the PR, record command output/evidence, and close the issue only when every acceptance checkbox is satisfied.

## Backlog invariants

- One issue owns one bounded schema/API/adapter/UI behavior.
- Dependencies are blocking and must not be bypassed by duplicate abstractions.
- The blueprint owns architecture; issue text owns execution detail.
- Protocol/vendor support requires explicit capability and simulator or hardware evidence.
- Production readiness requires security, performance, restore, and runbook evidence.
