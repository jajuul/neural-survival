---
name: verify-evidence
description: "Step 6 of the review workflow: build a requirements-to-evidence matrix for an OpenSpec change and state what could not be established. Use after the review gate and code review have run on a change, or when the user asks what evidence shows a change meets its spec."
---

Produce `openspec/changes/<change>/review/verification.md` for one OpenSpec change. The workflow is in `docs/agents/review-workflow.md`.

Your job is evidence, not judgement. Step 5's reviews already judged whether the code is right; you establish what proves each requirement holds, and say plainly where nothing does. Do not modify code, tests, or specs.

## Inputs

Ask for the change name if you weren't given it. Then read:

1. The change: `proposal.md`, `design.md`, `tasks.md`, and every delta spec under `specs/`.
2. `review/gate.md`, the review gate's recorded results. If it is missing or stale, stop and ask for `scripts/review-gate.sh <base> <change>` to be rerun. It is stale when a commit after the one its commit range ends at changes anything outside `openspec/changes/<change>/review/`: check that `git diff --name-only <that commit> HEAD -- . ':!openspec/changes/<change>/review/'` prints nothing. Commits that only add review files don't change the code the gate checked. Never rerun checks yourself and treat your run as the gate.
3. `review/code-review.md`, step 5's findings. If missing, say so in the report and continue.
4. `acceptance-approved`, which lists the human-approved acceptance tests.
5. If the `openspec-verify-change` skill is installed, run it on the change and use its output as one more input.

## Statuses

Give each requirement exactly one:

- **Verified**: every scenario has a `verified-by` link, the gate's Traceability and Tests checks both PASSED, and the linked tests actually exercise the scenario's WHEN and THEN. Read each linked test to confirm the last part; a test whose name matches but whose body checks something else is not evidence.
- **Partially verified**: some scenarios are verified, or the tests cover the scenario only in part (for example one input of several, or the THEN only loosely).
- **Failed**: a linked test fails, or step 5 found the implementation contradicts the requirement and nothing shows otherwise.
- **Unverified**: no executed evidence. Matching code, a test that exists but did not run, or an agent's assertion are not evidence.

If the gate's Traceability or Tests check did not PASS, no requirement can be Verified. If step 5 flagged a requirement, it cannot be Verified unless a test demonstrates step 5 was wrong; cite that test.

Note whether each test is an approved acceptance test (listed in `acceptance-approved`) or was written by the implementer. Implementer-written tests may share the implementation's misunderstanding, so a requirement verified only by them is weaker evidence; say so in its row.

## Report

Write `review/verification.md`:

```markdown
# Verification: <change>

Gate: <commit range from gate.md>, <n> PASS, <n> FAIL, <n> SKIPPED

## Requirements

| Requirement | Implementation | Evidence | Status | Missing evidence |
|---|---|---|---|---|
| <name from the delta spec> | `file.rs::symbol` | `test_fn` (acceptance, passed) | Verified | — |

## Skipped and failed checks

<Each gate check that did not PASS, and what it leaves unproven. A SKIPPED check is missing evidence, not a pass.>

## Unverified assumptions and residual risks

<For each: what could not be established, why, the impact if it is wrong, and what evidence would settle it.>
```

Cover at least: behaviour the tests cannot reach, scenarios tested only by implementer-written tests, surviving mutants from the gate (accepted ones included: each is behaviour no test checks yet, so name the requirement it weakens and the ticket expected to close it), and anything in `design.md` the spec scenarios don't express.

End by telling the user the counts per status and the single most significant unverified risk.
