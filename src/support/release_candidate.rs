use anyhow::{Result, anyhow};

pub fn validate_release_candidate(candidate: &str) -> Result<()> {
    let allowed = !candidate.is_empty()
        && !candidate.starts_with('.')
        && !candidate.contains("..")
        && candidate
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'));

    if allowed {
        return Ok(());
    }

    Err(anyhow!(
        "candidate name must contain only letters, numbers, dot, underscore, and dash, and must not contain path traversal"
    ))
}
