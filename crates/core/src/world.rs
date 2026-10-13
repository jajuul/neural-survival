//! One Arena of the world: Agents and food on a torus.

use rand::Rng as _;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::genome::{ControllerFamily, Genome, Network, Role};
use crate::geometry::{Torus, Vec2, normalize_angle};
use crate::rng::Rng;
use crate::sensing::Eye;
use crate::telemetry::{
    AgentTelemetry, AppliedOutputs, CoreSensors, EnergyCost, EnergyRule, EyeKind, EyeReading,
    RawOutputs,
};

/// Identifies an Agent within a Showcase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AgentId(pub u32);

/// A Genome embodied in the world for one Episode.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Agent {
    pub id: AgentId,
    pub role: Role,
    pub family: ControllerFamily,
    pub position: Vec2,
    /// Radians, counter-clockwise from the +x axis, in [-π, π).
    pub heading: f64,
    pub speed: f64,
    pub energy: f64,
    pub alive: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Food {
    pub position: Vec2,
}

/// Where and facing which way an Agent starts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    pub position: Vec2,
    pub heading: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Arena {
    torus: Torus,
    agents: Vec<Agent>,
    controllers: Vec<Network>,
    /// Last living step's telemetry per Agent, kept after death.
    telemetry: Vec<Option<AgentTelemetry>>,
    food: Vec<Food>,
    rng: Rng,
}

impl Arena {
    pub(crate) fn new(
        config: &Config,
        members: Vec<(AgentId, Genome, Placement)>,
        food: Vec<Food>,
        rng: Rng,
    ) -> Self {
        let (agents, controllers): (Vec<Agent>, Vec<Network>) = members
            .into_iter()
            .map(|(id, genome, placement)| {
                let agent = Agent {
                    id,
                    role: genome.role,
                    family: genome.family(),
                    position: placement.position,
                    heading: normalize_angle(placement.heading),
                    speed: 0.0,
                    energy: config.prey.starting_energy,
                    alive: true,
                };
                (agent, genome.network)
            })
            .unzip();
        let telemetry = vec![None; controllers.len()];
        Self {
            torus: torus(config),
            agents,
            controllers,
            telemetry,
            food,
            rng,
        }
    }

    pub fn width(&self) -> f64 {
        self.torus.width
    }

    pub fn height(&self) -> f64 {
        self.torus.height
    }

    pub fn agents(&self) -> &[Agent] {
        &self.agents
    }

    pub fn food(&self) -> &[Food] {
        &self.food
    }

    pub(crate) fn telemetry(&self, id: AgentId) -> Option<&AgentTelemetry> {
        let index = self.agents.iter().position(|a| a.id == id)?;
        self.telemetry[index].as_ref()
    }

    pub(crate) fn all_dead(&self) -> bool {
        self.agents.iter().all(|a| !a.alive)
    }

    /// Every living Agent senses and decides on the same world state, then
    /// each in id order moves, eats, and pays its energy costs.
    pub(crate) fn step(&mut self, config: &Config) {
        let decisions: Vec<Option<AgentTelemetry>> = (0..self.agents.len())
            .map(|i| {
                self.agents[i]
                    .alive
                    .then(|| self.sense_and_decide(config, i))
            })
            .collect();

        for (i, decision) in decisions.into_iter().enumerate() {
            let Some(mut telemetry) = decision else {
                continue;
            };
            self.apply_motion(config, i, telemetry.applied);
            self.eat_touching_food(config, i);
            telemetry.energy_costs = self.apply_energy_rules(config, i);
            self.telemetry[i] = Some(telemetry);
        }
    }

    fn sense_and_decide(&self, config: &Config, i: usize) -> AgentTelemetry {
        let agent = &self.agents[i];
        let eye = Eye {
            position: agent.position,
            heading: agent.heading,
            range: config.agent.vision_range,
            field_of_view: config.agent.field_of_view,
            bins: config.sensors.eye_bins,
        };
        let food_bins = eye.sense(self.torus, self.food.iter().map(|f| f.position));
        let mut telemetry = AgentTelemetry {
            eyes: vec![EyeReading {
                kind: EyeKind::Food,
                bins: food_bins,
            }],
            core: CoreSensors {
                energy: (agent.energy / config.sensors.energy_scale).clamp(0.0, 1.0),
                speed: if config.agent.max_speed > 0.0 {
                    agent.speed / config.agent.max_speed
                } else {
                    0.0
                },
            },
            raw_outputs: RawOutputs {
                steering: 0.0,
                throttle: 0.0,
            },
            applied: AppliedOutputs {
                steering: 0.0,
                thrust: 0.0,
            },
            energy_costs: Vec::new(),
        };
        let [steering, throttle] = self.controllers[i].act(&telemetry.controller_input());
        telemetry.raw_outputs = RawOutputs { steering, throttle };
        telemetry.applied = AppliedOutputs {
            steering,
            thrust: (throttle + 1.0) / 2.0,
        };
        telemetry
    }

    fn apply_motion(&mut self, config: &Config, i: usize, applied: AppliedOutputs) {
        let agent = &mut self.agents[i];
        agent.heading = normalize_angle(agent.heading + applied.steering * config.agent.turn_rate);
        agent.speed = (agent.speed * (1.0 - config.world.drag)
            + applied.thrust * config.agent.max_acceleration)
            .min(config.agent.max_speed);
        agent.position = self
            .torus
            .wrap(agent.position + Vec2::from_angle(agent.heading) * agent.speed);
    }

    /// Prey eat every food item they touch; each one respawns at once.
    fn eat_touching_food(&mut self, config: &Config, i: usize) {
        let agent = &mut self.agents[i];
        let reach = config.agent.body_radius + config.food.radius;
        for food in &mut self.food {
            if self.torus.distance(agent.position, food.position) <= reach {
                agent.energy += config.food.energy;
                food.position = random_position(self.torus, &mut self.rng);
            }
        }
    }

    fn apply_energy_rules(&mut self, config: &Config, i: usize) -> Vec<EnergyCost> {
        let costs = vec![EnergyCost {
            rule: EnergyRule::IdleDrain,
            amount: config.energy.idle_drain,
        }];
        let agent = &mut self.agents[i];
        agent.energy -= costs.iter().map(|c| c.amount).sum::<f64>();
        if agent.energy <= 0.0 {
            agent.energy = 0.0;
            agent.alive = false;
        }
        costs
    }
}

pub(crate) fn torus(config: &Config) -> Torus {
    Torus {
        width: config.world.width,
        height: config.world.height,
    }
}

pub(crate) fn random_position(torus: Torus, rng: &mut Rng) -> Vec2 {
    Vec2::new(
        rng.gen_range(0.0..torus.width),
        rng.gen_range(0.0..torus.height),
    )
}
