use agent_llm_mm::{interfaces::mcp::dto::EvidenceQueryDto, ports::EvidenceQuery};
use chrono::{DateTime, Utc};

#[test]
fn evidence_query_dto_parses_recency_window_fields() {
    let query: EvidenceQuery = serde_json::from_value::<EvidenceQueryDto>(serde_json::json!({
        "recorded_after": "2026-03-23T10:00:00Z",
        "recorded_before": "2026-03-23T11:00:00Z",
        "limit": 3
    }))
    .expect("dto should parse")
    .try_into()
    .expect("query should convert");

    assert_eq!(
        query.recorded_after,
        Some(
            DateTime::parse_from_rfc3339("2026-03-23T10:00:00Z")
                .unwrap()
                .with_timezone(&Utc)
        )
    );
    assert_eq!(
        query.recorded_before,
        Some(
            DateTime::parse_from_rfc3339("2026-03-23T11:00:00Z")
                .unwrap()
                .with_timezone(&Utc)
        )
    );
    assert_eq!(query.limit, Some(3));
}

#[test]
fn evidence_query_dto_rejects_invalid_recency_timestamp() {
    let dto = serde_json::from_value::<EvidenceQueryDto>(serde_json::json!({
        "recorded_after": "not-a-date"
    }))
    .expect("dto deserialization should keep validation in conversion");

    let error = EvidenceQuery::try_from(dto).expect_err("invalid timestamp should fail");

    assert!(error.to_string().contains("recorded_after"));
}
