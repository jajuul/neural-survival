# brain

## Purpose
This module defines the shared controller interface and the deterministic feed-forward MLP controller implementation.

## Behavior
- Exposes `NeuralController`, `ControllerFactory`, and `ControllerKind` as the common brain abstraction.
- Defines serializable `BrainGenome` payloads for controller parameters.
- Implements MLP genome encode/decode routines and deterministic matrix-vector inference.
- Maps MLP outputs into simulation actions (`steering`, `throttle`).

## Inputs
- `SensorVector` values from the simulation.
- Serialized controller genomes.
- Deterministic RNG (`StdRng`) for genome sampling.

## Outputs
- Runtime controller actions via `NeuralController::step`.
- Encoded/decoded genome parameter vectors.

## Side Effects
- No external side effects; all operations are in-memory.
