# brain

## Purpose
This module defines shared controller interfaces and deterministic MLP/LTC controller implementations.

## Behavior
- Exposes `NeuralController`, `ControllerFactory`, and `ControllerKind` as the common brain abstraction.
- Defines serializable `BrainGenome` payloads for controller parameters.
- Implements MLP and a conductance-based LTC genome encode/decode routines and deterministic stepping/inference.
- The LTC path uses gated sensory/recurrent synapses, leak conductance, membrane capacitance, and a fixed-count internal ODE unfold per simulation step.
- Includes a mixed-family controller factory that can generate and build both controller kinds.
- Uses config-driven controller capacity settings (`mlp_hidden_size`, `ltc_state_size`) when factories are created by genetics/experiment flows.
- Maps controller outputs into simulation actions (`steering`, `throttle`).
- Exposes `step_with_observability` so UI code can inspect raw steering/throttle signals alongside the applied action.

## Inputs
- `SensorVector` values from the simulation.
- Serialized controller genomes.
- Deterministic RNG (`StdRng`) for genome sampling.

## Outputs
- Runtime controller actions via `NeuralController::step`.
- Runtime controller observability via `NeuralController::step_with_observability`.
- Encoded/decoded genome parameter vectors.

## Side Effects
- No external side effects; all operations are in-memory.
