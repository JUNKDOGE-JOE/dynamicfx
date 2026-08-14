//! Effect-definition domain model for the target architecture.
//!
//! Host-agnostic by policy (CLAUDE.md): nothing under this module may depend
//! on AE SDK types. The AE-facing topology in `host/` consumes these types;
//! it never defines them.

pub mod effect;
pub mod param;
