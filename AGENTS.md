## General Info
- Work in the Rust programming language primarily

## Architecture and Code Guidelines
- Keep designs simple and maintainable.
- Prefer composition over inheritance.
- Write code in a way that supports straightforward unit testing.

## Reviewing a ticket

- Ticket work stacks on the current branch; `master` is stale. Diff a ticket against the commit before its first commit.
- Run mutation testing on the ticket's changes with `scripts/mutants-ticket.sh [<base>]`. The base defaults to HEAD, which covers uncommitted work. Done when every missed mutant is either caught by a new test or recorded as untestable for now on the ticket that will make it testable.

## Review workflow

Every change follows `docs/agents/review-workflow.md`: Claude Code implements, Codex writes acceptance tests and reviews. Rules that hold in every step:

- Classify each change Light, Standard, or Full before starting, and let the human confirm.
- Never edit an approved acceptance test or `acceptance-approved`. If one looks wrong, stop and ask.
- Never edit `docs/agents/always-read.txt`; suggest entries instead.
- Commit before review. `scripts/review-gate.sh` checks committed work only; report its results as recorded, never as you expect them.
- Review in a fresh session, never in the session that wrote the code.
- Never weaken a check, architecture constraint, or test to make it pass without the human's approval.

## Agent skills

### Issue tracker

Issues and specs live as local markdown files under `.scratch/<feature-slug>/`. See `docs/agents/issue-tracker.md`.

### Triage labels

Default vocabulary: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: one `CONTEXT.md` and `docs/adr/` at the repo root. See `docs/agents/domain.md`.
