//! Configurable, offline project reporting.
//!
//! Load a [`Workspace`], call [`Workspace::analyze`] to review the inputs, then
//! [`Workspace::export`] to create a new report bundle. The library does not
//! depend on a GUI or an organization-specific naming convention.
//!
//! ```no_run
//! use remain_core::Workspace;
//! let workspace = Workspace::load("examples/demo-en/workspace.json")?;
//! let analysis = workspace.analyze()?;
//! assert!(!analysis.projects.is_empty());
//! // Destination must not exist. Inputs are re-read during export.
//! workspace.export("/tmp/my-report")?;
//! # Ok::<(), anyhow::Error>(())

pub mod analysis;
pub mod config;
pub mod demo;
mod excel;
mod export;
pub mod locale;
mod pdf;
pub mod progress;
pub mod projects;
pub mod submissions;
mod workspace;

pub use workspace::Workspace;
