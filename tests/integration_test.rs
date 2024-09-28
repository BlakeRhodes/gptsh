use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_help() {
    let mut cmd = Command::cargo_bin("gptsh").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: gptsh [OPTIONS] [PROMPT]"));
}

#[test]
fn test_no_prompt() {
    let mut cmd = Command::cargo_bin("gptsh").unwrap();
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error: No prompt provided."));
}
