# 01: World: Prey forage in a Showcase

**What to build:** The first tracer bullet through the headless library. From a TOML config, build a single Arena on a 2D torus containing MLP Prey Agents with randomly initialised Genomes, plus stationary food that respawns when eaten. A Showcase can be stepped deterministically, and through the library's public interface a caller can read world state and per-Agent telemetry. There is no evolution, CLI, or GUI yet. Set up the workspace so the library has no GUI dependency (ADR-0003). Telemetry is shaped generically (a list of eye kinds with their bins, and a list of energy costs per Energy Rule) so that later tickets add entries without changing its shape.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] Config parses from TOML; an empty file yields a fully defaulted resolved config; an unknown key is an error naming the key; the resolved config serialises back out
- [ ] Agents and food wrap around the torus edges
- [ ] Agents move by steering and forward thrust: steering in [-1, 1] changes heading, throttle in [-1, 1] maps to thrust in [0, 1], with linear drag from config and a speed cap
- [ ] Prey eat food on contact and gain its energy; eaten food respawns immediately
- [ ] Idle drain reduces energy each step; an Agent at zero energy dies and stays dead
- [ ] Controller input is normalised energy, normalised speed, then N food Eye Bins (N from config); bin activation is `1 - distance/range` for the nearest food in that sector, else 0
- [ ] MLP controllers have hidden layer sizes from config, Xavier-scaled initial weights, and two tanh outputs
- [ ] Showcase exposes: step, has-ended (step limit or all dead), world state (Agents, food), and telemetry by Agent id (Eye Bins, core sensors, raw and applied outputs, energy costs per Energy Rule)
- [ ] Two Showcases with the same seed and config produce identical state after N steps, whether stepped one at a time or in batches
- [ ] Integration tests cover each of the above through the library's public interface only
