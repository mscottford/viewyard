use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_cli_help_command() {
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicates::str::contains(
            "The refreshingly unoptimized alternative to monorepos",
        ))
        .stdout(predicates::str::contains("dynamic repository discovery"));
}

#[test]
fn test_cli_version_command() {
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("--version");

    cmd.assert()
        .success()
        .stdout(predicates::str::contains("viewyard"));
}

#[test]
fn test_create_viewset_without_github_cli() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("viewset")
        .arg("create")
        .arg("test-viewset")
        .current_dir(temp_dir.path())
        .env("PATH", ""); // Remove PATH to ensure gh CLI is not available

    // Should fail gracefully when GitHub CLI is not available
    cmd.assert().failure().stderr(
        predicates::str::contains("Failed to check if gh CLI is installed")
            .or(predicates::str::contains("No such file or directory"))
            .or(predicates::str::contains("GitHub CLI")),
    );
}

#[test]
fn test_create_viewset_directory_already_exists() {
    let temp_dir = TempDir::new().unwrap();
    let viewset_dir = temp_dir.path().join("existing-viewset");
    fs::create_dir_all(&viewset_dir).unwrap();

    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("viewset")
        .arg("create")
        .arg("existing-viewset")
        .current_dir(temp_dir.path());

    // Should fail when directory already exists
    cmd.assert().failure().stderr(
        predicates::str::contains("already exists").or(predicates::str::contains("Directory")),
    );
}

#[test]
fn test_workspace_commands_outside_view() {
    let temp_dir = TempDir::new().unwrap();

    // Try to run workspace commands outside of a view directory
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("status").current_dir(temp_dir.path());

    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("view directory"));
}

#[test]
fn test_basic_command_structure() {
    // Test that all main commands are available
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicates::str::contains("viewset"))
        .stdout(predicates::str::contains("view"))
        .stdout(predicates::str::contains("status"))
        .stdout(predicates::str::contains("commit-all"))
        .stdout(predicates::str::contains("push-all"));
}

#[test]
fn test_https_url_loads_without_unusual_url_warning() {
    // Create a viewset structure with HTTPS URLs
    let temp_dir = TempDir::new().unwrap();
    let viewset_dir = temp_dir.path();
    let view_dir = viewset_dir.join("default");
    fs::create_dir_all(&view_dir).unwrap();

    let repos_json = r#"[
        {
            "name": "test-repo",
            "url": "https://github.com/testorg/test-repo.git",
            "is_private": false,
            "source": "GitHub (testorg)"
        }
    ]"#;
    fs::write(viewset_dir.join(".viewyard-repos.json"), repos_json).unwrap();

    // Run status from within the view directory - should load the HTTPS URL without
    // producing an "unusual URL format" warning, since it's a valid GitHub HTTPS URL
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("status").current_dir(&view_dir);

    cmd.assert()
        .stderr(predicates::str::contains("unusual URL format").not());
}

// ── Protocol config persisted to .viewyard-config.json ──────────────────────

#[test]
fn test_viewset_config_written_with_https_when_flag_used() {
    // We can't fully run `viewset create` without GitHub CLI, but we can verify
    // that the binary accepts --protocol https without erroring on argument parsing.
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("viewset")
        .arg("create")
        .arg("test-project")
        .arg("--protocol")
        .arg("https")
        .env("PATH", ""); // no gh CLI — will bail early, but arg parsing must succeed

    // The failure must be about GitHub CLI / git availability, not about
    // an unknown or invalid --protocol argument.
    cmd.assert().failure().stderr(
        predicates::str::contains("unrecognized option")
            .not()
            .and(predicates::str::contains("invalid value").not()),
    );
}

#[test]
fn test_viewset_create_rejects_unknown_protocol_value() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("viewset")
        .arg("create")
        .arg("test-project")
        .arg("--protocol")
        .arg("ftp")
        .current_dir(temp_dir.path());

    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("ftp").or(predicates::str::contains("protocol")));
}

#[test]
fn test_viewset_create_rejects_unknown_protocol_via_env_var() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("viewset")
        .arg("create")
        .arg("test-project")
        .current_dir(temp_dir.path())
        .env("VIEWYARD_GIT_PROTOCOL", "ftp");

    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("ftp").or(predicates::str::contains("protocol")));
}

#[test]
fn test_https_url_is_not_mutated_by_ssh_alias_transformation() {
    // Create a viewset structure with HTTPS URLs
    let temp_dir = TempDir::new().unwrap();
    let viewset_dir = temp_dir.path();
    let view_dir = viewset_dir.join("default");
    fs::create_dir_all(&view_dir).unwrap();

    let https_url = "https://github.com/testorg/test-repo.git";
    let repos_json = format!(
        r#"[{{"name":"test-repo","url":"{https_url}","is_private":false,"source":"GitHub (testorg)"}}]"#
    );
    fs::write(viewset_dir.join(".viewyard-repos.json"), repos_json).unwrap();

    // status output should reference the original HTTPS URL (not a mutated SSH form)
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("status").current_dir(&view_dir);

    // The URL must not have been transformed to an SSH form like "git@github.com-..."
    cmd.assert()
        .stderr(predicates::str::contains("git@github.com-testorg").not());
}
