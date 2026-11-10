# GitHub Issue Execution Standard

This standard defines the issue format for the e-ISP backlog. Every implementation issue must be small enough for a local AI coding agent to understand and complete autonomously without making architecture decisions outside its scope.

## 1. Issue design principles

- One issue = one bounded deliverable = one verifiable outcome.
- Prefer small vertical slices over broad phase-sized implementation tickets.
- Each issue must say what to change and what must explicitly not change.
- Every issue includes specification, design rules, acceptance checklist, verification commands/expectations, and dependencies.
- Keep the blueprint as the architecture source of truth. Do not copy the whole blueprint into every issue; link to the relevant section.
- Do not create two issues that own the same files, API, schema, or behavior. Shared foundations must be completed first.
- If work is blocked, update the issue with evidence and create a narrowly scoped prerequisite issue rather than implementing a workaround.
- No issue may claim vendor/protocol capability without a capability contract and test evidence.

## 2. Required issue body structure

Use this Markdown structure:

### Goal
One sentence describing the outcome.

### Context
Why this matters and the relevant blueprint section.

### Scope
Explicit files/modules/API/UI behavior expected to change.

### Out of scope
Adjacent work that must remain untouched.

### Specification
Concrete inputs, outputs, states, invariants, and behavior. Include API/data/event contracts when applicable.

### Design rules
Non-negotiable implementation constraints, ownership, security, UX, and style rules.

### Dependencies
List prerequisite issue numbers. Use `None` only when truly independent. Dependencies are blocking, not merely related.

### Acceptance criteria
A checkbox list of observable outcomes. Include negative cases when relevant.

### Verification
Exact test categories/commands or evidence expected. Commands may be refined to actual repository scripts after the bootstrap issue creates them.

### AI execution notes
Read the README, narrative, blueprint, engineering rules, and this issue. Inspect existing code before editing. Make only scoped changes. Do not invent requirements or expand scope. Report files changed, tests run, outcomes, and remaining gaps.

## 3. Titles and grouping

- Prefix titles with a stable domain key such as `[FOUNDATION]`, `[SECURITY]`, `[INVENTORY]`, `[COLLECTOR]`, `[TELEMETRY]`, `[TOPOLOGY]`, `[INCIDENT]`, `[ISP]`, `[FIBER]`, `[AUTOMATION]`, `[UI]`, or `[OPS]`.
- Keep one action in the title: `[INVENTORY] Define device identity and lifecycle contract`.
- Use GitHub milestones/projects for phase planning when those features are configured; do not encode status in title.
- Link dependencies using issue numbers once the backlog has been created.
- The index/epic issue is an index only; implementation must occur in child issues.

## 4. Size and acceptance quality

A good issue can usually be implemented and reviewed in one focused work session. Split it when it combines multiple independent schemas, adapters, screens, or failure domains.

Acceptance criteria must be:
- observable (test, API response, UI behavior, schema/migration, or document artifact),
- binary enough to check,
- scoped to this issue,
- include failure/negative conditions when relevant,
- free from vague words such as "properly", "robust", "complete", or "user-friendly" unless measurable behavior follows.

## 5. Definition of Done

- [ ] All acceptance checkboxes are satisfied with evidence.
- [ ] Required tests pass and relevant CI is green.
- [ ] No unrelated scope or duplicate abstraction was introduced.
- [ ] Security and tenant boundaries are tested when applicable.
- [ ] Documentation/API contracts are updated when behavior changes.
- [ ] PR links the issue and summarizes evidence and known limitations.
- [ ] Issue is closed only after review/verification, not merely after code is written.
