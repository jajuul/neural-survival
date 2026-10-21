//! The `eframe` window that renders the model and forwards input to it. It
//! makes no decisions of its own and isn't tested automatically.

use eframe::egui::{self, Color32, Pos2, Rect, Sense, Shape as EguiShape, Slider, Stroke, Ui};
use egui_plot::{Legend, Line, LineStyle, Plot};

use crate::model::{HEADING_MARKER_LENGTH, MAX_SPEED, MIN_SPEED, Rgb, Scene, Shape, Viewer};

/// Opens the viewer window and blocks until it closes.
pub fn run(viewer: Viewer) -> Result<(), Box<dyn std::error::Error>> {
    let title = format!(
        "Neural Survival: Generation {}",
        viewer.snapshot().generation
    );
    eframe::run_native(
        &title,
        eframe::NativeOptions::default(),
        Box::new(|_| Ok(Box::new(App { viewer }))),
    )?;
    Ok(())
}

struct App {
    viewer: Viewer,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let dt = ui.input(|input| input.stable_dt);
        self.viewer.frame(f64::from(dt));
        if !self.viewer.is_paused() && !self.viewer.has_ended() {
            ui.ctx().request_repaint();
        }
        egui::Panel::right("controls")
            .default_size(240.0)
            .show(ui, |ui| self.controls(ui));
        egui::Panel::bottom("fitness")
            .default_size(200.0)
            .show(ui, |ui| self.chart(ui));
        egui::CentralPanel::default().show(ui, |ui| self.world(ui));
    }
}

impl App {
    fn controls(&mut self, ui: &mut Ui) {
        let viewer = &mut self.viewer;
        ui.horizontal(|ui| {
            if viewer.is_paused() {
                if ui.button("Continue").clicked() {
                    viewer.play();
                }
            } else if ui.button("Pause").clicked() {
                viewer.pause();
            }
            let can_step = viewer.is_paused() && !viewer.has_ended();
            if ui
                .add_enabled(can_step, egui::Button::new("Step"))
                .clicked()
            {
                viewer.single_step();
            }
            if ui.button("Restart").clicked() {
                viewer.restart();
            }
        });
        let mut speed = viewer.speed();
        ui.add(
            Slider::new(&mut speed, MIN_SPEED..=MAX_SPEED)
                .logarithmic(true)
                .text("steps/s"),
        );
        viewer.set_speed(speed);
        ui.separator();

        let stats = viewer.stats();
        ui.label(format!("Generation {}", stats.generation));
        let ended = if viewer.has_ended() { " (ended)" } else { "" };
        ui.label(format!(
            "Step {} of {}{ended}",
            stats.step, stats.episode_steps
        ));
        ui.label(format!(
            "{} Episodes of {} steps this Generation",
            stats.episodes, stats.episode_steps
        ));
        ui.label(format!("{} alive, {} food", stats.alive, stats.food));
        ui.separator();
        for count in &stats.species {
            ui.label(format!(
                "{}: {} of {} alive",
                count.species, count.alive, count.total
            ));
        }
        for count in &stats.families {
            ui.label(format!(
                "{:?}: {} of {} alive",
                count.family, count.alive, count.total
            ));
        }
    }

    fn chart(&self, ui: &mut Ui) {
        Plot::new("fitness")
            .legend(Legend::default())
            .x_axis_label("Generation")
            .show(ui, |plot| {
                for series in self.viewer.fitness_series() {
                    plot.line(Line::new(format!("{} best", series.species), series.best));
                    plot.line(
                        Line::new(format!("{} mean", series.species), series.mean)
                            .style(LineStyle::dashed_loose()),
                    );
                }
            });
    }

    /// Paints the scene scaled to fit the panel, with world y pointing up.
    fn world(&self, ui: &mut Ui) {
        let world = &self.viewer.showcase().config().world;
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::hover());
        let panel = response.rect;
        let scale = (panel.width() / world.width as f32).min(panel.height() / world.height as f32);
        let area = Rect::from_min_size(
            panel.min,
            egui::vec2(world.width as f32 * scale, world.height as f32 * scale),
        );
        let painter = painter.with_clip_rect(area);
        painter.rect_filled(area, 0.0, Color32::from_gray(20));
        let to_screen = |x: f64, y: f64| {
            Pos2::new(
                area.left() + x as f32 * scale,
                area.bottom() - y as f32 * scale,
            )
        };

        let Scene { agents, food } = self.viewer.scene();
        for dot in food {
            painter.circle_filled(
                to_screen(dot.position.x, dot.position.y),
                dot.radius as f32 * scale,
                colour(dot.colour),
            );
        }
        for agent in agents {
            let centre = to_screen(agent.position.x, agent.position.y);
            let radius = agent.radius as f32 * scale;
            let fill = colour(agent.colour);
            let facing = egui::vec2(agent.heading.cos() as f32, -agent.heading.sin() as f32);
            match agent.shape {
                Shape::Circle => {
                    painter.circle_filled(centre, radius, fill);
                }
                Shape::Triangle => {
                    let side = egui::vec2(-facing.y, facing.x);
                    painter.add(EguiShape::convex_polygon(
                        vec![
                            centre + facing * radius,
                            centre - facing * radius + side * radius,
                            centre - facing * radius - side * radius,
                        ],
                        fill,
                        Stroke::NONE,
                    ));
                }
            }
            painter.line_segment(
                [
                    centre,
                    centre + facing * radius * HEADING_MARKER_LENGTH as f32,
                ],
                Stroke::new(1.5, Color32::WHITE),
            );
        }
    }
}

fn colour([r, g, b]: Rgb) -> Color32 {
    Color32::from_rgb(r, g, b)
}
