# 10: Day/night and sensor noise

**What to build:** Add an optional day/night cycle that scales sensing visibility, and optional sensor noise and dropout.

**Blocked by:** 04: Viewer: `view` plays a Showcase

**Status:** ready-for-agent

- [ ] Visibility follows a sine over a configured period in steps, starting at a configured phase each Episode, ranging from a configured night minimum to 1.0
- [ ] The visibility multiplier scales effective vision range for food, Prey, and Predator Eye Bins, but not Obstacle Eye Bins
- [ ] Optional noise and dropout perturb Eye Bins after they're computed, using a random stream seeded per Agent, so results stay deterministic
- [ ] Viewer shows the Light Phase (day, dusk, night, dawn) and visibility multiplier when the cycle is enabled
- [ ] Integration tests cover reduced sensing at night, unaffected Obstacle sensing, and deterministic noise
