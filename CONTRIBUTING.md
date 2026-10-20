# Contributing

Neural Survival evolves neural-network creatures in a 2D world; `CONTEXT.md` defines the domain terms. Code is written by AI agents and approved by a human, using a review workflow built so the human can approve without reading every line.

## Setup

Once per machine:

- **Rust** via rustup. The version is pinned in `rust-toolchain.toml`, and rustup installs it, with rustfmt and Clippy, on the first `cargo` command. Bump it deliberately, as its own Light change, so new Clippy lints show up there.
- **cargo-mutants**, for mutation testing:
  ```bash
  cargo install --locked cargo-mutants
  ```
- **OpenSpec CLI** (needs Node.js):
  ```bash
  npm install -g @fission-ai/openspec
  ```
  Then run `openspec config profile` and make sure the `verify` workflow is selected.
- **Claude Code** implements and **Codex CLI** reviews; install both.
- **Matt Pocock's skills** for Codex, from [mattpocock/skills](https://github.com/mattpocock/skills). Step 5 needs their `code-review`:
  ```bash
  codex plugin marketplace add mattpocock/skills
  codex plugin add mattpocock-skills@mattpocock
  ```
  The planning skills (`grill-with-docs`, `to-spec`, `to-tickets`) are optional for Claude Code: `claude plugin install mattpocock-skills@claude-plugins-official`. Install them only one way per agent, or every skill shows up twice. This repo already ran `/setup-matt-pocock-skills`; its choices are in `docs/agents/`.
- Optional: **cargo-deny** (`cargo install --locked cargo-deny`, plus a `deny.toml`) for the dependency audit. The review gate reports it SKIPPED until both exist. The **Archify** skill, for diagrams in walkthroughs.

Once per clone, enable the pre-commit hook (formatting, Clippy, tests):

```bash
git config core.hooksPath scripts/git-hooks
```

## How a change is made

Every change follows [`docs/agents/review-workflow.md`](docs/agents/review-workflow.md). In short:

| Step | Who | What | Produces |
|---|---|---|---|
| 0. Classify | Claude, you confirm | Light, Standard, or Full, by risk | `Tier:` line |
| 1. Specify | Claude, you approve | OpenSpec change for the ticket, checked against the requirements checklist | `openspec/changes/<change>/` |
| 2. Acceptance tests | Codex, you approve | Tests from the spec alone, before any code | `acceptance_*.rs`, `acceptance-approved` |
| 3. Implement | Claude | `/opsx:apply`, then commit | the code |
| 4. Gate | Claude | `scripts/review-gate.sh <base> <change>` | `review/gate.md` |
| 5. Review | Codex | `code-review` (standards, spec) and `codex review` (bugs) | `review/code-review.md` |
| 6. Verify | Codex | `verify-evidence` skill | `review/verification.md` |
| 7. Walkthrough | Codex | `architecture-walkthrough` skill | `review/walkthrough.md` |
| 8. Approve | You | Read what the walkthrough lists, then `/opsx:archive` | resolved ticket |

Light changes (no behaviour change) skip to steps 3, 4, and 8. Any correction reruns from step 4. Start your review at `review/walkthrough.md`; it summarises the rest.

## What only you decide

Agents propose; the human approves and edits these:

- the tier, the spec, and the acceptance tests (`acceptance-approved`)
- `docs/agents/always-read.txt`: code you read in full whenever it changes
- `accepted-mutants`: surviving mutants you accept as untestable for now

## Where things live

| Path | Contents |
|---|---|
| `.scratch/<feature>/` | Roadmap spec and tickets; see `docs/agents/issue-tracker.md` |
| `openspec/specs/` | Current behaviour, each scenario linked to its test |
| `openspec/changes/` | One change per ticket in progress, with its `review/` record |
| `docs/adr/` | Architecture decisions |
| `AGENTS.md` | Rules every agent follows (Claude Code reads it through `CLAUDE.md`) |
| `scripts/` | The review gate and its checks, mutation testing, the pre-commit hook |
