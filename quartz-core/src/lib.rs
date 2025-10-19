//! QuartzDB Core Module
//!
//! This module contains the core components of QuartzDB, including:
//! - Query processing
//! - Transaction management
//! - Distributed consensus
//! - Edge computing coordination

pub mod consensus;
pub mod error;
pub mod query;
pub mod transaction;
pub mod types;

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;

/// Version of the QuartzDB protocol
pub const PROTOCOL_VERSION: &str = "0.1.0";
