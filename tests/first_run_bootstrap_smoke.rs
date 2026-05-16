use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    process::Command,
    sync::{Mutex, OnceLock},
};

use serde_json::Value;
use tempfile::tempdir;

#[test]
fn first_run_bootstrap_smoke_runs_bootstrap_and_doctor_in_isolated_output_dir() {
    if !command_exists("bash") {
        return;
    }
    let temp_dir = tempdir().expect("temp dir");
    let output_dir = temp_dir.path().join("first run smoke");

    let output = Command::new("bash")
        .args([
            "scripts/first-run-bootstrap-smoke-local.sh",
            output_dir.to_str().expect("utf-8 output dir"),
        ])
        .output()
        .expect("first-run bootstrap smoke should run");

    assert!(
        output.status.success(),
        "first-run bootstrap smoke should pass; stdout={}; stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let doctor = read_json(output_dir.join("doctor.json"));
    assert_eq!(doctor["status"], "ok");
    assert_eq!(doctor["provider"], "mock");
    assert_eq!(doctor["self_revision_write_path"], "run_reflection");
    assert_eq!(doctor["daemon_enabled"], false);
    assert_eq!(doctor["daemon_observe_only"]["mode"], "observe_only");
    assert_eq!(doctor["daemon_observe_only"]["write_gate_approved"], false);
    assert_eq!(doctor["daemon_observe_only"]["writes_allowed"], false);
    assert_eq!(
        doctor["daemon_observe_only"]["remote_listener_enabled"],
        false
    );
    let resolved_output_dir = fs::canonicalize(&output_dir).expect("canonical output dir");
    assert_eq!(
        doctor["database_url"],
        sqlite_url(resolved_output_dir.join("first-run.sqlite"))
    );

    let summary = read_json(output_dir.join("summary.json"));
    assert_eq!(summary["kind"], "local_first_run_bootstrap_simulation");
    assert_eq!(summary["local_only"], true);
    assert_eq!(summary["fresh_machine_simulation"], true);
    assert_eq!(summary["real_fresh_machine_evidence"], false);
    assert_eq!(summary["doctor_status"], "ok");
    assert_eq!(summary["provider"], "mock");
    assert_eq!(summary["self_revision_write_path"], "run_reflection");
    assert_eq!(summary["daemon_enabled"], false);
    assert_eq!(summary["daemon_observe_only"]["mode"], "observe_only");
    assert_eq!(summary["daemon_observe_only"]["write_gate_approved"], false);
    assert_eq!(summary["daemon_observe_only"]["writes_allowed"], false);
    assert_eq!(
        summary["daemon_observe_only"]["remote_listener_enabled"],
        false
    );
    assert_eq!(summary["daemon_writes_allowed"], false);
    assert_eq!(summary["sqlite_database_exists"], true);
    assert_eq!(summary["started_serve"], false);
    assert_eq!(summary["ran_product_smoke"], false);
    assert_eq!(
        summary["database_url"],
        sqlite_url(resolved_output_dir.join("first-run.sqlite"))
    );
    assert!(
        output_dir.join("first-run.sqlite").exists(),
        "doctor should bootstrap the isolated sqlite database"
    );
}

#[test]
fn first_run_bootstrap_smoke_refuses_non_empty_output_dir() {
    if !command_exists("bash") {
        return;
    }
    let temp_dir = tempdir().expect("temp dir");
    let output_dir = temp_dir.path().join("non-empty");
    fs::create_dir(&output_dir).expect("create output dir");
    let marker = output_dir.join("marker.txt");
    fs::write(&marker, "keep me").expect("seed marker");

    let output = Command::new("bash")
        .args([
            "scripts/first-run-bootstrap-smoke-local.sh",
            output_dir.to_str().expect("utf-8 output dir"),
        ])
        .output()
        .expect("first-run bootstrap smoke should run");

    assert!(
        !output.status.success(),
        "first-run bootstrap smoke should refuse non-empty output dirs"
    );
    assert_eq!(
        fs::read_to_string(marker).expect("marker remains"),
        "keep me"
    );
    assert!(
        !output_dir.join("agent-llm-mm.local.toml").exists(),
        "refusal should happen before generating config"
    );
    assert!(
        !output_dir.join("doctor.json").exists(),
        "refusal should happen before writing doctor evidence"
    );
    assert!(
        !output_dir.join("summary.json").exists(),
        "refusal should happen before writing summary evidence"
    );
    assert!(
        !output_dir.join("first-run.sqlite").exists(),
        "refusal should happen before bootstrapping sqlite"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("output directory is not empty"),
        "stderr should explain stale evidence refusal"
    );
}

#[test]
fn first_run_bootstrap_smoke_ignores_real_home_and_database_url_env() {
    if !command_exists("bash") {
        return;
    }
    let temp_dir = tempdir().expect("temp dir");
    let fake_home = temp_dir.path().join("fake-home");
    let fake_xdg = temp_dir.path().join("fake-xdg");
    fs::create_dir(&fake_home).expect("create fake home");
    fs::create_dir(&fake_xdg).expect("create fake xdg");
    let env_database_path = temp_dir.path().join("env-override.sqlite");
    let env_database_url = sqlite_url(env_database_path.clone());
    let output_dir = temp_dir.path().join("first-run-env-isolation");
    let _guard = EnvGuard::apply([
        EnvChange::Set("HOME", fake_home.to_string_lossy().into_owned()),
        EnvChange::Set("XDG_DATA_HOME", fake_xdg.to_string_lossy().into_owned()),
        EnvChange::Set("AGENT_LLM_MM_DATABASE_URL", env_database_url.clone()),
    ]);

    let output = Command::new("bash")
        .args([
            "scripts/first-run-bootstrap-smoke-local.sh",
            output_dir.to_str().expect("utf-8 output dir"),
        ])
        .output()
        .expect("first-run bootstrap smoke should run");

    assert!(
        output.status.success(),
        "first-run bootstrap smoke should ignore env database overrides; stdout={}; stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let doctor = read_json(output_dir.join("doctor.json"));
    assert_eq!(
        doctor["database_url"],
        sqlite_url(
            fs::canonicalize(&output_dir)
                .expect("canonical output dir")
                .join("first-run.sqlite")
        )
    );
    assert_ne!(doctor["database_url"], env_database_url);
    assert!(
        !env_database_path.exists(),
        "env override database must not be touched"
    );
    assert!(
        fs::read_dir(&fake_home)
            .expect("fake home readable")
            .next()
            .is_none(),
        "smoke should not write into HOME"
    );
    assert!(
        fs::read_dir(&fake_xdg)
            .expect("fake xdg readable")
            .next()
            .is_none(),
        "smoke should not write into XDG_DATA_HOME"
    );
}

#[test]
fn first_run_bootstrap_smoke_stays_bootstrap_doctor_only() {
    let script = fs::read_to_string("scripts/first-run-bootstrap-smoke-local.sh")
        .expect("script should be readable");

    assert!(
        script.contains("bootstrap-local"),
        "smoke helper should exercise bootstrap-local"
    );
    assert!(
        script.contains(" doctor "),
        "smoke helper should exercise doctor"
    );
    assert!(
        !script.contains("agent-llm-mm.sh serve"),
        "smoke helper must not start serve"
    );
    assert!(
        !script.contains("product-smoke-local.sh"),
        "smoke helper must not call the broader product smoke script"
    );
    assert!(
        !script.contains("run-self-revision-demo.sh"),
        "smoke helper must not call the deterministic demo wrapper"
    );
    assert!(!script.contains(" ssh "), "smoke helper must not call ssh");
    assert!(!script.contains(" scp "), "smoke helper must not call scp");
    assert!(
        !script.contains(" rsync "),
        "smoke helper must not call rsync"
    );
    assert!(
        !script.contains("daemon start"),
        "smoke helper must not start daemon behavior"
    );
    assert!(
        !script.contains("agent-llm-mm.sh run_reflection")
            && !script.contains("-- run_reflection")
            && !script.contains(" run_reflection "),
        "smoke helper must not invoke durable reflection writes"
    );
}

fn command_exists(command: &str) -> bool {
    Command::new(command)
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

fn read_json(path: PathBuf) -> Value {
    serde_json::from_slice(&fs::read(path).expect("read json")).expect("parse json")
}

fn sqlite_url(path: PathBuf) -> String {
    format!("sqlite://{}", path.to_string_lossy().replace('\\', "/"))
}

enum EnvChange {
    Set(&'static str, String),
}

struct EnvGuard {
    _lock: std::sync::MutexGuard<'static, ()>,
    previous: HashMap<&'static str, Option<String>>,
}

impl EnvGuard {
    fn apply<const N: usize>(changes: [EnvChange; N]) -> Self {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

        let lock = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("env lock");

        let mut previous = HashMap::new();
        for change in changes {
            match change {
                EnvChange::Set(key, value) => {
                    previous.insert(key, std::env::var(key).ok());
                    unsafe { std::env::set_var(key, value) };
                }
            }
        }

        Self {
            _lock: lock,
            previous,
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (key, value) in &self.previous {
            match value {
                Some(value) => unsafe { std::env::set_var(key, value) },
                None => unsafe { std::env::remove_var(key) },
            }
        }
    }
}
