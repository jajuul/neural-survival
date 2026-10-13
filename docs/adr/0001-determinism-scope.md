# Determinism is per-platform and independent of presentation

The same seed, config, and binary must give identical results on the same machine, and a GUI continuation from a snapshot must give exactly the results a headless run would. We do not promise bit-identical results across operating systems or CPUs, because that would need software floating point or fixed-point maths throughout. So the simulation advances in fixed steps that are never tied to frame time: GUI speed, pausing, and single-stepping change only how often steps run, never what a step computes.
