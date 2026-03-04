# fitness

## Purpose
This module computes weighted scalar fitness values for evolutionary selection.

## Behavior
- Accepts episode outcome components (survival, food consumed, final energy).
- Applies config-provided weights to produce one deterministic fitness score.

## Inputs
- `FitnessComponents` per evaluated genome.
- `SimulationConfig` fitness weights.

## Outputs
- Scalar `f32` fitness value.

## Side Effects
- No side effects.
