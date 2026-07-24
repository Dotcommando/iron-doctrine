use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use serde_json::Value;

fn sim_cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sim-cli"))
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
}

fn empty_match_path() -> PathBuf {
    repository_root().join("scenarios").join("empty-match.json")
}

fn run_scenario(path: PathBuf) -> Output {
    sim_cli()
        .arg("run")
        .arg(path)
        .output()
        .expect("sim-cli process should run")
}

fn parse_success_json(output: &Output) -> Value {
    assert!(
        output.status.success(),
        "expected success, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON")
}

#[test]
fn empty_match_executes_three_ticks() {
    let output = run_scenario(empty_match_path());
    let json = parse_success_json(&output);

    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["initialTick"], 0);
    assert_eq!(json["completedTicks"], 3);
    assert_eq!(json["finalTick"], 3);
    assert!(
        json["stateHash"]
            .as_str()
            .is_some_and(|value| value.len() == 64)
    );
    assert!(json["trace"].is_array());
}

#[test]
fn repeated_execution_returns_the_same_hash() {
    let first = parse_success_json(&run_scenario(empty_match_path()));
    let second = parse_success_json(&run_scenario(empty_match_path()));

    assert_eq!(first["stateHash"], second["stateHash"]);
}

#[test]
fn invalid_scenario_fails() {
    let invalid_path = repository_root()
        .join("simulation")
        .join("target")
        .join("invalid-scenario-for-test.json");
    fs::write(
        &invalid_path,
        r#"{
            "schemaVersion": 1,
            "match": {
                "tickRateHz": 0,
                "seed": 123456
            },
            "runTicks": 3,
            "trace": false
        }"#,
    )
    .expect("invalid scenario fixture should be writable");

    let output = run_scenario(invalid_path);

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("invalid tick rate"),
        "stderr should explain validation failure"
    );
}

#[test]
fn successful_stdout_is_valid_json() {
    let output = run_scenario(empty_match_path());

    parse_success_json(&output);
}

#[test]
fn errors_are_not_mixed_into_successful_json_output() {
    let output = run_scenario(empty_match_path());

    assert!(
        output.status.success(),
        "expected success, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    serde_json::from_slice::<Value>(&output.stdout).expect("stdout should contain only JSON");
}
