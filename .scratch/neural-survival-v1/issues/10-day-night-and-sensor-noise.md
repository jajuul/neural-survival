# 10: Day/night and sensor noise

**What to build:** Add an optional day/night cycle that scales sensing visibility, and optional sensor noise and dropout.

**Blocked by:** 04: Viewer: `view` plays a Showcase

**Spec:** User Stories 58–59, 64–65, 80; Implementation Decisions › Day/night, Determinism; Testing Decisions › Seam 1 (World rules: night sensing) (`../spec.md`)

**Status:** ready-for-agent

- [ ] Visibility follows a sine over a configured period in steps, starting at a configured phase each Episode, ranging from a configured night minimum to 1.0
- [ ] The visibility multiplier scales effective vision range for food, Prey, and Predator Eye Bins, but not Obstacle Eye Bins
- [ ] Optional noise and dropout perturb Eye Bins after they're computed, using a random stream seeded per Agent, so results stay deterministic
- [ ] Viewer shows the Light Phase (day, dusk, night, dawn) and visibility multiplier when the cycle is enabled
- [ ] Integration tests cover reduced sensing at night, unaffected Obstacle sensing, and deterministic noise

## Comments

**From the 02 (Evolution library) implementation and review:**

- **There is no per-Agent random stream yet.** ADR-0001 splits streams per Generation, Episode, Arena and Agent. Ticket 02 added the Generation, Episode, Breeding and Showcase streams (`Stream` in `crates/core/src/rng.rs`); the per-Agent stream for sensor noise is still missing. Add a new `Stream` variant and derive each Agent's noise rng from the Episode seed (and Arena index) plus the Agent id. Don't draw noise from the Arena's own rng: that would shift food respawn positions whenever noise is toggled.
