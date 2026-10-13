//! Snapshots: a saved Generation as versioned JSON. They hold Genomes,
//! config, run seed, and metrics history, but no world state.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::Config;
use crate::genome::Genome;
use crate::metrics::MetricsRow;

/// The only Snapshot format this build reads and writes.
pub const SNAPSHOT_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Error)]
pub enum SnapshotError {
    #[error("cannot access snapshot {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("snapshot {path} is not valid: {source}")]
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error(
        "snapshot {path} has format version {found}, \
         but this build only supports version {SNAPSHOT_FORMAT_VERSION}"
    )]
    UnsupportedVersion { path: PathBuf, found: u64 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub format_version: u32,
    pub seed: u64,
    pub generation: u32,
    pub config: Config,
    /// Every Species' Genomes, Species by Species in `Species::ALL` order.
    pub genomes: Vec<Genome>,
    pub metrics: Vec<MetricsRow>,
}

/// Just enough of a Snapshot to check its version before reading the rest;
/// wide enough that any future version number reads as unsupported.
#[derive(Deserialize)]
struct VersionProbe {
    format_version: u64,
}

impl Snapshot {
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), SnapshotError> {
        let path = path.as_ref();
        let json = serde_json::to_string_pretty(self).map_err(|source| SnapshotError::Json {
            path: path.to_owned(),
            source,
        })?;
        std::fs::write(path, json).map_err(|source| SnapshotError::Io {
            path: path.to_owned(),
            source,
        })
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, SnapshotError> {
        let path = path.as_ref();
        let json_error = |source| SnapshotError::Json {
            path: path.to_owned(),
            source,
        };
        let text = std::fs::read_to_string(path).map_err(|source| SnapshotError::Io {
            path: path.to_owned(),
            source,
        })?;
        let probe: VersionProbe = serde_json::from_str(&text).map_err(json_error)?;
        if probe.format_version != u64::from(SNAPSHOT_FORMAT_VERSION) {
            return Err(SnapshotError::UnsupportedVersion {
                path: path.to_owned(),
                found: probe.format_version,
            });
        }
        serde_json::from_str(&text).map_err(json_error)
    }
}
