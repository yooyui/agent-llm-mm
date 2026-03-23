pub const INIT_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS events (
    event_id TEXT PRIMARY KEY,
    recorded_at TEXT NOT NULL,
    owner TEXT NOT NULL,
    kind TEXT NOT NULL,
    summary TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS claims (
    claim_id TEXT PRIMARY KEY,
    owner TEXT NOT NULL,
    subject TEXT NOT NULL,
    predicate TEXT NOT NULL,
    object TEXT NOT NULL,
    mode TEXT NOT NULL,
    status TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS evidence_links (
    claim_id TEXT NOT NULL,
    event_id TEXT NOT NULL,
    PRIMARY KEY (claim_id, event_id),
    FOREIGN KEY (claim_id) REFERENCES claims(claim_id),
    FOREIGN KEY (event_id) REFERENCES events(event_id)
);

CREATE TABLE IF NOT EXISTS episode_events (
    episode_reference TEXT NOT NULL,
    event_id TEXT NOT NULL,
    PRIMARY KEY (episode_reference, event_id),
    FOREIGN KEY (event_id) REFERENCES events(event_id)
);

CREATE TABLE IF NOT EXISTS reflections (
    reflection_id TEXT PRIMARY KEY,
    recorded_at TEXT NOT NULL,
    summary TEXT NOT NULL,
    superseded_claim_id TEXT,
    replacement_claim_id TEXT
);

CREATE TABLE IF NOT EXISTS identity_claims (
    position INTEGER NOT NULL PRIMARY KEY,
    claim TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS commitments (
    description TEXT PRIMARY KEY,
    owner TEXT NOT NULL
);
"#;
