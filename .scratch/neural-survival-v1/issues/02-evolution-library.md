# 02: Evolution library

**What to build:** Evolution in the headless library: from a resolved config and seed (or from a Snapshot), advance N Generations. Each Generation runs the Episodes the Evaluation Schedule prescribes, scores each Agent's Fitness, averages per Genome, and breeds the next Genomes within each Species. Snapshots save and load as versioned JSON. For now only the MLP-Prey Species exists, but per-Species structure is in place.

**Blocked by:** 01: World: Prey forage in a Showcase

**Status:** ready-for-agent

- [ ] Evolution can be constructed from (resolved config, seed) for Generation 0
- [ ] The Evaluation Schedule sets Episode count and Episode step limit and can vary by Generation
- [ ] Fitness per Agent per Episode is `w_survival * steps_alive + w_energy * energy_gained` with per-Role weights from config; a Genome's Fitness is the mean over its Episodes
- [ ] The GA runs per Species: tournament selection (configurable size), elitism (top k copied unchanged), uniform crossover, Gaussian mutation (configurable rate and sigma)
- [ ] Species population sizes stay fixed across Generations
- [ ] A progress callback fires every 10 Generations and on completion
- [ ] Per-Species metrics history (best Fitness, mean Fitness, alive at Episode end, food eaten) is exposed
- [ ] Snapshot holds a format version, run seed, Generation index, resolved config, all Genomes, and metrics history, with no world state; it round-trips through save/load; an unsupported version is rejected with a clear error
- [ ] Same seed and config give identical Genomes and metrics across two runs; random streams are split per Generation/Episode/Arena/Agent (ADR-0001)
- [ ] A Showcase can be constructed from a Snapshot with seed derived from (run seed, Generation)
- [ ] Integration tests cover each of the above through the library's public interface
