use crate::{
    cli::{
        Command, CommentCommand, ContentSource, DocumentCommand, DocumentTarget, IssueRef, Limit,
        MilestoneCommand, Mutation, Open, ProjectCommand, ProjectCreate, Scalar,
    },
    config,
    error::AppError,
    git,
    linear::transport::LinearClient,
    model::{Document, Issue, IssueIdentifier, Milestone, Project, State, Team, User},
    output,
};
use serde_json::{Map, Value, json};
use std::{
    env,
    io::{self, IsTerminal, Read, Write},
    process::Command as ProcessCommand,
};
pub fn run(command: Command, json_output: bool) -> Result<String, AppError> {
    match command {
        Command::Config(team) => config_cmd(team, json_output),
        Command::Completion(shell) => Ok(crate::cli::completion_script(shell)),
        Command::TeamList { page, open } => {
            let r = teams(&client()?, &page)?;
            if open != Open::None {
                let workspace = config::runtime_config()?.workspace.ok_or_else(|| AppError::input("workspace is required to open team URLs; set LINEAR_WORKSPACE or workspace in global config"))?;
                for item in &r.items {
                    open_url(
                        &format!("https://linear.app/{workspace}/team/{}/all", item.key),
                        open,
                    )?;
                }
            }
            Ok(output::team_collection(&r.items, &r.page_info, json_output))
        }
        Command::TeamId(key) => {
            let key = team(key.as_deref())?;
            Ok(output::scalar(
                "id",
                &find_team(&client()?, &key)?.id,
                json_output,
            ))
        }
        Command::TeamMembers {
            team: selected,
            include_disabled,
            page,
        } => {
            let key = team(selected.as_deref())?;
            let found = find_team(&client()?, &key)?;
            let r = users_pages(&page, |limit, after| {
                client()?.team_members(&found.id, include_disabled, limit, after)
            })?;
            Ok(output::user_collection(&r.items, &r.page_info, json_output))
        }
        Command::TeamCreate(input) => {
            let mut value = Map::new();
            value.insert("name".into(), json!(input.name));
            if let Some(key) = input.key {
                value.insert("key".into(), json!(key));
            }
            if let Some(description) = input.description {
                value.insert("description".into(), json!(description));
            }
            if let Some(color) = input.color {
                value.insert("color".into(), json!(color));
            }
            if let Some(icon) = input.icon {
                value.insert("icon".into(), json!(icon));
            }
            let found = client()?.team_create(Value::Object(value))?;
            Ok(output::team(&found, json_output))
        }
        Command::TeamAutolinks => Err(AppError::input(
            "team autolinks is unavailable: Linear's public API has no autolink operation",
        )),
        Command::UserList {
            include_disabled,
            page,
        } => {
            let r = users_pages(&page, |limit, after| {
                client()?.users(include_disabled, limit, after)
            })?;
            Ok(output::user_collection(&r.items, &r.page_info, json_output))
        }
        Command::Mine(mut list) => {
            let c = client()?;
            let selected = team(list.team.as_deref())?;
            resolve_issue_project_filters(&c, &mut list)?;
            let r = issues(&c, None, Some(&selected), &list)?;
            if list.open != Open::None {
                for issue in &r.items {
                    open_url(&issue.url, list.open)?;
                }
            }
            Ok(output::issue_collection(
                &r.items,
                &r.page_info,
                json_output,
            ))
        }
        Command::Query {
            text,
            mut list,
            all_teams,
        } => {
            let selected = if all_teams {
                None
            } else {
                Some(team(list.team.as_deref())?)
            };
            let c = client()?;
            resolve_issue_project_filters(&c, &mut list)?;
            let r = issues(&c, Some(&text), selected.as_deref(), &list)?;
            Ok(output::issue_collection(
                &r.items,
                &r.page_info,
                json_output,
            ))
        }
        Command::Project(command) => project_command(command, json_output),
        Command::Milestone(command) => milestone_command(command, json_output),
        Command::Document(command) => document_command(command, json_output),
        Command::View { issue, open } => {
            let id = issue_id(issue)?;
            let found = client()?.issue(id.as_str())?;
            if open != Open::None {
                open_url(&found.url, open)?;
            }
            Ok(output::issue(&found, json_output))
        }
        Command::Pr {
            issue,
            base,
            head,
            draft,
            title,
            web,
        } => {
            let id = issue_id(issue)?;
            let found = client()?.issue(id.as_str())?;
            pr(
                &found,
                base.as_deref(),
                head.as_deref(),
                draft,
                title.as_deref(),
                web,
                json_output,
            )
        }
        Command::Scalar { field, issue } => {
            let id = issue_id(issue)?;
            match field {
                Scalar::Id => Ok(output::scalar("id", id.as_str(), json_output)),
                Scalar::Title => Ok(output::scalar(
                    "title",
                    &client()?.issue(id.as_str())?.title,
                    json_output,
                )),
                Scalar::Url => Ok(output::scalar(
                    "url",
                    &client()?.issue(id.as_str())?.url,
                    json_output,
                )),
            }
        }
        Command::Create(mut m) => {
            prompt_create(&mut m, json_output)?;
            let c = client()?;
            let issue = c.create(mutation_input(&c, &m, None)?)?;
            Ok(output::issue(&issue, json_output))
        }
        Command::Update { issue, mutation } => {
            let id = issue_id(issue)?;
            let c = client()?;
            let issue = c.issue(id.as_str())?;
            let updated = c.update(id.as_str(), mutation_input(&c, &mutation, Some(&issue))?)?;
            Ok(output::issue(&updated, json_output))
        }
        Command::Delete {
            issue,
            confirm,
            dry_run,
        } => {
            let id = issue_id(issue)?;
            if confirm != id.as_str() {
                return Err(AppError::input(
                    "--confirm must exactly match the canonical issue identifier",
                ));
            }
            let found = client()?.issue(id.as_str())?;
            if dry_run {
                Ok(output::dry_run("archive", &found.identifier, json_output))
            } else {
                let entity = client()?.archive(&found.id)?;
                Ok(output::archived(&entity, json_output))
            }
        }
        Command::Comment(command) => comments(command, json_output),
        Command::Start(id) => start(id, json_output),
    }
}
fn client() -> Result<LinearClient, AppError> {
    Ok(LinearClient::new(config::api_key_from_env()?))
}
fn team(flag: Option<&str>) -> Result<String, AppError> {
    config::resolve_team(flag, config::default_team()?)
}
fn issue_id(id: Option<IssueRef>) -> Result<IssueIdentifier, AppError> {
    match id {
        Some(IssueRef::Identifier(id)) => Ok(id),
        Some(IssueRef::Number(number)) => Ok(format!("{}-{number}", team(None)?)
            .parse()
            .expect("validated team and number")),
        None => git::current_issue(
            &env::current_dir().map_err(|e| AppError::operational(e.to_string()))?,
        ),
    }
}
fn config_cmd(set: Option<String>, json_output: bool) -> Result<String, AppError> {
    if let Some(key) = set {
        let found = find_team(&client()?, &key)?;
        config::write_default_team(&found.key)?;
        Ok(output::config(&found, json_output))
    } else {
        let key = config::default_team()?.ok_or_else(|| {
            AppError::input(
                "no default team configured; run `workrus config TEAM` or set LINEAR_TEAM_ID",
            )
        })?;
        Ok(output::scalar("team", &key, json_output))
    }
}
fn teams(
    c: &LinearClient,
    page: &crate::cli::Page,
) -> Result<crate::linear::Collection<Team>, AppError> {
    let mut after = page.after.clone();
    let mut items = Vec::new();
    let mut guard = crate::linear::transport::PaginationGuard::default();
    loop {
        let request = c.teams(100, after.as_deref())?;
        let info = request.page_info.clone();
        let prior_len = items.len();
        items.extend(request.items);
        match page.limit {
            Limit::Bounded(limit) if items.len() >= limit as usize => {
                return Ok(crate::linear::Collection {
                    items: items.into_iter().take(limit as usize).collect(),
                    item_cursors: Vec::new(),
                    page_info: truncated_page_info(
                        info,
                        &request.item_cursors,
                        prior_len,
                        limit as usize,
                    ),
                });
            }
            Limit::Bounded(_) | Limit::Unlimited if !info.has_next_page => {
                return Ok(crate::linear::Collection {
                    items,
                    item_cursors: Vec::new(),
                    page_info: info,
                });
            }
            _ => after = guard.next_cursor(info.has_next_page, info.end_cursor.as_deref())?,
        }
    }
}
fn users_pages<F>(
    page: &crate::cli::Page,
    mut fetch: F,
) -> Result<crate::linear::Collection<crate::model::User>, AppError>
where
    F: FnMut(u8, Option<&str>) -> Result<crate::linear::Collection<crate::model::User>, AppError>,
{
    let mut after = page.after.clone();
    let mut items = Vec::new();
    let mut guard = crate::linear::transport::PaginationGuard::default();
    loop {
        let request = fetch(100, after.as_deref())?;
        let info = request.page_info.clone();
        let prior_len = items.len();
        items.extend(request.items);
        match page.limit {
            Limit::Bounded(limit) if items.len() >= limit as usize => {
                return Ok(crate::linear::Collection {
                    items: items.into_iter().take(limit as usize).collect(),
                    item_cursors: Vec::new(),
                    page_info: truncated_page_info(
                        info,
                        &request.item_cursors,
                        prior_len,
                        limit as usize,
                    ),
                });
            }
            Limit::Bounded(_) | Limit::Unlimited if !info.has_next_page => {
                return Ok(crate::linear::Collection {
                    items,
                    item_cursors: Vec::new(),
                    page_info: info,
                });
            }
            _ => after = guard.next_cursor(info.has_next_page, info.end_cursor.as_deref())?,
        }
    }
}
fn issues(
    c: &LinearClient,
    text: Option<&str>,
    selected_team: Option<&str>,
    list: &crate::cli::List,
) -> Result<crate::linear::Collection<Issue>, AppError> {
    let mut after = list.page.after.clone();
    let configured_sort = config::runtime_config()?.issue_sort.map(|sort| match sort {
        config::IssueSort::Manual => "manual",
        config::IssueSort::Priority => "priority",
    });
    let mut items = Vec::new();
    let mut guard = crate::linear::transport::PaginationGuard::default();
    loop {
        let request = c.issues_filtered(
            text,
            selected_team,
            list.state.as_deref(),
            list.sort.as_deref().or(configured_sort),
            list.project.as_deref(),
            list.milestone.as_deref(),
            100,
            after.as_deref(),
        )?;
        let mut page_info = request.page_info.clone();
        let prior_len = items.len();
        items.extend(request.items);
        match list.page.limit {
            Limit::Bounded(limit) if items.len() >= limit as usize => {
                items.truncate(limit as usize);
                page_info = truncated_page_info(
                    page_info,
                    &request.item_cursors,
                    prior_len,
                    limit as usize,
                );
                return Ok(crate::linear::Collection {
                    items,
                    item_cursors: Vec::new(),
                    page_info,
                });
            }
            Limit::Bounded(_) if !page_info.has_next_page => {
                return Ok(crate::linear::Collection {
                    items,
                    item_cursors: Vec::new(),
                    page_info,
                });
            }
            Limit::Unlimited if !page_info.has_next_page => {
                return Ok(crate::linear::Collection {
                    items,
                    item_cursors: Vec::new(),
                    page_info,
                });
            }
            _ => {
                after =
                    guard.next_cursor(page_info.has_next_page, page_info.end_cursor.as_deref())?
            }
        }
    }
}

fn truncated_page_info(
    mut page_info: crate::model::PageInfo,
    item_cursors: &[String],
    prior_len: usize,
    limit: usize,
) -> crate::model::PageInfo {
    let retained = limit.saturating_sub(prior_len);
    if retained < item_cursors.len() {
        page_info.end_cursor = item_cursors.get(retained.saturating_sub(1)).cloned();
        page_info.has_next_page = true;
    }
    page_info
}

fn scrub(command: &mut ProcessCommand) {
    // Reconstruct a minimal inherited environment rather than trying to enumerate
    // every Git routing/trace variable. PATH is retained so normal executable lookup works.
    command.env_clear();
    for (key, value) in env::vars_os() {
        let key_text = key.to_string_lossy();
        if key_text == "LINEAR_API_KEY"
            || key_text == "GIT_DIR"
            || key_text == "GIT_WORK_TREE"
            || key_text == "GIT_COMMON_DIR"
            || key_text == "GIT_EXTERNAL_DIFF"
            || key_text == "GIT_PAGER"
            || key_text == "GIT_EDITOR"
            || key_text == "GIT_ASKPASS"
            || key_text == "SSH_ASKPASS"
            || key_text == "GIT_SSH"
            || key_text == "GIT_SSH_COMMAND"
            || key_text == "GIT_CEILING_DIRECTORIES"
            || key_text.starts_with("GIT_CONFIG_")
            || key_text.starts_with("GIT_TRACE")
        {
            continue;
        }
        command.env(key, value);
    }
}
fn https_url(url: &str) -> Result<&str, AppError> {
    let host = url.strip_prefix("https://").and_then(|rest| {
        rest.split(['/', '?', '#'])
            .next()
            .filter(|host| !host.is_empty() && *host != ".")
    });
    if host.is_none_or(|host| host.starts_with(':') || host.contains('@'))
        || url.chars().any(|c| c.is_control() || c.is_whitespace())
    {
        return Err(AppError::operational("received an invalid non-HTTPS URL"));
    }
    Ok(url)
}
fn open_url(url: &str, open: Open) -> Result<(), AppError> {
    let url = https_url(url)?;
    let mut command = if matches!(open, Open::App) {
        if !cfg!(target_os = "macos") {
            return Err(AppError::input("--app is supported only on macOS"));
        }
        let mut c = ProcessCommand::new("open");
        c.args(["-a", "Linear", url]);
        c
    } else if cfg!(target_os = "macos") {
        let mut c = ProcessCommand::new("open");
        c.arg(url);
        c
    } else if cfg!(windows) {
        let mut c = ProcessCommand::new("explorer.exe");
        c.arg(url);
        c
    } else {
        let mut c = ProcessCommand::new("xdg-open");
        c.arg(url);
        c
    };
    scrub(&mut command);
    command
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|e| AppError::operational(format!("could not open URL: {e}")))
        .and_then(|s| {
            if s.success() {
                Ok(())
            } else {
                Err(AppError::operational("URL launcher failed"))
            }
        })
}
fn pr(
    issue: &Issue,
    base: Option<&str>,
    head: Option<&str>,
    draft: bool,
    title: Option<&str>,
    web: bool,
    json_output: bool,
) -> Result<String, AppError> {
    let mut command = ProcessCommand::new("gh");
    command.args([
        "pr",
        "create",
        "--title",
        title.unwrap_or(&format!("{} {}", issue.identifier, issue.title)),
        "--body",
        &issue.url,
    ]);
    if let Some(base) = base {
        command.args(["--base", base]);
    }
    if let Some(head) = head {
        command.args(["--head", head]);
    }
    if draft {
        command.arg("--draft");
    }
    // Always obtain the URL from gh first. `gh pr create --web` opens its own
    // browser and does not promise to print a URL, which would break our output contract.
    scrub(&mut command);
    let output = command
        .output()
        .map_err(|e| AppError::operational(format!("could not run gh: {e}")))?;
    if !output.status.success() {
        return Err(AppError::operational("gh pr create failed"));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let url = stdout
        .lines()
        .map(str::trim)
        .find_map(|line| https_url(line).ok())
        .ok_or_else(|| AppError::operational("gh did not return an HTTPS pull request URL"))?;
    if web {
        open_url(url, Open::Web)?;
    }
    output::pull_request(url, &issue.id, &issue.identifier, json_output)
}

fn read_content_file(path: &str) -> Result<String, AppError> {
    const MAX: usize = 1024 * 1024;
    #[cfg(unix)]
    let file = {
        use std::os::unix::fs::OpenOptionsExt;
        std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW)
            .open(path)
            .map_err(|e| AppError::input(format!("could not safely open content file: {e}")))?
    };
    #[cfg(windows)]
    let file = {
        use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
        const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;

        let file = std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(path)
            .map_err(|e| AppError::input(format!("could not safely open content file: {e}")))?;
        if file
            .metadata()
            .map_err(|e| AppError::operational(format!("could not inspect content file: {e}")))?
            .file_attributes()
            & FILE_ATTRIBUTE_REPARSE_POINT
            != 0
        {
            return Err(AppError::input("content file must not be a reparse point"));
        }
        file
    };
    #[cfg(not(any(unix, windows)))]
    let file = {
        let metadata = std::fs::symlink_metadata(path)
            .map_err(|e| AppError::operational(format!("could not read content file: {e}")))?;
        if metadata.file_type().is_symlink() {
            return Err(AppError::input("content file must not be a symlink"));
        }
        std::fs::File::open(path)
            .map_err(|e| AppError::operational(format!("could not read content file: {e}")))?
    };
    let metadata = file
        .metadata()
        .map_err(|e| AppError::operational(format!("could not inspect content file: {e}")))?;
    if !metadata.is_file() || metadata.len() > MAX as u64 {
        return Err(AppError::input(
            "content file must be a regular UTF-8 file no larger than 1 MiB",
        ));
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take((MAX + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| AppError::operational(format!("could not read content file: {e}")))?;
    if bytes.len() > MAX {
        return Err(AppError::input(
            "content file must be a regular UTF-8 file no larger than 1 MiB",
        ));
    }
    let content = String::from_utf8(bytes)
        .map_err(|_| AppError::input("content file must be valid UTF-8"))?;
    if content
        .chars()
        .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
    {
        return Err(AppError::input(
            "content file contains unsupported control characters",
        ));
    }
    Ok(content)
}
fn projects_pages(
    c: &LinearClient,
    team_id: Option<&str>,
    page: &crate::cli::Page,
) -> Result<crate::linear::Collection<Project>, AppError> {
    let mut after = page.after.clone();
    let mut items = Vec::new();
    let mut guard = crate::linear::transport::PaginationGuard::default();
    loop {
        // The nested team/lead/member selection exceeds Linear's query-complexity
        // budget at 100 projects per page. Fifty is accepted by the live schema.
        let request = c.projects(team_id, 50, after.as_deref())?;
        let info = request.page_info.clone();
        let prior = items.len();
        items.extend(request.items);
        match page.limit {
            Limit::Bounded(n) if items.len() >= n as usize => {
                return Ok(crate::linear::Collection {
                    items: items.into_iter().take(n as usize).collect(),
                    item_cursors: Vec::new(),
                    page_info: truncated_page_info(info, &request.item_cursors, prior, n as usize),
                });
            }
            Limit::Bounded(_) | Limit::Unlimited if !info.has_next_page => {
                return Ok(crate::linear::Collection {
                    items,
                    item_cursors: Vec::new(),
                    page_info: info,
                });
            }
            _ => after = guard.next_cursor(info.has_next_page, info.end_cursor.as_deref())?,
        }
    }
}
fn all_projects(c: &LinearClient) -> Result<Vec<Project>, AppError> {
    projects_pages(
        c,
        None,
        &crate::cli::Page {
            limit: Limit::Unlimited,
            after: None,
        },
    )
    .map(|r| r.items)
}
fn resolve_project(c: &LinearClient, value: &str) -> Result<Project, AppError> {
    let matches: Vec<_> = all_projects(c)?
        .into_iter()
        .filter(|p| p.id == value || p.slug_id.as_deref() == Some(value) || p.name == value)
        .collect();
    match matches.len() {
        1 => Ok(matches.into_iter().next().expect("one")),
        0 => Err(AppError::input(format!(
            "project {value} is not accessible; use its Linear ID, exact name, or slug"
        ))),
        _ => Err(AppError::input(format!(
            "project {value} is ambiguous; use one of: {}",
            matches
                .iter()
                .map(|p| format!("{} ({})", p.name, p.id))
                .collect::<Vec<_>>()
                .join(", ")
        ))),
    }
}
fn resolve_user(c: &LinearClient, value: &str) -> Result<User, AppError> {
    let page = crate::cli::Page {
        limit: Limit::Unlimited,
        after: None,
    };
    let matches: Vec<_> = users_pages(&page, |limit, after| c.users(true, limit, after))?
        .items
        .into_iter()
        .filter(|u| {
            u.id == value
                || u.name.as_deref() == Some(value)
                || u.display_name.as_deref() == Some(value)
        })
        .collect();
    match matches.len() {
        1 => Ok(matches.into_iter().next().expect("one")),
        0 => Err(AppError::input(format!(
            "user {value} is not accessible; use an exact ID, name, or display name"
        ))),
        _ => Err(AppError::input(format!(
            "user {value} is ambiguous; use one of: {}",
            matches
                .iter()
                .map(|u| format!(
                    "{} ({})",
                    u.display_name
                        .as_deref()
                        .or(u.name.as_deref())
                        .unwrap_or("unnamed"),
                    u.id
                ))
                .collect::<Vec<_>>()
                .join(", ")
        ))),
    }
}
fn resolve_issue_project_filters(
    c: &LinearClient,
    list: &mut crate::cli::List,
) -> Result<(), AppError> {
    if let Some(p) = list.project.take() {
        list.project = Some(resolve_project(c, &p)?.id);
    }
    if let Some(m) = list.milestone.take() {
        list.milestone = Some(resolve_milestone(c, &m, list.project.as_deref())?.id);
    }
    Ok(())
}
fn project_command(command: ProjectCommand, json_output: bool) -> Result<String, AppError> {
    match command {
        ProjectCommand::List { team, page, open } => {
            let c = client()?;
            let team_id = match team {
                Some(value) => Some(find_team(&c, &value)?.id),
                None => None,
            };
            let result = projects_pages(&c, team_id.as_deref(), &page)?;
            if open != Open::None {
                for project in &result.items {
                    if let Some(url) = &project.url {
                        open_url(url, open)?;
                    }
                }
            }
            Ok(output::project_collection(
                &result.items,
                &result.page_info,
                json_output,
            ))
        }
        ProjectCommand::View { project, open } => {
            let found = resolve_project(&client()?, &project)?;
            if open != Open::None {
                open_url(
                    found
                        .url
                        .as_deref()
                        .ok_or_else(|| AppError::operational("project has no HTTPS URL"))?,
                    open,
                )?;
            }
            Ok(output::project(&found, json_output))
        }
        ProjectCommand::Create(input) => create_project(input, json_output),
    }
}
fn create_project(input: ProjectCreate, json_output: bool) -> Result<String, AppError> {
    let c = client()?;
    let mut team_ids = Vec::new();
    for team in &input.teams {
        let id = find_team(&c, team)?.id;
        if !team_ids.contains(&id) {
            team_ids.push(id);
        }
    }
    let mut value = Map::new();
    value.insert("name".into(), json!(input.name));
    value.insert("teamIds".into(), json!(team_ids));
    if let Some(description) = input.description {
        value.insert("description".into(), json!(description));
    }
    if let Some(path) = input.content_file {
        value.insert("content".into(), json!(read_content_file(&path)?));
    }
    if let Some(lead) = input.lead {
        value.insert("leadId".into(), json!(resolve_user(&c, &lead)?.id));
    }
    if !input.members.is_empty() {
        let mut member_ids = Vec::new();
        for member in input.members {
            let id = resolve_user(&c, &member)?.id;
            if !member_ids.contains(&id) {
                member_ids.push(id);
            }
        }
        value.insert("memberIds".into(), json!(member_ids));
    }
    if let Some(date) = input.target_date {
        value.insert("targetDate".into(), json!(date));
    }
    Ok(output::project(
        &c.project_create(Value::Object(value))?,
        json_output,
    ))
}
fn milestones_pages(
    c: &LinearClient,
    project_id: Option<&str>,
    page: &crate::cli::Page,
) -> Result<crate::linear::Collection<Milestone>, AppError> {
    let mut after = page.after.clone();
    let mut items = Vec::new();
    let mut guard = crate::linear::transport::PaginationGuard::default();
    loop {
        let request = c.milestones(project_id, 100, after.as_deref())?;
        let info = request.page_info.clone();
        let prior = items.len();
        items.extend(request.items);
        match page.limit {
            Limit::Bounded(n) if items.len() >= n as usize => {
                return Ok(crate::linear::Collection {
                    items: items.into_iter().take(n as usize).collect(),
                    item_cursors: Vec::new(),
                    page_info: truncated_page_info(info, &request.item_cursors, prior, n as usize),
                });
            }
            Limit::Bounded(_) | Limit::Unlimited if !info.has_next_page => {
                return Ok(crate::linear::Collection {
                    items,
                    item_cursors: Vec::new(),
                    page_info: info,
                });
            }
            _ => after = guard.next_cursor(info.has_next_page, info.end_cursor.as_deref())?,
        }
    }
}
fn resolve_milestone(
    c: &LinearClient,
    value: &str,
    project_id: Option<&str>,
) -> Result<Milestone, AppError> {
    let page = crate::cli::Page {
        limit: Limit::Unlimited,
        after: None,
    };
    let matches: Vec<_> = milestones_pages(c, project_id, &page)?
        .items
        .into_iter()
        .filter(|m| m.id == value || m.name == value)
        .collect();
    match matches.len() {
        1 => Ok(matches.into_iter().next().expect("one")),
        0 => Err(AppError::input(format!(
            "milestone {value} is not accessible; use its Linear ID or exact name"
        ))),
        _ => Err(AppError::input(format!(
            "milestone {value} is ambiguous; use one of: {}",
            matches
                .iter()
                .map(|m| format!("{} ({})", m.name, m.id))
                .collect::<Vec<_>>()
                .join(", ")
        ))),
    }
}
fn milestone_command(command: MilestoneCommand, json_output: bool) -> Result<String, AppError> {
    let c = client()?;
    match command {
        MilestoneCommand::List { project, page } => {
            let project = resolve_project(&c, &project)?;
            let r = milestones_pages(&c, Some(&project.id), &page)?;
            Ok(output::milestone_collection(
                &r.items,
                &r.page_info,
                json_output,
            ))
        }
        MilestoneCommand::View { milestone } => {
            let found = resolve_milestone(&c, &milestone, None)?;
            Ok(output::milestone(&found, json_output))
        }
        MilestoneCommand::Create(mut input) => {
            if input.name.is_none() {
                if json_output || !io::stdin().is_terminal() || !io::stderr().is_terminal() {
                    return Err(AppError::input(
                        "milestone create requires --name NAME outside an interactive terminal",
                    ));
                }
                input.name = Some(prompt("Name", true)?);
            }
            let project = resolve_project(
                &c,
                input.project.as_deref().expect("parser requires project"),
            )?;
            let mut value = Map::new();
            value.insert("projectId".into(), json!(project.id));
            value.insert("name".into(), json!(input.name.expect("name prompted")));
            if let Some(x) = input.description {
                value.insert("description".into(), json!(x));
            }
            if let Some(x) = input.target_date {
                value.insert("targetDate".into(), json!(x));
            }
            Ok(output::milestone(
                &c.milestone_create(Value::Object(value))?,
                json_output,
            ))
        }
        MilestoneCommand::Update {
            milestone,
            mutation,
        } => {
            let found = resolve_milestone(&c, &milestone, None)?;
            let mut value = Map::new();
            if let Some(x) = mutation.name {
                value.insert("name".into(), json!(x));
            }
            if let Some(x) = mutation.description {
                value.insert("description".into(), json!(x));
            }
            if let Some(x) = mutation.target_date {
                value.insert("targetDate".into(), json!(x));
            }
            Ok(output::milestone(
                &c.milestone_update(&found.id, Value::Object(value))?,
                json_output,
            ))
        }
        MilestoneCommand::Delete {
            milestone,
            confirm,
            dry_run,
        } => {
            let found = resolve_milestone(&c, &milestone, None)?;
            if confirm != found.id && confirm != found.name {
                return Err(AppError::input(
                    "--confirm must exactly match the resolved milestone ID or name",
                ));
            }
            if dry_run {
                Ok(output::dry_run("delete", &found.id, json_output))
            } else {
                Ok(output::archived(
                    &c.milestone_delete(&found.id)?,
                    json_output,
                ))
            }
        }
    }
}
fn document_pages(
    c: &LinearClient,
    target: &DocumentTarget,
    page: &crate::cli::Page,
) -> Result<crate::linear::Collection<Document>, AppError> {
    let filter = match target {
        DocumentTarget::None => Value::Null,
        DocumentTarget::Project(value) => {
            json!({"project":{"id":{"eq":resolve_project(c, value)?.id}}})
        }
        DocumentTarget::Issue(issue) => {
            let id = issue_id(Some(issue.clone()))?;
            let found = c.issue(id.as_str())?;
            json!({"issue":{"id":{"eq":found.id}}})
        }
    };
    let mut after = page.after.clone();
    let mut items = Vec::new();
    let mut guard = crate::linear::transport::PaginationGuard::default();
    loop {
        // Each document includes up to 100 inline-comment anchors, so the outer
        // connection must stay small enough for Linear's query-complexity budget.
        let request = c.documents(filter.clone(), 25, after.as_deref())?;
        let info = request.page_info.clone();
        let prior = items.len();
        items.extend(request.items);
        match page.limit {
            Limit::Bounded(n) if items.len() >= n as usize => {
                return Ok(crate::linear::Collection {
                    items: items.into_iter().take(n as usize).collect(),
                    item_cursors: Vec::new(),
                    page_info: truncated_page_info(info, &request.item_cursors, prior, n as usize),
                });
            }
            Limit::Bounded(_) | Limit::Unlimited if !info.has_next_page => {
                return Ok(crate::linear::Collection {
                    items,
                    item_cursors: Vec::new(),
                    page_info: info,
                });
            }
            _ => after = guard.next_cursor(info.has_next_page, info.end_cursor.as_deref())?,
        }
    }
}
fn resolve_document(c: &LinearClient, value: &str) -> Result<Document, AppError> {
    // Public `document(id:)` accepts both ID and slug ID. Never guess by title.
    c.document(value)
}
fn document_content(source: ContentSource) -> Result<String, AppError> {
    match source {
        ContentSource::File(path) => read_content_file(&path),
        ContentSource::Stdin => {
            const MAX: usize = 1024 * 1024;
            let mut bytes = Vec::new();
            io::stdin()
                .take((MAX + 1) as u64)
                .read_to_end(&mut bytes)
                .map_err(|e| AppError::operational(format!("could not read stdin: {e}")))?;
            if bytes.len() > MAX {
                return Err(AppError::input(
                    "document content must be no larger than 1 MiB",
                ));
            }
            let content = String::from_utf8(bytes)
                .map_err(|_| AppError::input("document content must be valid UTF-8"))?;
            if content
                .chars()
                .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
            {
                return Err(AppError::input(
                    "document content contains unsupported control characters",
                ));
            }
            Ok(content)
        }
    }
}
fn document_target_input(
    c: &LinearClient,
    target: DocumentTarget,
) -> Result<Map<String, Value>, AppError> {
    let mut input = Map::new();
    match target {
        DocumentTarget::None => {}
        DocumentTarget::Project(value) => {
            input.insert("projectId".into(), json!(resolve_project(c, &value)?.id));
        }
        DocumentTarget::Issue(reference) => {
            let id = issue_id(Some(reference))?;
            input.insert("issueId".into(), json!(c.issue(id.as_str())?.id));
        }
    };
    Ok(input)
}
fn active_inline_comments(document: &Document) -> bool {
    document.comments.as_ref().is_some_and(|comments| {
        // A partial comment connection is conservatively treated as unsafe: another
        // page could contain an active inline comment.
        comments
            .page_info
            .as_ref()
            .is_some_and(|page| page.has_next_page)
            || comments.nodes.iter().any(|comment| {
                comment.document_content_id.is_some() && comment.resolved_at.is_none()
            })
    })
}
fn document_command(command: DocumentCommand, json_output: bool) -> Result<String, AppError> {
    let c = client()?;
    match command {
        DocumentCommand::List { target, page } => {
            let r = document_pages(&c, &target, &page)?;
            Ok(output::document_collection(
                &r.items,
                &r.page_info,
                json_output,
            ))
        }
        DocumentCommand::View { document, raw, web } => {
            let found = resolve_document(&c, &document)?;
            if web {
                open_url(
                    found
                        .url
                        .as_deref()
                        .ok_or_else(|| AppError::operational("document has no HTTPS URL"))?,
                    Open::Web,
                )?;
            }
            Ok(output::document(&found, raw, json_output))
        }
        DocumentCommand::Create(mutation) => {
            let mut input = document_target_input(&c, mutation.target)?;
            input.insert(
                "title".into(),
                json!(mutation.title.expect("parser requires title")),
            );
            if let Some(source) = mutation.content {
                input.insert("content".into(), json!(document_content(source)?));
            }
            Ok(output::document(
                &c.document_create(Value::Object(input))?,
                false,
                json_output,
            ))
        }
        DocumentCommand::Update {
            document,
            mutation,
            force,
        } => {
            let found = resolve_document(&c, &document)?;
            let replaces_content = mutation.content.is_some();
            let replaces_target = !matches!(mutation.target, DocumentTarget::None);
            if (replaces_content || replaces_target) && active_inline_comments(&found) && !force {
                return Err(AppError::input(
                    "document has active inline comments; use --force to replace content or target",
                ));
            }
            let mut input = document_target_input(&c, mutation.target)?;
            if let Some(title) = mutation.title {
                input.insert("title".into(), json!(title));
            }
            if let Some(source) = mutation.content {
                input.insert("content".into(), json!(document_content(source)?));
            }
            Ok(output::document(
                &c.document_update(&found.id, Value::Object(input))?,
                false,
                json_output,
            ))
        }
        DocumentCommand::Delete {
            documents,
            confirms,
            dry_run,
        } => {
            let mut found = Vec::new();
            for (document, confirm) in documents.iter().zip(confirms.iter()) {
                let item = resolve_document(&c, document)?;
                if confirm != &item.id && item.slug_id.as_deref() != Some(confirm) {
                    return Err(AppError::input(
                        "--confirm must exactly match the resolved document ID or slug",
                    ));
                }
                found.push(item);
            }
            if dry_run {
                return Ok(output::dry_run(
                    "archive",
                    &found
                        .iter()
                        .map(|d| d.id.as_str())
                        .collect::<Vec<_>>()
                        .join(","),
                    json_output,
                ));
            }
            let mut complete = Vec::new();
            for item in &found {
                if let Err(error) = c.document_delete(&item.id) {
                    if complete.is_empty() {
                        return Err(error);
                    }
                    let pending_documents = &documents[complete.len()..];
                    let pending_confirms = &confirms[complete.len()..];
                    let retry = format!(
                        "workrus document delete --bulk {} --confirm {}",
                        pending_documents.join(" "),
                        pending_confirms.join(" ")
                    );
                    return Err(AppError::partial(
                        error.message,
                        output::bulk_partial(&complete, &retry, json_output),
                    ));
                }
                complete.push(item.clone());
            }
            if found.len() == 1 {
                Ok(output::archived(&found[0].id, json_output))
            } else {
                Ok(output::archived_documents(&found, json_output))
            }
        }
    }
}
fn find_team(c: &LinearClient, key: &str) -> Result<Team, AppError> {
    use crate::linear::transport::PaginationGuard;
    let mut after = None;
    let mut guard = PaginationGuard::default();
    loop {
        let page = c.teams(100, after.as_deref())?;
        if let Some(team) = page.items.into_iter().find(|team| team.key == key) {
            return Ok(team);
        }
        after = guard.next_cursor(
            page.page_info.has_next_page,
            page.page_info.end_cursor.as_deref(),
        )?;
        if after.is_none() {
            return Err(AppError::input(format!("team {key} is not accessible")));
        }
    }
}
fn state(c: &LinearClient, team: &Team, value: &str) -> Result<State, AppError> {
    let states = c.team_states(&team.id)?;
    let needle = value.to_ascii_lowercase();
    let matches: Vec<_> = states
        .into_iter()
        .filter(|s| s.name.to_ascii_lowercase() == needle || s.kind.to_ascii_lowercase() == needle)
        .collect();
    match matches.len() {
        1 => Ok(matches.into_iter().next().expect("one")),
        0 => Err(AppError::input(format!(
            "state {value} is not available for team {}",
            team.key
        ))),
        _ => Err(AppError::input(format!(
            "state {value} is ambiguous for team {}",
            team.key
        ))),
    }
}
fn mutation_input(
    c: &LinearClient,
    m: &Mutation,
    current: Option<&Issue>,
) -> Result<Value, AppError> {
    let mut input = Map::new();
    if let Some(x) = &m.title {
        input.insert("title".into(), json!(x));
    }
    if let Some(x) = &m.description {
        input.insert("description".into(), json!(x));
    }

    // Creates always require a team; updates resolve state against the destination team.
    let target_team = if let Some(key) = m.team.as_deref() {
        Some(find_team(c, key)?)
    } else if let Some(issue) = current {
        Some(issue.team.clone())
    } else {
        let key = config::resolve_team(None, config::default_team()?)?;
        Some(find_team(c, &key)?)
    };
    if current.is_none() || m.team.is_some() {
        input.insert(
            "teamId".into(),
            json!(target_team.as_ref().expect("team resolved").id),
        );
    }
    if m.assignee_self {
        input.insert("assigneeId".into(), json!(c.viewer()?.id));
    }
    if m.unassign {
        input.insert("assigneeId".into(), Value::Null);
    }
    let resolved_project = m
        .project
        .as_deref()
        .map(|project| resolve_project(c, project))
        .transpose()?;
    if let Some(project) = &resolved_project {
        input.insert("projectId".into(), json!(project.id));
    }
    if m.remove_project {
        input.insert("projectId".into(), Value::Null);
    }
    if let Some(milestone) = &m.milestone {
        // When the command supplied a project, resolve the name only within that project.
        // Otherwise require global uniqueness rather than guessing a similarly named milestone.
        input.insert(
            "projectMilestoneId".into(),
            json!(
                resolve_milestone(
                    c,
                    milestone,
                    resolved_project.as_ref().map(|p| p.id.as_str())
                )?
                .id
            ),
        );
    }
    if m.remove_milestone {
        input.insert("projectMilestoneId".into(), Value::Null);
    }
    if let Some(priority) = m.priority {
        input.insert("priority".into(), json!(priority));
    }
    if let Some(value) = &m.state {
        let team = target_team.as_ref().expect("state has a target team");
        input.insert("stateId".into(), json!(state(c, team, value)?.id));
    }
    Ok(Value::Object(input))
}
fn prompt_create(m: &mut Mutation, json_output: bool) -> Result<(), AppError> {
    if m.title.is_none() {
        if json_output || !io::stdin().is_terminal() || !io::stderr().is_terminal() {
            return Err(AppError::input(
                "issue create requires --title TITLE outside an interactive terminal",
            ));
        }
        m.title = Some(prompt("Title", true)?);
        if m.description.is_none() {
            let description = prompt("Description (optional)", false)?;
            if !description.is_empty() {
                m.description = Some(description);
            }
        }
        if m.state.is_none() {
            let state = prompt("State (optional)", false)?;
            if !state.is_empty() {
                m.state = Some(state);
            }
        }
    }
    let settings = config::runtime_config()?;
    if !m.assignee_self && !m.unassign {
        match settings.issue_create_assign_self {
            Some(config::AssignSelf::Always) => m.assignee_self = true,
            Some(config::AssignSelf::Auto)
                if io::stdin().is_terminal()
                    && io::stderr().is_terminal()
                    && !json_output
                    && prompt("Assign to yourself? [Y/n]", false)?.is_empty() =>
            {
                m.assignee_self = true;
            }
            _ => {}
        }
    }
    // Project prompting is deliberately bounded to a line and only runs in human TTY mode.
    if settings.issue_create_ask_project == Some(true)
        && m.project.is_none()
        && io::stdin().is_terminal()
        && io::stderr().is_terminal()
        && !json_output
    {
        let project = prompt("Project (optional ID)", false)?;
        if !project.is_empty() {
            m.project = Some(project);
        }
    }
    Ok(())
}
fn prompt(label: &str, required: bool) -> Result<String, AppError> {
    const MAX: usize = 16 * 1024;
    let mut stderr = io::stderr().lock();
    write!(stderr, "{label}: ")
        .map_err(|e| AppError::operational(format!("could not prompt: {e}")))?;
    stderr
        .flush()
        .map_err(|e| AppError::operational(format!("could not prompt: {e}")))?;
    let mut bytes = Vec::new();
    io::stdin()
        .lock()
        .take((MAX + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| AppError::operational(format!("could not read prompt: {e}")))?;
    if bytes.len() > MAX {
        return Err(AppError::input("interactive input is invalid or too long"));
    }
    let value = String::from_utf8(bytes)
        .map_err(|_| AppError::input("interactive input is invalid or too long"))?;
    let value = value.trim_end_matches(['\r', '\n']).to_owned();
    if value.chars().any(|c| c.is_control() && c != '\t') {
        return Err(AppError::input("interactive input is invalid or too long"));
    }
    if required && value.trim().is_empty() {
        return Err(AppError::input("title must not be empty"));
    }
    Ok(value)
}
fn comments(command: CommentCommand, json_output: bool) -> Result<String, AppError> {
    match command {
        CommentCommand::List { issue, page } => {
            let id = issue_id(issue)?;
            let c = client()?;
            let mut after = page.after.clone();
            let mut items = Vec::new();
            let mut guard = crate::linear::transport::PaginationGuard::default();
            loop {
                let result = c.comments(id.as_str(), 100, after.as_deref())?;
                let info = result.page_info.clone();
                let prior = items.len();
                items.extend(result.items);
                match page.limit {
                    Limit::Bounded(n) if items.len() >= n as usize => {
                        items.truncate(n as usize);
                        let page_info =
                            truncated_page_info(info, &result.item_cursors, prior, n as usize);
                        return Ok(output::comment_collection(&items, &page_info, json_output));
                    }
                    Limit::Bounded(_) | Limit::Unlimited if !info.has_next_page => {
                        return Ok(output::comment_collection(&items, &info, json_output));
                    }
                    _ => {
                        after = guard.next_cursor(info.has_next_page, info.end_cursor.as_deref())?
                    }
                }
            }
        }
        CommentCommand::Add {
            issue,
            body,
            parent,
        } => {
            let id = issue_id(issue)?;
            let found = client()?.issue(id.as_str())?;
            let mut input = json!({"issueId":found.id,"body":body});
            if let Some(parent) = parent {
                input["parentId"] = json!(parent);
            }
            let comment = client()?.comment_create(input)?;
            Ok(output::comment(&comment, json_output))
        }
        CommentCommand::Update { id, body } => {
            let comment = client()?.comment_update(&id, json!({"body":body}))?;
            Ok(output::comment(&comment, json_output))
        }
        CommentCommand::Delete {
            id,
            confirm,
            dry_run,
        } => {
            if confirm != id {
                return Err(AppError::input("--confirm must exactly match COMMENT_ID"));
            }
            if dry_run {
                Ok(output::dry_run("delete", &id, json_output))
            } else {
                let entity = client()?.comment_delete(&id)?;
                Ok(output::archived(&entity, json_output))
            }
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
enum StartTransition {
    AlreadyStarted,
    Update,
}

fn start_transition(state_kind: Option<&str>) -> Result<StartTransition, AppError> {
    match state_kind {
        Some("started") => Ok(StartTransition::AlreadyStarted),
        Some("completed" | "canceled") => Err(AppError::input(
            "cannot start a completed or canceled issue",
        )),
        _ => Ok(StartTransition::Update),
    }
}

fn start_transition_after_git(
    state_kind: Option<&str>,
    partial_result: &impl Fn() -> String,
) -> Result<StartTransition, AppError> {
    start_transition(state_kind).map_err(|error| AppError::partial(error.message, partial_result()))
}

fn start(id: Option<IssueRef>, json_output: bool) -> Result<String, AppError> {
    let id = issue_id(id)?;
    let c = client()?;
    let issue = c.issue(id.as_str())?;
    let kind = issue.state.as_ref().map(|s| s.kind.as_str());
    if matches!(kind, Some("completed") | Some("canceled")) {
        return Err(AppError::input(
            "cannot start a completed or canceled issue",
        ));
    }
    let branch = issue
        .branch_name
        .as_deref()
        .filter(|x| !x.is_empty())
        .ok_or_else(|| AppError::input("Linear issue has no branch name"))?;
    let states = c.team_states(&issue.team.id)?;
    let started: Vec<_> = states.into_iter().filter(|s| s.kind == "started").collect();
    if started.len() != 1 {
        return Err(AppError::input(
            "team must have exactly one started workflow state",
        ));
    }
    let target = started.into_iter().next().expect("one");
    let action = git::prepare_start(
        &env::current_dir().map_err(|e| AppError::operational(e.to_string()))?,
        branch,
    )?;
    let partial = || {
        output::partial_start(
            &issue.identifier,
            &issue.id,
            branch,
            action,
            &target,
            json_output,
        )
    };
    let latest = c
        .issue(id.as_str())
        .map_err(|error| AppError::partial(error.message, partial()))?;
    let transition = start_transition_after_git(
        latest.state.as_ref().map(|state| state.kind.as_str()),
        &partial,
    )?;
    let workflow = match transition {
        StartTransition::AlreadyStarted => "already_started",
        StartTransition::Update => {
            c.update(id.as_str(), json!({"stateId":target.id}))
                .map_err(|error| AppError::partial(error.message, partial()))?;
            "updated"
        }
    };
    Ok(output::start(
        &issue.identifier,
        &issue.id,
        branch,
        action,
        workflow,
        &target,
        json_output,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_refuses_terminal_state_observed_after_git_switch() {
        for state_kind in ["completed", "canceled"] {
            let partial = || "git completed".to_owned();
            let error = start_transition_after_git(Some(state_kind), &partial).unwrap_err();

            assert_eq!(error.kind, crate::error::ErrorKind::Operational);
            assert!(error.message.contains("completed or canceled"));
            assert_eq!(error.partial_result.as_deref(), Some("git completed"));
        }
    }

    #[test]
    fn truncated_page_uses_last_retained_edge_cursor() {
        let info = crate::model::PageInfo {
            has_next_page: false,
            end_cursor: Some("server-end".into()),
        };
        let info = truncated_page_info(info, &["one".into(), "two".into(), "three".into()], 1, 3);
        assert_eq!(info.end_cursor.as_deref(), Some("two"));
        assert!(info.has_next_page);
        // Unlimited collections retain server metadata rather than synthesizing a cursor.
        let complete = truncated_page_info(info.clone(), &["one".into()], 0, 1);
        assert_eq!(complete.end_cursor, info.end_cursor);
    }

    #[cfg(unix)]
    #[test]
    fn browser_and_gh_use_argument_arrays_and_scrub_sensitive_environment() {
        use std::{
            fs,
            os::unix::fs::PermissionsExt,
            process::Command,
            time::{SystemTime, UNIX_EPOCH},
        };
        const CHILD: &str = "WORKRUS_PROCESS_TEST_CHILD";
        if env::var_os(CHILD).is_none() {
            let temp = env::temp_dir().join(format!(
                "workrus-process-{}",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir_all(&temp).unwrap();
            let script = "#!/bin/sh\nprintf '%s\\n' \"$@\" >> \"$WORKRUS_CAPTURE\"\nenv | grep -E '^(LINEAR_API_KEY|GIT_DIR|GIT_WORK_TREE|GIT_SSH_COMMAND)=' >> \"$WORKRUS_CAPTURE\" || true\nif [ \"$(basename \"$0\")\" = gh ]; then printf 'https://github.example/pr/1\\n'; fi\n";
            for executable in [
                "gh",
                if cfg!(target_os = "macos") {
                    "open"
                } else {
                    "xdg-open"
                },
            ] {
                let path = temp.join(executable);
                fs::write(&path, script).unwrap();
                let mut permissions = fs::metadata(&path).unwrap().permissions();
                permissions.set_mode(0o700);
                fs::set_permissions(path, permissions).unwrap();
            }
            let capture = temp.join("capture");
            let output = Command::new(env::current_exe().unwrap())
                .args(["--exact", "app::tests::browser_and_gh_use_argument_arrays_and_scrub_sensitive_environment"])
                .env(CHILD, "1")
                .env("PATH", format!("{}:{}", temp.display(), env::var("PATH").unwrap()))
                .env("WORKRUS_CAPTURE", &capture)
                .env("LINEAR_API_KEY", "must-not-leak")
                .env("GIT_DIR", "must-not-leak")
                .env("GIT_WORK_TREE", "must-not-leak")
                .env("GIT_SSH_COMMAND", "must-not-leak")
                .output().unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            fs::remove_dir_all(temp).unwrap();
            return;
        }
        let issue = Issue {
            id: "i1".into(),
            identifier: "ENG-1".into(),
            title: "Title".into(),
            description: None,
            url: "https://linear.app/issue/ENG-1".into(),
            priority: None,
            team: Team {
                id: "t".into(),
                key: "ENG".into(),
                name: "Engineering".into(),
                description: None,
                color: None,
                icon: None,
            },
            state: None,
            assignee: None,
            created_at: None,
            updated_at: None,
            branch_name: None,
        };
        let rendered = pr(
            &issue,
            Some("main"),
            Some("feature"),
            true,
            Some("Custom"),
            true,
            true,
        )
        .unwrap();
        assert!(rendered.contains("https://github.example/pr/1"));
        let captured = fs::read_to_string(env::var("WORKRUS_CAPTURE").unwrap()).unwrap();
        assert!(captured.contains("https://github.example/pr/1"));
        assert!(captured.contains("--base") && captured.contains("main"));
        assert!(captured.contains("--head") && captured.contains("feature"));
        assert!(captured.contains("--draft"));
        assert!(!captured.contains("--web"));
        assert!(!captured.contains("must-not-leak"));
    }
    #[test]
    fn content_file_is_read_from_a_bounded_open_handle() {
        let root = std::env::temp_dir().join(format!(
            "workrus-content-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("content.md");
        std::fs::write(&path, "safe markdown\n").unwrap();
        assert_eq!(
            read_content_file(path.to_str().unwrap()).unwrap(),
            "safe markdown\n"
        );
        std::fs::write(&path, vec![b'x'; 1024 * 1024 + 1]).unwrap();
        assert!(read_content_file(path.to_str().unwrap()).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn content_file_never_follows_a_symlink() {
        use std::os::unix::fs::symlink;

        let root = std::env::temp_dir().join(format!(
            "workrus-content-symlink-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let target = root.join("target.md");
        let link = root.join("link.md");
        std::fs::write(&target, "private\n").unwrap();
        symlink(&target, &link).unwrap();
        assert!(read_content_file(link.to_str().unwrap()).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn document_pagination_stays_below_linear_complexity_budget() {
        use crate::{config::ApiKey, linear::transport::tests::serve_once};

        let response = r#"{"data":{"documents":{"nodes":[],"edges":[],"pageInfo":{"hasNextPage":false,"endCursor":null}}}}"#;
        let (endpoint, captured) = serve_once("200 OK", response);
        let client = LinearClient::with_test_endpoint(ApiKey::for_test("secret"), &endpoint);
        let page = crate::cli::Page {
            limit: Limit::Bounded(1),
            after: None,
        };

        document_pages(&client, &DocumentTarget::None, &page).unwrap();

        let request = captured.join().unwrap();
        assert!(
            request.contains(r#""first":25"#),
            "document pages must stay below Linear's query-complexity limit"
        );
    }

    #[test]
    fn project_pagination_aggregates_and_preserves_retained_cursor() {
        use crate::config::ApiKey;
        use std::{
            io::{BufRead, BufReader, Read, Write},
            net::TcpListener,
            thread,
        };
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = format!("http://{}/graphql", listener.local_addr().unwrap());
        let server = thread::spawn(move || {
            for (id, next, cursor) in [("p1", true, "one"), ("p2", false, "two")] {
                let (mut stream, _) = listener.accept().unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut content_length = 0usize;
                loop {
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    if line == "\r\n" {
                        break;
                    }
                    if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                        content_length = value.trim().parse().unwrap();
                    }
                }
                let mut request_body = vec![0; content_length];
                reader.read_exact(&mut request_body).unwrap();
                let request_body = String::from_utf8(request_body).unwrap();
                assert!(
                    request_body.contains(r#""first":50"#),
                    "project pages must stay below Linear's query-complexity limit"
                );
                let body = format!(
                    r#"{{"data":{{"projects":{{"nodes":[{{"id":"{id}","name":"{id}","slugId":null,"description":null,"content":null,"url":null,"targetDate":null,"teams":{{"nodes":[]}},"lead":null,"members":{{"nodes":[]}}}}],"edges":[{{"cursor":"{cursor}"}}],"pageInfo":{{"hasNextPage":{next},"endCursor":"{cursor}"}}}}}}}}"#
                );
                write!(
                    stream,
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                )
                .unwrap();
            }
        });
        let client = LinearClient::with_test_endpoint(ApiKey::for_test("secret"), &endpoint);
        let page = crate::cli::Page {
            limit: Limit::Bounded(2),
            after: None,
        };
        let result = projects_pages(&client, None, &page).unwrap();
        assert_eq!(result.items.len(), 2);
        assert_eq!(result.page_info.end_cursor.as_deref(), Some("two"));
        server.join().unwrap();
    }
}
