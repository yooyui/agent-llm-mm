#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemoteTeamCapabilityState {
    Blocked,
    NotImplemented,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RemoteTeamCapabilityInventory {
    pub local_only: bool,
    pub support_bundle_upload_available: bool,
    pub capabilities: Vec<RemoteTeamCapability>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RemoteTeamCapability {
    pub name: &'static str,
    pub state: RemoteTeamCapabilityState,
    pub write_capable_route_exposed: bool,
    pub blocker: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RemoteTeamSecurityGateReport {
    pub remote_writes_allowed: bool,
    pub gates: Vec<RemoteTeamSecurityGate>,
}

impl RemoteTeamSecurityGateReport {
    pub fn blocked_gate_names(&self) -> Vec<&'static str> {
        self.gates
            .iter()
            .filter(|gate| !gate.satisfied)
            .map(|gate| gate.name)
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RemoteTeamSecurityGate {
    pub name: &'static str,
    pub satisfied: bool,
    pub blocker: &'static str,
}

pub fn remote_team_capability_inventory() -> RemoteTeamCapabilityInventory {
    RemoteTeamCapabilityInventory {
        local_only: true,
        support_bundle_upload_available: false,
        capabilities: vec![
            blocked_capability(
                "remote_read_dashboard",
                "remote dashboard is blocked until auth, authorization, audit, rate limit, and tenant isolation gates exist",
            ),
            blocked_capability(
                "remote_write_admin",
                "write-capable remote admin is blocked until auth, authorization, audit, rollback, and tenant isolation gates exist",
            ),
            blocked_capability(
                "team_shared_memory",
                "team mode is blocked until tenant isolation, authz, backup, migration, and audit gates exist",
            ),
            blocked_capability(
                "remote_support_bundle_upload",
                "support bundles remain local-only; no upload route is implemented",
            ),
        ],
    }
}

pub fn remote_team_security_gate_report() -> RemoteTeamSecurityGateReport {
    let gates = vec![
        security_gate("auth", "authentication is not implemented"),
        security_gate("authorization", "authorization roles are not implemented"),
        security_gate(
            "audit",
            "remote access audit persistence is not implemented",
        ),
        security_gate("rate_limit", "remote route rate limits are not implemented"),
        security_gate("tenant_isolation", "tenant isolation is not implemented"),
        security_gate("rollback", "remote write rollback is not implemented"),
    ];

    RemoteTeamSecurityGateReport {
        remote_writes_allowed: gates.iter().all(|gate| gate.satisfied),
        gates,
    }
}

fn blocked_capability(name: &'static str, blocker: &'static str) -> RemoteTeamCapability {
    RemoteTeamCapability {
        name,
        state: RemoteTeamCapabilityState::Blocked,
        write_capable_route_exposed: false,
        blocker,
    }
}

fn security_gate(name: &'static str, blocker: &'static str) -> RemoteTeamSecurityGate {
    RemoteTeamSecurityGate {
        name,
        satisfied: false,
        blocker,
    }
}
