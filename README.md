# Disclaimer
AI coding test. All code made with AI in this repo.

# neural-survival

Evolve neural-network-controlled creatures in a deterministic 2D toroidal world with a genetic algorithm. `CONTEXT.md` defines the domain terms.

The command line has two commands. `batch` evolves headlessly for a fixed number of Generations and writes run files:

```bash
cargo run -- batch --config <config.toml> --generations 50 --out results
```

Every config field has a default, and `--seed` is generated and printed when omitted. The roadmap and tickets are in `.scratch/neural-survival-v1/`.

`view` opens a window that plays a Showcase of a saved Generation, paused at step 0, with playback controls, stats, and the Fitness history chart:

```bash
cargo run -- view --snapshot results/snapshot_gen_50.json
```

### GUI prerequisites on Linux and WSL

Building needs no extra system packages: the windowing libraries are loaded when the window opens. To open it, Ubuntu 24.04 needs:

```bash
sudo apt install libxkbcommon-x11-0 libxcursor1 libxi6 libxrandr2 libwayland-client0 libgl1 libegl1 libgl1-mesa-dri
```

Under WSL the window needs WSLg. `libEGL warning` and `MESA: error: ZINK` lines on startup are harmless there: Mesa falls back to software rendering.

## Contributing

Code here is written by AI agents and approved by a human through a review workflow. Start with [CONTRIBUTING.md](CONTRIBUTING.md) for setup and an overview of how a change is made.
