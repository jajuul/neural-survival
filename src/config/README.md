# config

## Purpose
This module defines simulation configuration data and validation rules.

## Behavior
It loads TOML configuration into strongly typed Rust structs, applies spec defaults, and validates key numeric ranges.

## Inputs
- A TOML document from disk or in-memory text.

## Outputs
- A validated `SimulationConfig` value.
- A `ConfigError` when parsing or validation fails.

## Side Effects
- File I/O when loading a config from path.
