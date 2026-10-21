---
name: architecture-walkthrough
description: "Step 7 of the review workflow: walk a human through what an OpenSpec change actually built, how it fits the architecture, and exactly which code they should read before approving. Read-only. Use after verification, when the user wants to understand a change without reading every line, or asks follow-up questions about how it works."
---

Explain one change the way a senior engineer walks a colleague through a pull request: what changed, how it is structured, why, and where the human's attention is needed. The workflow is in `docs/agents/review-workflow.md`.

This skill is read-only. Do not modify code, tests, specs, or the always-read list.

## Inputs

Ask for the change name and base commit if you weren't given them. Then read:

1. The diff: `git diff <base>...HEAD` and `git log <base>..HEAD --oneline`.
2. The change's `proposal.md`, `design.md`, and delta specs.
3. The ADRs in `docs/adr/` that the design cites or the diff touches, and `CONTEXT.md` for domain terms.
4. `review/gate.md`, `review/code-review.md`, and `review/verification.md`. Build on them; do not redo their work.
5. `docs/agents/always-read.txt`.

Before writing anything, check that the review files describe the current code. Each review step commits its own file, so they name earlier commits than `HEAD`. A file is current when no commit after the one it names changes anything outside `openspec/changes/<change>/review/`: `git diff --name-only <named commit> HEAD -- . ':!openspec/changes/<change>/review/'` prints nothing. Check:

- the commit that `gate.md`'s `Commits:` range ends at is current;
- `verification.md`'s `Gate:` line names the same range as `gate.md`;
- the commit in `code-review.md`'s `Reviewed:` line is current.

If any file is missing or stale, stop and say which steps must be rerun. Never walk through code the reviews didn't cover.

Read the surrounding code the diff calls and is called by, not only the changed lines. Ground every technical claim in a file and symbol (`crates/core/src/world.rs::World::step`). If you cannot ground a claim, say it is an inference.

## Report

Write `openspec/changes/<change>/review/walkthrough.md`, short enough to read in five minutes. It is the file the human opens first, so it starts with the whole review in brief:

0. **Review summary**: the commit range; the gate result (PASS, FAIL, or SKIPPED per check, and any accepted surviving mutants); code-review findings per axis; the verification status counts; and the single most significant open risk. Link [gate](gate.md), [code review](code-review.md), and [verification](verification.md). End with a one-line verdict: ready for the human's targeted reading, or blocked and by what.
1. **What changed**: a few sentences in domain language (`CONTEXT.md` terms).
2. **How it is structured**: the components changed or added, what each is responsible for, who owns which state, and the interfaces between them. List new dependencies between modules or crates explicitly.
3. **Runtime paths**: trace one representative path and one failure path through the new code, step by step, with file and symbol references.
4. **Design and ADR conformance**: where the implementation differs from `design.md` or an ADR, and the trade-offs it made. Say "none found" if so.
5. **Risks**: risks that follow from the structure (shared state, ordering, determinism under ADR-0001, unclear ownership), taking the open items in `verification.md` into account. Do not repeat code-review findings; refer to them.
6. **Read in full**: every always-read path the diff touches, each with one line on what changed in it. These are not optional and do not count toward the three below.
7. **Inspect these**: at most three further locations, ranked by risk, each with the file and line range, what to look for, and why it matters. Choose them for the risk they carry, not their size.

A diagram is optional. Use the Archify skill only when a component or sequence view makes the structure clearer than the text, and save it under `review/`, not `docs/architecture/`.

## Follow-up questions

After the report, offer to answer questions such as tracing a scenario end to end, explaining why an abstraction was introduced, or showing which test proves a behaviour. Answer from the code, with references, and say when a question needs evidence that does not exist yet.
