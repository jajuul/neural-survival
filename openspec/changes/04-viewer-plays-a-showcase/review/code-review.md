Reviewed: b317232

Committed scope: `git diff 2a2a8fe...b317232`. Full tier, as recorded in the proposal. Reviews ran in a fresh session; no implementation or protected files were edited.

## Standards

Hard violations: none found, excluding checks already enforced by tooling.

Judgement-call smells: none warrant reporting.

The changes preserve the relevant architecture rules in `AGENTS.md` and ADR-0001/0003: playback schedules fixed core steps; presentation stays in the viewer crate; the model exposes behaviour for headless tests. The public `Showcase::config()` getter supplies scene radii through an immutable reference. `Episode::config()` remains inside a private module. Passing an already constructed `Viewer` to `run` keeps snapshot loading and its failures outside window creation.

The running step total is an understandable alternative to storing a remainder; it does not itself breach a documented standard. `--test-workspace=true` broadens mutation detection to downstream acceptance tests without weakening the check. The 19 GUI survivors match the explicitly excluded GUI testing scope; the remaining stats survivor is tracked on ticket 06. Additional playback tests written after implementation are permitted by review-workflow step 3 ("Add further tests freely"); test-first scene work introduces no standards issue.

Limitations: this axis assessed committed source and recorded workflow evidence. It did not establish historical human approvals independently, rerun the gate, or exercise the GUI.

## Spec

- **[P3] Wrap heading markers that cross an edge independently of the body — `crates/viewer/src/model.rs:294`, `crates/viewer/src/app.rs:167`.** The drawn-scene requirement at `specs/viewer/spec.md:218` says: "Anything overlapping a torus edge SHALL also be drawn shifted by the world's width, height, or both, so it shows on both sides." Edge copies use the body radius, but the painter extends the heading marker to `1.6 * radius`. In a 50 × 40 world, an Agent at `(1.2, 20)`, radius 1, heading π has a marker ending at x = −0.4. Its body does not overlap the edge, so no shifted Agent is generated. The marker is clipped at the left boundary (`app.rs:127`), and its corresponding segment at x = 49.6–50 is absent. This is a localized visual defect; simulation behaviour is unaffected. Include the marker's extent when deciding which shifted shapes to emit.

Scope creep: no findings.

The immutable config getter and `run(Viewer)` fit the revised proposal/design and solve concrete scene/initialization needs without changing core simulation behaviour. The mutation-script change enables tests in consuming crates to detect mutations. The stats survivor (`&&` → `||`) is equivalent with the sole current Species, MLP-Prey; ticket 06 explicitly calls for mixed-Role stats tests once Predators exist. GUI survivors remain manual-review obligations, not evidence that painting or widgets work. The running clock preserves the requested ordinary finite-frame budget, including fractional carry, speed changes, and reset. Post-implementation playback tests are permitted; their timing does not establish test-driven development or diminish what their assertions cover.

This axis used committed source inspection, not a GUI run.

Axis summary: Standards 0 findings; Spec 1 finding, worst P3 (heading marker torus wrap).

## Bugs

Built-in review stdout (verbatim; exit 0):

```text
No actionable regressions were identified in the diff. All workspace tests passed. The mutation-testing rerun remains incomplete; no unaccepted survivors have been observed so far.
```

Invocation: `codex review --base review-base`, with `review-base` set to `2a2a8fe`. The installed CLI rejected the exact requested invocation with exit 2 and `error: the argument '--base <BRANCH>' cannot be used with '[PROMPT]'`. Consequently this run used the default review prompt; the requested bugs/error-handling prompt and style exclusion could not be supplied with `--base`. After automatic approval review initially blocked external source transmission, the human explicitly approved this CLI review and it ran successfully.

The built-in review's incomplete mutation rerun is separate from this session's completed run recorded below. Its no-regressions verdict does not negate the Spec axis's localized visual finding.

## Validation evidence

The existing committed `review/gate.md` records checks over `2a2a8fe..c048991`: formatting, Clippy, tests, architecture, OpenSpec validation, traceability, acceptance lock and tier passed. Dependency audit was skipped because there is no `deny.toml`. Mutation testing is recorded as PASS with underlying exit code 2 and 20 accepted survivors. Those are recorded results, not a fresh gate run.

Fresh `scripts/mutants-ticket.sh 2a2a8fe`: exit 2; 101 mutants tested, 37 missed, 53 caught, 11 unviable, no timeouts. The 37 missed cases normalize to 20 distinct file-and-mutation signatures, all already present in `accepted-mutants`; no new exception is proposed. This rerun also built and passed the unmutated workspace baseline. No acceptance tests or existing mutation exceptions were changed.
