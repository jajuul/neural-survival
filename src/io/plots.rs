use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use crate::experiment::GenerationMetrics;

const SVG_WIDTH: f32 = 960.0;
const SVG_HEIGHT: f32 = 540.0;
const MARGIN_LEFT: f32 = 70.0;
const MARGIN_RIGHT: f32 = 24.0;
const MARGIN_TOP: f32 = 24.0;
const MARGIN_BOTTOM: f32 = 56.0;

/// Write an SVG chart plotting best and mean fitness over generations.
///
/// Inputs: output directory and per-generation metrics.
/// Output: written file path.
/// Side effects: writes `fitness.svg` in `out_dir`.
pub fn write_fitness_svg(out_dir: &Path, metrics: &[GenerationMetrics]) -> anyhow::Result<PathBuf> {
    let path = out_dir.join("fitness.svg");
    let mut svg = String::with_capacity(16 * 1024);

    write!(
        &mut svg,
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{SVG_WIDTH}" height="{SVG_HEIGHT}" viewBox="0 0 {SVG_WIDTH} {SVG_HEIGHT}" role="img" aria-label="Fitness over generations">"##
    )?;
    svg.push_str(
        r##"<rect x="0" y="0" width="100%" height="100%" fill="#12161d"/><text x="70" y="20" fill="#dce6f2" font-size="16" font-family="sans-serif">Fitness Over Generations</text>"##,
    );

    let plot_left = MARGIN_LEFT;
    let plot_top = MARGIN_TOP + 16.0;
    let plot_width = SVG_WIDTH - MARGIN_LEFT - MARGIN_RIGHT;
    let plot_height = SVG_HEIGHT - MARGIN_TOP - MARGIN_BOTTOM - 16.0;
    let plot_right = plot_left + plot_width;
    let plot_bottom = plot_top + plot_height;

    write!(
        &mut svg,
        r##"<rect x="{plot_left}" y="{plot_top}" width="{plot_width}" height="{plot_height}" fill="#1b2330" stroke="#3b4b63" stroke-width="1"/>"##
    )?;

    if metrics.is_empty() {
        write!(
            &mut svg,
            r##"<text x="{}" y="{}" fill="#c4cfdd" font-size="14" font-family="sans-serif">No generation metrics available</text>"##,
            plot_left + 16.0,
            plot_top + 32.0
        )?;
        svg.push_str("</svg>");
        fs::write(&path, svg)?;
        return Ok(path);
    }

    let min_generation = metrics.first().map(|m| m.generation_index).unwrap_or(0);
    let max_generation = metrics.last().map(|m| m.generation_index).unwrap_or(0);

    let max_fitness = metrics
        .iter()
        .flat_map(|m| [m.best_fitness, m.mean_fitness])
        .fold(0.0_f32, f32::max)
        .max(1.0);

    let x_for_index = |idx: usize, total: usize| -> f32 {
        if total <= 1 {
            return plot_left + plot_width * 0.5;
        }
        plot_left + (idx as f32 / (total.saturating_sub(1) as f32)) * plot_width
    };
    let y_for_value = |value: f32| -> f32 {
        let ratio = (value / max_fitness).clamp(0.0, 1.0);
        plot_bottom - ratio * plot_height
    };

    let mut best_points = String::new();
    let mut mean_points = String::new();
    for (idx, metric) in metrics.iter().enumerate() {
        let x = x_for_index(idx, metrics.len());
        let best_y = y_for_value(metric.best_fitness);
        let mean_y = y_for_value(metric.mean_fitness);
        write!(&mut best_points, "{x:.2},{best_y:.2} ")?;
        write!(&mut mean_points, "{x:.2},{mean_y:.2} ")?;
    }

    let y_axis_label_x = plot_left - 54.0;
    let x_axis_label_y = plot_bottom + 42.0;
    let top_label_y = plot_top + 4.0;
    let mid_y = plot_top + plot_height * 0.5;

    write!(
        &mut svg,
        r##"<line x1="{plot_left}" y1="{plot_bottom}" x2="{plot_right}" y2="{plot_bottom}" stroke="#7d8ea7" stroke-width="1"/><line x1="{plot_left}" y1="{plot_top}" x2="{plot_left}" y2="{plot_bottom}" stroke="#7d8ea7" stroke-width="1"/>"##
    )?;
    write!(
        &mut svg,
        r##"<line x1="{plot_left}" y1="{mid_y}" x2="{plot_right}" y2="{mid_y}" stroke="#2d3c52" stroke-width="1" stroke-dasharray="4 4"/>"##
    )?;

    write!(
        &mut svg,
        r##"<polyline fill="none" stroke="#f39c12" stroke-width="2.5" points="{best_points}"/><polyline fill="none" stroke="#4ea8de" stroke-width="2.5" points="{mean_points}"/>"##
    )?;

    write!(
        &mut svg,
        r##"<text x="{y_axis_label_x}" y="{top_label_y}" fill="#c4cfdd" font-size="12" font-family="sans-serif">{max_fitness:.2}</text><text x="{y_axis_label_x}" y="{plot_bottom}" fill="#c4cfdd" font-size="12" font-family="sans-serif">0.00</text>"##
    )?;
    write!(
        &mut svg,
        r##"<text x="{plot_left}" y="{x_axis_label_y}" fill="#c4cfdd" font-size="12" font-family="sans-serif">gen {min_generation}</text><text x="{plot_right}" y="{x_axis_label_y}" text-anchor="end" fill="#c4cfdd" font-size="12" font-family="sans-serif">gen {max_generation}</text>"##
    )?;

    let legend_x = plot_right - 170.0;
    let legend_y = plot_top + 16.0;
    write!(
        &mut svg,
        r##"<line x1="{legend_x}" y1="{legend_y}" x2="{}" y2="{legend_y}" stroke="#f39c12" stroke-width="3"/><text x="{}" y="{}" fill="#dce6f2" font-size="12" font-family="sans-serif">best fitness</text>"##,
        legend_x + 18.0,
        legend_x + 24.0,
        legend_y + 4.0
    )?;
    write!(
        &mut svg,
        r##"<line x1="{legend_x}" y1="{}" x2="{}" y2="{}" stroke="#4ea8de" stroke-width="3"/><text x="{}" y="{}" fill="#dce6f2" font-size="12" font-family="sans-serif">mean fitness</text>"##,
        legend_y + 18.0,
        legend_x + 18.0,
        legend_y + 18.0,
        legend_x + 24.0,
        legend_y + 22.0
    )?;

    svg.push_str("</svg>");
    fs::write(&path, svg)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::experiment::GenerationMetrics;

    use super::write_fitness_svg;

    fn temp_dir(prefix: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after UNIX_EPOCH")
            .as_nanos();
        std::env::temp_dir().join(format!("neural-survival-{prefix}-{nanos}"))
    }

    #[test]
    fn write_fitness_svg_outputs_best_and_mean_series() {
        let out_dir = temp_dir("fitness-plot");
        fs::create_dir_all(&out_dir).expect("temp dir should be created");

        let metrics = vec![
            GenerationMetrics {
                generation_index: 0,
                best_fitness: 1.2,
                mean_fitness: 0.6,
                survival_min: 1,
                survival_mean: 2.0,
                survival_max: 3,
                population_size: 4,
                controller_mlp: 2,
                controller_ltc: 2,
            },
            GenerationMetrics {
                generation_index: 1,
                best_fitness: 2.4,
                mean_fitness: 1.1,
                survival_min: 1,
                survival_mean: 2.4,
                survival_max: 4,
                population_size: 4,
                controller_mlp: 2,
                controller_ltc: 2,
            },
        ];

        let path = write_fitness_svg(&out_dir, &metrics).expect("plot writing should succeed");
        let content = fs::read_to_string(path).expect("plot should be readable");

        assert!(content.contains("best fitness"));
        assert!(content.contains("mean fitness"));
        assert!(content.contains("<polyline"));
    }
}
