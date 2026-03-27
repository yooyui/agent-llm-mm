use agent_llm_mm::domain::{
    claim::ClaimDraft,
    types::{Mode, Namespace, Owner},
};

#[test]
fn inferred_claim_requires_external_evidence() {
    let draft = ClaimDraft::new_inferred(Owner::Self_, "self.role", "is", "architect");
    assert!(draft.validate(0).is_err());
}

#[test]
fn identity_core_updates_are_not_allowed_from_any_ingest_mode() {
    let ingest_modes = [Mode::Observed, Mode::Said, Mode::Acted, Mode::Inferred];

    for mode in ingest_modes {
        let result = agent_llm_mm::domain::identity_core::allow_direct_ingest_update(mode);
        assert!(
            !result,
            "identity_core direct updates should stay blocked for ingest mode {:?}",
            mode
        );
    }
}

#[test]
fn claim_defaults_namespace_from_owner_scope() {
    let self_claim = ClaimDraft::new(Owner::Self_, "self.role", "is", "architect", Mode::Observed);
    let user_claim = ClaimDraft::new(
        Owner::User,
        "user.preference",
        "likes",
        "concise",
        Mode::Observed,
    );
    let world_claim = ClaimDraft::new(
        Owner::World,
        "project.memory",
        "needs",
        "structure",
        Mode::Observed,
    );

    assert_eq!(self_claim.namespace().as_str(), "self");
    assert_eq!(user_claim.namespace().as_str(), "user/default");
    assert_eq!(world_claim.namespace().as_str(), "world");
}

#[test]
fn explicit_namespace_must_match_owner_scope() {
    let draft = ClaimDraft::new_with_namespace(
        Owner::Self_,
        Namespace::for_user("default"),
        "self.role",
        "is",
        "architect",
        Mode::Observed,
    );

    assert!(draft.validate(1).is_err());
}
