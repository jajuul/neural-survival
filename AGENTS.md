## General Info
- Work in the Rust programming language primarily

## Architecture and Code Guidelines
- Keep designs simple and maintainable.
- Prefer composition over inheritance.
- Write code in a way that supports straightforward unit testing.

## Reviewing a ticket

- Run mutation testing on the ticket's changes: `git diff <base> -- crates/*/src > <diff>`, then `cargo mutants --workspace --in-diff <diff>` (install with `cargo install --locked cargo-mutants`). Done when every missed mutant is either caught by a new test or recorded as untestable for now on the ticket that will make it testable.


## Agent skills

### Issue tracker

Issues and specs live as local markdown files under `.scratch/<feature-slug>/`. See `docs/agents/issue-tracker.md`.

### Triage labels

Default vocabulary: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: one `CONTEXT.md` and `docs/adr/` at the repo root. See `docs/agents/domain.md`.
