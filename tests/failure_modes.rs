use std::sync::{Arc, Mutex};

use agent_llm_mm::{
    application::{
        auto_reflect_if_needed::{self, AutoReflectInput},
        build_self_snapshot::{BuildSelfSnapshotInput, execute as execute_build_snapshot},
        ingest_interaction::{IngestInput, execute as execute_ingest},
        run_reflection::{ReflectionInput, execute as execute_reflection},
    },
    domain::{
        claim::ClaimDraft,
        commitment::Commitment,
        event::Event,
        identity_core::IdentityCore,
        reflection::Reflection,
        self_revision::TriggerType,
        snapshot::SnapshotBudget,
        types::{EventKind, Mode, Namespace, Owner},
    },
    error::AppError,
    ports::{
        ClaimStatus, ClaimStore, Clock, CommitmentStore, EpisodeStore, EventStore, EvidenceQuery,
        IdGenerator, IdentityStore, IngestTransaction, IngestTransactionRunner, ModelDecision,
        ModelInput, ModelPort, ReflectionTransaction, ReflectionTransactionRunner, StoredClaim,
        StoredEvent, StoredReflection, StoredTriggerLedgerEntry, TriggerLedgerStatus,
        TriggerLedgerStore,
    },
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[tokio::test]
async fn inferred_claim_without_evidence_is_rejected() {
    let deps = test_support::deps_for_failure_modes();

    let result = execute_ingest(&deps, test_support::inferred_without_evidence()).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn reflection_marks_conflict_as_disputed_instead_of_deleting_history() {
    let deps = test_support::deps_for_failure_modes();

    let result = execute_reflection(&deps, test_support::conflicting_reflection())
        .await
        .unwrap();

    let original_claim = deps
        .claim("claim-conflict")
        .expect("original conflicting claim should remain in history");
    assert_eq!(original_claim.status, ClaimStatus::Disputed);
    assert_ne!(original_claim.status, ClaimStatus::Superseded);
    assert_eq!(result.replacement_claim_id, None);
    assert!(
        deps.claim("id-1:replacement").is_none(),
        "conflict-only reflections must not create a replacement claim"
    );
    assert_eq!(
        deps.claim_count(),
        1,
        "marking a conflict should preserve the original claim instead of replacing it"
    );
}

#[tokio::test]
async fn reflection_rejects_identity_or_commitment_updates_without_resolved_evidence() {
    let deps = test_support::deps_for_failure_modes();

    let result =
        execute_reflection(&deps, test_support::reflection_updates_without_evidence()).await;

    assert!(matches!(result, Err(AppError::InvalidParams(_))));
    assert_eq!(
        deps.identity().canonical_claims(),
        &["identity:self=architect".to_string()]
    );
    assert_eq!(
        deps.commitments(),
        vec![Commitment::new(
            Owner::Self_,
            "forbid:write_identity_core_directly",
        )]
    );
    assert!(deps.reflection("id-1").is_none());
}

#[tokio::test]
async fn reflection_rejects_empty_identity_update_even_with_supporting_evidence() {
    let deps = test_support::deps_for_failure_modes();

    let result = execute_reflection(
        &deps,
        test_support::reflection_updates_with_empty_identity(),
    )
    .await;

    assert!(matches!(result, Err(AppError::InvalidParams(_))));
    assert_eq!(
        deps.identity().canonical_claims(),
        &["identity:self=architect".to_string()]
    );
    assert!(deps.reflection("id-1").is_none());
}

#[tokio::test]
async fn reflection_commit_failure_rolls_back_claim_identity_commitment_and_audit_updates() {
    let deps = test_support::deps_with_fail_point(FailPoint::CommitReflection);

    let result = execute_reflection(&deps, test_support::reflection_updates_with_evidence()).await;

    assert!(
        matches!(result, Err(AppError::Message(message)) if message == "injected reflection commit failure")
    );
    assert_eq!(
        deps.claim("claim-conflict")
            .expect("original claim should remain")
            .status,
        ClaimStatus::Active
    );
    assert!(deps.claim("id-1:replacement").is_none());
    assert_eq!(
        deps.identity().canonical_claims(),
        &["identity:self=architect".to_string()]
    );
    assert_eq!(
        deps.commitments(),
        vec![Commitment::new(
            Owner::Self_,
            "forbid:write_identity_core_directly",
        )]
    );
    assert!(deps.reflection("id-1").is_none());
    assert!(deps.evidence_links().is_empty());
}

#[tokio::test]
async fn auto_reflection_returns_structured_diagnostics_for_recursion_guard_skip() {
    let deps = test_support::deps_for_failure_modes();

    let result = auto_reflect_if_needed::execute(
        &deps,
        AutoReflectInput::for_periodic(
            Namespace::for_project("agent-llm-mm"),
            vec!["periodic".to_string()],
        )
        .with_recursion_guard(
            agent_llm_mm::application::auto_reflect_if_needed::RecursionGuard::SkipAutoReflection,
        ),
    )
    .await
    .unwrap();

    assert!(!result.triggered);
    assert_eq!(result.trigger_type, Some(TriggerType::Periodic));
    assert_eq!(result.reflection_id, None);
    assert_eq!(result.ledger_status, None);
    assert_eq!(result.reason.as_deref(), Some("recursion guard enabled"));
    assert_eq!(
        result.trigger_key.as_deref(),
        Some("project/agent-llm-mm:periodic")
    );
    assert!(result.evidence_event_ids.is_empty());
    assert_eq!(result.cooldown_until, None);
    assert_eq!(result.suppression_reason, None);
    assert_eq!(result.diagnostics.trigger_type, TriggerType::Periodic);
    assert_eq!(
        result.diagnostics.outcome,
        agent_llm_mm::domain::self_revision::AutoReflectOutcome::Skipped
    );
    assert_eq!(result.diagnostics.rejection_reason, None);
    assert_eq!(result.diagnostics.suppression_reason, None);
    assert_eq!(result.diagnostics.cooldown_boundary, None);
    assert_eq!(result.diagnostics.evidence_window_size, 0);
    assert!(result.diagnostics.selected_evidence_event_ids.is_empty());
    assert_eq!(result.diagnostics.durable_write_path, "run_reflection");
    let diagnostics_json = serde_json::to_value(&result.diagnostics).unwrap();
    assert_eq!(diagnostics_json["trigger_type"], "periodic");
    assert_eq!(diagnostics_json["outcome"], "skipped");
}

#[tokio::test]
async fn auto_reflection_returns_structured_diagnostics_for_not_triggered_case() {
    let deps = test_support::deps_for_failure_modes();

    let result = auto_reflect_if_needed::execute(
        &deps,
        AutoReflectInput::for_failure(
            Namespace::for_project("agent-llm-mm"),
            vec!["failure".to_string()],
        ),
    )
    .await
    .unwrap();

    assert!(!result.triggered);
    assert_eq!(result.trigger_type, Some(TriggerType::Failure));
    assert_eq!(result.reflection_id, None);
    assert_eq!(result.ledger_status, None);
    assert_eq!(result.reason, None);
    assert_eq!(
        result.trigger_key.as_deref(),
        Some("project/agent-llm-mm:failure")
    );
    assert!(result.evidence_event_ids.is_empty());
    assert_eq!(result.cooldown_until, None);
    assert_eq!(result.suppression_reason, None);
    assert_eq!(result.diagnostics.trigger_type, TriggerType::Failure);
    assert_eq!(
        result.diagnostics.outcome,
        agent_llm_mm::domain::self_revision::AutoReflectOutcome::NotTriggered
    );
    assert_eq!(result.diagnostics.rejection_reason, None);
    assert_eq!(result.diagnostics.suppression_reason, None);
    assert_eq!(result.diagnostics.cooldown_boundary, None);
    assert_eq!(result.diagnostics.evidence_window_size, 0);
    assert!(result.diagnostics.selected_evidence_event_ids.is_empty());
    assert_eq!(result.diagnostics.durable_write_path, "run_reflection");
    let diagnostics_json = serde_json::to_value(&result.diagnostics).unwrap();
    assert_eq!(diagnostics_json["trigger_type"], "failure");
    assert_eq!(diagnostics_json["outcome"], "not_triggered");
}

#[tokio::test]
async fn auto_reflection_returns_structured_diagnostics_for_rejected_proposal() {
    let deps = test_support::deps_for_failure_modes();
    deps.seed_project_reflection_event();

    let result = auto_reflect_if_needed::execute(
        &deps,
        AutoReflectInput::for_periodic(
            Namespace::for_project("agent-llm-mm"),
            vec!["periodic".to_string()],
        ),
    )
    .await
    .unwrap();

    assert!(!result.triggered);
    assert_eq!(result.trigger_type, Some(TriggerType::Periodic));
    assert_eq!(result.reflection_id, None);
    assert_eq!(result.ledger_status, Some(TriggerLedgerStatus::Rejected));
    assert_eq!(
        result.reason.as_deref(),
        Some("mock model did not detect a valid Periodic revision")
    );
    assert_eq!(result.diagnostics.trigger_type, TriggerType::Periodic);
    assert_eq!(
        result.diagnostics.outcome,
        agent_llm_mm::domain::self_revision::AutoReflectOutcome::Rejected
    );
    assert_eq!(
        result.diagnostics.rejection_reason.as_deref(),
        Some("mock model did not detect a valid Periodic revision")
    );
    assert_eq!(result.diagnostics.suppression_reason, None);
    assert_eq!(result.diagnostics.cooldown_boundary, None);
    assert_eq!(result.diagnostics.evidence_window_size, 1);
    assert!(result.diagnostics.selected_evidence_event_ids.is_empty());
    assert_eq!(result.diagnostics.durable_write_path, "run_reflection");
    let diagnostics_json = serde_json::to_value(&result.diagnostics).unwrap();
    assert_eq!(diagnostics_json["trigger_type"], "periodic");
    assert_eq!(diagnostics_json["outcome"], "rejected");
    assert_eq!(
        result.trigger_key.as_deref(),
        Some("project/agent-llm-mm:periodic")
    );
    assert_eq!(
        result.evidence_event_ids,
        vec!["evt-reflection-1".to_string()]
    );
    assert_eq!(result.cooldown_until, None);
    assert_eq!(result.suppression_reason, None);
}

#[tokio::test]
async fn snapshot_budget_deduplicates_recent_duplicate_evidence_before_truncating() {
    let deps = test_support::deps_for_failure_modes();

    let snapshot = execute_build_snapshot(&deps, test_support::budgeted_snapshot())
        .await
        .unwrap()
        .snapshot;

    assert_eq!(
        snapshot.evidence,
        vec![
            "event:recent-duplicate".to_string(),
            "event:anchor".to_string(),
            "event:baseline".to_string(),
        ],
        "the current data model only tracks ordered event references, so this regression models a recent-event hijack as duplicate recent references exhausting the budget"
    );
}

#[tokio::test]
async fn auto_reflection_rejects_identity_patch_without_minimum_support_and_records_ledger() {
    let deps = test_support::deps_for_failure_modes();
    deps.set_self_revision_proposal(test_support::identity_only_auto_reflection_proposal());
    deps.seed_identity_support_context(
        vec!["episode-001".to_string()],
        vec![
            StoredClaim::new(
                "claim-supporting-1".to_string(),
                ClaimDraft::new(
                    Owner::Self_,
                    "self.role",
                    "is",
                    "principal_architect",
                    Mode::Observed,
                ),
                ClaimStatus::Active,
            ),
            StoredClaim::new(
                "claim-conflicting-1".to_string(),
                ClaimDraft::new(Owner::Self_, "self.role", "is", "architect", Mode::Observed),
                ClaimStatus::Active,
            ),
        ],
    );

    let result = auto_reflect_if_needed::execute(
        &deps,
        AutoReflectInput::for_conflict(
            Namespace::self_(),
            vec!["conflict".to_string(), "identity".to_string()],
        ),
    )
    .await;

    assert!(matches!(result, Err(AppError::InvalidParams(_))));
    assert_eq!(
        deps.latest_trigger_status(),
        Some(TriggerLedgerStatus::Rejected)
    );
    assert!(deps.reflection("id-2").is_none());
}

#[tokio::test]
async fn auto_reflection_rejects_model_proposed_evidence_outside_trigger_window() {
    let deps = test_support::deps_for_failure_modes();
    deps.seed_failure_window(vec![
        (
            "evt-failure-1",
            "rollback after violating a hard commitment",
        ),
        (
            "evt-failure-2",
            "second rollback after violating the same hard commitment",
        ),
    ]);
    deps.set_self_revision_proposal(
        test_support::commitment_only_auto_reflection_proposal_with_policy(
            vec!["evt-outside-1".to_string(), "evt-outside-2".to_string()],
            None,
        ),
    );

    let result = auto_reflect_if_needed::execute(
        &deps,
        AutoReflectInput::for_failure(
            Namespace::for_project("agent-llm-mm"),
            vec!["failure".to_string(), "rollback".to_string()],
        ),
    )
    .await;

    assert!(matches!(result, Err(AppError::InvalidParams(_))));
    assert_eq!(
        deps.latest_trigger_status(),
        Some(TriggerLedgerStatus::Rejected)
    );
}

#[tokio::test]
async fn auto_reflection_rejects_mixed_valid_and_invalid_model_proposed_evidence_ids() {
    let deps = test_support::deps_for_failure_modes();
    deps.seed_failure_window(vec![
        (
            "evt-failure-1",
            "rollback after violating a hard commitment",
        ),
        (
            "evt-failure-2",
            "second rollback after violating the same hard commitment",
        ),
    ]);
    deps.set_self_revision_proposal(
        test_support::commitment_only_auto_reflection_proposal_with_policy(
            vec!["evt-failure-2".to_string(), "evt-outside-1".to_string()],
            None,
        ),
    );

    let result = auto_reflect_if_needed::execute(
        &deps,
        AutoReflectInput::for_failure(
            Namespace::for_project("agent-llm-mm"),
            vec!["failure".to_string(), "rollback".to_string()],
        ),
    )
    .await;

    assert!(matches!(result, Err(AppError::InvalidParams(_))));
    assert_eq!(
        deps.latest_trigger_status(),
        Some(TriggerLedgerStatus::Rejected)
    );
    assert!(deps.latest_reflection().is_none());
}

#[tokio::test]
async fn auto_reflection_rejects_model_proposed_evidence_ids_that_do_not_match_query_policy() {
    let deps = test_support::deps_for_failure_modes();
    deps.seed_events(vec![
        StoredEvent::new(
            "evt-conflict-outside-window".to_string(),
            chrono::DateTime::parse_from_rfc3339("2026-03-23T09:55:00Z")
                .unwrap()
                .with_timezone(&Utc),
            Event::new(
                Owner::World,
                EventKind::Observation,
                "older conflicting observation outside the current trigger window",
            ),
        ),
        StoredEvent::new(
            "evt-conflict-1".to_string(),
            chrono::DateTime::parse_from_rfc3339("2026-03-23T10:01:00Z")
                .unwrap()
                .with_timezone(&Utc),
            Event::new(
                Owner::User,
                EventKind::Conversation,
                "user raised a possible commitment conflict",
            ),
        ),
        StoredEvent::new(
            "evt-conflict-2".to_string(),
            chrono::DateTime::parse_from_rfc3339("2026-03-23T10:02:00Z")
                .unwrap()
                .with_timezone(&Utc),
            Event::new(
                Owner::World,
                EventKind::Observation,
                "current conflicting observation inside the trigger window",
            ),
        ),
        StoredEvent::new(
            "evt-conflict-3".to_string(),
            chrono::DateTime::parse_from_rfc3339("2026-03-23T10:03:00Z")
                .unwrap()
                .with_timezone(&Utc),
            Event::new(
                Owner::Self_,
                EventKind::Action,
                "self attempted a conflicting overwrite",
            ),
        ),
        StoredEvent::new(
            "evt-conflict-4".to_string(),
            chrono::DateTime::parse_from_rfc3339("2026-03-23T10:04:00Z")
                .unwrap()
                .with_timezone(&Utc),
            Event::new(
                Owner::User,
                EventKind::Conversation,
                "user requested commitment clarification",
            ),
        ),
        StoredEvent::new(
            "evt-conflict-5".to_string(),
            chrono::DateTime::parse_from_rfc3339("2026-03-23T10:05:00Z")
                .unwrap()
                .with_timezone(&Utc),
            Event::new(
                Owner::Self_,
                EventKind::Action,
                "self retried the conflicting overwrite",
            ),
        ),
    ]);
    deps.set_self_revision_proposal(
        test_support::commitment_only_auto_reflection_proposal_with_policy(
            vec!["evt-conflict-3".to_string()],
            Some(EvidenceQuery {
                namespace: None,
                owner: Some(Owner::World),
                kind: Some(EventKind::Observation),
                limit: Some(5),
            }),
        ),
    );

    let result = auto_reflect_if_needed::execute(
        &deps,
        AutoReflectInput::for_conflict(
            Namespace::self_(),
            vec!["conflict".to_string(), "commitment".to_string()],
        ),
    )
    .await;

    assert!(matches!(result, Err(AppError::InvalidParams(_))));
    assert_eq!(
        deps.latest_trigger_status(),
        Some(TriggerLedgerStatus::Rejected)
    );
    assert!(deps.latest_reflection().is_none());
}

#[tokio::test]
async fn auto_reflection_keeps_explicit_ids_authoritative_when_query_limit_only_matches_outside_window()
 {
    let deps = test_support::deps_for_failure_modes();
    deps.seed_events(vec![
        StoredEvent::new(
            "evt-outside-newest".to_string(),
            chrono::DateTime::parse_from_rfc3339("2026-03-23T10:06:00Z")
                .unwrap()
                .with_timezone(&Utc),
            Event::new(
                Owner::User,
                EventKind::Conversation,
                "newest non-failure event outside the failure trigger window",
            ),
        ),
        StoredEvent::new(
            "evt-failure-1".to_string(),
            chrono::DateTime::parse_from_rfc3339("2026-03-23T10:01:00Z")
                .unwrap()
                .with_timezone(&Utc),
            Event::new(
                Owner::Self_,
                EventKind::Action,
                "rollback after violating a hard commitment",
            ),
        ),
        StoredEvent::new(
            "evt-failure-2".to_string(),
            chrono::DateTime::parse_from_rfc3339("2026-03-23T10:02:00Z")
                .unwrap()
                .with_timezone(&Utc),
            Event::new(
                Owner::Self_,
                EventKind::Action,
                "second rollback after violating the same hard commitment",
            ),
        ),
    ]);
    deps.set_self_revision_proposal(
        test_support::commitment_only_auto_reflection_proposal_with_policy(
            vec!["evt-failure-2".to_string()],
            Some(EvidenceQuery {
                namespace: None,
                owner: None,
                kind: None,
                limit: Some(1),
            }),
        ),
    );

    let result = auto_reflect_if_needed::execute(
        &deps,
        AutoReflectInput::for_failure(
            Namespace::for_project("agent-llm-mm"),
            vec!["failure".to_string(), "rollback".to_string()],
        ),
    )
    .await
    .unwrap();

    assert!(result.triggered);
    assert_eq!(result.evidence_event_ids, vec!["evt-failure-2".to_string()]);
}

#[tokio::test]
async fn auto_reflection_applies_query_limit_within_current_trigger_window_when_ids_are_empty() {
    let deps = test_support::deps_for_failure_modes();
    deps.seed_events(vec![
        StoredEvent::new(
            "evt-outside-newest-1".to_string(),
            chrono::DateTime::parse_from_rfc3339("2026-03-23T10:06:00Z")
                .unwrap()
                .with_timezone(&Utc),
            Event::new(
                Owner::User,
                EventKind::Conversation,
                "newest non-failure event outside the failure trigger window",
            ),
        ),
        StoredEvent::new(
            "evt-outside-newest-2".to_string(),
            chrono::DateTime::parse_from_rfc3339("2026-03-23T10:05:30Z")
                .unwrap()
                .with_timezone(&Utc),
            Event::new(
                Owner::World,
                EventKind::Observation,
                "second newest non-failure event outside the failure trigger window",
            ),
        ),
        StoredEvent::new(
            "evt-failure-1".to_string(),
            chrono::DateTime::parse_from_rfc3339("2026-03-23T10:01:00Z")
                .unwrap()
                .with_timezone(&Utc),
            Event::new(
                Owner::Self_,
                EventKind::Action,
                "rollback after violating a hard commitment",
            ),
        ),
        StoredEvent::new(
            "evt-failure-2".to_string(),
            chrono::DateTime::parse_from_rfc3339("2026-03-23T10:02:00Z")
                .unwrap()
                .with_timezone(&Utc),
            Event::new(
                Owner::Self_,
                EventKind::Action,
                "second rollback after violating the same hard commitment",
            ),
        ),
    ]);
    deps.set_self_revision_proposal(
        test_support::commitment_only_auto_reflection_proposal_with_policy(
            Vec::new(),
            Some(EvidenceQuery {
                namespace: None,
                owner: None,
                kind: None,
                limit: Some(1),
            }),
        ),
    );

    let result = auto_reflect_if_needed::execute(
        &deps,
        AutoReflectInput::for_failure(
            Namespace::for_project("agent-llm-mm"),
            vec!["failure".to_string(), "rollback".to_string()],
        ),
    )
    .await
    .unwrap();

    assert!(result.triggered);
    assert_eq!(result.evidence_event_ids, vec!["evt-failure-2".to_string()]);
    let reflection = deps
        .latest_reflection()
        .expect("query-limited auto-reflection should persist a reflection");
    assert_eq!(
        reflection.supporting_evidence_event_ids,
        vec!["evt-failure-2".to_string()]
    );
    let handled_entry = deps
        .latest_trigger_entry()
        .expect("handled auto-reflection should persist a trigger ledger entry");
    assert_eq!(
        handled_entry.evidence_window,
        vec!["evt-failure-2".to_string(), "evt-failure-1".to_string()]
    );
}

#[tokio::test]
async fn auto_reflection_applies_model_proposed_evidence_subset_but_preserves_full_trigger_window_in_handled_ledger()
 {
    let deps = test_support::deps_for_failure_modes();
    deps.seed_failure_window(vec![
        (
            "evt-failure-1",
            "rollback after violating a hard commitment",
        ),
        (
            "evt-failure-2",
            "second rollback after violating the same hard commitment",
        ),
    ]);
    deps.set_self_revision_proposal(
        test_support::commitment_only_auto_reflection_proposal_with_policy(
            vec!["evt-failure-2".to_string()],
            None,
        ),
    );

    let result = auto_reflect_if_needed::execute(
        &deps,
        AutoReflectInput::for_failure(
            Namespace::for_project("agent-llm-mm"),
            vec!["failure".to_string(), "rollback".to_string()],
        ),
    )
    .await
    .unwrap();

    assert!(result.triggered);
    assert_eq!(result.evidence_event_ids, vec!["evt-failure-2".to_string()]);
    assert_eq!(result.diagnostics.trigger_type, TriggerType::Failure);
    assert_eq!(
        result.diagnostics.outcome,
        agent_llm_mm::domain::self_revision::AutoReflectOutcome::Handled
    );
    assert_eq!(result.diagnostics.rejection_reason, None);
    assert_eq!(result.diagnostics.suppression_reason, None);
    assert_eq!(result.diagnostics.evidence_window_size, 2);
    assert_eq!(
        result.diagnostics.selected_evidence_event_ids,
        vec!["evt-failure-2".to_string()]
    );
    assert_eq!(result.diagnostics.durable_write_path, "run_reflection");
    let diagnostics_json = serde_json::to_value(&result.diagnostics).unwrap();
    assert_eq!(diagnostics_json["trigger_type"], "failure");
    assert_eq!(diagnostics_json["outcome"], "handled");
    let reflection = deps
        .latest_reflection()
        .expect("handled auto-reflection should persist a reflection");
    assert_eq!(
        reflection.supporting_evidence_event_ids,
        vec!["evt-failure-2".to_string()]
    );
    let handled_entry = deps
        .latest_trigger_entry()
        .expect("handled auto-reflection should persist a trigger ledger entry");
    assert_eq!(
        handled_entry.evidence_window,
        vec!["evt-failure-2".to_string(), "evt-failure-1".to_string()]
    );
}

#[tokio::test]
async fn auto_reflection_scopes_trigger_window_to_input_namespace() {
    let deps = test_support::deps_for_failure_modes();
    let project_namespace = Namespace::for_project("agent-llm-mm");
    deps.seed_events(vec![
        StoredEvent::new(
            "evt-project-a-1".to_string(),
            chrono::DateTime::parse_from_rfc3339("2026-03-23T10:01:00Z")
                .unwrap()
                .with_timezone(&Utc),
            Event::new_with_namespace(
                Owner::World,
                project_namespace.clone(),
                EventKind::Observation,
                "project A evidence should remain in the trigger window",
            )
            .unwrap(),
        ),
        StoredEvent::new(
            "evt-project-a-2".to_string(),
            chrono::DateTime::parse_from_rfc3339("2026-03-23T10:02:00Z")
                .unwrap()
                .with_timezone(&Utc),
            Event::new_with_namespace(
                Owner::World,
                project_namespace.clone(),
                EventKind::Observation,
                "newer project A evidence should remain in the trigger window",
            )
            .unwrap(),
        ),
        StoredEvent::new(
            "evt-project-b-newer".to_string(),
            chrono::DateTime::parse_from_rfc3339("2026-03-23T10:03:00Z")
                .unwrap()
                .with_timezone(&Utc),
            Event::new_with_namespace(
                Owner::World,
                Namespace::for_project("other"),
                EventKind::Observation,
                "newer project B evidence must not enter project A trigger window",
            )
            .unwrap(),
        ),
        StoredEvent::new(
            "evt-world-newest".to_string(),
            chrono::DateTime::parse_from_rfc3339("2026-03-23T10:04:00Z")
                .unwrap()
                .with_timezone(&Utc),
            Event::new(
                Owner::World,
                EventKind::Observation,
                "newest world evidence must not enter project A trigger window",
            ),
        ),
    ]);
    deps.set_self_revision_proposal(test_support::commitment_only_auto_reflection_proposal());

    let result = auto_reflect_if_needed::execute(
        &deps,
        AutoReflectInput::for_conflict(project_namespace, vec!["conflict".to_string()]),
    )
    .await
    .unwrap();

    assert!(result.triggered);
    assert_eq!(
        result.evidence_event_ids,
        vec!["evt-project-a-2".to_string(), "evt-project-a-1".to_string()]
    );
    let reflection = deps
        .latest_reflection()
        .expect("namespace-scoped auto-reflection should persist a reflection");
    assert_eq!(
        reflection.supporting_evidence_event_ids,
        vec!["evt-project-a-2".to_string(), "evt-project-a-1".to_string()]
    );
    let handled_entry = deps
        .latest_trigger_entry()
        .expect("namespace-scoped auto-reflection should persist a trigger ledger entry");
    assert_eq!(
        handled_entry.evidence_window,
        vec!["evt-project-a-2".to_string(), "evt-project-a-1".to_string()]
    );
}

#[tokio::test]
async fn auto_reflection_suppresses_unchanged_failure_window_after_cooldown_when_prior_reflection_used_subset()
 {
    let deps = test_support::deps_for_failure_modes();
    deps.seed_failure_window(vec![
        (
            "evt-failure-1",
            "rollback after violating a hard commitment",
        ),
        (
            "evt-failure-2",
            "second rollback after violating the same hard commitment",
        ),
    ]);
    deps.set_self_revision_proposal(
        test_support::commitment_only_auto_reflection_proposal_with_policy(
            vec!["evt-failure-2".to_string()],
            None,
        ),
    );

    let first = auto_reflect_if_needed::execute(
        &deps,
        AutoReflectInput::for_failure(
            Namespace::for_project("agent-llm-mm"),
            vec!["failure".to_string(), "rollback".to_string()],
        ),
    )
    .await
    .unwrap();
    deps.advance_now_by_hours(25);
    let second = auto_reflect_if_needed::execute(
        &deps,
        AutoReflectInput::for_failure(
            Namespace::for_project("agent-llm-mm"),
            vec!["failure".to_string(), "rollback".to_string()],
        ),
    )
    .await
    .unwrap();

    assert!(first.triggered);
    assert!(!second.triggered);
    assert_eq!(second.ledger_status, Some(TriggerLedgerStatus::Suppressed));
    assert_eq!(
        second.suppression_reason.as_deref(),
        Some("evidence_window_unchanged")
    );
    assert_eq!(deps.reflections().len(), 1);
}

#[tokio::test]
async fn auto_reflection_preserves_handled_baseline_across_cooldown_suppression_for_unchanged_failure_window()
 {
    let deps = test_support::deps_for_failure_modes();
    deps.seed_failure_window(vec![
        (
            "evt-failure-1",
            "rollback after violating a hard commitment",
        ),
        (
            "evt-failure-2",
            "second rollback after violating the same hard commitment",
        ),
    ]);
    deps.set_self_revision_proposal(
        test_support::commitment_only_auto_reflection_proposal_with_policy(
            vec!["evt-failure-2".to_string()],
            None,
        ),
    );

    let first = auto_reflect_if_needed::execute(
        &deps,
        AutoReflectInput::for_failure(
            Namespace::for_project("agent-llm-mm"),
            vec!["failure".to_string(), "rollback".to_string()],
        ),
    )
    .await
    .unwrap();
    let second = auto_reflect_if_needed::execute(
        &deps,
        AutoReflectInput::for_failure(
            Namespace::for_project("agent-llm-mm"),
            vec!["failure".to_string(), "rollback".to_string()],
        ),
    )
    .await
    .unwrap();
    deps.advance_now_by_hours(25);
    let third = auto_reflect_if_needed::execute(
        &deps,
        AutoReflectInput::for_failure(
            Namespace::for_project("agent-llm-mm"),
            vec!["failure".to_string(), "rollback".to_string()],
        ),
    )
    .await
    .unwrap();

    assert!(first.triggered);
    assert_eq!(first.ledger_status, Some(TriggerLedgerStatus::Handled));

    assert!(!second.triggered);
    assert_eq!(second.ledger_status, Some(TriggerLedgerStatus::Suppressed));
    assert_eq!(
        second.suppression_reason.as_deref(),
        Some("cooldown_active")
    );

    assert!(!third.triggered);
    assert_eq!(third.ledger_status, Some(TriggerLedgerStatus::Suppressed));
    assert_eq!(
        third.suppression_reason.as_deref(),
        Some("evidence_window_unchanged")
    );
    assert_eq!(
        deps.reflections().len(),
        1,
        "cooldown-expired unchanged failure windows should stay suppressed against the last handled baseline"
    );
}

#[tokio::test]
async fn auto_reflection_rejects_empty_proposed_evidence_query_instead_of_widening() {
    let deps = test_support::deps_for_failure_modes();
    deps.seed_failure_window(vec![
        (
            "evt-failure-1",
            "rollback after violating a hard commitment",
        ),
        (
            "evt-failure-2",
            "second rollback after violating the same hard commitment",
        ),
    ]);
    deps.set_self_revision_proposal(
        test_support::commitment_only_auto_reflection_proposal_with_policy(
            Vec::new(),
            Some(EvidenceQuery {
                namespace: None,
                owner: Some(Owner::World),
                kind: Some(EventKind::Observation),
                limit: Some(5),
            }),
        ),
    );

    let result = auto_reflect_if_needed::execute(
        &deps,
        AutoReflectInput::for_failure(
            Namespace::for_project("agent-llm-mm"),
            vec!["failure".to_string(), "rollback".to_string()],
        ),
    )
    .await;

    assert!(matches!(
        result,
        Err(AppError::InvalidParams(message))
            if message.contains("proposed evidence query did not match the current trigger window")
    ));
    assert_eq!(
        deps.latest_trigger_status(),
        Some(TriggerLedgerStatus::Rejected)
    );
    assert!(deps.latest_reflection().is_none());
}

#[tokio::test]
async fn auto_reflection_rejects_namespace_filter_with_no_trigger_window_intersection() {
    let deps = test_support::deps_for_failure_modes();
    deps.seed_events(vec![
        StoredEvent::new(
            "evt-conflict-1".to_string(),
            chrono::DateTime::parse_from_rfc3339("2026-03-23T10:01:00Z")
                .unwrap()
                .with_timezone(&Utc),
            Event::new_with_namespace(
                Owner::World,
                Namespace::for_project("agent-llm-mm"),
                EventKind::Observation,
                "current conflicting observation inside the trigger window",
            )
            .unwrap(),
        ),
        StoredEvent::new(
            "evt-conflict-2".to_string(),
            chrono::DateTime::parse_from_rfc3339("2026-03-23T10:02:00Z")
                .unwrap()
                .with_timezone(&Utc),
            Event::new_with_namespace(
                Owner::World,
                Namespace::for_project("agent-llm-mm"),
                EventKind::Observation,
                "second current conflicting observation inside the trigger window",
            )
            .unwrap(),
        ),
    ]);
    deps.set_self_revision_proposal(
        test_support::commitment_only_auto_reflection_proposal_with_policy(
            Vec::new(),
            Some(EvidenceQuery {
                namespace: Some(Namespace::for_project("other")),
                owner: Some(Owner::World),
                kind: Some(EventKind::Observation),
                limit: Some(5),
            }),
        ),
    );

    let result = auto_reflect_if_needed::execute(
        &deps,
        AutoReflectInput::for_conflict(
            Namespace::for_project("agent-llm-mm"),
            vec!["conflict".to_string(), "commitment".to_string()],
        ),
    )
    .await;

    assert!(matches!(
        result,
        Err(AppError::InvalidParams(message))
            if message.contains("proposed evidence query did not match the current trigger window")
    ));
    assert_eq!(
        deps.latest_trigger_status(),
        Some(TriggerLedgerStatus::Rejected)
    );
    assert!(deps.latest_reflection().is_none());
}

#[tokio::test]
async fn auto_reflection_rejects_noop_proposal_when_query_has_no_trigger_window_intersection() {
    let deps = test_support::deps_for_failure_modes();
    deps.seed_events(vec![StoredEvent::new(
        "evt-conflict-1".to_string(),
        chrono::DateTime::parse_from_rfc3339("2026-03-23T10:01:00Z")
            .unwrap()
            .with_timezone(&Utc),
        Event::new_with_namespace(
            Owner::World,
            Namespace::for_project("agent-llm-mm"),
            EventKind::Observation,
            "current conflicting observation inside the trigger window",
        )
        .unwrap(),
    )]);
    deps.set_self_revision_proposal(test_support::noop_auto_reflection_proposal_with_policy(
        Some(EvidenceQuery {
            namespace: Some(Namespace::for_project("other")),
            owner: Some(Owner::World),
            kind: Some(EventKind::Observation),
            limit: Some(5),
        }),
    ));

    let result = auto_reflect_if_needed::execute(
        &deps,
        AutoReflectInput::for_conflict(
            Namespace::for_project("agent-llm-mm"),
            vec!["conflict".to_string(), "commitment".to_string()],
        ),
    )
    .await;

    assert!(matches!(
        result,
        Err(AppError::InvalidParams(message))
            if message.contains("proposed evidence query did not match the current trigger window")
    ));
    assert_eq!(
        deps.latest_trigger_status(),
        Some(TriggerLedgerStatus::Rejected)
    );
    assert!(deps.latest_reflection().is_none());
}

#[tokio::test]
async fn auto_reflection_intersects_proposed_evidence_query_with_current_trigger_window_when_ids_are_empty()
 {
    let deps = test_support::deps_for_failure_modes();
    deps.seed_events(vec![
        StoredEvent::new(
            "evt-conflict-outside-window".to_string(),
            chrono::DateTime::parse_from_rfc3339("2026-03-23T09:55:00Z")
                .unwrap()
                .with_timezone(&Utc),
            Event::new(
                Owner::World,
                EventKind::Observation,
                "older conflicting observation outside the current trigger window",
            ),
        ),
        StoredEvent::new(
            "evt-conflict-1".to_string(),
            chrono::DateTime::parse_from_rfc3339("2026-03-23T10:01:00Z")
                .unwrap()
                .with_timezone(&Utc),
            Event::new(
                Owner::User,
                EventKind::Conversation,
                "user raised a possible commitment conflict",
            ),
        ),
        StoredEvent::new(
            "evt-conflict-2".to_string(),
            chrono::DateTime::parse_from_rfc3339("2026-03-23T10:02:00Z")
                .unwrap()
                .with_timezone(&Utc),
            Event::new(
                Owner::World,
                EventKind::Observation,
                "current conflicting observation inside the trigger window",
            ),
        ),
        StoredEvent::new(
            "evt-conflict-3".to_string(),
            chrono::DateTime::parse_from_rfc3339("2026-03-23T10:03:00Z")
                .unwrap()
                .with_timezone(&Utc),
            Event::new(
                Owner::Self_,
                EventKind::Action,
                "self attempted a conflicting overwrite",
            ),
        ),
        StoredEvent::new(
            "evt-conflict-4".to_string(),
            chrono::DateTime::parse_from_rfc3339("2026-03-23T10:04:00Z")
                .unwrap()
                .with_timezone(&Utc),
            Event::new(
                Owner::User,
                EventKind::Conversation,
                "user requested commitment clarification",
            ),
        ),
        StoredEvent::new(
            "evt-conflict-5".to_string(),
            chrono::DateTime::parse_from_rfc3339("2026-03-23T10:05:00Z")
                .unwrap()
                .with_timezone(&Utc),
            Event::new(
                Owner::Self_,
                EventKind::Action,
                "self retried the conflicting overwrite",
            ),
        ),
    ]);
    deps.set_self_revision_proposal(
        test_support::commitment_only_auto_reflection_proposal_with_policy(
            Vec::new(),
            Some(EvidenceQuery {
                namespace: None,
                owner: Some(Owner::World),
                kind: Some(EventKind::Observation),
                limit: Some(5),
            }),
        ),
    );

    let result = auto_reflect_if_needed::execute(
        &deps,
        AutoReflectInput::for_conflict(
            Namespace::self_(),
            vec!["conflict".to_string(), "commitment".to_string()],
        ),
    )
    .await
    .unwrap();

    assert!(result.triggered);
    assert_eq!(
        result.evidence_event_ids,
        vec!["evt-conflict-2".to_string()]
    );
    let reflection = deps
        .latest_reflection()
        .expect("query-intersected auto-reflection should persist a reflection");
    assert_eq!(
        reflection.supporting_evidence_event_ids,
        vec!["evt-conflict-2".to_string()]
    );
    let handled_entry = deps
        .latest_trigger_entry()
        .expect("handled auto-reflection should persist a trigger ledger entry");
    assert_eq!(
        handled_entry.evidence_window,
        vec![
            "evt-conflict-5".to_string(),
            "evt-conflict-4".to_string(),
            "evt-conflict-3".to_string(),
            "evt-conflict-2".to_string(),
            "evt-conflict-1".to_string(),
        ]
    );
}

#[tokio::test]
async fn auto_reflection_preserves_handled_reflection_id_when_unchanged_suppression_follows_rejected_attempt()
 {
    let deps = test_support::deps_for_failure_modes();
    deps.seed_failure_window(vec![
        (
            "evt-failure-1",
            "rollback after violating a hard commitment",
        ),
        (
            "evt-failure-2",
            "second rollback after violating the same hard commitment",
        ),
    ]);
    deps.set_self_revision_proposal(
        test_support::commitment_only_auto_reflection_proposal_with_policy(
            vec!["evt-failure-2".to_string()],
            None,
        ),
    );

    let handled = auto_reflect_if_needed::execute(
        &deps,
        AutoReflectInput::for_failure(
            Namespace::for_project("agent-llm-mm"),
            vec!["failure".to_string(), "rollback".to_string()],
        ),
    )
    .await
    .unwrap();

    deps.advance_now_by_hours(25);
    deps.seed_failure_window(vec![
        (
            "evt-failure-2",
            "second rollback after violating a hard commitment",
        ),
        (
            "evt-failure-3",
            "third rollback after violating a different hard commitment",
        ),
    ]);
    deps.set_self_revision_proposal(
        agent_llm_mm::domain::self_revision::SelfRevisionProposal::no_revision(
            "mock model did not detect a valid Failure revision".to_string(),
        ),
    );

    let rejected = auto_reflect_if_needed::execute(
        &deps,
        AutoReflectInput::for_failure(
            Namespace::for_project("agent-llm-mm"),
            vec!["failure".to_string(), "rollback".to_string()],
        ),
    )
    .await
    .unwrap();

    deps.seed_failure_window(vec![
        (
            "evt-failure-1",
            "rollback after violating a hard commitment",
        ),
        (
            "evt-failure-2",
            "second rollback after violating the same hard commitment",
        ),
    ]);

    let suppressed = auto_reflect_if_needed::execute(
        &deps,
        AutoReflectInput::for_failure(
            Namespace::for_project("agent-llm-mm"),
            vec!["failure".to_string(), "rollback".to_string()],
        ),
    )
    .await
    .unwrap();

    assert!(handled.triggered);
    assert_eq!(handled.ledger_status, Some(TriggerLedgerStatus::Handled));

    assert!(!rejected.triggered);
    assert_eq!(rejected.ledger_status, Some(TriggerLedgerStatus::Rejected));

    assert!(!suppressed.triggered);
    assert_eq!(
        suppressed.ledger_status,
        Some(TriggerLedgerStatus::Suppressed)
    );
    assert_eq!(
        suppressed.suppression_reason.as_deref(),
        Some("evidence_window_unchanged")
    );
    assert_eq!(suppressed.reflection_id, handled.reflection_id);
    assert_eq!(deps.reflections().len(), 1);
}

#[tokio::test]
async fn auto_reflection_rejected_identity_attempt_does_not_start_cooldown_for_later_valid_retry() {
    let deps = test_support::deps_for_failure_modes();
    deps.set_self_revision_proposal(test_support::identity_only_auto_reflection_proposal());
    deps.seed_identity_support_context(
        vec!["episode-001".to_string()],
        vec![StoredClaim::new(
            "claim-supporting-1".to_string(),
            ClaimDraft::new(
                Owner::Self_,
                "self.role",
                "is",
                "principal_architect",
                Mode::Observed,
            ),
            ClaimStatus::Active,
        )],
    );

    let first = auto_reflect_if_needed::execute(
        &deps,
        AutoReflectInput::for_conflict(
            Namespace::self_(),
            vec!["conflict".to_string(), "identity".to_string()],
        ),
    )
    .await;

    assert!(matches!(first, Err(AppError::InvalidParams(_))));
    assert_eq!(
        deps.latest_trigger_status(),
        Some(TriggerLedgerStatus::Rejected)
    );
    let rejected_entry = deps
        .latest_trigger_entry()
        .expect("rejected attempt should remain auditable");
    assert_eq!(rejected_entry.handled_at, None);
    assert_eq!(rejected_entry.cooldown_until, None);

    deps.seed_identity_support_context(
        vec![
            "episode-101".to_string(),
            "episode-202".to_string(),
            "episode-303".to_string(),
        ],
        vec![
            StoredClaim::new(
                "claim-supporting-1".to_string(),
                ClaimDraft::new(
                    Owner::Self_,
                    "self.role",
                    "is",
                    "principal_architect",
                    Mode::Observed,
                ),
                ClaimStatus::Active,
            ),
            StoredClaim::new(
                "claim-supporting-2".to_string(),
                ClaimDraft::new(
                    Owner::Self_,
                    "self.role",
                    "is",
                    "principal_architect",
                    Mode::Observed,
                ),
                ClaimStatus::Active,
            ),
            StoredClaim::new(
                "claim-supporting-3".to_string(),
                ClaimDraft::new(
                    Owner::Self_,
                    "self.role",
                    "is",
                    "principal_architect",
                    Mode::Observed,
                ),
                ClaimStatus::Active,
            ),
        ],
    );

    let second = auto_reflect_if_needed::execute(
        &deps,
        AutoReflectInput::for_conflict(
            Namespace::self_(),
            vec!["conflict".to_string(), "identity".to_string()],
        ),
    )
    .await
    .unwrap();

    assert!(second.triggered);
    assert_eq!(second.trigger_type, Some(TriggerType::Conflict));
    assert_eq!(
        deps.latest_trigger_status(),
        Some(TriggerLedgerStatus::Handled)
    );
    let reflection = deps
        .latest_reflection()
        .expect("later valid retry should persist a reflection");
    assert_eq!(reflection.superseded_claim_id, None);
    assert_eq!(reflection.replacement_claim_id, None);
}

#[tokio::test]
async fn auto_reflection_handled_ledger_failure_rolls_back_reflection_updates() {
    let deps = test_support::deps_with_fail_point(FailPoint::AppendHandledTriggerLedger);
    deps.set_self_revision_proposal(test_support::commitment_only_auto_reflection_proposal());

    let result = auto_reflect_if_needed::execute(
        &deps,
        AutoReflectInput::for_conflict(
            Namespace::self_(),
            vec!["conflict".to_string(), "commitment".to_string()],
        ),
    )
    .await;

    assert!(
        matches!(result, Err(AppError::Message(message)) if message == "injected handled trigger ledger failure")
    );
    assert!(deps.reflections().is_empty());
    assert_eq!(
        deps.commitments(),
        vec![Commitment::new(
            Owner::Self_,
            "forbid:write_identity_core_directly",
        )]
    );
    assert_eq!(
        deps.latest_trigger_status(),
        Some(TriggerLedgerStatus::Rejected)
    );

    deps.clear_fail_point();

    let retry = auto_reflect_if_needed::execute(
        &deps,
        AutoReflectInput::for_conflict(
            Namespace::self_(),
            vec!["conflict".to_string(), "commitment".to_string()],
        ),
    )
    .await
    .unwrap();

    assert!(retry.triggered);
    assert_eq!(
        deps.latest_trigger_status(),
        Some(TriggerLedgerStatus::Handled)
    );
    assert_eq!(deps.reflections().len(), 1);
}

#[tokio::test]
async fn auto_reflection_suppresses_periodic_trigger_during_cooldown() {
    let deps = test_support::deps_for_failure_modes();
    deps.seed_periodic_cooldown("project/agent-llm-mm:periodic");

    let result = auto_reflect_if_needed::execute(
        &deps,
        AutoReflectInput::for_periodic(
            Namespace::for_project("agent-llm-mm"),
            vec!["periodic".to_string()],
        ),
    )
    .await
    .unwrap();

    assert!(!result.triggered);
    assert_eq!(result.trigger_type, Some(TriggerType::Periodic));
    assert_eq!(
        deps.latest_trigger_status(),
        Some(TriggerLedgerStatus::Suppressed)
    );
    assert!(deps.reflection("id-2").is_none());
}

#[tokio::test]
async fn auto_reflection_returns_structured_diagnostics_for_suppressed_trigger() {
    let deps = test_support::deps_for_failure_modes();
    deps.seed_project_reflection_event();
    deps.seed_periodic_cooldown("project/agent-llm-mm:periodic");

    let result = auto_reflect_if_needed::execute(
        &deps,
        AutoReflectInput::for_periodic(
            Namespace::for_project("agent-llm-mm"),
            vec!["periodic".to_string()],
        ),
    )
    .await
    .unwrap();

    assert_eq!(result.ledger_status, Some(TriggerLedgerStatus::Suppressed));
    assert_eq!(result.reflection_id.as_deref(), Some("seeded-reflection"));
    assert_eq!(result.diagnostics.trigger_type, TriggerType::Periodic);
    assert_eq!(
        result.diagnostics.outcome,
        agent_llm_mm::domain::self_revision::AutoReflectOutcome::Suppressed
    );
    assert_eq!(result.diagnostics.rejection_reason, None);
    assert_eq!(
        result.trigger_key.as_deref(),
        Some("project/agent-llm-mm:periodic")
    );
    assert!(result.cooldown_until.is_some());
    assert_eq!(result.diagnostics.cooldown_boundary, result.cooldown_until);
    assert_eq!(
        result.suppression_reason.as_deref(),
        Some("cooldown_active")
    );
    assert_eq!(
        result.diagnostics.suppression_reason.as_deref(),
        Some("cooldown_active")
    );
    assert_eq!(result.diagnostics.evidence_window_size, 1);
    assert!(result.diagnostics.selected_evidence_event_ids.is_empty());
    assert_eq!(result.diagnostics.durable_write_path, "run_reflection");
    let diagnostics_json = serde_json::to_value(&result.diagnostics).unwrap();
    assert_eq!(diagnostics_json["trigger_type"], "periodic");
    assert_eq!(diagnostics_json["outcome"], "suppressed");
}

#[tokio::test]
async fn auto_reflection_repeated_suppression_does_not_extend_existing_cooldown() {
    let deps = test_support::deps_for_failure_modes();
    deps.seed_periodic_cooldown("project/agent-llm-mm:periodic");

    let first = auto_reflect_if_needed::execute(
        &deps,
        AutoReflectInput::for_periodic(
            Namespace::for_project("agent-llm-mm"),
            vec!["periodic".to_string()],
        ),
    )
    .await
    .unwrap();
    let first_entry = deps
        .latest_trigger_entry()
        .expect("suppressed trigger should be recorded");

    let second = auto_reflect_if_needed::execute(
        &deps,
        AutoReflectInput::for_periodic(
            Namespace::for_project("agent-llm-mm"),
            vec!["periodic".to_string()],
        ),
    )
    .await
    .unwrap();
    let second_entry = deps
        .latest_trigger_entry()
        .expect("repeated suppressed trigger should be recorded");

    assert!(!first.triggered);
    assert!(!second.triggered);
    assert_eq!(first.ledger_status, Some(TriggerLedgerStatus::Suppressed));
    assert_eq!(second.ledger_status, Some(TriggerLedgerStatus::Suppressed));
    assert_eq!(first_entry.cooldown_until, second_entry.cooldown_until);
    assert_eq!(
        second_entry.cooldown_until,
        Some(
            chrono::DateTime::parse_from_rfc3339("2026-03-24T09:50:00Z")
                .unwrap()
                .with_timezone(&Utc)
        )
    );
}

#[tokio::test]
async fn auto_reflection_preserves_reflection_id_for_non_cooldown_suppression() {
    let deps = test_support::deps_for_failure_modes();
    deps.seed_periodic_watermark_suppression("project/agent-llm-mm:periodic");

    let result = auto_reflect_if_needed::execute(
        &deps,
        AutoReflectInput::for_periodic(
            Namespace::for_project("agent-llm-mm"),
            vec!["periodic".to_string()],
        ),
    )
    .await
    .unwrap();

    assert_eq!(result.ledger_status, Some(TriggerLedgerStatus::Suppressed));
    assert_eq!(result.reflection_id.as_deref(), Some("seeded-reflection"));
    assert_eq!(
        result.suppression_reason.as_deref(),
        Some("episode_watermark_unchanged")
    );
}

mod test_support {
    use super::*;

    pub fn deps_for_failure_modes() -> FailureModeDeps {
        FailureModeDeps::new(State::default())
    }

    pub fn deps_with_fail_point(fail_point: FailPoint) -> FailureModeDeps {
        FailureModeDeps::new(State {
            fail_point: Some(fail_point),
            ..State::default()
        })
    }

    pub fn inferred_without_evidence() -> IngestInput {
        IngestInput::new(
            Event::new(
                Owner::User,
                EventKind::Conversation,
                "The user guessed at an unsupported identity change.",
            ),
            vec![ClaimDraft::new(
                Owner::Self_,
                "self.role",
                "is",
                "principal_architect",
                Mode::Inferred,
            )],
            None,
        )
    }

    pub fn conflicting_reflection() -> ReflectionInput {
        ReflectionInput::new(
            Reflection::new("A conflicting reflection should mark the claim as disputed."),
            "claim-conflict",
            None,
            Vec::new(),
        )
    }

    pub fn reflection_updates_without_evidence() -> ReflectionInput {
        ReflectionInput::new(
            Reflection::new("Identity-only updates still require supporting evidence."),
            "claim-conflict",
            None,
            Vec::new(),
        )
        .with_identity_update(vec!["identity:self=principal_architect".to_string()])
        .with_commitment_updates(vec![Commitment::new(
            Owner::Self_,
            "prefer:reflect_before_identity_changes",
        )])
    }

    pub fn reflection_updates_with_evidence() -> ReflectionInput {
        ReflectionInput::new(
            Reflection::new("Reflection should replace claim, identity, and commitments together."),
            "claim-conflict",
            Some(ClaimDraft::new(
                Owner::Self_,
                "self.role",
                "is",
                "principal_architect",
                Mode::Observed,
            )),
            vec!["evt-reflection-1".to_string()],
        )
        .with_identity_update(vec!["identity:self=principal_architect".to_string()])
        .with_commitment_updates(vec![Commitment::new(
            Owner::Self_,
            "prefer:reflect_before_identity_changes",
        )])
    }

    pub fn reflection_updates_with_empty_identity() -> ReflectionInput {
        ReflectionInput::new(
            Reflection::new("Identity updates cannot clear the canonical identity list."),
            "claim-conflict",
            None,
            vec!["evt-reflection-1".to_string()],
        )
        .with_identity_update(Vec::new())
    }

    pub fn budgeted_snapshot() -> BuildSelfSnapshotInput {
        BuildSelfSnapshotInput {
            budget: SnapshotBudget::new(3),
        }
    }

    pub fn commitment_only_auto_reflection_proposal()
    -> agent_llm_mm::domain::self_revision::SelfRevisionProposal {
        agent_llm_mm::domain::self_revision::SelfRevisionProposal {
            should_reflect: true,
            rationale: "repeated rollback should tighten future commitments".to_string(),
            machine_patch: agent_llm_mm::domain::self_revision::SelfRevisionPatch {
                identity_patch: None,
                commitment_patch: Some(
                    agent_llm_mm::domain::self_revision::SelfRevisionCommitmentPatch::new(vec![
                        "prefer:reflect_after_repeated_rollback".to_string(),
                    ]),
                ),
            },
            proposed_evidence_event_ids: Vec::new(),
            proposed_evidence_query: None,
            confidence: None,
        }
    }

    pub fn commitment_only_auto_reflection_proposal_with_policy(
        proposed_evidence_event_ids: Vec<String>,
        proposed_evidence_query: Option<EvidenceQuery>,
    ) -> agent_llm_mm::domain::self_revision::SelfRevisionProposal {
        agent_llm_mm::domain::self_revision::SelfRevisionProposal {
            should_reflect: true,
            rationale: "repeated rollback should tighten future commitments".to_string(),
            machine_patch: agent_llm_mm::domain::self_revision::SelfRevisionPatch {
                identity_patch: None,
                commitment_patch: Some(
                    agent_llm_mm::domain::self_revision::SelfRevisionCommitmentPatch::new(vec![
                        "prefer:reflect_after_repeated_rollback".to_string(),
                    ]),
                ),
            },
            proposed_evidence_event_ids,
            proposed_evidence_query,
            confidence: Some("medium".to_string()),
        }
    }

    pub fn noop_auto_reflection_proposal_with_policy(
        proposed_evidence_query: Option<EvidenceQuery>,
    ) -> agent_llm_mm::domain::self_revision::SelfRevisionProposal {
        agent_llm_mm::domain::self_revision::SelfRevisionProposal {
            should_reflect: true,
            rationale: "record-only proposal should still respect evidence query governance"
                .to_string(),
            machine_patch: agent_llm_mm::domain::self_revision::SelfRevisionPatch::default(),
            proposed_evidence_event_ids: Vec::new(),
            proposed_evidence_query,
            confidence: Some("medium".to_string()),
        }
    }

    pub fn identity_only_auto_reflection_proposal()
    -> agent_llm_mm::domain::self_revision::SelfRevisionProposal {
        agent_llm_mm::domain::self_revision::SelfRevisionProposal {
            should_reflect: true,
            rationale: "conflict-backed identity rewrite".to_string(),
            machine_patch: agent_llm_mm::domain::self_revision::SelfRevisionPatch {
                identity_patch: Some(
                    agent_llm_mm::domain::self_revision::SelfRevisionIdentityPatch::new(vec![
                        "identity:self=principal_architect".to_string(),
                    ]),
                ),
                commitment_patch: None,
            },
            proposed_evidence_event_ids: Vec::new(),
            proposed_evidence_query: None,
            confidence: None,
        }
    }
}

#[derive(Clone)]
struct FailureModeDeps {
    state: Arc<Mutex<State>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FailPoint {
    CommitReflection,
    AppendHandledTriggerLedger,
}

#[derive(Clone)]
struct State {
    committed: CommittedState,
    now: DateTime<Utc>,
    next_id: usize,
    fail_point: Option<FailPoint>,
    model_calls: Vec<ModelInput>,
    self_revision_proposal: agent_llm_mm::domain::self_revision::SelfRevisionProposal,
}

#[derive(Clone)]
struct CommittedState {
    claims: Vec<StoredClaim>,
    commitments: Vec<Commitment>,
    identity: IdentityCore,
    event_references: Vec<String>,
    episode_references: Vec<String>,
    reflections: Vec<StoredReflection>,
    trigger_ledger: Vec<StoredTriggerLedgerEntry>,
    evidence_links: Vec<(String, String)>,
    events: Vec<StoredEvent>,
}

#[derive(Default)]
struct PendingIngest {
    claims: Vec<StoredClaim>,
    evidence_links: Vec<(String, String)>,
    events: Vec<StoredEvent>,
}

#[derive(Default)]
struct PendingReflection {
    claims: Vec<StoredClaim>,
    evidence_links: Vec<(String, String)>,
    reflections: Vec<StoredReflection>,
    trigger_ledger: Vec<StoredTriggerLedgerEntry>,
    status_updates: Vec<(String, ClaimStatus)>,
    identity: Option<IdentityCore>,
    commitments: Option<Vec<Commitment>>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            committed: CommittedState {
                claims: vec![StoredClaim::new(
                    "claim-conflict".to_string(),
                    ClaimDraft::new(Owner::Self_, "self.role", "is", "architect", Mode::Observed),
                    ClaimStatus::Active,
                )],
                commitments: vec![Commitment::new(
                    Owner::Self_,
                    "forbid:write_identity_core_directly",
                )],
                identity: IdentityCore::new(vec!["identity:self=architect".to_string()]),
                event_references: vec![
                    "event:recent-duplicate".to_string(),
                    "event:recent-duplicate".to_string(),
                    "event:recent-duplicate".to_string(),
                    "event:anchor".to_string(),
                    "event:baseline".to_string(),
                ],
                episode_references: vec!["episode:failure-modes".to_string()],
                reflections: Vec::new(),
                trigger_ledger: Vec::new(),
                evidence_links: Vec::new(),
                events: vec![StoredEvent::new(
                    "evt-reflection-1".to_string(),
                    chrono::DateTime::parse_from_rfc3339("2026-03-23T10:01:00Z")
                        .unwrap()
                        .with_timezone(&Utc),
                    Event::new(Owner::World, EventKind::Observation, "evt-reflection-1"),
                )],
            },
            now: chrono::DateTime::parse_from_rfc3339("2026-03-23T10:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            next_id: 1,
            fail_point: None,
            model_calls: Vec::new(),
            self_revision_proposal:
                agent_llm_mm::domain::self_revision::SelfRevisionProposal::no_revision(
                    "mock model did not detect a valid Failure revision".to_string(),
                ),
        }
    }
}

impl FailureModeDeps {
    fn new(state: State) -> Self {
        Self {
            state: Arc::new(Mutex::new(state)),
        }
    }

    fn claim(&self, claim_id: &str) -> Option<StoredClaim> {
        self.state
            .lock()
            .unwrap()
            .committed
            .claims
            .iter()
            .find(|claim| claim.claim_id == claim_id)
            .cloned()
    }

    fn claim_count(&self) -> usize {
        self.state.lock().unwrap().committed.claims.len()
    }

    fn identity(&self) -> IdentityCore {
        self.state.lock().unwrap().committed.identity.clone()
    }

    fn commitments(&self) -> Vec<Commitment> {
        self.state.lock().unwrap().committed.commitments.clone()
    }

    fn reflection(&self, reflection_id: &str) -> Option<StoredReflection> {
        self.state
            .lock()
            .unwrap()
            .committed
            .reflections
            .iter()
            .find(|reflection| reflection.reflection_id == reflection_id)
            .cloned()
    }

    fn reflections(&self) -> Vec<StoredReflection> {
        self.state.lock().unwrap().committed.reflections.clone()
    }

    fn latest_reflection(&self) -> Option<StoredReflection> {
        self.state
            .lock()
            .unwrap()
            .committed
            .reflections
            .last()
            .cloned()
    }

    fn evidence_links(&self) -> Vec<(String, String)> {
        self.state.lock().unwrap().committed.evidence_links.clone()
    }

    fn latest_trigger_status(&self) -> Option<TriggerLedgerStatus> {
        self.state
            .lock()
            .unwrap()
            .committed
            .trigger_ledger
            .last()
            .map(|entry| entry.status)
    }

    fn latest_trigger_entry(&self) -> Option<StoredTriggerLedgerEntry> {
        self.state
            .lock()
            .unwrap()
            .committed
            .trigger_ledger
            .last()
            .cloned()
    }

    fn clear_fail_point(&self) {
        self.state.lock().unwrap().fail_point = None;
    }

    fn set_self_revision_proposal(
        &self,
        proposal: agent_llm_mm::domain::self_revision::SelfRevisionProposal,
    ) {
        self.state.lock().unwrap().self_revision_proposal = proposal;
    }

    fn seed_identity_support_context(
        &self,
        episode_references: Vec<String>,
        claims: Vec<StoredClaim>,
    ) {
        let mut state = self.state.lock().unwrap();
        state.committed.episode_references = episode_references;
        state.committed.claims = claims;
    }

    fn seed_failure_window(&self, events: Vec<(&str, &str)>) {
        let mut state = self.state.lock().unwrap();
        state.committed.events = events
            .into_iter()
            .enumerate()
            .map(|(index, (event_id, summary))| {
                StoredEvent::new(
                    event_id.to_string(),
                    chrono::DateTime::parse_from_rfc3339(&format!(
                        "2026-03-23T10:0{}:00Z",
                        index + 1
                    ))
                    .unwrap()
                    .with_timezone(&Utc),
                    Event::new(Owner::Self_, EventKind::Action, summary),
                )
            })
            .collect();
    }

    fn seed_events(&self, events: Vec<StoredEvent>) {
        let mut event_references = events
            .iter()
            .map(|event| (event.recorded_at, format!("event:{}", event.event_id)))
            .collect::<Vec<_>>();
        event_references.sort_by(|lhs, rhs| rhs.0.cmp(&lhs.0).then_with(|| rhs.1.cmp(&lhs.1)));
        let event_references = event_references
            .into_iter()
            .map(|(_, event_reference)| event_reference)
            .collect();
        let mut state = self.state.lock().unwrap();
        state.committed.events = events;
        state.committed.event_references = event_references;
    }

    fn seed_project_reflection_event(&self) {
        self.seed_events(vec![StoredEvent::new(
            "evt-reflection-1".to_string(),
            chrono::DateTime::parse_from_rfc3339("2026-03-23T10:01:00Z")
                .unwrap()
                .with_timezone(&Utc),
            Event::new_with_namespace(
                Owner::World,
                Namespace::for_project("agent-llm-mm"),
                EventKind::Observation,
                "evt-reflection-1",
            )
            .unwrap(),
        )]);
    }

    fn advance_now_by_hours(&self, hours: i64) {
        let mut state = self.state.lock().unwrap();
        state.now += chrono::Duration::hours(hours);
    }

    fn seed_periodic_cooldown(&self, trigger_key: &str) {
        self.state
            .lock()
            .unwrap()
            .committed
            .trigger_ledger
            .push(StoredTriggerLedgerEntry {
                ledger_id: "ledger-seeded-periodic".to_string(),
                trigger_type: TriggerType::Periodic,
                namespace: Namespace::for_project("agent-llm-mm"),
                trigger_key: trigger_key.to_string(),
                status: TriggerLedgerStatus::Handled,
                evidence_window: vec!["evt-reflection-1".to_string()],
                handled_at: Some(
                    chrono::DateTime::parse_from_rfc3339("2026-03-23T09:50:00Z")
                        .unwrap()
                        .with_timezone(&Utc),
                ),
                cooldown_until: Some(
                    chrono::DateTime::parse_from_rfc3339("2026-03-24T09:50:00Z")
                        .unwrap()
                        .with_timezone(&Utc),
                ),
                episode_watermark: Some(1),
                reflection_id: Some("seeded-reflection".to_string()),
            });
    }

    fn seed_periodic_watermark_suppression(&self, trigger_key: &str) {
        self.state
            .lock()
            .unwrap()
            .committed
            .trigger_ledger
            .push(StoredTriggerLedgerEntry {
                ledger_id: "ledger-seeded-periodic-watermark".to_string(),
                trigger_type: TriggerType::Periodic,
                namespace: Namespace::for_project("agent-llm-mm"),
                trigger_key: trigger_key.to_string(),
                status: TriggerLedgerStatus::Handled,
                evidence_window: vec!["evt-unrelated".to_string()],
                handled_at: Some(
                    chrono::DateTime::parse_from_rfc3339("2026-03-22T09:50:00Z")
                        .unwrap()
                        .with_timezone(&Utc),
                ),
                cooldown_until: Some(
                    chrono::DateTime::parse_from_rfc3339("2026-03-23T09:30:00Z")
                        .unwrap()
                        .with_timezone(&Utc),
                ),
                episode_watermark: Some(1),
                reflection_id: Some("seeded-reflection".to_string()),
            });
    }
}

fn upsert_claim(target: &mut Vec<StoredClaim>, claim: StoredClaim) {
    if let Some(existing) = target
        .iter_mut()
        .find(|stored_claim| stored_claim.claim_id == claim.claim_id)
    {
        *existing = claim;
    } else {
        target.push(claim);
    }
}

#[async_trait]
impl Clock for FailureModeDeps {
    async fn now(&self) -> Result<DateTime<Utc>, AppError> {
        Ok(self.state.lock().unwrap().now)
    }
}

#[async_trait]
impl IdGenerator for FailureModeDeps {
    async fn next_id(&self) -> Result<String, AppError> {
        let mut state = self.state.lock().unwrap();
        let next_id = format!("id-{}", state.next_id);
        state.next_id += 1;
        Ok(next_id)
    }
}

#[async_trait]
impl ModelPort for FailureModeDeps {
    async fn decide(&self, input: ModelInput) -> Result<ModelDecision, AppError> {
        self.state.lock().unwrap().model_calls.push(input);
        Ok(ModelDecision::new("Proceed".to_string()))
    }

    async fn propose_self_revision(
        &self,
        request: agent_llm_mm::domain::self_revision::SelfRevisionRequest,
    ) -> Result<agent_llm_mm::domain::self_revision::SelfRevisionProposal, AppError> {
        let state = self.state.lock().unwrap();
        if state.self_revision_proposal.should_reflect {
            return Ok(state.self_revision_proposal.clone());
        }

        Ok(
            agent_llm_mm::domain::self_revision::SelfRevisionProposal::no_revision(format!(
                "mock model did not detect a valid {:?} revision",
                request.trigger_type
            )),
        )
    }
}

#[async_trait]
impl IdentityStore for FailureModeDeps {
    async fn load_identity(&self) -> Result<IdentityCore, AppError> {
        Ok(self.state.lock().unwrap().committed.identity.clone())
    }

    async fn save_identity(&self, identity: IdentityCore) -> Result<(), AppError> {
        self.state.lock().unwrap().committed.identity = identity;
        Ok(())
    }
}

#[async_trait]
impl CommitmentStore for FailureModeDeps {
    async fn list_commitments(&self) -> Result<Vec<Commitment>, AppError> {
        Ok(self.state.lock().unwrap().committed.commitments.clone())
    }
}

#[async_trait]
impl EventStore for FailureModeDeps {
    async fn append_event(&self, event: StoredEvent) -> Result<(), AppError> {
        self.state.lock().unwrap().committed.events.push(event);
        Ok(())
    }

    async fn list_event_references(&self) -> Result<Vec<String>, AppError> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .committed
            .event_references
            .clone())
    }

    async fn query_evidence_event_ids(
        &self,
        query: EvidenceQuery,
    ) -> Result<Vec<String>, AppError> {
        let mut events = self.state.lock().unwrap().committed.events.clone();
        filter_and_order_events(&mut events, query.namespace, query.owner, query.kind);

        let limit = query.limit.unwrap_or(10);
        if limit == 0 {
            return Ok(Vec::new());
        }

        Ok(events
            .into_iter()
            .take(limit)
            .map(|event| event.event_id)
            .collect())
    }

    async fn query_evidence_event_ids_unbounded(
        &self,
        query: EvidenceQuery,
    ) -> Result<Vec<String>, AppError> {
        let mut events = self.state.lock().unwrap().committed.events.clone();
        filter_and_order_events(&mut events, query.namespace, query.owner, query.kind);

        let events = if let Some(limit) = query.limit {
            events.into_iter().take(limit).collect()
        } else {
            events
        };

        Ok(events.into_iter().map(|event| event.event_id).collect())
    }

    async fn has_event(&self, event_id: &str) -> Result<bool, AppError> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .committed
            .events
            .iter()
            .any(|event| event.event_id == event_id))
    }
}

fn filter_and_order_events(
    events: &mut Vec<StoredEvent>,
    namespace: Option<Namespace>,
    owner: Option<Owner>,
    kind: Option<EventKind>,
) {
    if let Some(namespace) = namespace {
        events.retain(|event| event.event.namespace() == &namespace);
    }

    if let Some(owner) = owner {
        events.retain(|event| event.event.owner() == owner);
    }

    if let Some(kind) = kind {
        events.retain(|event| event.event.kind() == kind);
    }

    events.sort_by(|lhs, rhs| {
        rhs.recorded_at
            .cmp(&lhs.recorded_at)
            .then_with(|| rhs.event_id.cmp(&lhs.event_id))
    });
}

#[async_trait]
impl EpisodeStore for FailureModeDeps {
    async fn record_event_in_episode(
        &self,
        episode_reference: String,
        _event_id: String,
    ) -> Result<(), AppError> {
        self.state
            .lock()
            .unwrap()
            .committed
            .episode_references
            .push(episode_reference);
        Ok(())
    }

    async fn list_episode_references(&self) -> Result<Vec<String>, AppError> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .committed
            .episode_references
            .clone())
    }
}

#[async_trait]
impl ClaimStore for FailureModeDeps {
    async fn upsert_claim(&self, claim: StoredClaim) -> Result<(), AppError> {
        upsert_claim(&mut self.state.lock().unwrap().committed.claims, claim);
        Ok(())
    }

    async fn link_evidence(&self, claim_id: String, event_id: String) -> Result<(), AppError> {
        self.state
            .lock()
            .unwrap()
            .committed
            .evidence_links
            .push((claim_id, event_id));
        Ok(())
    }

    async fn list_active_claims(&self) -> Result<Vec<StoredClaim>, AppError> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .committed
            .claims
            .iter()
            .filter(|claim| claim.status == ClaimStatus::Active)
            .cloned()
            .collect())
    }

    async fn update_claim_status(
        &self,
        claim_id: &str,
        status: ClaimStatus,
    ) -> Result<(), AppError> {
        let mut state = self.state.lock().unwrap();
        let claim = state
            .committed
            .claims
            .iter_mut()
            .find(|claim| claim.claim_id == claim_id)
            .ok_or_else(|| AppError::Message(format!("missing claim: {claim_id}")))?;
        claim.status = status;
        Ok(())
    }
}

#[async_trait]
impl IngestTransactionRunner for FailureModeDeps {
    async fn begin_ingest_transaction(
        &self,
    ) -> Result<Box<dyn IngestTransaction + Send + '_>, AppError> {
        Ok(Box::new(FailureModeIngestTransaction {
            deps: self.clone(),
            pending: PendingIngest::default(),
        }))
    }
}

#[async_trait]
impl ReflectionTransactionRunner for FailureModeDeps {
    async fn begin_reflection_transaction(
        &self,
    ) -> Result<Box<dyn ReflectionTransaction + Send + '_>, AppError> {
        Ok(Box::new(FailureModeReflectionTransaction {
            deps: self.clone(),
            pending: PendingReflection::default(),
        }))
    }
}

#[async_trait]
impl TriggerLedgerStore for FailureModeDeps {
    async fn record_trigger_attempt(
        &self,
        entry: StoredTriggerLedgerEntry,
    ) -> Result<(), AppError> {
        self.state
            .lock()
            .unwrap()
            .committed
            .trigger_ledger
            .push(entry);
        Ok(())
    }

    async fn latest_trigger_entry(
        &self,
        trigger_key: &str,
    ) -> Result<Option<StoredTriggerLedgerEntry>, AppError> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .committed
            .trigger_ledger
            .iter()
            .rev()
            .find(|entry| entry.trigger_key == trigger_key)
            .cloned())
    }

    async fn latest_handled_trigger_entry(
        &self,
        trigger_key: &str,
    ) -> Result<Option<StoredTriggerLedgerEntry>, AppError> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .committed
            .trigger_ledger
            .iter()
            .rev()
            .find(|entry| {
                entry.trigger_key == trigger_key && entry.status == TriggerLedgerStatus::Handled
            })
            .cloned())
    }
}

struct FailureModeIngestTransaction {
    deps: FailureModeDeps,
    pending: PendingIngest,
}

#[async_trait]
impl IngestTransaction for FailureModeIngestTransaction {
    async fn append_event(&mut self, event: StoredEvent) -> Result<(), AppError> {
        self.pending.events.push(event);
        Ok(())
    }

    async fn record_event_in_episode(
        &mut self,
        _episode_reference: String,
        _event_id: String,
    ) -> Result<(), AppError> {
        Ok(())
    }

    async fn upsert_claim(&mut self, claim: StoredClaim) -> Result<(), AppError> {
        self.pending.claims.push(claim);
        Ok(())
    }

    async fn link_evidence(&mut self, claim_id: String, event_id: String) -> Result<(), AppError> {
        self.pending.evidence_links.push((claim_id, event_id));
        Ok(())
    }

    async fn commit(self: Box<Self>) -> Result<(), AppError> {
        let mut state = self.deps.state.lock().unwrap();
        state.committed.events.extend(self.pending.events);
        state
            .committed
            .evidence_links
            .extend(self.pending.evidence_links);
        for claim in self.pending.claims {
            upsert_claim(&mut state.committed.claims, claim);
        }
        Ok(())
    }
}

struct FailureModeReflectionTransaction {
    deps: FailureModeDeps,
    pending: PendingReflection,
}

#[async_trait]
impl ReflectionTransaction for FailureModeReflectionTransaction {
    async fn upsert_claim(&mut self, claim: StoredClaim) -> Result<(), AppError> {
        self.pending.claims.push(claim);
        Ok(())
    }

    async fn link_evidence(&mut self, claim_id: String, event_id: String) -> Result<(), AppError> {
        self.pending.evidence_links.push((claim_id, event_id));
        Ok(())
    }

    async fn append_reflection(&mut self, reflection: StoredReflection) -> Result<(), AppError> {
        self.pending.reflections.push(reflection);
        Ok(())
    }

    async fn append_trigger_ledger(
        &mut self,
        entry: StoredTriggerLedgerEntry,
    ) -> Result<(), AppError> {
        if self.deps.state.lock().unwrap().fail_point == Some(FailPoint::AppendHandledTriggerLedger)
            && entry.status == TriggerLedgerStatus::Handled
        {
            return Err(AppError::Message(
                "injected handled trigger ledger failure".to_string(),
            ));
        }

        self.pending.trigger_ledger.push(entry);
        Ok(())
    }

    async fn load_identity(&mut self) -> Result<IdentityCore, AppError> {
        if let Some(identity) = &self.pending.identity {
            return Ok(identity.clone());
        }

        Ok(self.deps.state.lock().unwrap().committed.identity.clone())
    }

    async fn replace_identity(&mut self, identity: IdentityCore) -> Result<(), AppError> {
        self.pending.identity = Some(identity);
        Ok(())
    }

    async fn load_commitments(&mut self) -> Result<Vec<Commitment>, AppError> {
        if let Some(commitments) = &self.pending.commitments {
            return Ok(commitments.clone());
        }

        Ok(self
            .deps
            .state
            .lock()
            .unwrap()
            .committed
            .commitments
            .clone())
    }

    async fn replace_commitments(&mut self, commitments: Vec<Commitment>) -> Result<(), AppError> {
        self.pending.commitments = Some(commitments);
        Ok(())
    }

    async fn update_claim_status(
        &mut self,
        claim_id: &str,
        status: ClaimStatus,
    ) -> Result<(), AppError> {
        self.pending
            .status_updates
            .push((claim_id.to_string(), status));
        Ok(())
    }

    async fn commit(self: Box<Self>) -> Result<(), AppError> {
        if self.deps.state.lock().unwrap().fail_point == Some(FailPoint::CommitReflection) {
            return Err(AppError::Message(
                "injected reflection commit failure".to_string(),
            ));
        }
        let mut state = self.deps.state.lock().unwrap();
        for claim in self.pending.claims {
            upsert_claim(&mut state.committed.claims, claim);
        }
        state
            .committed
            .evidence_links
            .extend(self.pending.evidence_links);
        state.committed.reflections.extend(self.pending.reflections);
        state
            .committed
            .trigger_ledger
            .extend(self.pending.trigger_ledger);
        if let Some(identity) = self.pending.identity {
            state.committed.identity = identity;
        }
        if let Some(commitments) = self.pending.commitments {
            state.committed.commitments = commitments;
        }
        for (claim_id, status) in self.pending.status_updates {
            let claim = state
                .committed
                .claims
                .iter_mut()
                .find(|claim| claim.claim_id == claim_id)
                .ok_or_else(|| AppError::Message(format!("missing claim: {claim_id}")))?;
            claim.status = status;
        }
        Ok(())
    }
}
