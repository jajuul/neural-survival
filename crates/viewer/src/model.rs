//! The viewer's headless model: playback, stats, the drawn scene, and the
//! Fitness chart data.

use neural_survival_core::{
    Agent, Arena, ControllerFamily, MetricsRow, Role, Showcase, ShowcaseError, Snapshot, Species,
    Vec2,
};

/// Simulation speed in steps per second when a viewer opens.
pub const DEFAULT_SPEED: u32 = 30;
pub const MIN_SPEED: u32 = 1;
pub const MAX_SPEED: u32 = 1000;
/// The most elapsed time one frame counts for, so a slow frame can't run a
/// burst of steps.
pub const MAX_FRAME_SECONDS: f64 = 0.25;

/// Plays a Showcase of a Snapshot's Generation. Playback only decides when
/// `Showcase::step` runs, never what it computes (ADR-0001).
pub struct Viewer {
    snapshot: Snapshot,
    showcase: Showcase,
    paused: bool,
    speed: u32,
    /// Steps' worth of playing time since playback last started. Kept as a
    /// running total rather than a remainder so frame timing can't drift.
    clock: f64,
    /// Steps due by `clock` that have already been run.
    steps_run: u32,
}

impl Viewer {
    /// Opens paused at step 0 on a Showcase of the Snapshot's Generation.
    pub fn open(snapshot: Snapshot) -> Result<Self, ShowcaseError> {
        let showcase = Showcase::from_snapshot(&snapshot)?;
        Ok(Self {
            snapshot,
            showcase,
            paused: true,
            speed: DEFAULT_SPEED,
            clock: 0.0,
            steps_run: 0,
        })
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn step_count(&self) -> u32 {
        self.showcase.step_count()
    }

    pub fn has_ended(&self) -> bool {
        self.showcase.has_ended()
    }

    /// Steps per second.
    pub fn speed(&self) -> u32 {
        self.speed
    }

    pub fn showcase(&self) -> &Showcase {
        &self.showcase
    }

    pub fn snapshot(&self) -> &Snapshot {
        &self.snapshot
    }

    pub fn play(&mut self) {
        self.paused = false;
    }

    /// Pauses; playing time not yet spent on a step is dropped.
    pub fn pause(&mut self) {
        self.paused = true;
        self.clock = 0.0;
        self.steps_run = 0;
    }

    /// Advances one step while paused; does nothing while playing.
    pub fn single_step(&mut self) {
        if self.paused {
            self.showcase.step();
        }
    }

    /// Sets the speed in steps per second, clamped to [`MIN_SPEED`, `MAX_SPEED`].
    pub fn set_speed(&mut self, speed: u32) {
        self.speed = speed.clamp(MIN_SPEED, MAX_SPEED);
    }

    /// Runs the whole steps that `dt` seconds of playing allow, counting at
    /// most [`MAX_FRAME_SECONDS`] and carrying any fraction to the next frame.
    pub fn frame(&mut self, dt: f64) {
        if self.paused {
            return;
        }
        self.clock += dt.clamp(0.0, MAX_FRAME_SECONDS) * f64::from(self.speed);
        let due = self.clock.floor() as u32;
        self.showcase.step_many(due - self.steps_run);
        self.steps_run = due;
    }

    /// Back to step 0, paused, on the Showcase the viewer opened with.
    pub fn restart(&mut self) {
        self.showcase = Showcase::from_snapshot(&self.snapshot)
            .expect("the Snapshot built a Showcase when the viewer opened");
        self.pause();
    }

    pub fn stats(&self) -> Stats {
        let agents: Vec<&Agent> = self
            .showcase
            .arenas()
            .iter()
            .flat_map(|arena| arena.agents())
            .collect();
        let species = Species::ALL
            .iter()
            .map(|&species| {
                let (alive, total) = tally(&agents, |a| {
                    a.family == species.family && a.role == species.role
                });
                SpeciesCount {
                    species,
                    alive,
                    total,
                }
            })
            .collect();
        let mut families: Vec<ControllerFamily> = Vec::new();
        for species in Species::ALL {
            if !families.contains(&species.family) {
                families.push(species.family);
            }
        }
        let families = families
            .into_iter()
            .map(|family| {
                let (alive, total) = tally(&agents, |a| a.family == family);
                FamilyCount {
                    family,
                    alive,
                    total,
                }
            })
            .collect();
        let plan = self
            .snapshot
            .config
            .evaluation
            .plan(self.snapshot.generation);
        Stats {
            generation: self.snapshot.generation,
            step: self.showcase.step_count(),
            alive: agents.iter().filter(|agent| agent.alive).count(),
            food: self
                .showcase
                .arenas()
                .iter()
                .map(|arena| arena.food().len())
                .sum(),
            species,
            families,
            episodes: plan.episodes,
            episode_steps: plan.episode_steps,
        }
    }

    pub fn scene(&self) -> Scene {
        Scene::of(&self.showcase)
    }

    /// Best and mean Fitness per Generation for each Species in the
    /// Snapshot's metrics history, in `Species::ALL` order.
    pub fn fitness_series(&self) -> Vec<FitnessSeries> {
        Species::ALL
            .iter()
            .map(|&species| {
                let rows = self
                    .snapshot
                    .metrics
                    .iter()
                    .filter(|row| row.species == species);
                let points = |value: fn(&MetricsRow) -> f64| {
                    rows.clone()
                        .map(|row| [f64::from(row.generation), value(row)])
                        .collect()
                };
                FitnessSeries {
                    species,
                    best: points(|row| row.best_fitness),
                    mean: points(|row| row.mean_fitness),
                }
            })
            .filter(|series| !series.best.is_empty())
            .collect()
    }
}

/// Living and total Agents among those that `belongs` picks.
fn tally(agents: &[&Agent], belongs: impl Fn(&Agent) -> bool) -> (usize, usize) {
    let members = agents.iter().filter(|agent| belongs(agent));
    let alive = members.clone().filter(|agent| agent.alive).count();
    (alive, members.count())
}

/// Overview numbers for the side panel.
#[derive(Debug, Clone, PartialEq)]
pub struct Stats {
    pub generation: u32,
    pub step: u32,
    pub alive: usize,
    pub food: usize,
    /// One entry per Species, in `Species::ALL` order.
    pub species: Vec<SpeciesCount>,
    /// One entry per Controller Family, in `Species::ALL` order.
    pub families: Vec<FamilyCount>,
    /// The Evaluation Schedule's plan for the Generation.
    pub episodes: u32,
    pub episode_steps: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpeciesCount {
    pub species: Species,
    pub alive: usize,
    pub total: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FamilyCount {
    pub family: ControllerFamily,
    pub alive: usize,
    pub total: usize,
}

/// An sRGB colour.
pub type Rgb = [u8; 3];

/// Indexed by Role: Prey.
pub const ROLE_COLOURS: [Rgb; 1] = [[80, 160, 255]];
pub const FOOD_COLOUR: Rgb = [110, 200, 90];

/// How far an Agent's heading marker reaches from its centre, in body radii.
/// It reaches past the body, so it decides which edges an Agent overlaps.
pub const HEADING_MARKER_LENGTH: f64 = 1.5;

/// The shape an Agent is drawn as, one per Controller Family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    /// MLP.
    Circle,
    /// Reserved for LTC (ticket 07).
    Triangle,
}

/// What the world view draws, in world units.
#[derive(Debug, Clone, PartialEq)]
pub struct Scene {
    pub agents: Vec<AgentShape>,
    pub food: Vec<FoodDot>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AgentShape {
    pub position: Vec2,
    pub radius: f64,
    /// Radians, counter-clockwise from the +x axis.
    pub heading: f64,
    pub colour: Rgb,
    pub shape: Shape,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FoodDot {
    pub position: Vec2,
    pub radius: f64,
    pub colour: Rgb,
}

impl Scene {
    /// The Showcase's living Agents and food, each also drawn shifted across
    /// any torus edge it overlaps.
    pub fn of(showcase: &Showcase) -> Self {
        let config = showcase.config();
        let mut scene = Scene {
            agents: Vec::new(),
            food: Vec::new(),
        };
        for arena in showcase.arenas() {
            let copies = |position, radius| edge_copies(position, radius, arena);
            for agent in arena.agents().iter().filter(|agent| agent.alive) {
                let radius = config.agent.body_radius;
                scene
                    .agents
                    .extend(copies(agent.position, radius * HEADING_MARKER_LENGTH).map(
                        |position| AgentShape {
                            position,
                            radius,
                            heading: agent.heading,
                            colour: role_colour(agent.role),
                            shape: family_shape(agent.family),
                        },
                    ));
            }
            for food in arena.food() {
                let radius = config.food.radius;
                scene
                    .food
                    .extend(copies(food.position, radius).map(|position| FoodDot {
                        position,
                        radius,
                        colour: FOOD_COLOUR,
                    }));
            }
        }
        scene
    }
}

fn role_colour(role: Role) -> Rgb {
    match role {
        Role::Prey => ROLE_COLOURS[0],
    }
}

fn family_shape(family: ControllerFamily) -> Shape {
    match family {
        ControllerFamily::Mlp => Shape::Circle,
    }
}

/// `position` and its copies shifted by the Arena's width, height, or both,
/// for each edge a circle of `radius` there overlaps.
fn edge_copies(position: Vec2, radius: f64, arena: &Arena) -> impl Iterator<Item = Vec2> {
    let shifts = |at: f64, size: f64| {
        let mut shifts = vec![0.0];
        if at - radius < 0.0 {
            shifts.push(size);
        }
        if at + radius > size {
            shifts.push(-size);
        }
        shifts
    };
    let xs = shifts(position.x, arena.width());
    let ys = shifts(position.y, arena.height());
    xs.into_iter().flat_map(move |dx| {
        ys.clone()
            .into_iter()
            .map(move |dy| Vec2::new(position.x + dx, position.y + dy))
    })
}

/// One Species' Fitness history as chart points `[Generation, Fitness]`.
#[derive(Debug, Clone, PartialEq)]
pub struct FitnessSeries {
    pub species: Species,
    pub best: Vec<[f64; 2]>,
    pub mean: Vec<[f64; 2]>,
}
