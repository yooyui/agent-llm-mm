use agent_llm_mm::{
    application::{
        build_self_snapshot::BuildSelfSnapshotInput,
        get_memory::{GetMemoryInput, MemoryRecordReference},
    },
    domain::{event::MAX_EVIDENCE_MANIFEST_ITEMS, types::Owner},
    interfaces::mcp::dto::{
        BuildSelfSnapshotParams, EvidenceQueryDto, GetMemoryParams, MemoryRecordTypeDto,
    },
    ports::EvidenceQuery,
};
use chrono::{DateTime, Utc};

#[test]
fn get_memory_dto_defaults_ambiguous_raw_ids_to_event_for_compatibility() {
    let input = GetMemoryInput::try_from(GetMemoryParams {
        namespace: "project/dto".to_string(),
        id: "claim:legacy-event-id".to_string(),
        record_type: None,
    })
    .expect("omitted record type should preserve Event parsing");

    match input.id {
        MemoryRecordReference::Event(reference) => {
            assert_eq!(reference.event_id(), "claim:legacy-event-id");
        }
        MemoryRecordReference::Claim(_) => panic!("omitted record type must not select Claim"),
    }
}

#[test]
fn get_memory_dto_uses_explicit_claim_type_for_raw_or_canonical_claim_ids() {
    for id in ["stored-claim-id", "claim:stored-claim-id"] {
        let input = GetMemoryInput::try_from(GetMemoryParams {
            namespace: "project/dto".to_string(),
            id: id.to_string(),
            record_type: Some(MemoryRecordTypeDto::Claim),
        })
        .expect("explicit Claim record type should parse the claim ID");

        match input.id {
            MemoryRecordReference::Claim(reference) => {
                assert_eq!(reference.claim_id(), "stored-claim-id");
            }
            MemoryRecordReference::Event(_) => panic!("explicit Claim must not select Event"),
        }
    }
}

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

#[test]
fn evidence_query_dto_parses_event_id_prefix() {
    let query: EvidenceQuery = serde_json::from_value::<EvidenceQueryDto>(serde_json::json!({
        "event_id_prefix": "alpha-"
    }))
    .expect("dto should parse")
    .try_into()
    .expect("query should convert");

    assert_eq!(query.event_id_prefix, Some("alpha-".to_string()));

    let empty = serde_json::from_value::<EvidenceQueryDto>(serde_json::json!({
        "event_id_prefix": "   "
    }))
    .expect("dto deserialization keeps validation in conversion");
    let error = EvidenceQuery::try_from(empty).expect_err("empty prefix should fail");
    assert!(error.to_string().contains("event_id_prefix"));
}

#[test]
fn evidence_query_dto_rejects_zero_limit() {
    let dto = serde_json::from_value::<EvidenceQueryDto>(serde_json::json!({
        "limit": 0
    }))
    .expect("dto deserialization keeps validation in conversion");

    let error = EvidenceQuery::try_from(dto).expect_err("zero limit should fail");

    assert!(error.to_string().contains("at least 1"));
}

#[test]
fn snapshot_dto_derives_owner_from_namespace_without_accepting_owner_input() {
    let params = serde_json::from_value::<BuildSelfSnapshotParams>(serde_json::json!({
        "budget": 4,
        "namespace": "project/agent-llm-mm"
    }))
    .expect("snapshot params should parse");

    let input = BuildSelfSnapshotInput::try_from(params).expect("scope should convert");

    assert_eq!(input.scope.owner(), Some(Owner::World));
    assert_eq!(
        input.scope.namespace().map(|namespace| namespace.as_str()),
        Some("project/agent-llm-mm")
    );
}

#[test]
fn snapshot_dto_keeps_omitted_namespace_as_legacy_compatibility_scope() {
    let params = serde_json::from_value::<BuildSelfSnapshotParams>(serde_json::json!({
        "budget": 4
    }))
    .expect("legacy snapshot params should parse");

    let input = BuildSelfSnapshotInput::try_from(params).expect("legacy scope should convert");

    assert!(input.scope.is_legacy_unscoped());
    assert!(input.time_window.is_unbounded());
}

#[test]
fn snapshot_dto_parses_and_normalizes_time_window() {
    let params = serde_json::from_value::<BuildSelfSnapshotParams>(serde_json::json!({
        "budget": 4,
        "namespace": "project/agent-llm-mm",
        "recorded_after": "2026-07-11T10:00:00+08:00",
        "recorded_before": "2026-07-11T03:00:00Z"
    }))
    .expect("snapshot params should parse");

    let input = BuildSelfSnapshotInput::try_from(params).expect("time window should convert");

    assert_eq!(
        input.time_window.recorded_after,
        Some(
            DateTime::parse_from_rfc3339("2026-07-11T02:00:00Z")
                .unwrap()
                .with_timezone(&Utc)
        )
    );
    assert_eq!(
        input.time_window.recorded_before,
        Some(
            DateTime::parse_from_rfc3339("2026-07-11T03:00:00Z")
                .unwrap()
                .with_timezone(&Utc)
        )
    );
}

#[test]
fn snapshot_dto_rejects_invalid_or_reversed_time_window() {
    let invalid_timestamp = serde_json::from_value::<BuildSelfSnapshotParams>(serde_json::json!({
        "budget": 4,
        "namespace": "project/agent-llm-mm",
        "recorded_after": "not-a-date"
    }))
    .unwrap();
    let error = BuildSelfSnapshotInput::try_from(invalid_timestamp)
        .expect_err("invalid snapshot timestamp should fail");
    assert!(error.to_string().contains("recorded_after"));

    let reversed = serde_json::from_value::<BuildSelfSnapshotParams>(serde_json::json!({
        "budget": 4,
        "namespace": "project/agent-llm-mm",
        "recorded_after": "2026-07-11T04:00:00Z",
        "recorded_before": "2026-07-11T03:00:00Z"
    }))
    .unwrap();
    let error = BuildSelfSnapshotInput::try_from(reversed)
        .expect_err("reversed snapshot time window should fail");
    assert!(error.to_string().contains("less than or equal"));
}

#[test]
fn snapshot_dto_requires_namespace_for_any_explicit_time_bound() {
    for params in [
        serde_json::json!({
            "budget": 4,
            "recorded_after": "2026-07-11T02:00:00Z"
        }),
        serde_json::json!({
            "budget": 4,
            "recorded_before": "2026-07-11T03:00:00Z"
        }),
    ] {
        let params = serde_json::from_value::<BuildSelfSnapshotParams>(params).unwrap();
        let error = BuildSelfSnapshotInput::try_from(params)
            .expect_err("time-bounded snapshots must not use legacy unscoped reads");
        assert!(error.to_string().contains("explicit namespace"));
    }
}

#[test]
fn snapshot_dto_rejects_non_empty_manifest_without_explicit_namespace() {
    let params = serde_json::from_value::<BuildSelfSnapshotParams>(serde_json::json!({
        "budget": 4,
        "evidence_manifest": ["evt-1"]
    }))
    .expect("validation should stay in conversion");

    let error = BuildSelfSnapshotInput::try_from(params)
        .expect_err("manifest lookup must not use the legacy unscoped compatibility path");

    assert!(
        error
            .to_string()
            .contains("evidence_manifest requires an explicit namespace")
    );
}

#[test]
fn snapshot_dto_rejects_empty_manifest_without_explicit_namespace() {
    let params = serde_json::from_value::<BuildSelfSnapshotParams>(serde_json::json!({
        "budget": 4,
        "evidence_manifest": []
    }))
    .expect("validation should stay in conversion");

    let error = BuildSelfSnapshotInput::try_from(params)
        .expect_err("an explicit empty manifest must still require a bounded scope");

    assert!(
        error
            .to_string()
            .contains("evidence_manifest requires an explicit namespace")
    );
}

#[test]
fn snapshot_dto_rejects_invalid_namespace() {
    let params = serde_json::from_value::<BuildSelfSnapshotParams>(serde_json::json!({
        "budget": 4,
        "namespace": "tenant/not-supported"
    }))
    .expect("validation should stay in conversion");

    assert!(BuildSelfSnapshotInput::try_from(params).is_err());
}

#[test]
fn snapshot_dto_canonicalizes_and_deduplicates_evidence_manifest() {
    let params = serde_json::from_value::<BuildSelfSnapshotParams>(serde_json::json!({
        "budget": 4,
        "namespace": "project/agent-llm-mm",
        "evidence_manifest": ["evt-1", "event:evt-1", "event:evt-2"]
    }))
    .expect("snapshot params should parse");

    let input = BuildSelfSnapshotInput::try_from(params).expect("manifest should convert");
    let manifest = input
        .evidence_manifest
        .expect("manifest should remain explicit");

    assert_eq!(
        manifest
            .iter()
            .map(|reference| reference.canonical())
            .collect::<Vec<_>>(),
        vec!["event:evt-1", "event:evt-2"]
    );
}

#[test]
fn snapshot_dto_rejects_invalid_evidence_manifest_entry() {
    for invalid in ["", "event:", "event:event:evt-1", "evt 1"] {
        let params = serde_json::from_value::<BuildSelfSnapshotParams>(serde_json::json!({
            "budget": 4,
            "namespace": "project/agent-llm-mm",
            "evidence_manifest": [invalid]
        }))
        .expect("validation should stay in conversion");

        assert!(BuildSelfSnapshotInput::try_from(params).is_err());
    }
}

#[test]
fn snapshot_dto_bounds_evidence_manifest_before_query_construction() {
    let at_limit = (0..MAX_EVIDENCE_MANIFEST_ITEMS)
        .map(|index| format!("evt-{index}"))
        .collect::<Vec<_>>();
    let input = BuildSelfSnapshotInput::try_from(BuildSelfSnapshotParams {
        budget: 4,
        namespace: Some("project/agent-llm-mm".to_string()),
        evidence_manifest: Some(at_limit),
        recorded_after: None,
        recorded_before: None,
        auto_reflect_namespace: None,
    })
    .expect("manifest at the documented limit should convert");
    assert_eq!(
        input
            .evidence_manifest
            .as_deref()
            .map(|manifest| manifest.len()),
        Some(MAX_EVIDENCE_MANIFEST_ITEMS)
    );

    let oversized = (0..=MAX_EVIDENCE_MANIFEST_ITEMS)
        .map(|index| format!("evt-{index}"))
        .collect::<Vec<_>>();
    let error = BuildSelfSnapshotInput::try_from(BuildSelfSnapshotParams {
        budget: 4,
        namespace: Some("project/agent-llm-mm".to_string()),
        evidence_manifest: Some(oversized),
        recorded_after: None,
        recorded_before: None,
        auto_reflect_namespace: None,
    })
    .expect_err("oversized manifests must fail before SQLite bind construction");
    assert!(
        error
            .to_string()
            .contains("evidence_manifest must contain at most 256 entries")
    );
}
