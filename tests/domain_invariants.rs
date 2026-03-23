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
