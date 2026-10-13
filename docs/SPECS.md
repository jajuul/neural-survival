# Neural Survival v1 Behavior Specification

## Purpose
`neural-survival` is a native Rust desktop application where the user evolves neural-network-controlled agents in a 2D world, inspects the resulting populations, and continues evolution from saved snapshots.

This document describes the externally visible behavior of the application as it exists now. It focuses on what a user can run, see, save, and inspect without describing the internal implementation details.

## Product Goals
1. Evolve agents with a genetic algorithm in a deterministic 2D toroidal environment.
2. Compare MLP and LTC controller families under the same world rules.
3. Model mutable traits that affect sensing, movement, and energy economics.
4. Support configurable food replenishment, obstacles, lighting, and simple predator/prey ecology.
5. Support both headless experiment workflows and GUI-based inspection/continuation.
6. Export enough artifacts to compare runs offline and reopen saved populations later.
7. Surface progress during long-running evolution in both CLI and GUI workflows.

## Non-Goals
1. Multi-food edibility behavior in live simulation.
2. Topology-evolving neuroevolution methods such as NEAT.
3. GPU or distributed training.
4. Full ecology beyond the current predator/prey split, food clusters, day/night visibility, and obstacles.

## User Workflows

### Train Then Visualize
The user runs the `run` command with a config path, output directory, a requested number of headless generations, and optionally a seed. The application evolves the population headlessly, writes the run artifacts, and then opens the GUI using the final snapshot. The GUI starts paused and shows the loaded population, world state, and historical fitness data carried in the snapshot.

### Batch Experiments
The user runs the `batch` command for a fixed number of generations. The application performs headless evolution only, writes the required artifacts, and exits. The output directory must either not exist yet or be empty before the run starts so previous artifacts are not mixed with the new run.

### Interactive Inspection
The user opens a snapshot in the GUI with the `view` command or arrives in the GUI through `run`. The GUI renders the world, supports pausing and stepping, exposes speed control, shows top-level world statistics, and lets the user inspect one selected agent in detail. The user can continue evolution from the loaded population by editing continuation parameters directly in the viewer.

## CLI Behavior
The application exposes these commands:

- `neural-survival run --config <path> [--seed <u64>] --headless-generations <u32> --out <dir>`
- `neural-survival batch --config <path> [--seed <u64>] --generations <u32> --out <dir>`
- `neural-survival view --snapshot <path>`

For `run` and `batch`, the application reports generation progress every 10 generations and also reports completion. If `--seed` is omitted, the application generates one and prints it so the run can be reproduced later.

## Simulation Behavior
The simulated world is a 2D torus, so agents and food wrap around the world edges. Inside that toroidal world, the simulation may also contain static round obstacles that block agent movement and remain visible in the GUI.

Food exists as point-like items with collision radius. Depending on configuration, food may:
- remain stationary
- drift through the world
- respawn immediately when consumed
- spawn at a fixed deterministic rate over simulated time
- belong to moving food clusters whose density changes over the day/night cycle

Agents move with steering and forward acceleration rather than direct speed assignment. They lose energy over time, may gain energy from food or prey capture, and die when their energy reaches zero.

The simulation supports two ecological roles:
- prey consume food
- predators capture prey at close range and do not consume food directly

The controller input is a fixed-width sensor vector containing:
- normalized energy and speed
- food eye bins
- prey eye bins
- predator eye bins
- obstacle eye bins

The optional day/night cycle changes sensing visibility. Optional deterministic sensor noise and dropout can further perturb sensed values. The controller then produces steering and throttle actions each step.

The system supports two controller families:
- feed-forward MLP
- conductance-based LTC

Both controller families may exist in the same run, but reproduction is partitioned so MLP and LTC do not interbreed.

## GUI Behavior
The GUI shows the current world state and enough observability to inspect both population-level behavior and one selected agent.

The GUI displays:
- current generation index
- current world step
- alive agent count
- food remaining
- best and mean fitness history
- controller-family counts
- active evaluation schedule
- active rule-toggle states
- current light phase and visibility multiplier when day/night is enabled
- continuation progress while GUI-triggered evolution is running

The world view displays:
- prey and predator agents with different colors
- MLP and LTC agents with different shapes
- food items
- food-cluster overlays when clusters are enabled
- round obstacle overlays when obstacles are enabled

The selected-agent inspector displays:
- controller family and ecological role
- current food, prey, predator, and obstacle eye-bin activations
- core sensor values
- raw controller outputs and applied throttle
- last-step energy-cost breakdown
- a live network visualization
- the last captured telemetry even after the selected agent has died

The GUI allows:
- pause
- continue
- single-step while paused
- simulation speed changes
- toggling supported energy rules
- restarting from the snapshot seed/config state
- saving a new viewer snapshot
- editing continuation parameters from the full runtime config surface
- editing the continuation evaluation schedule
- continuing evolution from the viewer
- selecting and clearing agent selection with clicks

The GUI continuation editor includes the current runtime configuration surface for world, food, obstacles, clusters, GA, fitness, sensors, predator/prey settings, and rule toggles. Some fields are initialization-oriented rather than retroactive; the GUI should not imply that every edited default immediately rewrites already-evolved genomes in place.

## Data Export Behavior
Headless runs write:
- `metrics.csv`
- `fitness.svg`
- `run_summary.json`
- `snapshot_gen_<n>.json`

Viewer snapshot saves write files named like:
- `snapshot_gen_<n>_step_<m>.json`

Snapshots contain enough information to reopen the viewer with the saved world state, saved population, resolved config, run seed, and historical metrics. Older snapshots are expected to continue loading even when controller input width has expanded, for example after adding new sensor channels such as obstacle sensing.

## Acceptance Scenarios

### Feature: Batch experiment outputs
```gherkin
Feature: Batch experiment outputs

  Scenario: A headless batch run writes the required artifacts
    Given a valid config file and seed 42
    And an output directory that does not exist
    When the user runs "neural-survival batch --config config.toml --seed 42 --generations 10 --out results"
    Then the command completes successfully
    And the directory "results" contains "metrics.csv"
    And the directory "results" contains "fitness.svg"
    And the directory "results" contains "run_summary.json"
    And the directory "results" contains a snapshot file for the last generation

  Scenario: A batch run rejects a non-empty output directory
    Given a valid config file and seed 42
    And the output directory "results" already contains files
    When the user runs "neural-survival batch --config config.toml --seed 42 --generations 10 --out results"
    Then the command fails
    And the error explains that the output directory must be absent or empty
```

### Feature: Train then open the viewer
```gherkin
Feature: Train then open the viewer

  Scenario: Headless generations are completed before the GUI starts
    Given a valid config file and seed 7
    And an empty output directory
    When the user runs "neural-survival run --config config.toml --seed 7 --headless-generations 20 --out results"
    Then the application evolves exactly 20 headless generations
    And the GUI opens using the generation-20 snapshot
    And the GUI shows the historical best and mean fitness carried in that snapshot
```

### Feature: Snapshot inspection and continuation
```gherkin
Feature: Snapshot inspection and continuation

  Scenario: A saved snapshot can be opened for inspection
    Given a previously exported snapshot file
    When the user runs "neural-survival view --snapshot snapshot_gen_20.json"
    Then the GUI opens using that snapshot
    And the user can pause and resume the simulation
    And the user can change the simulation speed
    And the user can select an agent to inspect its telemetry

  Scenario: The user continues evolution from the GUI
    Given the GUI is open with a loaded snapshot
    And the user has entered continuation parameters and a generation count
    When the user starts continuation evolution
    Then the GUI shows continuation progress every 10 generations
    And the best and mean fitness history updates as continuation proceeds
```

### Feature: Selected-agent inspection
```gherkin
Feature: Selected-agent inspection

  Scenario: Selecting an agent exposes its current telemetry
    Given the GUI is rendering a running simulation
    When the user clicks an agent
    Then the inspector shows the agent's food, prey, predator, and obstacle eye-bin activations
    And the inspector shows the agent's core sensor values
    And the inspector shows raw controller outputs and applied throttle
    And the inspector shows the last-step energy breakdown
    And the inspector shows a live network visualization for that controller

  Scenario: A selected agent remains inspectable after death
    Given an agent is currently selected in the GUI
    When that agent dies
    Then the inspector continues showing the last captured telemetry for that agent
```

### Feature: World-rule toggles
```gherkin
Feature: World-rule toggles

  Scenario: Energy rule toggles can be changed during GUI inspection
    Given the GUI is open with a loaded simulation
    When the user toggles an energy rule
    Then the GUI reflects the new toggle state
    And subsequent simulation steps use the updated rule state
```
