#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DomainError {
    InsufficientEvidence,
}

pub mod claim;
pub mod commitment;
pub mod episode;
pub mod event;
pub mod evidence_link;
pub mod identity_core;
pub mod reflection;
pub mod rules;
pub mod snapshot;
pub mod types;
