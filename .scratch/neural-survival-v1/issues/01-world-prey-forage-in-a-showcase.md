# 01: World: Prey forage in a Showcase

**What to build:** The first tracer bullet through the headless library. From a TOML config, build a single Arena on a 2D torus containing MLP Prey Agents with randomly initialised Genomes, plus stationary food that respawns when eaten. A Showcase can be stepped deterministically, and through the library's public interface a caller can read world state and per-Agent telemetry. There is no evolution, CLI, or GUI yet. Set up the workspace so the library has no GUI dependency (ADR-0003). Telemetry is shaped generically (a list of eye kinds with their bins, and a list of energy costs per Energy Rule) so that later tickets add entries without changing its shape.

**Blocked by:** None (can start immediately)

**Spec:** User Stories 9–12, 21, 33, 36–41, 43, 45, 47, 52 (respawn-on-eat), 60–62 (food bins), 66; Implementation Decisions › Workspace shape, The library's public interface (Config, Showcase), Determinism, Controllers (MLP), Input vector layout, Episode; Testing Decisions › Seam 1 (Config, Determinism, World rules) (`../spec.md`)

**Status:** resolved

- [x] Config parses from TOML; an empty file yields a fully defaulted resolved config; an unknown key is an error naming the key; the resolved config serialises back out
- [x] Agents and food wrap around the torus edges
- [x] Agents move by steering and forward thrust: steering in [-1, 1] changes heading, throttle in [-1, 1] maps to thrust in [0, 1], with linear drag from config and a speed cap
- [x] Prey eat food on contact and gain its energy; eaten food respawns immediately
- [x] Idle drain reduces energy each step; an Agent at zero energy dies and stays dead
- [x] Controller input is normalised energy, normalised speed, then N food Eye Bins (N from config); bin activation is `1 - distance/range` for the nearest food in that sector, else 0
- [x] MLP controllers have hidden layer sizes from config, Xavier-scaled initial weights, and two tanh outputs
- [x] Showcase exposes: step, has-ended (step limit or all dead), world state (Agents, food), and telemetry by Agent id (Eye Bins, core sensors, raw and applied outputs, energy costs per Energy Rule)
- [x] Two Showcases with the same seed and config produce identical state after N steps, whether stepped one at a time or in batches
- [x] Integration tests cover each of the above through the library's public interface only
