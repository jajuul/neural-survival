# Review workflow

How a change gets from idea to approval without the human reading every line. The human's trust rests on checks they or a script can confirm: an approved spec, approved acceptance tests, a gate that records real exit codes, and a short list of code to read.

Claude Code implements. Codex writes the acceptance tests and does all reviewing, each step in a fresh session, so the model that wrote the code never grades it.

## Tiers

Every change gets a tier in step 0. Classify by risk, not size.

| Tier | When | Steps |
|---|---|---|
| Light | No behaviour change: refactors, renames, docs, formatting | 3, 4, 8. No OpenSpec change. |
| Standard | Behaviour changes inside existing component boundaries | All. The design can be brief; at least one acceptance test per behaviour change. |
| Full | New components, changed interfaces between modules or crates, dependency changes, or any path in `docs/agents/always-read.txt` | All, in full. |

`scripts/review-gate.sh` raises the tier from the diff and fails when the declared tier is too low: always-read paths and any `Cargo.toml` or `Cargo.lock` change require Full; removed or changed assertions in a Light change require Standard. The tier only ever goes up.

## Steps

**0. Classify** (Claude, the human confirms). Propose a tier with one line of reasoning.

**1. Specify** (Claude, the human approves). `/opsx:propose`, named `<NN>-<slug>` after the ticket in `.scratch/`. `proposal.md` carries `Tier: <tier>` and `Base: <commit>`, the commit the change starts from. Every scenario is concrete enough to test.

**2. Acceptance tests** (Codex, fresh session; the human skims and approves). From the delta specs only, not from any implementation, write tests in `tests/acceptance_<NN>_<slug>.rs` or `crates/core/tests/acceptance_<NN>_<slug>.rs`. They may call functions that don't exist yet. Link each scenario with `<!-- verified-by: <path>::<test_fn> -->`. Commit them (the pre-commit hook only format-checks a commit of OpenSpec files and acceptance tests), then record the approval in a second commit as `openspec/changes/<change>/acceptance-approved`:

```
commit <sha of the acceptance-test commit>
crates/core/tests/acceptance_<NN>_<slug>.rs
```

**3. Implement** (Claude). `/opsx:apply`. Never edit an approved acceptance test or `acceptance-approved`; if one is wrong, stop and tell the human. Add further tests freely in other files, linked the same way. End with a commit: review covers committed work only.

**4. Gate** (Claude runs it; the human can rerun it). `scripts/review-gate.sh <base> <change>` (Light: `scripts/review-gate.sh <base>`). It runs formatting, Clippy, tests, architecture checks, OpenSpec validation, the dependency audit, traceability, the acceptance-test lock, the tier check, and mutation testing, and writes `review/gate.md`. Fix failures and rerun until it passes. Surviving mutants follow the rule in `AGENTS.md`.

**5. Review** (Codex, fresh session). Both reviews against `<base>`, saved together as `openspec/changes/<change>/review/code-review.md`:
- `code-review`, Matt Pocock's skill, with fixed point `<base>` and the change folder as the spec: coding standards and spec compliance, including scope creep.
- `/review`, Codex's built-in review: bugs, edge cases, and error handling.

**6. Verify** (Codex, fresh session). The `verify-evidence` skill writes `review/verification.md`: the requirements-to-evidence matrix and what could not be established. It records evidence; it does not re-judge step 5.

**7. Walkthrough** (Codex). The `architecture-walkthrough` skill writes `review/walkthrough.md`: the structure, ADR conformance, the always-read paths to read in full, and at most three further places to inspect.

**8. Approve** (human). Read what step 7 lists. Any correction goes back to Claude and the workflow reruns from step 4. Once approved, `/opsx:archive` the change and resolve its ticket as `docs/agents/issue-tracker.md` describes.

## Review output

Everything under `openspec/changes/<change>/review/` belongs to that one change and is archived with it. It is a record of one review, not documentation to keep up to date.
