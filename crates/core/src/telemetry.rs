//! What an Agent sensed, decided, and spent on its last living step. Eyes and
//! energy costs are lists keyed by kind, so new eye kinds and Energy Rules
//! add entries without changing the shape.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentTelemetry {
    pub eyes: Vec<EyeReading>,
    pub core: CoreSensors,
    pub raw_outputs: RawOutputs,
    pub applied: AppliedOutputs,
    pub energy_costs: Vec<EnergyCost>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EyeKind {
    Food,
}

/// One eye's Eye Bin activations, bin 0 at the clockwise edge of the field of view.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EyeReading {
    pub kind: EyeKind,
    pub bins: Vec<f64>,
}

/// Normalised core sensor values fed to the controller.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CoreSensors {
    pub energy: f64,
    pub speed: f64,
}

/// Controller outputs, each in [-1, 1].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RawOutputs {
    pub steering: f64,
    pub throttle: f64,
}

/// Outputs as applied to the body: steering in [-1, 1], thrust in [0, 1].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AppliedOutputs {
    pub steering: f64,
    pub thrust: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnergyRule {
    IdleDrain,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EnergyCost {
    pub rule: EnergyRule,
    pub amount: f64,
}

impl AgentTelemetry {
    /// The controller input vector: core sensors, then each eye's bins in order.
    pub(crate) fn controller_input(&self) -> Vec<f64> {
        let mut input = vec![self.core.energy, self.core.speed];
        for eye in &self.eyes {
            input.extend(&eye.bins);
        }
        input
    }
}
