# Disclaimer
AI coding test. All code made with AI in this repo.

# neural-survival

Evolve neural-network-controlled creatures in a deterministic 2D toroidal world with a genetic algorithm. `CONTEXT.md` defines the domain terms.

The command line currently has one command, `batch`, which evolves headlessly for a fixed number of Generations and writes run files:

```bash
cargo run -- batch --config <config.toml> --generations 50 --out results
```

Every config field has a default, and `--seed` is generated and printed when omitted. The roadmap and tickets are in `.scratch/neural-survival-v1/`.

## Contributing

Code here is written by AI agents and approved by a human through a review workflow. Start with [CONTRIBUTING.md](CONTRIBUTING.md) for setup and an overview of how a change is made.
