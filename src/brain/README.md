# brain

## Purpose
This module defines shared controller interfaces and deterministic MLP/LTC controller implementations.

## Behavior
- Exposes `NeuralController`, `ControllerFactory`, and `ControllerKind` as the common brain abstraction.
- Defines serializable `BrainGenome` payloads for controller parameters.
- Implements MLP and LTC genome encode/decode routines and deterministic stepping/inference.
- Includes a mixed-family controller factory that can generate and build both controller kinds.
- Uses config-driven controller capacity settings (`mlp_hidden_size`, `ltc_state_size`) when factories are created by genetics/experiment flows.
- Maps controller outputs into simulation actions (`steering`, `throttle`).

## Inputs
- `SensorVector` values from the simulation.
- Serialized controller genomes.
- Deterministic RNG (`StdRng`) for genome sampling.

## Outputs
- Runtime controller actions via `NeuralController::step`.
- Encoded/decoded genome parameter vectors.

## Side Effects
- No external side effects; all operations are in-memory.
