## General Info
- Work in the Rust programming language primarily

## Architecture and Code Guidelines
- Keep designs simple and maintainable.
- Prefer composition over inheritance.
- Write code in a way that supports straightforward unit testing.

## Reviewing a ticket

- Ticket work stacks on the current branch; `master` is stale. Diff a ticket against the commit before its first commit.
- Run mutation testing on the ticket's changes with `scripts/mutants-ticket.sh [<base>]`. The base defaults to HEAD, which covers uncommitted work. Done when every missed mutant is either caught by a new test or recorded as untestable for now on the ticket that will make it testable.

## Agent skills

### Issue tracker

Issues and specs live as local markdown files under `.scratch/<feature-slug>/`. See `docs/agents/issue-tracker.md`.

### Triage labels

Default vocabulary: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: one `CONTEXT.md` and `docs/adr/` at the repo root. See `docs/agents/domain.md`.
