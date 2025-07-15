// src/compliance/mod.rs
//! Compliance Check Module: Policy-Driven Data Sanitization
//! 
//! Strategic Principle: "Shift Left on Compliance"
//! Don't just validate data at the end of a process; build the data correctly 
//! and securely from the very beginning. This means you should sanitize data 
//! at the point of creation, ensuring sensitive PII never touches your core 
//! application logic or logs.

pub mod context_builder;
pub mod data_policies;
pub mod pii_detector;
pub mod audit_logger;

pub use context_builder::*;
pub use data_policies::*;
pub use pii_detector::*;
pub use audit_logger::*;