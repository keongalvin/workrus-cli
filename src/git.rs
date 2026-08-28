use crate::{
    error::AppError,
    model::{IssueIdentifier, issue_identifier_in_branch},
};
use std::{path::Path, process::Command};

pub(crate) fn command(dir: &Path, args: &[&str]) -> Command {
    let mut command = Command::new("git");
    command.args(args).current_dir(dir).env_clear();
    for (name, value) in std::env::vars_os() {
        let key = name.to_string_lossy();
        if key == "LINEAR_API_KEY"
            || matches!(
                key.as_ref(),
                "GIT_DIR"
                    | "GIT_WORK_TREE"
                    | "GIT_COMMON_DIR"
                    | "GIT_EXTERNAL_DIFF"
                    | "GIT_PAGER"
                    | "GIT_EDITOR"
                    | "GIT_ASKPASS"
                    | "SSH_ASKPASS"
                    | "GIT_SSH"
                    | "GIT_SSH_COMMAND"
                    | "GIT_CEILING_DIRECTORIES"
            )
            || key.starts_with("GIT_CONFIG_")
            || key.starts_with("GIT_TRACE")
        {
            continue;
        }
        command.env(name, value);
    }
    // Keep explicit removals visible to Command inspection and robust against a
    // future change from env_clear to selective inheritance.
    command.env_remove("LINEAR_API_KEY");
    command
}

#[cfg(windows)]
const DISABLED_HOOKS_PATH: &str = "NUL";
#[cfg(not(windows))]
const DISABLED_HOOKS_PATH: &str = "/dev/null";

fn git(dir: &Path, args: &[&str]) -> Result<std::process::Output, AppError> {
    command(dir, args)
        .output()
        .map_err(|e| AppError::operational(format!("could not run git: {e}")))
}
pub fn current_issue(dir: &Path) -> Result<IssueIdentifier, AppError> {
    let out = git(dir, &["branch", "--show-current"])?;
    if !out.status.success() {
        return Err(AppError::input(
            "could not determine the current Git branch",
        ));
    }
    let b = String::from_utf8(out.stdout)
        .map_err(|_| AppError::operational("git returned a non-Unicode branch name"))?;
    issue_identifier_in_branch(b.trim()).ok_or_else(|| {
        AppError::input(
            "could not infer an issue identifier from the current Git branch; pass an ID",
        )
    })
}
pub fn prepare_start(dir: &Path, branch: &str) -> Result<&'static str, AppError> {
    let head = git(dir, &["rev-parse", "--verify", "HEAD"])?;
    if !head.status.success() {
        return Err(AppError::input(
            "issue start requires a Git worktree with a resolvable HEAD",
        ));
    }
    let valid = git(dir, &["check-ref-format", "--branch", branch])?;
    if !valid.status.success() || String::from_utf8_lossy(&valid.stdout).trim_end() != branch {
        return Err(AppError::input(
            "Linear returned an invalid Git branch name",
        ));
    }
    let current = git(dir, &["branch", "--show-current"])?;
    if String::from_utf8_lossy(&current.stdout).trim() == branch {
        return Ok("already_current");
    }
    let exists = git(
        dir,
        &[
            "show-ref",
            "--verify",
            "--quiet",
            &format!("refs/heads/{branch}"),
        ],
    )?;
    let branch_exists = match exists.status.code() {
        Some(0) => true,
        Some(1) => false,
        _ => {
            return Err(AppError::operational(
                "Git could not check whether the branch exists",
            ));
        }
    };
    let args: Vec<&str> = if branch_exists {
        vec!["switch", "--", branch]
    } else {
        vec!["switch", "--create", branch]
    };
    let hooks_config = format!("core.hooksPath={DISABLED_HOOKS_PATH}");
    let mut switch_args = vec!["-c", hooks_config.as_str()];
    switch_args.extend(args);
    let out = git(dir, &switch_args)?;
    if !out.status.success() {
        return Err(AppError::operational(format!(
            "Git could not switch branch: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(if branch_exists { "switched" } else { "created" })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        ffi::OsStr,
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

    fn temp_repo() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "workrus-git-{}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        run_git(&path, &["init", "--quiet"]);
        run_git(&path, &["config", "user.name", "Workrus Test"]);
        run_git(&path, &["config", "user.email", "workrus@example.invalid"]);
        fs::write(path.join("README.md"), "test\n").unwrap();
        run_git(&path, &["add", "README.md"]);
        run_git(&path, &["commit", "--quiet", "-m", "initial"]);
        path
    }

    fn run_git(dir: &Path, args: &[&str]) {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(dir)
                .status()
                .unwrap()
                .success()
        );
    }

    #[test]
    fn git_subprocesses_remove_linear_api_key() {
        let command = command(Path::new("."), &["status"]);

        assert!(
            command
                .get_envs()
                .all(|(name, value)| name != OsStr::new("LINEAR_API_KEY") || value.is_none())
        );
    }

    #[cfg(unix)]
    #[test]
    fn checkout_hook_cannot_read_linear_api_key() {
        const CHILD_MARKER: &str = "WORKRUS_GIT_HOOK_TEST_CHILD";
        if std::env::var_os(CHILD_MARKER).is_some() {
            use std::os::unix::fs::PermissionsExt;

            let repo = temp_repo();
            let hook = repo.join(".git/hooks/post-checkout");
            fs::write(
                &hook,
                "#!/bin/sh\necho executed > repository-hook-ran\nif [ -n \"$LINEAR_API_KEY\" ]; then echo leaked > api-key-leak; fi\n",
            )
            .unwrap();
            let mut permissions = fs::metadata(&hook).unwrap().permissions();
            permissions.set_mode(0o700);
            fs::set_permissions(&hook, permissions).unwrap();

            assert_eq!(
                prepare_start(&repo, "alice/eng-123-hook-test").unwrap(),
                "created"
            );
            assert!(!repo.join("repository-hook-ran").exists());
            assert!(!repo.join("api-key-leak").exists());
            fs::remove_dir_all(repo).unwrap();
            return;
        }

        let output = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "git::tests::checkout_hook_cannot_read_linear_api_key",
                "--nocapture",
            ])
            .env(CHILD_MARKER, "1")
            .env("LINEAR_API_KEY", "lin_api_hook_secret")
            .output()
            .unwrap();

        assert!(
            output.status.success(),
            "child hook test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!String::from_utf8_lossy(&output.stdout).contains("lin_api_hook_secret"));
        assert!(!String::from_utf8_lossy(&output.stderr).contains("lin_api_hook_secret"));
    }

    #[test]
    fn current_issue_ignores_identifier_shaped_title_suffix() {
        let repo = temp_repo();
        run_git(
            &repo,
            &["switch", "--quiet", "--create", "alice/eng-123-oauth-2"],
        );

        let issue = current_issue(&repo).unwrap();

        assert_eq!(issue.as_str(), "ENG-123");
        fs::remove_dir_all(repo).unwrap();
    }

    #[test]
    fn prepare_start_creates_then_recognizes_branch() {
        let repo = temp_repo();

        assert_eq!(
            prepare_start(&repo, "alice/eng-123-example").unwrap(),
            "created"
        );
        assert_eq!(
            prepare_start(&repo, "alice/eng-123-example").unwrap(),
            "already_current"
        );
        fs::remove_dir_all(repo).unwrap();
    }
}
