//! The viewer: plays a Showcase of a saved Generation. `model` holds all the
//! behaviour with no `egui` types; `app` only renders it (ADR-0003).

#![forbid(unsafe_code)]

mod app;
mod model;

pub use app::run;
pub use model::*;
