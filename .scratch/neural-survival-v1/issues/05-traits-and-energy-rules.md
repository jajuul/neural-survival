# 05: Traits and Energy Rules

**What to build:** Agents get six heritable Traits (vision range, field of view, max speed, max acceleration, turn rate, body radius), each with a configured range and an energy cost that grows with its value. Energy drain splits into separate Energy Rules: idle drain (always on), movement, turning, sensing, and trait upkeep. Every rule except idle drain can be toggled, including live from the viewer.

**Blocked by:** 04: Viewer: `view` plays a Showcase

**Status:** ready-for-agent

- [ ] Generation-0 Traits are drawn uniformly within configured ranges; mutation clamps Traits to their ranges
- [ ] Traits take effect: vision range and field of view shape Eye Bins, max speed caps speed, max acceleration scales thrust, turn rate scales steering, body radius sets contact size
- [ ] Each Energy Rule's per-step cost appears separately in telemetry
- [ ] Toggling a rule off through the Showcase stops that cost on the next step; idle drain cannot be toggled
- [ ] Rule toggles are part of the config
- [ ] Viewer exposes the toggles, shows their current state, and the running Showcase uses the new state on subsequent steps
- [ ] Integration tests cover Trait clamping, per-rule costs, and toggle effects
