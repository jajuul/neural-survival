## General Info
- Work in the Rust programming language primarily

## ExecPlans
When writing complex features or significant refactors, use an ExecPlan (as described in .agent/PLANS.md) from design to implementation.

## Specs
- See docs/SPECS.md for the specs of the applications. 

## Test Expectations
- New code should include unit tests where relevant.
- Nodes should include module-level integration tests verifying expected inputs and outputs.
- Before handing off changes, run tests for affected packages at minimum.

## Documentation Expectations
- Add comments at non-obvious or high-impact logic boundaries.
- Functions should document behavior, inputs, outputs, and side effects.
- Each module should have a `README.md` describing purpose, behavior, inputs, and outputs.
- Git commit often and with good descriptions of the content of the commit

## Architecture and Code Guidelines
- Keep designs simple and maintainable.
- Prefer composition over inheritance.
- Write code in a way that supports straightforward unit testing.

## When Stuck
- Ask clarifying questions.
- If uncertainty remains, propose a short implementation plan and request feedback before proceeding.

## Ask First (Non-Exhaustive)
- If doing destructive and / or mutating commands outside the current project directory
- Installing software or packages.
- Accessing files outside the project workspace.
