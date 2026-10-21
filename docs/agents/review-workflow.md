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

**1. Specify** (Claude, the human approves). `/opsx:propose`, named `<NN>-<slug>` after the ticket in `.scratch/`. `proposal.md` carries `Tier: <tier>` and `Base: <commit>`, the commit the change starts from.

Before asking for approval, Claude checks the delta specs against this list and reports each failure with the fix it proposes. The human approves against the same list:

- Every requirement has at least one scenario.
- Every scenario names concrete inputs or state in WHEN and an observable outcome in THEN: values, counts, files, or messages a test can check.
- No vague words a test can't measure: fast, robust, reasonable, appropriate, correctly, as expected.
- Scenarios describe behaviour, not implementation: no private types, functions, or data structures.
- Failure and edge cases are covered: invalid input, empty input, boundaries, wrap-around on the Torus.
- Terms match `CONTEXT.md`, and nothing contradicts an accepted ADR.
- The proposal says what is out of scope.
- A bug fix has the three scenarios step 2 needs.
- `design.md` has a **Test interface** section naming (a) the public functions, types, or CLI commands the acceptance tests will call, with their signatures, and (b) the seams Claude will test-drive in step 3. Approving the design approves both, so step 2 tests against an agreed interface and step 3 needs no second round of seam questions.

**2. Acceptance tests** (Codex, fresh session; the human skims and approves). First run the step 1 checklist again and report any scenario that can't become a test; it goes back to step 1 rather than being guessed at. Then, from the delta specs only, not from any implementation, write tests in `tests/acceptance_<NN>_<slug>.rs` or `crates/core/tests/acceptance_<NN>_<slug>.rs`. They call only the interface named in `design.md`'s Test interface section, never internals, with a few tests per scenario; that interface may not exist yet. If a scenario can't be tested through it, report that rather than reaching past it. Where a requirement states an invariant or a rule over many inputs (energy never negative, positions stay on the Torus, same seed gives the same result), write it as a `proptest` property rather than a few examples; add `proptest` as a dev-dependency the first time, which makes that change Full.

A bug fix gets three groups of tests:

- **Reproduces the bug**: fails on the base commit, for the reason the bug report gives. Run it there before approval to prove it.
- **Fix holds**: the corrected behaviour, including inputs near the one that triggered the bug.
- **Nothing else changed**: behaviour next to the bug that must stay as it was, so the fix can't trade one bug for another.

The first two can be one test when it fails before the fix and passes after; say so in the test's comment.

Link each scenario with `<!-- verified-by: <path>::<test_fn> -->`. Commit them (the pre-commit hook only format-checks a commit of OpenSpec files and acceptance tests), then record the approval in a second commit as `openspec/changes/<change>/acceptance-approved`:

```
commit <sha of the acceptance-test commit>
crates/core/tests/acceptance_<NN>_<slug>.rs
```

**3. Implement** (Claude). `/opsx:apply`, using Matt Pocock's `tdd` skill at the seams `design.md` lists: red → green one slice at a time, until the acceptance tests pass. The seams are already approved, so don't ask for them again; a new seam the design missed is a question for the human. Use `tdd`, not Matt's `implement`: `/opsx:apply` tracks the tasks, and the review comes in step 5. Never edit an approved acceptance test or `acceptance-approved`; if one is wrong, or implementing shows the Test interface is wrong, stop and tell the human, who approves a revised interface and tests. Add further tests freely in other files, linked the same way. End with a commit: review covers committed work only.

**4. Gate** (Claude runs it; the human can rerun it). `scripts/review-gate.sh <base> <change>` (Light: `scripts/review-gate.sh <base>`). It runs formatting, Clippy, tests, architecture checks, OpenSpec validation, the dependency audit, traceability, the acceptance-test lock, and the tier check, and writes `review/gate.md`. Fix failures and rerun until it passes.

Mutation testing is slow, so it runs only with `--mutants` (`scripts/review-gate.sh --mutants <base> <change>`); without it the check is recorded SKIPPED. Iterate without the flag. Run the gate once with `--mutants` on the final code before step 5: that is the `gate.md` the reviews read, and they treat a SKIPPED check as missing evidence.

A surviving mutant fails the gate unless the human has accepted it. When one genuinely can't be tested yet, Claude records it on the ticket that will make it testable (see `AGENTS.md`) and proposes it; the human adds it to `openspec/changes/<change>/accepted-mutants` and commits:

```
# <file>: <mutation as in mutants.out/missed.txt>  # <ticket that will make it testable>
crates/core/src/world.rs: replace Torus::wrap -> f32 with 0.0  # 06-predators-and-captures
```

Entries match on file and mutation, not line number. Accepted survivors stay listed in `gate.md`.

**5. Review** (Codex, fresh session). Both reviews against `<base>`, saved together as `openspec/changes/<change>/review/code-review.md`, which starts with `Reviewed: <short HEAD commit>` so later steps can tell whether it is current:
- `code-review`, Matt Pocock's skill, with fixed point `<base>` and the change folder as the spec: coding standards and spec compliance, including scope creep.
- Codex's built-in review, for bugs, edge cases, and error handling. `--base` takes a branch, so point one at the base commit first:

  ```bash
  git branch -f review-base <base>
  codex review --base review-base "Find bugs, edge cases, and error-handling gaps. Ignore style; another review covers it."
  ```

  It prints to stdout; append its output to `code-review.md` under `## Bugs`.

**6. Verify** (Codex, fresh session). The `verify-evidence` skill writes `review/verification.md`: the requirements-to-evidence matrix and what could not be established. It records evidence; it does not re-judge step 5.

**7. Walkthrough** (Codex). The `architecture-walkthrough` skill writes `review/walkthrough.md`: a summary of the whole review with links to the other review files, then the structure, ADR conformance, the always-read paths to read in full, and at most three further places to inspect. This is the file the human opens first.

**8. Approve** (human). Read what step 7 lists. Any correction goes back to Claude and the workflow reruns from step 4. Once approved, `/opsx:archive` the change and resolve its ticket as `docs/agents/issue-tracker.md` describes.

## Review output

Everything under `openspec/changes/<change>/review/` belongs to that one change and is archived with it. It is a record of one review, not documentation to keep up to date.
