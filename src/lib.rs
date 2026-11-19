//! Shared library for the OpenCode token dashboard.
//!
//! The GUI and the web server are thin frontends over the same data layer,
//! so both always show the same numbers.

pub mod chart;
pub mod cli;
pub mod cost;
pub mod csv;
pub mod format;
pub mod gui;
pub mod insights;
pub mod settings;
pub mod usage;
pub mod web;
