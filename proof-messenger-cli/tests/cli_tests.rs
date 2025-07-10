//! Integration tests for the CLI application

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("proof-messenger-cli").unwrap();
    cmd.arg("--help");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Proof Messenger CLI"))
        .stdout(predicate::str::contains("invite"))
        .stdout(predicate::str::contains("onboard"))
        .stdout(predicate::str::contains("send"))
        .stdout(predicate::str::contains("receive"));
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("proof-messenger-cli").unwrap();
    cmd.arg("--version");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("proof-messenger-cli"));
}

#[test]
fn test_invite_generation() {
    let temp_dir = TempDir::new().unwrap();
    
    let mut cmd = Command::cargo_bin("proof-messenger-cli").unwrap();
    cmd.arg("--config")
        .arg(temp_dir.path().join("config.toml"))
        .arg("invite")
        .arg("--generate");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Invitation generated successfully"))
        .stdout(predicate::str::contains("Invitation Code:"));
}

#[test]
fn test_invite_list_empty() {
    let temp_dir = TempDir::new().unwrap();
    
    let mut cmd = Command::cargo_bin("proof-messenger-cli").unwrap();
    cmd.arg("--config")
        .arg(temp_dir.path().join("config.toml"))
        .arg("invite")
        .arg("--list");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No invitations found"));
}

#[test]
fn test_identity_generation() {
    let temp_dir = TempDir::new().unwrap();
    
    let mut cmd = Command::cargo_bin("proof-messenger-cli").unwrap();
    cmd.arg("--config")
        .arg(temp_dir.path().join("config.toml"))
        .arg("identity")
        .arg("--generate");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("New identity generated"))
        .stdout(predicate::str::contains("Public key:"));
}

#[test]
fn test_identity_show_none() {
    let temp_dir = TempDir::new().unwrap();
    
    let mut cmd = Command::cargo_bin("proof-messenger-cli").unwrap();
    cmd.arg("--config")
        .arg(temp_dir.path().join("config.toml"))
        .arg("identity")
        .arg("--show");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No identity found"));
}

#[test]
fn test_demo_e2e() {
    let temp_dir = TempDir::new().unwrap();
    
    let mut cmd = Command::cargo_bin("proof-messenger-cli").unwrap();
    cmd.arg("--config")
        .arg(temp_dir.path().join("config.toml"))
        .arg("demo")
        .arg("--e2e");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Running End-to-End Demo"))
        .stdout(predicate::str::contains("demo completed successfully"));
}

#[test]
fn test_demo_benchmark() {
    let temp_dir = TempDir::new().unwrap();
    
    let mut cmd = Command::cargo_bin("proof-messenger-cli").unwrap();
    cmd.arg("--config")
        .arg(temp_dir.path().join("config.toml"))
        .arg("demo")
        .arg("--benchmark");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Running Cryptographic Benchmarks"))
        .stdout(predicate::str::contains("Benchmark completed"));
}

#[test]
fn test_invalid_onboard_code() {
    let temp_dir = TempDir::new().unwrap();
    
    let mut cmd = Command::cargo_bin("proof-messenger-cli").unwrap();
    cmd.arg("--config")
        .arg(temp_dir.path().join("config.toml"))
        .arg("onboard")
        .arg("INVALID");
    
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Invalid invitation code format"));
}

#[test]
fn test_send_without_identity() {
    let temp_dir = TempDir::new().unwrap();
    
    let mut cmd = Command::cargo_bin("proof-messenger-cli").unwrap();
    cmd.arg("--config")
        .arg(temp_dir.path().join("config.toml"))
        .arg("send")
        .arg("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef")
        .arg("Hello");
    
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No identity found"));
}

#[test]
fn test_receive_empty() {
    let temp_dir = TempDir::new().unwrap();
    
    let mut cmd = Command::cargo_bin("proof-messenger-cli").unwrap();
    cmd.arg("--config")
        .arg(temp_dir.path().join("config.toml"))
        .arg("receive");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No messages found"));
}