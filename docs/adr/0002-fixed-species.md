# Role and Controller Family are fixed per Species

A Genome's Role (Prey or Predator) and Controller Family (MLP or LTC) never mutate. The population is split into four Species, one per combination, and each has its own breeding pool and fixed population size. We chose this over letting Role or Family mutate so that selection pressure cannot drive one combination extinct, which would end the MLP-vs-LTC comparison the project exists to make, and so that crossover only ever combines Genomes with the same network shape.
