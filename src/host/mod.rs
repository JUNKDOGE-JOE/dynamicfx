//! AE-facing host layer. Everything that touches the AE SDK lives here or in
//! `lib.rs`; the `definition`/`frontend`/`binding` domain layers stay
//! host-agnostic by policy (CLAUDE.md).

pub mod idle;
pub mod params;
