//! Run files: the metrics, chart, summary, and Snapshots an evolution run
//! leaves in its output directory.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use serde::Serialize;
use thiserror::Error;

use crate::config::Config;
use crate::evolution::{Evolution, Progress};
use crate::metrics::MetricsRow;
use crate::snapshot::SnapshotError;
use crate::species::Species;

const METRICS_FILE: &str = "metrics.csv";
const FITNESS_CHART_FILE: &str = "fitness.svg";
const RUN_SUMMARY_FILE: &str = "run_summary.json";

#[derive(Debug, Error)]
pub enum RunFilesError {
    #[error("output directory {path} is not empty; it must be absent or empty")]
    OutputDirNotEmpty { path: PathBuf },
    #[error("cannot write {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),
}

/// The file a Snapshot of `generation` is saved as.
pub fn snapshot_file_name(generation: u32) -> String {
    format!("snapshot_gen_{generation}.json")
}

/// Create `dir` if it is absent; refuse it if it exists and is not empty, so
/// files from different runs never mix.
pub fn prepare_output_dir(dir: impl AsRef<Path>) -> Result<(), RunFilesError> {
    let dir = dir.as_ref();
    let io_error = |source| RunFilesError::Io {
        path: dir.to_owned(),
        source,
    };
    if dir.exists() && dir.read_dir().map_err(io_error)?.next().is_some() {
        return Err(RunFilesError::OutputDirNotEmpty {
            path: dir.to_owned(),
        });
    }
    std::fs::create_dir_all(dir).map_err(io_error)
}

/// Write `metrics.csv`, `fitness.svg`, and `run_summary.json` for a run's
/// full metrics history into `dir`, replacing any earlier versions.
pub fn write_run_files(
    dir: impl AsRef<Path>,
    config: &Config,
    seed: u64,
    metrics: &[MetricsRow],
) -> Result<(), RunFilesError> {
    let dir = dir.as_ref();
    write_file(&dir.join(METRICS_FILE), &metrics_csv(metrics))?;
    write_file(&dir.join(FITNESS_CHART_FILE), &fitness_svg(metrics))?;
    let summary = RunSummary::new(config, seed, metrics);
    let json = serde_json::to_string_pretty(&summary).expect("run summary always serialises");
    write_file(&dir.join(RUN_SUMMARY_FILE), &json)
}

/// Advance `evolution` by `generations`, saving a Snapshot into `dir` whenever
/// the Generation reaches a multiple of `PROGRESS_INTERVAL`, then a Snapshot
/// of the final Generation and the run files. `on_progress` is called as by
/// `Evolution::advance`.
///
/// A failed write stops further writes but not the evolution; the first
/// error is returned once it finishes.
pub fn record_evolution(
    evolution: &mut Evolution,
    generations: u32,
    dir: impl AsRef<Path>,
    mut on_progress: impl FnMut(&Progress),
) -> Result<(), RunFilesError> {
    let dir = dir.as_ref();
    let mut result = Ok(());
    evolution.advance(generations, |progress| {
        // The callback fires at each multiple of PROGRESS_INTERVAL and on
        // completion; the final Snapshot is saved below, even for 0.
        if result.is_ok() && progress.completed != progress.total {
            result = save_snapshot(progress.evolution, dir);
        }
        on_progress(progress);
    });
    result?;
    save_snapshot(evolution, dir)?;
    write_run_files(
        dir,
        evolution.config(),
        evolution.seed(),
        evolution.metrics(),
    )
}

fn save_snapshot(evolution: &Evolution, dir: &Path) -> Result<(), RunFilesError> {
    let path = dir.join(snapshot_file_name(evolution.generation()));
    Ok(evolution.snapshot().save(path)?)
}

fn write_file(path: &Path, contents: &str) -> Result<(), RunFilesError> {
    std::fs::write(path, contents).map_err(|source| RunFilesError::Io {
        path: path.to_owned(),
        source,
    })
}

#[derive(Serialize)]
struct RunSummary<'a> {
    seed: u64,
    /// How many Generations have been evaluated.
    generations: u32,
    /// The last evaluated Generation's rows, one per Species.
    final_metrics: &'a [MetricsRow],
    config: &'a Config,
}

impl<'a> RunSummary<'a> {
    fn new(config: &'a Config, seed: u64, metrics: &'a [MetricsRow]) -> Self {
        let generations = metrics.last().map_or(0, |row| row.generation + 1);
        let final_start = metrics
            .iter()
            .position(|row| row.generation + 1 == generations)
            .unwrap_or(metrics.len());
        Self {
            seed,
            generations,
            final_metrics: &metrics[final_start..],
            config,
        }
    }
}

fn metrics_csv(metrics: &[MetricsRow]) -> String {
    let mut csv = String::from(
        "generation,species,episodes,episode_steps,best_fitness,mean_fitness,alive_at_end,food_eaten\n",
    );
    for row in metrics {
        writeln!(
            csv,
            "{},{},{},{},{},{},{},{}",
            row.generation,
            row.species,
            row.episodes,
            row.episode_steps,
            row.best_fitness,
            row.mean_fitness,
            row.alive_at_end,
            row.food_eaten,
        )
        .expect("writing to a String cannot fail");
    }
    csv
}

const CHART_WIDTH: f64 = 800.0;
const CHART_HEIGHT: f64 = 450.0;
const PLOT_LEFT: f64 = 70.0;
const PLOT_RIGHT: f64 = 620.0;
const PLOT_TOP: f64 = 40.0;
const PLOT_BOTTOM: f64 = 390.0;
const LEGEND_TOP: f64 = 50.0;
const LEGEND_SPACING: f64 = 18.0;
/// One colour per Species, in `Species::ALL` order.
const SERIES_COLOURS: [&str; 4] = ["#2a78c8", "#d4572a", "#2f9e61", "#8e4fc2"];

/// One of the lines drawn for each Species.
struct Line {
    statistic: &'static str,
    /// Extra SVG attributes for the stroke.
    stroke_style: &'static str,
    value: fn(&MetricsRow) -> f64,
}

const LINES: [Line; 2] = [
    Line {
        statistic: "best",
        stroke_style: "",
        value: |row| row.best_fitness,
    },
    Line {
        statistic: "mean",
        stroke_style: r#" stroke-dasharray="6 4""#,
        value: |row| row.mean_fitness,
    },
];

/// Best (solid) and mean (dashed) Fitness per Species against Generation.
fn fitness_svg(metrics: &[MetricsRow]) -> String {
    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{CHART_WIDTH}" height="{CHART_HEIGHT}" viewBox="0 0 {CHART_WIDTH} {CHART_HEIGHT}" font-family="sans-serif" font-size="12">
<rect width="100%" height="100%" fill="white"/>
<text x="{PLOT_LEFT}" y="24" font-size="16">Fitness per Species</text>
<line x1="{PLOT_LEFT}" y1="{PLOT_BOTTOM}" x2="{PLOT_RIGHT}" y2="{PLOT_BOTTOM}" stroke="black"/>
<line x1="{PLOT_LEFT}" y1="{PLOT_TOP}" x2="{PLOT_LEFT}" y2="{PLOT_BOTTOM}" stroke="black"/>
<text x="{x}" y="{y}" text-anchor="middle">Generation</text>
<text x="18" y="{mid}" text-anchor="middle" transform="rotate(-90 18 {mid})">Fitness</text>
"#,
        x = (PLOT_LEFT + PLOT_RIGHT) / 2.0,
        y = PLOT_BOTTOM + 40.0,
        mid = (PLOT_TOP + PLOT_BOTTOM) / 2.0,
    );
    if let Some(scale) = Scale::fit(metrics) {
        scale.write_ticks(&mut svg);
        let mut legend_y = LEGEND_TOP;
        for (&species, colour) in Species::ALL.iter().zip(SERIES_COLOURS) {
            let rows: Vec<&MetricsRow> = metrics
                .iter()
                .filter(|row| row.species == species)
                .collect();
            for Line {
                statistic,
                stroke_style,
                value,
            } in LINES
            {
                let points: Vec<String> = rows
                    .iter()
                    .map(|row| {
                        let (x, y) = scale.point(row.generation, value(row));
                        format!("{x:.2},{y:.2}")
                    })
                    .collect();
                writeln!(
                    svg,
                    r#"<polyline data-series="{species} {statistic}" fill="none" stroke="{colour}" stroke-width="2"{stroke_style} points="{points}"/>
<line x1="{x1}" y1="{legend_y}" x2="{x2}" y2="{legend_y}" stroke="{colour}" stroke-width="2"{stroke_style}/>
<text x="{text_x}" y="{text_y}">{species} {statistic}</text>"#,
                    points = points.join(" "),
                    x1 = PLOT_RIGHT + 20.0,
                    x2 = PLOT_RIGHT + 50.0,
                    text_x = PLOT_RIGHT + 58.0,
                    text_y = legend_y + 4.0,
                )
                .expect("writing to a String cannot fail");
                legend_y += LEGEND_SPACING;
            }
        }
    } else {
        writeln!(
            svg,
            r#"<text x="{x}" y="{y}" text-anchor="middle">No Generations evaluated yet</text>"#,
            x = (PLOT_LEFT + PLOT_RIGHT) / 2.0,
            y = (PLOT_TOP + PLOT_BOTTOM) / 2.0,
        )
        .expect("writing to a String cannot fail");
    }
    svg.push_str("</svg>\n");
    svg
}

/// Maps Generations and Fitness values onto the plot area.
struct Scale {
    first_generation: u32,
    last_generation: u32,
    min_fitness: f64,
    max_fitness: f64,
}

impl Scale {
    /// `None` when there is nothing to plot.
    fn fit(metrics: &[MetricsRow]) -> Option<Self> {
        let first_generation = metrics.iter().map(|row| row.generation).min()?;
        let last_generation = metrics.iter().map(|row| row.generation).max()?;
        let values = metrics
            .iter()
            .flat_map(|row| [row.best_fitness, row.mean_fitness]);
        let min_fitness = values.clone().fold(f64::INFINITY, f64::min);
        let max_fitness = values.fold(f64::NEG_INFINITY, f64::max);
        Some(Self {
            first_generation,
            last_generation,
            min_fitness,
            max_fitness,
        })
    }

    fn point(&self, generation: u32, fitness: f64) -> (f64, f64) {
        let x = interpolate(
            f64::from(generation),
            f64::from(self.first_generation),
            f64::from(self.last_generation),
            PLOT_LEFT,
            PLOT_RIGHT,
        );
        let y = interpolate(
            fitness,
            self.min_fitness,
            self.max_fitness,
            PLOT_BOTTOM,
            PLOT_TOP,
        );
        (x, y)
    }

    fn write_ticks(&self, svg: &mut String) {
        writeln!(
            svg,
            r#"<text x="{PLOT_LEFT}" y="{y}" text-anchor="middle">{first}</text>
<text x="{PLOT_RIGHT}" y="{y}" text-anchor="middle">{last}</text>
<text x="{x}" y="{PLOT_BOTTOM}" text-anchor="end">{min:.2}</text>
<text x="{x}" y="{top}" text-anchor="end">{max:.2}</text>"#,
            y = PLOT_BOTTOM + 18.0,
            x = PLOT_LEFT - 6.0,
            top = PLOT_TOP + 4.0,
            first = self.first_generation,
            last = self.last_generation,
            min = self.min_fitness,
            max = self.max_fitness,
        )
        .expect("writing to a String cannot fail");
    }
}

/// Map `value` from `[from_low, from_high]` onto `[to_low, to_high]`; a
/// degenerate source range maps to the middle of the target.
fn interpolate(value: f64, from_low: f64, from_high: f64, to_low: f64, to_high: f64) -> f64 {
    let span = from_high - from_low;
    if span > 0.0 {
        to_low + (value - from_low) / span * (to_high - to_low)
    } else {
        (to_low + to_high) / 2.0
    }
}
