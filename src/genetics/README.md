# genetics

## Purpose
This module provides genome types and genetic operators for population evolution.

## Behavior
- Defines `TraitGenome` and `AgentGenome` structures.
- Evolves sensor morphology traits including perception range, eye count, and per-eye angular width.
- Initializes mixed-family populations from deterministic RNG.
- Evolves populations using elitism, tournament selection, crossover, and mutation.
- Runs separate evolution pools per controller family (MLP and LTC), preserving per-family counts and preventing cross-family interbreeding.
- Enforces trait mutation bounds and stable brain parameter mutation paths.

## Inputs
- `SimulationConfig` GA parameters.
- Deterministic RNG streams.
- Scored genomes from episode evaluations.

## Outputs
- Initial populations.
- Next-generation populations with preserved size.

## Side Effects
- No external side effects.
