# Review gate

- Change: 04-viewer-plays-a-showcase
- Commits: 2a2a8fe..c048991
- Tier: full (required by the diff: full)
- Run: 2026-10-21T12:01:45+02:00

| Check | Result | Exit code | Detail |
|---|---|---|---|
| Formatting | PASS | 0 | `cargo fmt --all --check` |
| Clippy | PASS | 0 | `cargo clippy --locked --workspace --all-targets -- -D warnings` |
| Tests | PASS | 0 | `cargo test --locked --workspace` |
| Architecture | PASS | 0 | `scripts/check-architecture.sh` |
| OpenSpec validation | PASS | 0 | `openspec validate --all --strict --no-interactive` |
| Dependency audit | SKIPPED | - | no deny.toml |
| Traceability | PASS | 0 | `scripts/check-traceability.sh 04-viewer-plays-a-showcase` |
| Acceptance-test lock | PASS | - | 1 file(s) unchanged since 2b92f447c061dc8335b1510b104563aac472eabe |
| Tier | PASS | - | full |
| Mutation testing | PASS | 2 | 20 surviving mutant(s), all accepted |

## Accepted surviving mutants

Untested for now, accepted in accepted-mutants:

- crates/viewer/src/app.rs: delete ! in <impl eframe::App for App>::ui
- crates/viewer/src/app.rs: delete ! in App::controls
- crates/viewer/src/app.rs: delete - in App::world
- crates/viewer/src/app.rs: replace && with || in <impl eframe::App for App>::ui
- crates/viewer/src/app.rs: replace && with || in App::controls
- crates/viewer/src/app.rs: replace * with + in App::world
- crates/viewer/src/app.rs: replace * with / in App::world
- crates/viewer/src/app.rs: replace + with * in App::world
- crates/viewer/src/app.rs: replace + with - in App::world
- crates/viewer/src/app.rs: replace - with + in App::world
- crates/viewer/src/app.rs: replace - with / in App::world
- crates/viewer/src/app.rs: replace / with % in App::world
- crates/viewer/src/app.rs: replace / with * in App::world
- crates/viewer/src/app.rs: replace <impl eframe::App for App>::ui with ()
- crates/viewer/src/app.rs: replace App::chart with ()
- crates/viewer/src/app.rs: replace App::controls with ()
- crates/viewer/src/app.rs: replace App::world with ()
- crates/viewer/src/app.rs: replace colour -> Color32 with Default::default()
- crates/viewer/src/app.rs: replace run -> Result<(), Box<dyn std::error::Error>> with Ok(())
- crates/viewer/src/model.rs: replace && with || in Viewer::stats
