# 11: Food motion and replenishment

**What to build:** Make food motion and replenishment configurable, as independent settings.

**Blocked by:** 04: Viewer: `view` plays a Showcase

**Spec:** User Stories 51–52, 54; Implementation Decisions › Food; Testing Decisions › Seam 1 (`../spec.md`)

**Status:** ready-for-agent

- [ ] Food motion is stationary or drifting; drifting food wraps around the torus
- [ ] Replenishment is respawn-on-eat or spawning at a fixed deterministic rate up to a cap
- [ ] The settings combine freely
- [ ] Integration tests cover each mode
