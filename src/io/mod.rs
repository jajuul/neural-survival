//! Artifact persistence helpers for metrics, summaries, and snapshots.

use std::fs;
use std::path::Path;

pub mod metrics;
pub mod plots;
pub mod snapshot;
pub mod summary;

/// Ensure output directory exists and is empty.
///
/// Inputs: output directory path.
/// Output: `Ok(())` when directory is ready for writing.
/// Side effects: may create directory; returns error if directory is non-empty.
pub fn ensure_output_dir(path: &Path) -> anyhow::Result<()> {
    if path.exists() {
        if !path.is_dir() {
            anyhow::bail!(
                "output path exists but is not a directory: {}",
                path.display()
            );
        }

        let mut entries = fs::read_dir(path)?;
        if entries.next().transpose()?.is_some() {
            anyhow::bail!(
                "output directory already exists and is not empty: {}",
                path.display()
            );
        }
    } else {
        fs::create_dir_all(path)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::ensure_output_dir;

    fn temp_dir(prefix: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after UNIX_EPOCH")
            .as_nanos();
        std::env::temp_dir().join(format!("neural-survival-{prefix}-{nanos}"))
    }

    #[test]
    fn ensure_output_dir_accepts_new_directory() {
        let dir = temp_dir("io-new");
        ensure_output_dir(&dir).expect("new directory should be accepted");
        assert!(dir.exists());
    }

    #[test]
    fn ensure_output_dir_rejects_non_empty_directory() {
        let dir = temp_dir("io-non-empty");
        fs::create_dir_all(&dir).expect("temporary directory must be created");
        fs::write(dir.join("existing.txt"), "data").expect("seed file must be written");

        let result = ensure_output_dir(&dir);
        assert!(result.is_err());
    }
}
