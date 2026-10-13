# 09: Obstacles

**What to build:** Add static round Obstacles, placed from the Episode seed, that block movement and are sensed through Obstacle Eye Bins.

**Blocked by:** 04: Viewer: `view` plays a Showcase

**Status:** ready-for-agent

- [ ] Obstacle count and radius range come from config; placement is non-overlapping and drawn from the Episode seed
- [ ] Agents and food never spawn inside Obstacles
- [ ] An Agent colliding with an Obstacle loses the velocity component pointing into it and slides along the edge; it never ends a step inside an Obstacle
- [ ] Controller input adds N Obstacle Eye Bins; distance is measured to where a ray meets the circle's edge
- [ ] Viewer draws Obstacles
- [ ] Integration tests cover placement, sliding, and Obstacle Eye Bin activation for an Obstacle in a known sector
