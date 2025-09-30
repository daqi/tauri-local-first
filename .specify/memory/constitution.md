<!--
Sync Impact Report
Version change: N/A → 1.0.0
Modified principles: (initial adoption)
Added sections: Core Principles, Additional Constraints & Standards, Development Workflow & Quality Gates, Governance
Removed sections: None
Templates requiring updates:
 - .specify/templates/plan-template.md (Constitution version & gates) ✅
 - .specify/templates/spec-template.md (no direct version reference) ✅ (no change needed)
 - .specify/templates/tasks-template.md (no direct version reference) ✅ (no change needed)
Follow-up TODOs: None
-->

# Tauri Local-First Suite Constitution

## Core Principles

### 1. Local-First & Offline by Default

All applications MUST function fully without network connectivity unless a feature explicitly requires remote sync. Any optional online/sync capability MUST be an opt-in user action and clearly isolated. Persistent data MUST reside locally (e.g., SQLite, file system) and remain exportable in open formats. No hidden telemetry, analytics, or background network requests are permitted. Rationale: Guarantees privacy, portability, and predictable performance on constrained systems.

### 2. Minimal Footprint & Performance Efficiency

Each app (and shared crate/package) MUST justify every dependency. Large meta-frameworks, unused UI kits, and redundant libraries are prohibited. Baselines: cold start < 300ms UI interactive target, memory idle baseline kept minimal (prefer < ~80MB per app in debug where feasible), binary/install footprint minimized by stripping unused features and enabling Tauri's allowlist. Performance regressions MUST include a measurable explanation and a remediation issue before merge. Rationale: Sustains a snappy multi-tool environment and differentiates from heavier Electron alternatives.

### 3. Composable Multi-App Architecture

The workspace MUST organize capabilities into reusable Rust crates (`crates/*`) and UI component packages (`packages/ui`) instead of duplicating logic inside app folders. Cross-app interaction MUST prefer (a) in-process event bus, (b) deeplink (`<scheme>://`), (c) command-line / protocol handler—in that order. Shared contracts (descriptor `tlfsuite.json`, deep link formats, event payloads) MUST remain backward compatible or declare a breaking change plan. Rationale: Encourages modular evolution and reduces coupling.

### 4. Least-Privilege Security & Auditable IPC

Tauri capabilities/allowlist MUST enable only APIs actually used. Any operation requiring elevated permissions (e.g., modifying `/etc/hosts`) MUST present an explicit confirmation UI and log the action (where logging exists). IPC commands MUST validate all inputs (types, ranges, path constraints) and reject on ambiguity. No dynamic code execution (eval, Function constructor) is allowed. Rationale: Minimizes attack surface and accidental damage.

### 5. Testable Modular Reuse & Type Safety

New logic that can be isolated MUST launch as a focused crate/module with unit tests. Critical user flows MUST have integration tests (hosts rule toggle, descriptor discovery, deep link routing). TypeScript MUST run in strict mode; Rust crates MUST compile with warnings treated as errors in CI. Before implementing a feature, a minimal test or contract artifact SHOULD exist when practical; skipping MUST be justified in the PR description. Rationale: Maintains reliability as surface area grows.

## Additional Constraints & Standards

Technology Stack: Tauri v2, Rust stable (workspace), React 18 + Radix UI, pnpm workspaces for JS/TS, Cargo workspace for Rust. Feature flags and lazy-loading MUST be used to keep startup lean. All scripts MUST use `pnpm` (no `npm`/`yarn`).

Configuration: Shared ESLint + Prettier + TypeScript strict config; Clippy MUST pass (no warnings) for production crates. Build scripts MUST NOT perform network fetches beyond dependency resolution.

Binary & Resource Policy: Bundle only assets actually referenced. Large assets (>200KB) require justification. Icons and descriptors MUST follow documented discovery precedence.

Inter-App Contracts: `tlfsuite.json` MUST include stable `id`, optional `scheme`, `actions` with argument definitions. Deep links MUST be deterministic and URL-encoded. Event bus payloads MUST be typed (TypeScript interface or Rust struct) and versioned if shape changes.

Versioning: Semantic Versioning for published crates/packages. Breaking interface changes (deep link format, descriptor fields, IPC command signature) MUST increment MAJOR and document migration steps in the PR.

Observability (Lightweight): Prefer deterministic reproduction over heavy logging. Where logging exists, it MUST be structured (JSON or key=value) and omit sensitive data. Debug-only instrumentation MUST be gated/compiled out for release.

Internationalization & Accessibility: UI components MUST be keyboard navigable (Radix primitives) and not block future i18n (no hard string concatenation that hinders translation).

## Development Workflow & Quality Gates

Pull Request Requirements:

- Checklist MUST confirm: Principles unaffected or justified deviations listed.
- Added dependencies MUST include a one-line justification in the PR body.
- Features touching security-sensitive paths (file system mutations, elevated operations) MUST include at least one test (unit/integration) or explicit TODO issue reference.

Quality Gates (CI):

- TypeScript: no `any` introduced without `// justify:` comment.
- Rust: `cargo clippy -- -D warnings` passes; tests run (`cargo test`).
- Build: `pnpm build` for affected apps completes without size regressions >10% (otherwise flagged).
- Lint: ESLint & Prettier consistency.

Testing Categories:

- Unit: Pure functions / crate internals.
- Integration: Cross-layer scenarios (e.g., rule enabling modifies hosts file abstraction).
- Contract: Descriptor discovery, deep link parsing, IPC command validation.

Documentation:

- README high-level principles kept in sync (no principle drift).
- New deep link or descriptor fields documented in `docs/CONVENTIONS.md`.

Release & Version Bumps:

- Use Changesets (or equivalent) summarizing user-visible changes.
- Security-relevant changes MUST note threat mitigation rationale.

Deviation Handling:

- Any intentional breach of a principle MUST add a "Deviation" section in PR description with mitigation plan or sunset date.

## Governance

Authority: This Constitution supersedes ad-hoc practices. Conflicts resolve in favor of explicit text herein.

Amendments: Proposed via PR modifying this file. PR MUST include: (a) rationale, (b) impact on existing apps/crates, (c) migration/transition notes if breaking. A maintainer review + one additional contributor approval required.

Versioning of Constitution: Semantic versioning applied:

- MAJOR: Removal or fundamental redefinition of a principle or governance rule.
- MINOR: Addition of a new principle, section, or materially expanded guidance.
- PATCH: Clarifications, wording refinements, non-behavioral corrections.

Review Cadence: Quarterly (or earlier if friction emerges) review to assess continued relevance. Each review MUST either log "No Change" or produce an amendment PR.

Compliance: Each PR reviewer MUST check for principle adherence. If uncertain, label `needs-constitution-review` before merge.

Change Log: Rely on Git history for diff context; no separate manual changelog maintained inside this file.

Enforcement: Repeated unapproved deviations may trigger a follow-up refactor issue before additional features accepted in that area.

**Version**: 1.0.0 | **Ratified**: 2025-09-30 | **Last Amended**: 2025-09-30
