# 14: Inspector

**What to build:** Selecting an Agent in the viewer shows its full telemetry, kept after death.

**Blocked by:** 05: Traits and Energy Rules

**Spec:** User Stories 85–92; Implementation Decisions › The library's public interface (Showcase telemetry); Acceptance Scenarios › Selected-agent inspection (`../spec.md`)

**Status:** ready-for-agent

- [ ] Clicking an Agent selects it; clicking empty space clears the selection
- [ ] Inspector shows Controller Family, Role, and Traits
- [ ] Inspector shows every Eye Bin kind present in telemetry and the core sensor values
- [ ] Inspector shows raw steering and throttle outputs and the applied throttle
- [ ] Inspector shows last step's energy cost per Energy Rule
- [ ] Inspector shows a live network visualisation for MLP and, when present, LTC controllers
- [ ] After the selected Agent dies, the inspector keeps showing its last captured telemetry

## Comments

**From the 04 (Showcase viewer) implementation and review:**

- `crates/viewer/src/app.rs::App::world` currently requests only `Sense::hover`; `model.rs::AgentShape` carries no AgentId, and edge copies duplicate positions. Selection needs identity preserved through the scene, screen-to-world conversion, and hit testing that maps wrapped copies to the same Agent. Read telemetry through `Showcase::telemetry`; keep selection decisions testable in the model.
- Scene edge reach currently covers bodies and heading markers only (`model.rs::HEADING_MARKER_LENGTH`, `Scene::of`, `edge_copies`). Inspector vision overlays need their own extents and wrap coverage.
