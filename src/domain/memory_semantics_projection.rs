use crate::domain::self_revision::SELF_REVISION_DURABLE_WRITE_PATH;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemorySemanticsProjectionInput {
    pub evidence_relation_count: usize,
    pub episode_summary_count: usize,
    pub semantic_claim_count: usize,
    pub procedural_memory_count: usize,
    pub self_model_write_migration_present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MemorySemanticsProjection {
    pub read_only: bool,
    pub writes_performed: bool,
    pub durable_self_model_write_path: String,
    pub capabilities: Vec<MemorySemanticCapability>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MemorySemanticCapability {
    pub capability: String,
    pub status: String,
    pub observed_count: usize,
    pub writes_allowed: bool,
}

pub fn build_memory_semantics_projection(
    input: MemorySemanticsProjectionInput,
) -> MemorySemanticsProjection {
    MemorySemanticsProjection {
        read_only: true,
        writes_performed: false,
        durable_self_model_write_path: SELF_REVISION_DURABLE_WRITE_PATH.to_string(),
        capabilities: vec![
            count_based_capability("evidence_relations", input.evidence_relation_count),
            count_based_capability("episode_summaries", input.episode_summary_count),
            count_based_capability("semantic_claims", input.semantic_claim_count),
            count_based_capability("procedural_memory", input.procedural_memory_count),
            capability(
                "durable_self_model_writes",
                durable_self_model_write_status(input.self_model_write_migration_present),
                0,
            ),
        ],
        non_claims: vec![
            "not full ranking engine".to_string(),
            "not durable self-model write migration".to_string(),
        ],
    }
}

fn count_based_capability(
    capability_name: &str,
    observed_count: usize,
) -> MemorySemanticCapability {
    capability(
        capability_name,
        status_for_count(observed_count),
        observed_count,
    )
}

fn capability(
    capability_name: &str,
    status: &str,
    observed_count: usize,
) -> MemorySemanticCapability {
    MemorySemanticCapability {
        capability: capability_name.to_string(),
        status: status.to_string(),
        observed_count,
        writes_allowed: false,
    }
}

fn status_for_count(count: usize) -> &'static str {
    if count == 0 {
        "not_implemented"
    } else {
        "partial"
    }
}

fn durable_self_model_write_status(migration_present: bool) -> &'static str {
    if migration_present {
        "partial"
    } else {
        "blocked"
    }
}
