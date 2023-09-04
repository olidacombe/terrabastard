//! Parse a terraform monorepo and report on bad things

#![deny(clippy::pedantic)]
#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

pub mod cli;
pub mod terraform;
pub mod walk;
