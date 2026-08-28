use std::{
    fs,
    path::Path,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

fn workrus(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_workrus"))
        .args(args)
        .output()
        .unwrap()
}

fn workrus_in(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_workrus"))
        .args(args)
        .current_dir(dir)
        .env_remove("LINEAR_API_KEY")
        .output()
        .unwrap()
}

fn temp_dir() -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "workrus-cli-{}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn parse_errors_honor_json_mode() {
    for args in [
        vec!["--json", "issue", "id", "bad"],
        vec!["--json", "issue", "mine", "--unknown"],
        vec!["--json", "team", "list", "--limit"],
        vec![
            "--json",
            "issue",
            "query",
            "x",
            "--team",
            "ENG",
            "--all-teams",
        ],
    ] {
        let output = workrus(&args);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        let error: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
        assert_eq!(error["error"]["code"], "invalid_input");
    }
}

#[test]
fn scalar_id_is_offline() {
    let output = workrus(&["issue", "id", "ENG-1"]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"ENG-1\n");
}

#[test]
fn destructive_parser_failures_remain_json_stderr_only() {
    let output = workrus(&["--json", "issue", "delete", "ENG-1"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let value: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(value["error"]["code"], "invalid_input");

    let output = workrus(&[
        "--json",
        "issue",
        "comment",
        "delete",
        "comment-id",
        "--confirm",
        "other",
    ]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
}

#[test]
fn explicit_team_does_not_require_repository_configuration() {
    let directory = temp_dir();

    let output = workrus_in(&directory, &["issue", "mine", "--team", "ENG"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("LINEAR_API_KEY"));
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn completion_scripts_are_plain_text_and_aliases_match() {
    for shell in ["bash", "zsh", "fish", "powershell"] {
        let output = workrus(&["completion", shell]);
        assert!(output.status.success(), "{shell}");
        assert!(output.stderr.is_empty());
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.contains("workrus"));
        assert!(text.ends_with('\n'));
    }
    let canonical = workrus(&["completion", "bash"]);
    let alias = workrus(&["completions", "bash"]);
    assert_eq!(canonical.stdout, alias.stdout);

    let output = workrus(&["--json", "completion", "bash"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());

    let help = workrus(&["--json", "--help"]);
    assert!(help.status.success());
    assert!(help.stderr.is_empty());
    let help = String::from_utf8(help.stdout).unwrap();
    assert!(help.contains("completion|completions <bash|zsh|fish|powershell>"));
    assert!(help.contains("milestone|m"));
    assert!(help.contains("document|docs"));
}

#[test]
fn milestone_alias_parse_errors_remain_json_stderr_only() {
    let output = workrus(&[
        "--json",
        "m",
        "create",
        "--project",
        "p",
        "--target-date",
        "bad",
    ]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let value: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(value["error"]["code"], "invalid_input");
}
