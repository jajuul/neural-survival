# Neural Survival v1

Status: ready-for-agent

## Problem Statement

I want to watch neural-network-controlled creatures evolve survival behaviour in a 2D world, and compare how two controller families (feed-forward MLP and liquid time-constant LTC) fare under the same world rules. An earlier build of this project exists only as a behaviour spec (`docs/SPECS.md`); I want a clean rebuild from that spec, with the ambiguities in it resolved. Concretely, I want to train headlessly for a while, look at how the current population actually behaves, tweak the world, train some more, and keep reproducible records of each run so I can compare them offline.

## Solution

A native Rust desktop application with three commands:

- `batch` evolves the population headlessly for a fixed number of Generations and writes run artifacts.
- `run` does the same and then opens the viewer on the final Snapshot.
- `view` opens the viewer on any saved Snapshot.

The population is four fixed Species (MLP-Prey, MLP-Predator, LTC-Prey, LTC-Predator). Each Generation evaluates every Genome over the Episodes prescribed by the Evaluation Schedule, scores Fitness per Role, and breeds each Species separately. Controller Families can live in Separated Arenas (identical layouts, independent ecologies) or one Shared Arena, and the Arena Mode can be switched between Generations.

The viewer plays a Showcase: an unscored Episode of the whole current population, which I can pause, step, speed up, and inspect Agent by Agent. From the viewer I can edit the full config and continue evolution for y more Generations. The Showcase keeps animating while training runs in the background, then switches to a Showcase of the new Generation.

Everything is deterministic per machine: the same seed, config, and binary give identical results, and a continuation from the viewer gives exactly what a headless run would.

The domain vocabulary is in `CONTEXT.md`, and decisions are recorded in `docs/adr/` (0001 determinism scope, 0002 fixed Species, 0003 eframe/egui over a headless core).

## User Stories

### Running experiments

1. As an experimenter, I want to run `neural-survival batch --config <path> [--seed <u64>] --generations <u32> --out <dir>`, so that I can evolve a population headlessly and get artifacts to analyse.
2. As an experimenter, I want `batch` to refuse an output directory that exists and is not empty, with an error explaining it must be absent or empty, so that artifacts from different runs never mix.
3. As an experimenter, I want to run `neural-survival run --config <path> [--seed <u64>] --headless-generations <u32> --out <dir>`, so that I can train for a while and then immediately watch the result.
4. As an experimenter, I want `run` to accept a non-empty output directory and clear only the run files it owns (`metrics.csv`, `fitness.svg`, `run_summary.json`, `snapshot_gen_*.json`) before starting, so that I can reuse one directory for repeated attempts without stale files mixing in, and without losing unrelated files.
5. As an experimenter, I want `run` to evolve exactly the requested number of Generations before the viewer opens, so that what I see is the trained population I asked for.
6. As an experimenter, I want `neural-survival view --snapshot <path>`, so that I can reopen any saved Generation later.
7. As an experimenter, I want the application to print a generated seed when I omit `--seed`, so that I can reproduce the run later.
8. As an experimenter, I want progress reported every 10 Generations and on completion, so that I know a long run is alive and how far along it is.
9. As an experimenter, I want the same seed, config, and binary to give identical results on the same machine, so that experiments are reproducible.
10. As an experimenter, I want to write the config as TOML in which every field has a default, so that a minimal or empty config still runs.
11. As an experimenter, I want unknown config keys to be a hard error, so that a misspelt setting is never silently ignored.
12. As an experimenter, I want the fully resolved config, with defaults filled in, recorded in `run_summary.json` and every Snapshot, so that I always know exactly what a run used.

### Evolution

13. As an experimenter, I want the population split into four Species (MLP-Prey, MLP-Predator, LTC-Prey, LTC-Predator), each with a fixed population size set in the config, so that no family/Role combination can go extinct and end the comparison.
14. As an experimenter, I want Role and Controller Family never to mutate, so that each Species stays a stable lineage.
15. As an experimenter, I want each Species to breed only within itself, so that MLP and LTC never interbreed and crossover always combines compatible networks.
16. As an experimenter, I want tournament selection with a configurable tournament size, so that I can tune selection pressure.
17. As an experimenter, I want elitism that copies the top k Genomes of each Species unchanged, so that the best solutions are never lost.
18. As an experimenter, I want uniform crossover over network parameters and Traits, so that offspring combine features of both parents.
19. As an experimenter, I want Gaussian mutation with a configurable rate and sigma, with Traits clamped to their configured ranges, so that I can control exploration.
20. As an experimenter, I want network shape set per Controller Family in the config and fixed for the whole run, so that Genomes stay compatible across Generations.
21. As an experimenter, I want Generation-0 MLP weights initialised with Xavier-style scaling and LTC parameters with the published defaults, so that early Generations behave sensibly.
22. As an experimenter, I want Generation-0 Traits drawn uniformly from their configured ranges, so that the initial population is diverse.
23. As an experimenter, I want each Generation to run the number of Episodes, of the length, given by the Evaluation Schedule, with each Genome's Fitness averaged over them, so that a lucky Episode doesn't dominate selection.
24. As an experimenter, I want the Evaluation Schedule to be able to vary by Generation, so that I can make evaluation harder or longer as evolution progresses.
25. As an experimenter, I want Fitness to be a per-Role weighted sum of steps survived and energy gained, with weights in the config, so that I can steer what each Role is rewarded for.
26. As an experimenter, I want Prey energy gained to count food and Predator energy gained to count Captures, so that each Role is rewarded for its own way of making a living.

### Arenas

27. As an experimenter, I want an Arena Mode setting with Separated and Shared values, so that I can study MLP and LTC in isolation and then put them together.
28. As an experimenter, I want each Controller Family in Separated mode to get its own full-size Arena built from the whole world config, so that each family has a complete ecology of its own.
29. As an experimenter, I want all Arenas of one Episode to share the Episode seed, so that MLP and LTC face identical food and obstacle layouts and the comparison is controlled.
30. As an experimenter, I want Shared mode to put all four Species into one Arena of the configured size without automatic scaling, so that nothing changes behind my back; I adjust food or size myself if I want.
31. As an experimenter, I want to change Arena Mode at any Generation boundary, in either direction, so that I can bring families together and separate them again.
32. As an experimenter, I want each metrics row to record the Arena Mode, so that I can explain Fitness jumps caused by a mode change.

### World and Agents

33. As an experimenter, I want the world to be a 2D torus in which Agents and food wrap around the edges, so that there are no walls to exploit.
34. As an experimenter, I want each Agent to have six mutable Traits (vision range, field of view, max speed, max acceleration, turn rate, body radius), each with a configured range, so that body and senses evolve alongside behaviour.
35. As an experimenter, I want each Trait to cost energy that grows with its value, so that every Trait is a real trade-off.
36. As an experimenter, I want Agents to move by steering and forward thrust rather than having their speed set directly, so that movement has inertia and must be learned.
37. As an experimenter, I want steering output in [−1, 1] multiplied by the turn-rate Trait to give the heading change, so that turning ability is heritable.
38. As an experimenter, I want throttle output in [−1, 1] mapped to forward thrust in [0, 1], scaled by the max-acceleration Trait, with no reverse, so that the applied throttle is well defined.
39. As an experimenter, I want linear drag, with a coefficient in the world config, and speed capped at the max-speed Trait, so that Agents can't accelerate without bound.
40. As an experimenter, I want Agents to start each Episode with a per-Role starting energy from the config, so that Prey and Predators can be balanced separately.
41. As an experimenter, I want an Agent to die when its energy reaches zero and stay dead for the rest of the Episode, so that survival matters.
42. As an experimenter, I want no births during an Episode, so that reproduction happens only between Generations through the GA.
43. As an experimenter, I want an Episode to end at its step limit or when every Agent is dead, whichever comes first, so that Episodes don't run forever.
44. As an experimenter, I want Prey to keep going if every Predator has died, so that an Episode measures Prey survival in full.
45. As an experimenter, I want each Energy Rule (idle drain, movement, turning, sensing, trait upkeep) to be a separate source of drain, so that I can see and control where energy goes.
46. As an experimenter, I want every Energy Rule except idle drain to be toggleable, so that I can experiment with energy economics without making Agents immortal.

### Ecology

47. As an experimenter, I want Prey to eat food on contact and gain its energy, so that Prey have a way to survive.
48. As an experimenter, I want Predators never to eat food, so that the Roles stay ecologically distinct.
49. As an experimenter, I want a Capture to happen when a Predator comes within capture range of a Prey, killing the Prey and giving the Predator a fixed configured amount of energy, so that Predators have a reliable learning signal.
50. As an experimenter, I want a configurable cooldown after each Capture, so that one Predator can't wipe out a group of Prey in a single step.
51. As an experimenter, I want food motion to be either stationary or drifting, so that I can make foraging harder.
52. As an experimenter, I want food replenishment to be either respawn-on-eat or spawn-at-a-fixed-rate up to a cap, so that I can control scarcity.
53. As an experimenter, I want an optional Food Clusters setting in which food spawns around drifting clusters whose density follows the Light Phase, so that food is patchy and changes with time of day.
54. As an experimenter, I want food motion, replenishment, and clusters to be independent settings that combine, so that I can build any mix.
55. As an experimenter, I want Obstacles placed from the Episode seed, with count and radius range in the config and no overlaps, so that every Episode gets a fresh but reproducible layout.
56. As an experimenter, I want Agents to slide along an Obstacle's edge rather than stop dead, so that collisions don't trap Agents.
57. As an experimenter, I want Agents and food never to spawn inside Obstacles, so that every Episode starts valid.
58. As an experimenter, I want an optional day/night cycle in which a sine wave over a configured number of steps moves the visibility multiplier between a configured night minimum and 1.0, so that sensing gets harder at night.
59. As an experimenter, I want the Light Phase to start at a configured phase in every Episode, so that Episodes are comparable.

### Sensing and control

60. As an experimenter, I want each Agent's controller input to be a fixed-width vector of normalised energy, normalised speed, and food, Prey, Predator, and Obstacle Eye Bins, so that all Genomes of a Controller Family share one input layout.
61. As an experimenter, I want each eye to split the field of view into N equal sectors, with N set in the config, fixed for the run, and shared by all four eye kinds, so that input width never changes mid-run.
62. As an experimenter, I want each Eye Bin's activation to be `1 − distance / range` for the nearest matching object in that sector, or 0 when there is none, so that closer objects give stronger signals.
63. As an experimenter, I want Obstacle Eye Bins to measure distance to where a ray meets the circle's edge rather than its centre, so that large Obstacles are sensed correctly.
64. As an experimenter, I want the visibility multiplier to scale the effective vision range for food, Prey, and Predator eyes but not Obstacle eyes, so that night is harder without making Agents blind to walls.
65. As an experimenter, I want optional sensor noise and dropout applied after the Eye Bins are computed, drawn from a random stream seeded per Agent, so that robustness can be selected for while staying deterministic.
66. As an experimenter, I want the controller to produce steering and throttle outputs each step, so that behaviour is entirely network-driven.
67. As an experimenter, I want LTC controllers to advance a configurable number of semi-implicit solver sub-steps per sim step (default about 6), so that their dynamics are integrated stably.
68. As an experimenter, I want LTC neuron state to persist across steps within an Episode and reset at the start of each Episode, so that LTC can use memory but Episodes stay independent.

### Viewer

69. As an experimenter, I want `run` and `view` to open the viewer paused on a Showcase of the loaded Generation, so that I can get my bearings before anything moves.
70. As an experimenter, I want the Showcase to contain the whole current population in the current Arena Mode, so that I see real ecological interactions.
71. As an experimenter, I want the Showcase seed derived from the run seed and the Generation, so that the same Snapshot always gives the same Showcase.
72. As an experimenter, I want the Showcase never to be scored or to affect evolution, so that watching changes nothing.
73. As an experimenter, I want to pause, continue, and single-step the Showcase, so that I can study behaviour closely.
74. As an experimenter, I want to change simulation speed without changing what happens, so that I can fast-forward safely.
75. As an experimenter, I want the Showcase to stop when its Episode ends and offer a restart that replays it exactly, so that I can rewatch interesting moments.
76. As an experimenter, I want to see the current Generation, world step, alive Agent count, and food remaining, so that I have an overview.
77. As an experimenter, I want to see the population count of each Species and Controller Family, so that I know the composition.
78. As an experimenter, I want to see the active Evaluation Schedule and Arena Mode, so that I know how the population was trained.
79. As an experimenter, I want to see the state of each Energy Rule toggle, so that I know which rules are active.
80. As an experimenter, I want to see the current Light Phase (day, dusk, night, dawn) and visibility multiplier when day/night is on, so that I can relate behaviour to visibility.
81. As an experimenter, I want a best and mean Fitness chart with one line pair per Species and a marker wherever Arena Mode changed, so that I can follow progress over the whole history.
82. As an experimenter, I want Prey and Predators drawn in different colours, and MLP and LTC in different shapes, so that I can tell all four Species apart at a glance.
83. As an experimenter, I want food, Food Clusters, and Obstacles drawn in the world view when enabled, so that I can see what Agents are reacting to.
84. As an experimenter, I want the separate Arenas shown side by side in Separated mode, so that I can compare MLP and LTC directly.

### Inspector

85. As an experimenter, I want to click an Agent to select it and click empty space to clear the selection, so that inspection is direct.
86. As an experimenter, I want the inspector to show the selected Agent's Controller Family, Role, and Traits, so that I know what I'm looking at.
87. As an experimenter, I want the inspector to show the current food, Prey, Predator, and Obstacle Eye Bin activations, so that I can see what the Agent senses.
88. As an experimenter, I want the inspector to show normalised energy and speed, so that I can see its internal state.
89. As an experimenter, I want the inspector to show raw steering and throttle outputs alongside the applied throttle, so that I can see how outputs become motion.
90. As an experimenter, I want the inspector to show last step's energy cost broken down by Energy Rule, so that I can see what is draining the Agent.
91. As an experimenter, I want a live network visualisation of the selected Agent's controller, so that I can see activations flow.
92. As an experimenter, I want the inspector to keep showing the last captured telemetry after the selected Agent dies, so that I can study why it died.

### Editing and continuation

93. As an experimenter, I want to toggle Energy Rules in the viewer and have subsequent Showcase steps use the new state immediately, so that I can see the effect live.
94. As an experimenter, I want rule toggles made in the viewer carried into the next continuation's config, so that what I tuned is what gets trained.
95. As an experimenter, I want a continuation editor exposing the full config (world, food, Obstacles, Food Clusters, GA, Fitness, sensors, Predator/Prey settings, Arena Mode, Evaluation Schedule, Energy Rules), so that I can change anything between training bursts.
96. As an experimenter, I want the editor to make clear which fields only affect initialisation, such as network shape and Generation-0 Trait ranges, so that I don't expect them to rewrite already-evolved Genomes.
97. As an experimenter, I want to enter a number of Generations y and start continuation from the viewer, so that I can train more without leaving the GUI.
98. As an experimenter, I want the Showcase to keep animating while continuation trains in the background, so that the viewer stays responsive.
99. As an experimenter, I want continuation progress shown every 10 Generations, with the Fitness chart updating as it goes, so that I can follow training.
100. As an experimenter, I want the viewer to switch to a Showcase of the new Generation automatically when continuation finishes, so that I can immediately see what changed.
101. As an experimenter, I want a GUI continuation to give exactly the results a headless run with the same Snapshot and config would, so that training in the viewer is as trustworthy as batch.
102. As an experimenter, I want continuation to write into the loaded Snapshot's directory (a snapshot every 10 Generations and at the end, plus rewritten `metrics.csv`, `fitness.svg`, and `run_summary.json` with the full history), so that the run's directory stays complete.
103. As an experimenter, I want to save the current Generation with my edited config from the viewer as `snapshot_gen_<n>_edit_<k>.json`, where k counts up from 1, so that I can keep a tuned setup without overwriting the original Snapshot.

### Artifacts

104. As an experimenter, I want headless runs to write `metrics.csv`, `fitness.svg`, `run_summary.json`, and `snapshot_gen_<n>.json` for the last Generation, so that I can analyse runs offline.
105. As an experimenter, I want headless runs to also write a Snapshot every 10 Generations, so that I can reopen intermediate points.
106. As an experimenter, I want `metrics.csv` to have one row per Generation per Species, with best Fitness, mean Fitness, Agents alive at Episode end, Captures or food eaten, and Arena Mode, so that the four Species can be compared.
107. As an experimenter, I want `fitness.svg` to plot best and mean Fitness per Species with Arena Mode change markers, so that I get a quick visual summary.
108. As an experimenter, I want each Snapshot to hold the Generation's Genomes, the resolved config, the run seed, and the full metrics history, so that it is self-contained.
109. As an experimenter, I want Snapshots to contain no world state, so that they stay small and opening one always starts a fresh, reproducible Showcase.
110. As an experimenter, I want Snapshots to carry a format version and to be rejected with a clear error if the version is unsupported, so that future format changes can be migrated deliberately.

## Implementation Decisions

- **Workspace shape:** a headless library holds the simulation, GA, config, persistence, and artifacts and has no GUI dependency. A GUI built on `eframe`/`egui` (with `egui_plot` for charts) depends on it. A thin binary provides the CLI and launches the GUI. The library never depends on the GUI (ADR-0003).
- **The library's public interface is the main seam.** It has four entry points:
  - **Config:** parse TOML into a resolved config, filling defaults and rejecting unknown keys. The resolved config can be serialised back out.
  - **Evolution:** construct from (resolved config, seed) for Generation 0 or from a Snapshot. Advance N Generations with a progress callback that is invoked every 10 Generations and on completion. Expose the current Generation, its Genomes, and per-Species metrics history. Produce a Snapshot.
  - **Showcase:** construct from a Snapshot, with the seed derived from (run seed, Generation). Step once; report whether the Episode has ended; read world state (Agents, food, Food Clusters, Obstacles, Light Phase) per Arena; read telemetry for an Agent by id (Eye Bins, core sensors, raw and applied outputs, energy breakdown, network activations), with the last telemetry retained after death; set Energy Rule toggles; restart.
  - **Snapshot:** save to and load from a path. The format is versioned JSON.
- **Artifact writing** is a library function that takes the metrics history, resolved config, and seed and writes `metrics.csv`, `fitness.svg`, and `run_summary.json` into a directory. The CLI and GUI continuation both call it.
- **Determinism (ADR-0001):** a single seeded random source is split deterministically into independent streams per Generation, per Episode, per Arena, and per Agent (for sensor noise). Iteration order is always deterministic, with no hash-map ordering in the simulation path. The simulation advances in fixed steps that are never tied to frame time. If evaluation is parallelised, results must still be reduced in a fixed order.
- **Species (ADR-0002):** Species = Controller Family × Role. Population size is configured per Species. The GA runs independently per Species.
- **Genome contents:** Controller Family, Role, network parameters (shape fixed per Controller Family from config), and the six Traits. Traits are clamped to configured ranges after mutation.
- **Controllers:**
  - MLP: a feed-forward network with configured hidden layer sizes and a tanh output.
  - LTC: a conductance-based liquid time-constant network with a configured neuron count, advanced by the semi-implicit (fused) solver with a configurable number of sub-steps per sim step. Neuron state resets at Episode start.
  - Both map the same fixed-width input vector to two outputs (steering, throttle) in [−1, 1].
- **Input vector layout:** normalised energy, normalised speed, then N food bins, N Prey bins, N Predator bins, and N Obstacle bins. N is configured once per run.
- **Arena Mode:**
  - Separated: one Arena per Controller Family, all built from the same Episode seed, so food, Obstacles, and spawn layouts are identical. Each holds that family's Prey and Predator Species.
  - Shared: a single Arena with all four Species.
  - The mode is part of the config and may change between Generations. It is recorded per metrics row.
- **Episode:** ends at the step limit from the Evaluation Schedule or when every Agent in the Arena is dead. Separated Arenas within one Episode run independently. There are no births; the dead stay dead.
- **Fitness:** per Agent per Episode, `w_survival[Role] * steps_alive + w_energy[Role] * energy_gained`. A Genome's Fitness is the mean over its Generation's Episodes.
- **Energy Rules:** idle drain (always on), movement, turning, sensing (scales with vision range and field of view), and trait upkeep. Each step's per-rule cost is recorded for telemetry. The toggles are part of the config.
- **Capture:** within capture range, the Prey dies and the Predator gains a fixed energy amount and enters a configurable cooldown.
- **Food:** three independent settings: motion (stationary | drift), replenishment (respawn-on-eat | fixed-rate up to a cap), and Food Clusters (on | off; clusters drift, with density following the Light Phase).
- **Obstacles:** a count and radius range are configured. Placement is non-overlapping and drawn from the Episode seed. Collision removes the velocity component pointing into the Obstacle, so Agents slide. Spawning avoids Obstacles.
- **Day/night:** sinusoidal over a configured period, starting at a configured phase each Episode. The visibility multiplier ranges from the configured night minimum to 1.0 and scales the effective range of the food, Prey, and Predator eyes only.
- **CLI:**
  - `batch` rejects a non-empty existing output directory.
  - `run` clears only its own known artifact files from the output directory, then proceeds.
  - Both print the seed when it was generated, report progress every 10 Generations and on completion, and write a Snapshot every 10 Generations and at the end.
  - `run` has a test-only way to skip opening the window, such as a hidden flag.
- **GUI continuation:** runs Evolution on a background thread over a copy of the Generation and edited config, sending progress messages back. On completion the viewer swaps in a new Showcase. Output goes to the loaded Snapshot's directory.
- **Viewer saves:** named `snapshot_gen_<n>_edit_<k>.json` with the smallest unused k ≥ 1, holding the current Genomes and the edited config.
- **Snapshot schema:** format version, run seed, Generation index, resolved config, Genomes for all Species, and the metrics history. There is no world state, and no backward compatibility with the earlier build's files.

## Testing Decisions

- **Good tests check external behaviour through public seams**, not internals. Tests set up small worlds and scenarios through the config (tiny Arenas, a handful of Agents, specific food or Obstacle settings) and assert on what the library reports. They do not call sensor maths, network code, or GA operators directly.
- **Seam 1, the library's public interface (the main test suite):**
  - Config: defaults fill in from an empty file; unknown keys fail; the resolved config round-trips.
  - Determinism: same seed and config give identical metrics and Genomes across two runs; a Snapshot continued for N Generations matches an uninterrupted run of the same total length; Showcase stepping gives identical state regardless of how steps are batched.
  - Evolution: Species sizes stay fixed; Role and Family never change; elites survive unchanged; Traits stay within ranges; the Fitness formula holds for constructed scenarios; the Evaluation Schedule controls the Episode count and length; an Arena Mode change takes effect at the next Generation and is recorded in metrics.
  - World rules, through the Showcase: an Agent with no energy dies and stays dead; toroidal wrap; Capture kills Prey, credits the Predator, and respects the cooldown; Predators don't eat food; Agents slide along and never enter Obstacles; Eye Bins activate for an object placed in a known sector; night reduces food, Prey, and Predator sensing but not Obstacle sensing; Energy Rule toggles change the reported energy breakdown on the next step; telemetry remains readable after the selected Agent dies.
  - Separated mode: both Arenas of an Episode have identical food and Obstacle layouts.
  - Snapshot: save/load round-trip preserves Genomes, config, seed, and history; an unsupported version is rejected.
- **Seam 2, the CLI binary as a process (acceptance tests),** covering the spec's Gherkin scenarios:
  - `batch` writes `metrics.csv`, `fitness.svg`, `run_summary.json`, and the last-Generation Snapshot.
  - `batch` fails with an explanatory error on a non-empty output directory.
  - `run` (with the window suppressed) completes exactly the requested Generations, clears stale run files but keeps unrelated ones, and its final Snapshot carries the full Fitness history.
  - The seed is printed when omitted; progress lines appear every 10 Generations.
- **The GUI is not tested automatically.** It is a thin layer; everything it displays comes from Seam 1.
- **Prior art:** none in this repo (fresh build). Use Rust's built-in test harness with integration tests per seam. Keep test configs tiny so the suite runs in seconds.

## Out of Scope

- Agents that can eat more than one kind of food.
- Topology-evolving neuroevolution such as NEAT; network shape is fixed per Controller Family.
- GPU or distributed training.
- Ecology beyond Prey/Predator, food, Food Clusters, day/night, and Obstacles.
- Bit-identical results across operating systems or CPUs (ADR-0001).
- Role or Controller Family mutation (ADR-0002).
- World state in Snapshots; resuming a Showcase mid-Episode.
- Loading Snapshots from the earlier build.
- Automatic scaling of food or world size when switching Arena Mode.
- Births during an Episode.
- Reverse thrust.

## Further Notes

- Build in stages: (1) library core with the Showcase stepping (world, sensing, controllers, Energy Rules); (2) Evolution, Config, Snapshot, and artifacts; (3) CLI; (4) GUI. Each stage is testable through Seam 1 before the next begins.
- `docs/SPECS.md` is the original behaviour spec. This document supersedes it where they differ: snapshot contents, viewer save naming, `run` output directory handling, and the addition of Arena Mode.
- Use the vocabulary in `CONTEXT.md` throughout code and docs: Genome, Agent, Species, Arena, Episode, Showcase, and so on.
