# Neural Survival

A 2D toroidal world where populations of neural-network-controlled agents are evolved by a genetic algorithm, inspected, and continued from saved snapshots.

## Evolution

**Genome**:
The heritable unit the genetic algorithm selects, recombines, and mutates: controller family, network parameters, and mutable traits.
_Avoid_: Individual, creature, organism, brain

**Species**:
A fixed breeding pool defined by one Controller Family and one Role, with a fixed population size. Genomes never move between Species.
_Avoid_: Subpopulation, group, team

**Controller Family**:
The kind of neural network a Genome encodes: either MLP (feed-forward) or LTC (conductance-based liquid time-constant).
_Avoid_: Brain type, network type

**Role**:
A Genome's fixed ecological part: Prey eat food; Predators capture Prey and never eat food.

**Trait**:
A mutable, heritable Genome parameter shaping an Agent's body and senses: vision range, field of view, max speed, max acceleration, turn rate, or body radius. Each Trait carries an energy cost that grows with its value.

**Generation**:
One round of the genetic algorithm: every Genome is evaluated over the Episodes the Evaluation Schedule prescribes, then the next set of Genomes is bred.

**Episode**:
A single simulated run of the world in which Agents live and are scored. A Generation consists of one or more Episodes whose scores are averaged.
_Avoid_: Trial, round, epoch

**Evaluation Schedule**:
The rule giving how many Episodes a Generation runs and how long each lasts, possibly varying by Generation.

**Fitness**:
An Agent's Episode score, a per-Role weighted sum of steps survived and energy gained (from food for Prey, from captures for Predators); a Genome's Fitness is the mean over its Generation's Episodes.

**Showcase**:
An Episode run in the viewer to watch the current Genomes live; it is never scored and never affects evolution.
_Avoid_: Demo, replay, preview

**Snapshot**:
A saved Generation: its Genomes, the resolved config, the run seed, and the metrics history. It holds no world state; opening it starts a fresh Showcase.

## World

**Arena**:
One instance of the world within an Episode, built from the full world config.
_Avoid_: Map, level

**Arena Mode**:
Whether Controller Families live apart or together. **Separated**: each Controller Family gets its own Arena, all Arenas of an Episode sharing one Episode seed. **Shared**: all Species live in one Arena.

**Agent**:
A Genome embodied in the world for one Episode, with position, heading, speed, and energy.
_Avoid_: Individual, creature, bot

**Energy Rule**:
One source of energy drain applied to Agents each step (idle drain, movement, turning, sensing, trait upkeep). Every rule except idle drain can be toggled.
_Avoid_: Cost model, energy toggle

**Eye Bin**:
One angular sector of an Agent's field of view for one kind of object (food, Prey, Predator, or obstacle). Its activation grows as the nearest such object in that sector gets closer.
_Avoid_: Ray, sensor slot

**Capture**:
A Predator coming within capture range of a Prey: the Prey dies and the Predator gains a fixed amount of energy, then cannot capture again until a cooldown passes.
_Avoid_: Kill, eat (eating is for food)

**Obstacle**:
A static, round, impassable region of an Arena, placed from the Episode seed. Agents slide along its edge rather than stopping.

**Food Cluster**:
A drifting region around which food spawns, whose density follows the Light Phase.

**Light Phase**:
The current point in the day/night cycle (day, dusk, night, dawn), which sets the visibility multiplier scaling Agents' effective vision range.
_Avoid_: Time of day, lighting
