//! Integration tests for CLI JSON output functionality
//!
//! These tests verify that the CLI produces valid, machine-readable JSON output
//! when the --output json flag is used, enabling scripting and automation.

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::error::Error;

/// TDD Step 1: Write the failing test for keygen JSON output
#[test]
fn keygen_command_produces_valid_json_output() -> Result<(), Box<dyn Error>> {
    // ARRANGE: Prepare the command to run the CLI binary
    let mut cmd = Command::cargo_bin("proof-messenger-cli")?;
    cmd.arg("keygen").arg("--output").arg("json");

    // ACT & ASSERT: Run the command and assert that it succeeds
    let output = cmd.assert().success().get_output().stdout.clone();
    let output_str = String::from_utf8(output)?;

    // Assert that the output string is valid JSON
    let json: Value = serde_json::from_str(&output_str)?;

    // Assert that the JSON contains the expected fields
    assert!(json["status"].is_string());
    assert!(json["publicKeyHex"].is_string());
    assert!(json["keypairFile"].is_string());
    
    // Verify specific values
    assert_eq!(json["status"].as_str().unwrap(), "success");
    assert!(predicate::str::contains("keypair.json")
        .eval(&json["keypairFile"].as_str().unwrap()));
    
    // Verify public key is valid hex
    let public_key_hex = json["publicKeyHex"].as_str().unwrap();
    assert_eq!(public_key_hex.len(), 64); // 32 bytes = 64 hex chars
    assert!(public_key_hex.chars().all(|c| c.is_ascii_hexdigit()));

    Ok(())
}

/// Test that invite command produces valid JSON output
#[test]
fn invite_command_produces_valid_json_output() -> Result<(), Box<dyn Error>> {
    // ARRANGE: Prepare the command with JSON output
    let mut cmd = Command::cargo_bin("proof-messenger-cli")?;
    cmd.arg("invite").arg("--seed").arg("42").arg("--output").arg("json");

    // ACT & ASSERT: Run and validate JSON output
    let output = cmd.assert().success().get_output().stdout.clone();
    let output_str = String::from_utf8(output)?;
    let json: Value = serde_json::from_str(&output_str)?;

    // Verify JSON structure
    assert!(json["status"].is_string());
    assert!(json["inviteData"].is_string());
    assert!(json["publicKeyHex"].is_string());
    assert!(json["seed"].is_number());
    
    assert_eq!(json["status"].as_str().unwrap(), "success");
    assert_eq!(json["seed"].as_u64().unwrap(), 42);

    Ok(())
}

/// Test that onboard command produces valid JSON output
#[test]
fn onboard_command_produces_valid_json_output() -> Result<(), Box<dyn Error>> {
    // ARRANGE: Prepare the command with JSON output
    let mut cmd = Command::cargo_bin("proof-messenger-cli")?;
    cmd.arg("onboard").arg("123").arg("--output").arg("json");

    // ACT & ASSERT: Run and validate JSON output
    let output = cmd.assert().success().get_output().stdout.clone();
    let output_str = String::from_utf8(output)?;
    let json: Value = serde_json::from_str(&output_str)?;

    // Verify JSON structure
    assert!(json["status"].is_string());
    assert!(json["proofHex"].is_string());
    assert!(json["publicKeyHex"].is_string());
    assert!(json["inviteSeed"].is_number());
    
    assert_eq!(json["status"].as_str().unwrap(), "success");
    assert_eq!(json["inviteSeed"].as_u64().unwrap(), 123);

    Ok(())
}

/// Test that verify command produces valid JSON output
#[test]
fn verify_command_produces_valid_json_output() -> Result<(), Box<dyn Error>> {
    // ARRANGE: Prepare the command with JSON output
    let mut cmd = Command::cargo_bin("proof-messenger-cli")?;
    cmd.arg("verify")
        .arg("test_proof_hex")
        .arg("42")
        .arg("--output")
        .arg("json");

    // ACT & ASSERT: Run and validate JSON output
    let output = cmd.assert().success().get_output().stdout.clone();
    let output_str = String::from_utf8(output)?;
    let json: Value = serde_json::from_str(&output_str)?;

    // Verify JSON structure
    assert!(json["status"].is_string());
    assert!(json["verified"].is_boolean());
    assert!(json["proof"].is_string());
    assert!(json["inviteSeed"].is_number());
    
    assert_eq!(json["proof"].as_str().unwrap(), "test_proof_hex");
    assert_eq!(json["inviteSeed"].as_u64().unwrap(), 42);

    Ok(())
}

/// Test that send command produces valid JSON output
#[test]
fn send_command_produces_valid_json_output() -> Result<(), Box<dyn Error>> {
    // ARRANGE: Prepare the command with JSON output
    let mut cmd = Command::cargo_bin("proof-messenger-cli")?;
    cmd.arg("send")
        .arg("--to-pubkey")
        .arg("test_pubkey")
        .arg("--msg")
        .arg("Hello World")
        .arg("--output")
        .arg("json");

    // ACT & ASSERT: Run and validate JSON output
    let output = cmd.assert().success().get_output().stdout.clone();
    let output_str = String::from_utf8(output)?;
    let json: Value = serde_json::from_str(&output_str)?;

    // Verify JSON structure
    assert!(json["status"].is_string());
    assert!(json["message"].is_string());
    assert!(json["recipient"].is_string());
    
    assert_eq!(json["status"].as_str().unwrap(), "success");
    assert_eq!(json["message"].as_str().unwrap(), "Hello World");
    assert_eq!(json["recipient"].as_str().unwrap(), "test_pubkey");

    Ok(())
}

/// Test that default output is text (not JSON)
#[test]
fn default_output_is_text_format() -> Result<(), Box<dyn Error>> {
    // ARRANGE: Run command without --output flag
    let mut cmd = Command::cargo_bin("proof-messenger-cli")?;
    cmd.arg("keygen");

    // ACT: Run and capture output
    let output = cmd.assert().success().get_output().stdout.clone();
    let output_str = String::from_utf8(output)?;

    // ASSERT: Output should be human-readable text, not JSON
    assert!(output_str.contains("✅"));
    assert!(output_str.contains("Keypair generated"));
    assert!(output_str.contains("Public Key:"));
    assert!(output_str.contains("Saved to:"));
    
    // Should NOT be valid JSON
    assert!(serde_json::from_str::<Value>(&output_str).is_err());

    Ok(())
}

/// Test that invalid output format produces error
#[test]
fn invalid_output_format_produces_error() -> Result<(), Box<dyn Error>> {
    // ARRANGE: Use invalid output format
    let mut cmd = Command::cargo_bin("proof-messenger-cli")?;
    cmd.arg("keygen").arg("--output").arg("xml");

    // ACT & ASSERT: Should fail with error
    cmd.assert().failure();

    Ok(())
}

/// Test JSON output consistency across multiple runs
#[test]
fn json_output_is_consistent_for_deterministic_commands() -> Result<(), Box<dyn Error>> {
    // ARRANGE: Run the same command twice with same seed
    let mut cmd1 = Command::cargo_bin("proof-messenger-cli")?;
    cmd1.arg("invite").arg("--seed").arg("42").arg("--output").arg("json");
    
    let mut cmd2 = Command::cargo_bin("proof-messenger-cli")?;
    cmd2.arg("invite").arg("--seed").arg("42").arg("--output").arg("json");

    // ACT: Run both commands
    let output1 = cmd1.assert().success().get_output().stdout.clone();
    let output2 = cmd2.assert().success().get_output().stdout.clone();
    
    let json1: Value = serde_json::from_str(&String::from_utf8(output1)?)?;
    let json2: Value = serde_json::from_str(&String::from_utf8(output2)?)?;

    // ASSERT: Should produce identical results for deterministic operations
    assert_eq!(json1["inviteData"], json2["inviteData"]);
    assert_eq!(json1["publicKeyHex"], json2["publicKeyHex"]);
    assert_eq!(json1["seed"], json2["seed"]);

    Ok(())
}

/// Test that JSON output is properly formatted (pretty-printed)
#[test]
fn json_output_is_properly_formatted() -> Result<(), Box<dyn Error>> {
    // ARRANGE: Run command with JSON output
    let mut cmd = Command::cargo_bin("proof-messenger-cli")?;
    cmd.arg("keygen").arg("--output").arg("json");

    // ACT: Run and capture output
    let output = cmd.assert().success().get_output().stdout.clone();
    let output_str = String::from_utf8(output)?;

    // ASSERT: Should be pretty-printed JSON (contains newlines and indentation)
    assert!(output_str.contains('\n'));
    assert!(output_str.contains("  ")); // Indentation
    
    // Should still be valid JSON
    let _json: Value = serde_json::from_str(&output_str)?;

    Ok(())
}