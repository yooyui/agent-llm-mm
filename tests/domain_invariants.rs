use agent_llm_mm::domain::{
    claim::ClaimDraft,
    types::{Mode, Owner},
};

#[test]
fn inferred_claim_requires_external_evidence() {
    let draft = ClaimDraft::new_inferred(Owner::Self_, "self.role", "is", "architect");
    assert!(draft.validate(0).is_err());
}

#[test]
fn identity_core_updates_are_not_allowed_from_ingest_mode() {
    let result = agent_llm_mm::domain::identity_core::allow_direct_ingest_update(Mode::Said);
    assert!(!result);
}
