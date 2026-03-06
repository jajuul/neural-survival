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

/// Write an SVG chart plotting species-split best and mean fitness over generations.
///
/// Inputs: output directory and per-generation metrics.
/// Output: written file path.
/// Side effects: writes `fitness.svg` in `out_dir`.
pub fn write_fitness_svg(out_dir: &Path, metrics: &[GenerationMetrics]) -> anyhow::Result<PathBuf> {
    let path = out_dir.join("fitness.svg");
    let mut svg = String::with_capacity(16 * 1024);

    write!(
        &mut svg,
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{SVG_WIDTH}" height="{SVG_HEIGHT}" viewBox="0 0 {SVG_WIDTH} {SVG_HEIGHT}" role="img" aria-label="Species fitness over generations">"##
    )?;
    svg.push_str(
        r##"<rect x="0" y="0" width="100%" height="100%" fill="#12161d"/><text x="70" y="20" fill="#dce6f2" font-size="16" font-family="sans-serif">Fitness Over Generations By Species</text>"##,
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
        .flat_map(|m| {
            [
                Some(m.best_fitness),
                Some(m.mean_fitness),
                m.mlp_best_fitness,
                m.mlp_mean_fitness,
                m.ltc_best_fitness,
                m.ltc_mean_fitness,
            ]
        })
        .flatten()
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

    let mlp_best_points = build_species_points(
        metrics,
        |metric| metric.mlp_best_fitness,
        &x_for_index,
        &y_for_value,
    );
    let mlp_mean_points = build_species_points(
        metrics,
        |metric| metric.mlp_mean_fitness,
        &x_for_index,
        &y_for_value,
    );
    let ltc_best_points = build_species_points(
        metrics,
        |metric| metric.ltc_best_fitness,
        &x_for_index,
        &y_for_value,
    );
    let ltc_mean_points = build_species_points(
        metrics,
        |metric| metric.ltc_mean_fitness,
        &x_for_index,
        &y_for_value,
    );

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

    append_polyline(&mut svg, &mlp_best_points, "#2d6cdf", false)?;
    append_polyline(&mut svg, &mlp_mean_points, "#76a9ff", true)?;
    append_polyline(&mut svg, &ltc_best_points, "#f08a24", false)?;
    append_polyline(&mut svg, &ltc_mean_points, "#ffbe73", true)?;

    write!(
        &mut svg,
        r##"<text x="{y_axis_label_x}" y="{top_label_y}" fill="#c4cfdd" font-size="12" font-family="sans-serif">{max_fitness:.2}</text><text x="{y_axis_label_x}" y="{plot_bottom}" fill="#c4cfdd" font-size="12" font-family="sans-serif">0.00</text>"##
    )?;
    write!(
        &mut svg,
        r##"<text x="{plot_left}" y="{x_axis_label_y}" fill="#c4cfdd" font-size="12" font-family="sans-serif">gen {min_generation}</text><text x="{plot_right}" y="{x_axis_label_y}" text-anchor="end" fill="#c4cfdd" font-size="12" font-family="sans-serif">gen {max_generation}</text>"##
    )?;

    let legend_x = plot_right - 200.0;
    let legend_y = plot_top + 16.0;
    append_legend_entry(&mut svg, legend_x, legend_y, "#2d6cdf", false, "MLP best")?;
    append_legend_entry(
        &mut svg,
        legend_x,
        legend_y + 18.0,
        "#76a9ff",
        true,
        "MLP mean",
    )?;
    append_legend_entry(
        &mut svg,
        legend_x,
        legend_y + 36.0,
        "#f08a24",
        false,
        "LTC best",
    )?;
    append_legend_entry(
        &mut svg,
        legend_x,
        legend_y + 54.0,
        "#ffbe73",
        true,
        "LTC mean",
    )?;

    svg.push_str("</svg>");
    fs::write(&path, svg)?;
    Ok(path)
}

fn build_species_points<Fx, Fy, Fv>(
    metrics: &[GenerationMetrics],
    value_for_metric: Fv,
    x_for_index: &Fx,
    y_for_value: &Fy,
) -> Vec<(f32, f32)>
where
    Fx: Fn(usize, usize) -> f32,
    Fy: Fn(f32) -> f32,
    Fv: Fn(&GenerationMetrics) -> Option<f32>,
{
    metrics
        .iter()
        .enumerate()
        .filter_map(|(idx, metric)| {
            value_for_metric(metric)
                .map(|value| ((x_for_index)(idx, metrics.len()), (y_for_value)(value)))
        })
        .collect()
}

fn append_polyline(
    svg: &mut String,
    points: &[(f32, f32)],
    color: &str,
    dashed: bool,
) -> anyhow::Result<()> {
    if points.len() < 2 {
        return Ok(());
    }

    let mut serialized_points = String::new();
    for (x, y) in points {
        write!(&mut serialized_points, "{x:.2},{y:.2} ")?;
    }

    if dashed {
        write!(
            svg,
            r##"<polyline fill="none" stroke="{color}" stroke-width="2.5" stroke-dasharray="8 6" points="{serialized_points}"/>"##
        )?;
    } else {
        write!(
            svg,
            r##"<polyline fill="none" stroke="{color}" stroke-width="2.5" points="{serialized_points}"/>"##
        )?;
    }
    Ok(())
}

fn append_legend_entry(
    svg: &mut String,
    x: f32,
    y: f32,
    color: &str,
    dashed: bool,
    label: &str,
) -> anyhow::Result<()> {
    if dashed {
        write!(
            svg,
            r##"<line x1="{x}" y1="{y}" x2="{}" y2="{y}" stroke="{color}" stroke-width="3" stroke-dasharray="8 6"/><text x="{}" y="{}" fill="#dce6f2" font-size="12" font-family="sans-serif">{label}</text>"##,
            x + 18.0,
            x + 24.0,
            y + 4.0
        )?;
    } else {
        write!(
            svg,
            r##"<line x1="{x}" y1="{y}" x2="{}" y2="{y}" stroke="{color}" stroke-width="3"/><text x="{}" y="{}" fill="#dce6f2" font-size="12" font-family="sans-serif">{label}</text>"##,
            x + 18.0,
            x + 24.0,
            y + 4.0
        )?;
    }
    Ok(())
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
                mlp_best_fitness: Some(1.2),
                mlp_mean_fitness: Some(0.6),
                ltc_best_fitness: Some(0.8),
                ltc_mean_fitness: Some(0.4),
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
                mlp_best_fitness: Some(2.4),
                mlp_mean_fitness: Some(1.1),
                ltc_best_fitness: Some(1.7),
                ltc_mean_fitness: Some(0.9),
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

        assert!(content.contains("MLP best"));
        assert!(content.contains("MLP mean"));
        assert!(content.contains("LTC best"));
        assert!(content.contains("LTC mean"));
        assert!(content.contains("<polyline"));
    }
}
