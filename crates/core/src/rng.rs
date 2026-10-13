//! Deterministic random streams (ADR-0001). A parent seed is split into
//! independent child streams by hashing it with a stream tag and index, so
//! adding draws to one stream never shifts another.

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

pub(crate) type Rng = ChaCha8Rng;

#[derive(Debug, Clone, Copy)]
#[repr(u64)]
pub(crate) enum Stream {
    Genomes = 1,
    Arena = 2,
}

pub(crate) fn derive_seed(parent: u64, stream: Stream, index: u64) -> u64 {
    splitmix64(splitmix64(parent ^ splitmix64(stream as u64)) ^ index)
}

pub(crate) fn stream_rng(parent: u64, stream: Stream, index: u64) -> Rng {
    Rng::seed_from_u64(derive_seed(parent, stream, index))
}

fn splitmix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}
