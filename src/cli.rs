use crate::{error::AppError, model::IssueIdentifier};

pub const USAGE: &str = "workrus [--json] <command>\n\nCommands:\n  config [TEAM]\n  completion|completions <bash|zsh|fish|powershell>\n  team list [--limit N] [--after CURSOR] [--web|--app]\n  team id [TEAM]\n  team members [TEAM] [--all] [--limit N] [--after CURSOR]\n  team create --name NAME [--key KEY] [--description TEXT] [--color COLOR] [--icon ICON]\n  team autolinks\n  user list [--all] [--limit N] [--after CURSOR]\n  project list [--team TEAM] [--limit N] [--after CURSOR] [--web|--app]\n  project view PROJECT [--web|--app]\n  project create --name NAME --team TEAM [--team TEAM...] [--description TEXT] [--content-file PATH] [--lead USER] [--member USER...] [--target-date YYYY-MM-DD]\n  milestone|m list --project PROJECT [--limit N] [--after CURSOR]\n  milestone|m view MILESTONE\n  milestone|m create --project PROJECT --name NAME [--description TEXT] [--target-date YYYY-MM-DD]\n  milestone|m update MILESTONE [--name NAME] [--description TEXT] [--target-date YYYY-MM-DD]\n  milestone|m delete MILESTONE --confirm MILESTONE [--dry-run]\n  document|docs list [--project PROJECT|--issue ISSUE] [--limit N] [--after CURSOR]\n  document|docs view DOCUMENT [--raw] [-w|--web]\n  document|docs create --title TITLE (--project PROJECT|--issue ISSUE) [--content-file PATH|--stdin]\n  document|docs update DOCUMENT [--title TITLE] [--content-file PATH|--stdin] [--project PROJECT|--issue ISSUE] [--force]\n  document|docs delete DOCUMENT --confirm DOCUMENT [--dry-run]\n  document|docs delete --bulk DOCUMENT... --confirm DOCUMENT... [--dry-run]\n  issue mine|list|l [--team TEAM] [-s STATE] [--sort manual|priority] [--project PROJECT] [--milestone MILESTONE] [--limit N] [--after CURSOR] [--web|--app]\n  issue query <TEXT|--search TEXT> [--team TEAM|--all-teams] [-s STATE] [--sort manual|priority] [--project PROJECT] [--milestone MILESTONE] [--limit N] [--after CURSOR]\n  issue view [ID|NUMBER] [--web|--app]\n  issue pr|pull-request [ID|NUMBER] [--base BRANCH] [--head BRANCH] [--draft] [-t TITLE] [-w|--web]\n  issue create [-t TITLE] [-d DESCRIPTION] [--team TEAM] [--state STATE] [--assignee self] [--project PROJECT] [--milestone MILESTONE] [--priority 0..4]\n  issue update [ID|NUMBER] [-t TITLE] [-d DESCRIPTION] [--team TEAM] [--state STATE] [--assignee self|--unassign] [--project PROJECT|--remove-project] [--milestone MILESTONE|--remove-milestone] [--priority 0..4]\n  issue delete [ID|NUMBER] --confirm CANONICAL_ID [--dry-run]\n  issue comment list [ID|NUMBER] [--limit N] [--after CURSOR]\n  issue comment add [ID|NUMBER] --body TEXT [-p COMMENT_ID]\n  issue comment update COMMENT_ID --body TEXT\n  issue comment delete COMMENT_ID --confirm COMMENT_ID [--dry-run]\n  issue start [ID|NUMBER]\n  issue id|title|url [ID|NUMBER]\n";
#[derive(Debug, PartialEq, Eq)]
pub enum ParseResult {
    Help,
    Version,
    Command { json: bool, command: Command },
}
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Config(Option<String>),
    TeamList {
        page: Page,
        open: Open,
    },
    TeamId(Option<String>),
    TeamMembers {
        team: Option<String>,
        include_disabled: bool,
        page: Page,
    },
    TeamCreate(TeamCreate),
    TeamAutolinks,
    UserList {
        include_disabled: bool,
        page: Page,
    },
    Project(ProjectCommand),
    Milestone(MilestoneCommand),
    Document(DocumentCommand),
    Completion(CompletionShell),
    Mine(List),
    Query {
        text: String,
        list: List,
        all_teams: bool,
    },
    View {
        issue: Option<IssueRef>,
        open: Open,
    },
    Pr {
        issue: Option<IssueRef>,
        base: Option<String>,
        head: Option<String>,
        draft: bool,
        title: Option<String>,
        web: bool,
    },
    Scalar {
        field: Scalar,
        issue: Option<IssueRef>,
    },
    Create(Mutation),
    Update {
        issue: Option<IssueRef>,
        mutation: Mutation,
    },
    Delete {
        issue: Option<IssueRef>,
        confirm: String,
        dry_run: bool,
    },
    Comment(CommentCommand),
    Start(Option<IssueRef>),
}
#[derive(Debug, PartialEq, Eq)]
pub enum CommentCommand {
    List {
        issue: Option<IssueRef>,
        page: Page,
    },
    Add {
        issue: Option<IssueRef>,
        body: String,
        parent: Option<String>,
    },
    Update {
        id: String,
        body: String,
    },
    Delete {
        id: String,
        confirm: String,
        dry_run: bool,
    },
}
#[derive(Debug, PartialEq, Eq)]
pub enum DocumentCommand {
    List {
        target: DocumentTarget,
        page: Page,
    },
    View {
        document: String,
        raw: bool,
        web: bool,
    },
    Create(DocumentMutation),
    Update {
        document: String,
        mutation: DocumentMutation,
        force: bool,
    },
    Delete {
        documents: Vec<String>,
        confirms: Vec<String>,
        dry_run: bool,
    },
}
#[derive(Debug, PartialEq, Eq, Default)]
pub struct DocumentMutation {
    pub title: Option<String>,
    pub content: Option<ContentSource>,
    pub target: DocumentTarget,
}
#[derive(Debug, PartialEq, Eq, Default)]
pub enum DocumentTarget {
    #[default]
    None,
    Project(String),
    Issue(IssueRef),
}
#[derive(Debug, PartialEq, Eq)]
pub enum ContentSource {
    File(String),
    Stdin,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompletionShell {
    Bash,
    Zsh,
    Fish,
    Powershell,
}

/// The command words shared by help-derived completion generation.
const ROOT_COMMANDS: &[&str] = &[
    "config",
    "team",
    "user",
    "project",
    "milestone",
    "m",
    "document",
    "docs",
    "issue",
    "completion",
    "completions",
];
const TEAM_COMMANDS: &[&str] = &["list", "id", "members", "create", "autolinks"];
const PROJECT_COMMANDS: &[&str] = &["list", "view", "create"];
const MILESTONE_COMMANDS: &[&str] = &["list", "view", "create", "update", "delete"];
const DOCUMENT_COMMANDS: &[&str] = &["list", "view", "create", "update", "delete"];
const ISSUE_COMMANDS: &[&str] = &[
    "mine",
    "list",
    "l",
    "query",
    "view",
    "pr",
    "pull-request",
    "id",
    "title",
    "url",
    "create",
    "update",
    "delete",
    "comment",
    "start",
];
const COMMENT_COMMANDS: &[&str] = &["list", "add", "update", "delete"];

pub fn completion_script(shell: CompletionShell) -> String {
    let root = ROOT_COMMANDS.join(" ");
    let team = TEAM_COMMANDS.join(" ");
    let project = PROJECT_COMMANDS.join(" ");
    let milestone = MILESTONE_COMMANDS.join(" ");
    let document = DOCUMENT_COMMANDS.join(" ");
    let issue = ISSUE_COMMANDS.join(" ");
    let comment = COMMENT_COMMANDS.join(" ");
    match shell {
        CompletionShell::Bash => format!(
            "# bash completion for workrus\n_workrus() {{\n  local cur\n  cur=\"${{COMP_WORDS[COMP_CWORD]}}\"\n  case \"${{COMP_WORDS[1]}}\" in\n    team) COMPREPLY=( $(compgen -W '{team}' -- \"$cur\") ) ;;\n    project) COMPREPLY=( $(compgen -W '{project}' -- \"$cur\") ) ;;\n    milestone|m) COMPREPLY=( $(compgen -W '{milestone}' -- \"$cur\") ) ;;\n    document|docs) COMPREPLY=( $(compgen -W '{document}' -- \"$cur\") ) ;;\n    issue)\n      if [[ ${{COMP_WORDS[2]}} == comment ]]; then COMPREPLY=( $(compgen -W '{comment}' -- \"$cur\") ); else COMPREPLY=( $(compgen -W '{issue}' -- \"$cur\") ); fi ;;\n    *) COMPREPLY=( $(compgen -W '{root}' -- \"$cur\") ) ;;\n  esac\n}}\ncomplete -F _workrus workrus\n"
        ),
        CompletionShell::Zsh => format!(
            "#compdef workrus\n\n_workrus() {{\n  local -a commands\n  commands=({root})\n  if (( CURRENT == 2 )); then\n    _describe -t commands 'workrus command' commands\n  else\n    case $words[2] in\n      team) _values 'team command' {team} ;;\n      project) _values 'project command' {project} ;;\n      milestone|m) _values 'milestone command' {milestone} ;;\n      document|docs) _values 'document command' {document} ;;\n      issue)\n        if [[ $words[3] == comment ]]; then _values 'comment command' {comment}; else _values 'issue command' {issue}; fi ;;\n    esac\n  fi\n}}\n_workrus \"$@\"\n"
        ),
        CompletionShell::Fish => format!(
            "# fish completion for workrus\ncomplete -c workrus -f\ncomplete -c workrus -n '__fish_use_subcommand' -a '{root}'\ncomplete -c workrus -n '__fish_seen_subcommand_from team' -a '{team}'\ncomplete -c workrus -n '__fish_seen_subcommand_from project' -a '{project}'\ncomplete -c workrus -n '__fish_seen_subcommand_from milestone m' -a '{milestone}'\ncomplete -c workrus -n '__fish_seen_subcommand_from document docs' -a '{document}'\ncomplete -c workrus -n '__fish_seen_subcommand_from issue' -a '{issue}'\ncomplete -c workrus -n '__fish_seen_subcommand_from comment' -a '{comment}'\n"
        ),
        CompletionShell::Powershell => format!(
            "# PowerShell completion for workrus\nRegister-ArgumentCompleter -Native -CommandName workrus -ScriptBlock {{\n  param($wordToComplete, $commandAst, $cursorPosition)\n  $words = @($commandAst.CommandElements | ForEach-Object {{ $_.ToString() }})\n  $candidates = switch ($words[1]) {{\n    'team' {{ '{team}' }}\n    'project' {{ '{project}' }}\n    'milestone' {{ '{milestone}' }}\n    'm' {{ '{milestone}' }}\n    'document' {{ '{document}' }}\n    'docs' {{ '{document}' }}\n    'issue' {{ if ($words[2] -eq 'comment') {{ '{comment}' }} else {{ '{issue}' }} }}\n    default {{ '{root}' }}\n  }}\n  $candidates -split ' ' | Where-Object {{ $_ -like \"$wordToComplete*\" }} | ForEach-Object {{ [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_) }}\n}}\n"
        ),
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum MilestoneCommand {
    List {
        project: String,
        page: Page,
    },
    View {
        milestone: String,
    },
    Create(MilestoneMutation),
    Update {
        milestone: String,
        mutation: MilestoneMutation,
    },
    Delete {
        milestone: String,
        confirm: String,
        dry_run: bool,
    },
}
#[derive(Debug, PartialEq, Eq, Default)]
pub struct MilestoneMutation {
    pub project: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub target_date: Option<String>,
}
#[derive(Debug, PartialEq, Eq)]
pub enum ProjectCommand {
    List {
        team: Option<String>,
        page: Page,
        open: Open,
    },
    View {
        project: String,
        open: Open,
    },
    Create(ProjectCreate),
}
#[derive(Debug, PartialEq, Eq, Default)]
pub struct ProjectCreate {
    pub name: String,
    pub teams: Vec<String>,
    pub description: Option<String>,
    pub content_file: Option<String>,
    pub lead: Option<String>,
    pub members: Vec<String>,
    pub target_date: Option<String>,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Scalar {
    Id,
    Title,
    Url,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IssueRef {
    Identifier(IssueIdentifier),
    Number(u64),
}
#[derive(Debug, PartialEq, Eq)]
pub enum Limit {
    Bounded(u16),
    Unlimited,
}
impl Default for Limit {
    fn default() -> Self {
        Self::Bounded(50)
    }
}
#[derive(Debug, PartialEq, Eq, Default)]
pub struct Page {
    pub limit: Limit,
    pub after: Option<String>,
}
#[derive(Debug, PartialEq, Eq, Default)]
pub struct List {
    pub page: Page,
    pub open: Open,
    pub team: Option<String>,
    pub state: Option<String>,
    pub sort: Option<String>,
    pub project: Option<String>,
    pub milestone: Option<String>,
}
#[derive(Debug, PartialEq, Eq, Default)]
pub struct TeamCreate {
    pub name: String,
    pub key: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
}
#[derive(Debug, PartialEq, Eq, Default)]
pub struct Mutation {
    pub title: Option<String>,
    pub description: Option<String>,
    pub team: Option<String>,
    pub assignee_self: bool,
    pub unassign: bool,
    pub state: Option<String>,
    pub project: Option<String>,
    pub remove_project: bool,
    pub milestone: Option<String>,
    pub remove_milestone: bool,
    pub priority: Option<u8>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Open {
    #[default]
    None,
    Web,
    App,
}

pub fn parse<I, T>(args: I) -> Result<ParseResult, AppError>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString>,
{
    let mut v = Vec::new();
    let mut json = false;
    for a in args {
        let s = a
            .into()
            .into_string()
            .map_err(|_| AppError::input("arguments must be Unicode"))?;
        if s == "--json" {
            json = true
        } else {
            v.push(s)
        }
    }
    if v.iter().any(|x| matches!(x.as_str(), "--help" | "-h")) {
        return Ok(ParseResult::Help);
    }
    if v.iter().any(|x| matches!(x.as_str(), "--version" | "-V")) {
        return Ok(ParseResult::Version);
    }
    if v.is_empty() {
        return Err(AppError::input("missing command; run `workrus --help`"));
    }
    if matches!(v[0].as_str(), "completion" | "completions") && json {
        return Err(AppError::input(
            "completion output is plain text; --json is unsupported",
        ));
    }
    let command = match (v[0].as_str(), v.get(1).map(String::as_str)) {
        ("config", _) => {
            if v.len() > 2 {
                return Err(extra());
            }
            Command::Config(v.get(1).cloned())
        }
        ("completion" | "completions", Some(shell)) if v.len() == 2 => {
            Command::Completion(match shell {
                "bash" => CompletionShell::Bash,
                "zsh" => CompletionShell::Zsh,
                "fish" => CompletionShell::Fish,
                "powershell" => CompletionShell::Powershell,
                _ => {
                    return Err(AppError::input(
                        "completion shell must be bash, zsh, fish, or powershell",
                    ));
                }
            })
        }
        ("team", Some("list")) => Command::TeamList {
            page: team_list_page(&v[2..])?,
            open: project_open(&v[2..])?,
        },
        ("team", Some("id")) => Command::TeamId(optional_positional(&v[2..], "team")?),
        ("team", Some("members")) => {
            let (team, rest) = optional_positional_with_rest(&v[2..], "team")?;
            let (include_disabled, page) = users_page(rest)?;
            Command::TeamMembers {
                team,
                include_disabled,
                page,
            }
        }
        ("team", Some("create")) => Command::TeamCreate(team_create(&v[2..])?),
        ("team", Some("autolinks")) if v.len() == 2 => Command::TeamAutolinks,
        ("user", Some("list")) => {
            let (include_disabled, page) = users_page(&v[2..])?;
            Command::UserList {
                include_disabled,
                page,
            }
        }
        ("document" | "docs", Some("list")) => Command::Document(document_list(&v[2..])?),
        ("document" | "docs", Some("view")) => Command::Document(document_view(&v[2..])?),
        ("document" | "docs", Some("create")) => {
            Command::Document(DocumentCommand::Create(document_mutation(&v[2..], true)?))
        }
        ("document" | "docs", Some("update")) => {
            let (document, rest) = required_positional_with_rest(&v[2..], "document")?;
            Command::Document(DocumentCommand::Update {
                document,
                mutation: document_mutation(rest, false)?,
                force: flag(rest, "--force")?,
            })
        }
        ("document" | "docs", Some("delete")) => Command::Document(document_delete(&v[2..])?),
        ("milestone" | "m", Some("list")) => {
            let project = required(&v[2..], "--project")?;
            Command::Milestone(MilestoneCommand::List {
                project,
                page: milestone_page(&v[2..])?,
            })
        }
        ("milestone" | "m", Some("view")) => {
            let (milestone, rest) = required_positional_with_rest(&v[2..], "milestone")?;
            if !rest.is_empty() {
                return Err(extra());
            }
            Command::Milestone(MilestoneCommand::View { milestone })
        }
        ("milestone" | "m", Some("create")) => {
            Command::Milestone(MilestoneCommand::Create(milestone_create(&v[2..])?))
        }
        ("milestone" | "m", Some("update")) => {
            let (milestone, rest) = required_positional_with_rest(&v[2..], "milestone")?;
            let mutation = milestone_update(rest)?;
            Command::Milestone(MilestoneCommand::Update {
                milestone,
                mutation,
            })
        }
        ("milestone" | "m", Some("delete")) => {
            let (milestone, rest) = required_positional_with_rest(&v[2..], "milestone")?;
            {
                validate_destructive(rest, &["--confirm"], &["--dry-run"])?;
                Command::Milestone(MilestoneCommand::Delete {
                    confirm: required(rest, "--confirm")?,
                    dry_run: flag(rest, "--dry-run")?,
                    milestone,
                })
            }
        }
        ("project", Some("list")) => Command::Project(ProjectCommand::List {
            team: option(&v[2..], "--team")?,
            page: project_page(&v[2..])?,
            open: project_open(&v[2..])?,
        }),
        ("project", Some("view")) => {
            let (project, rest) = required_positional_with_rest(&v[2..], "project")?;
            Command::Project(ProjectCommand::View {
                project,
                open: open_option(rest)?,
            })
        }
        ("project", Some("create")) => {
            Command::Project(ProjectCommand::Create(project_create(&v[2..])?))
        }
        ("issue", Some("mine" | "list" | "l")) => Command::Mine(list(&v[2..], true)?.0),
        ("issue", Some("query")) => {
            let (text, rest) = query_text(&v[2..])?;
            let (l, all) = list(rest, false)?;
            if all && l.team.is_some() {
                return Err(AppError::input("--all-teams conflicts with --team"));
            }
            Command::Query {
                text,
                list: l,
                all_teams: all,
            }
        }
        ("issue", Some("view")) => {
            let (issue, open) = issue_open(&v[2..])?;
            Command::View { issue, open }
        }
        ("issue", Some("pr" | "pull-request")) => {
            let (i, r) = issue_prefix(&v[2..])?;
            validate_options(
                r,
                &["--base", "--head", "--title", "-t"],
                &["--draft", "--web", "-w"],
            )?;
            Command::Pr {
                issue: i,
                base: option(r, "--base")?,
                head: option(r, "--head")?,
                draft: flag(r, "--draft")?,
                title: option2(r, "--title", "-t")?,
                web: flag(r, "--web")? || flag(r, "-w")?,
            }
        }
        ("issue", Some("id")) => Command::Scalar {
            field: Scalar::Id,
            issue: issue_only(&v[2..])?,
        },
        ("issue", Some("title")) => Command::Scalar {
            field: Scalar::Title,
            issue: issue_only(&v[2..])?,
        },
        ("issue", Some("url")) => Command::Scalar {
            field: Scalar::Url,
            issue: issue_only(&v[2..])?,
        },
        ("issue", Some("create")) => {
            let m = mutation(&v[2..])?;
            if m.unassign {
                return Err(AppError::input("--unassign is only valid for issue update"));
            }
            Command::Create(m)
        }
        ("issue", Some("update")) => {
            let (i, r) = issue_prefix(&v[2..])?;
            let m = mutation(r)?;
            if m == Mutation::default() {
                return Err(AppError::input("issue update requires a mutation flag"));
            }
            Command::Update {
                issue: i,
                mutation: m,
            }
        }
        ("issue", Some("delete")) => {
            let (i, r) = issue_prefix(&v[2..])?;
            validate_destructive(r, &["--confirm"], &["--dry-run"])?;
            Command::Delete {
                issue: i,
                confirm: required(r, "--confirm")?,
                dry_run: flag(r, "--dry-run")?,
            }
        }
        ("issue", Some("comment")) => Command::Comment(comment(&v[2..])?),
        ("issue", Some("start")) => Command::Start(issue_only(&v[2..])?),
        _ => {
            return Err(AppError::input(
                "unknown or incomplete command; run `workrus --help`",
            ));
        }
    };
    Ok(ParseResult::Command { json, command })
}
fn extra() -> AppError {
    AppError::input("too many arguments")
}
fn nonempty(s: &str, n: &str) -> Result<String, AppError> {
    if s.trim().is_empty() {
        Err(AppError::input(format!("{n} must not be empty")))
    } else {
        Ok(s.into())
    }
}
fn issue(s: &str) -> Result<IssueRef, AppError> {
    if s.bytes().all(|b| b.is_ascii_digit()) {
        let n = s
            .parse()
            .map_err(|_| AppError::input("issue number is invalid"))?;
        if n == 0 {
            return Err(AppError::input("issue number must be positive"));
        }
        Ok(IssueRef::Number(n))
    } else {
        Ok(IssueRef::Identifier(s.parse()?))
    }
}
fn issue_prefix(v: &[String]) -> Result<(Option<IssueRef>, &[String]), AppError> {
    if v.first().is_some_and(|x| !x.starts_with('-')) {
        Ok((Some(issue(&v[0])?), &v[1..]))
    } else {
        Ok((None, v))
    }
}
fn issue_only(v: &[String]) -> Result<Option<IssueRef>, AppError> {
    let (i, r) = issue_prefix(v)?;
    if r.is_empty() { Ok(i) } else { Err(extra()) }
}
fn issue_open(v: &[String]) -> Result<(Option<IssueRef>, Open), AppError> {
    let (i, r) = issue_prefix(v)?;
    match r {
        [] => Ok((i, Open::None)),
        [x] if x == "--web" || x == "-w" => Ok((i, Open::Web)),
        [x] if x == "--app" || x == "-a" => Ok((i, Open::App)),
        _ => Err(extra()),
    }
}
fn required_positional_with_rest<'a>(
    v: &'a [String],
    name: &str,
) -> Result<(String, &'a [String]), AppError> {
    let value = v
        .first()
        .filter(|x| !x.starts_with('-'))
        .ok_or_else(|| AppError::input(format!("{name} is required")))?;
    Ok((nonempty(value, name)?, &v[1..]))
}
fn open_option(v: &[String]) -> Result<Open, AppError> {
    match v {
        [] => Ok(Open::None),
        [x] if x == "--web" || x == "-w" => Ok(Open::Web),
        [x] if x == "--app" || x == "-a" => Ok(Open::App),
        _ => Err(extra()),
    }
}
fn project_open(v: &[String]) -> Result<Open, AppError> {
    let mut open = Open::None;
    let mut i = 0;
    while i < v.len() {
        match v[i].as_str() {
            "--web" | "-w" => {
                if open != Open::None {
                    return Err(AppError::input("duplicate open option"));
                }
                open = Open::Web;
                i += 1
            }
            "--app" | "-a" => {
                if open != Open::None {
                    return Err(AppError::input("duplicate open option"));
                }
                open = Open::App;
                i += 1
            }
            "--team" | "--limit" | "--after" => i += 2,
            _ => return Err(AppError::input(format!("unknown option {}", v[i]))),
        }
    }
    Ok(open)
}
fn team_list_page(v: &[String]) -> Result<Page, AppError> {
    let filtered: Vec<_> = v
        .iter()
        .filter(|x| !matches!(x.as_str(), "--web" | "-w" | "--app" | "-a"))
        .cloned()
        .collect();
    page(&filtered)
}
fn project_page(v: &[String]) -> Result<Page, AppError> {
    let mut args = Vec::new();
    let mut i = 0;
    while i < v.len() {
        match v[i].as_str() {
            "--team" => {
                i += 2;
            }
            "--web" | "-w" | "--app" | "-a" => i += 1,
            "--limit" | "--after" => {
                args.push(v[i].clone());
                i += 1;
                args.push(
                    v.get(i)
                        .ok_or_else(|| AppError::input("pagination option requires a value"))?
                        .clone(),
                );
                i += 1
            }
            _ => return Err(AppError::input(format!("unknown option {}", v[i]))),
        }
    }
    page(&args)
}
fn page(v: &[String]) -> Result<Page, AppError> {
    let mut p = Page {
        limit: Limit::Bounded(50),
        after: None,
    };
    let mut i = 0;
    let mut saw_limit = false;
    let mut saw_after = false;
    while i < v.len() {
        let f = &v[i];
        i += 1;
        let x = v
            .get(i)
            .ok_or_else(|| AppError::input(format!("{f} requires a value")))?;
        match f.as_str() {
            "--limit" => {
                if saw_limit {
                    return Err(AppError::input("duplicate option --limit"));
                }
                saw_limit = true;
                let n = x
                    .parse::<u16>()
                    .map_err(|_| AppError::input("--limit must be a non-negative integer"))?;
                p.limit = if n == 0 {
                    Limit::Unlimited
                } else {
                    Limit::Bounded(n)
                }
            }
            "--after" => {
                if saw_after {
                    return Err(AppError::input("duplicate option --after"));
                }
                saw_after = true;
                p.after = Some(nonempty(x, "cursor")?);
            }
            _ => return Err(AppError::input(format!("unknown option {f}"))),
        }
        i += 1
    }
    Ok(p)
}
fn optional_positional(v: &[String], name: &str) -> Result<Option<String>, AppError> {
    match v {
        [] => Ok(None),
        [value] if !value.starts_with('-') => Ok(Some(nonempty(value, name)?)),
        _ => Err(extra()),
    }
}
fn optional_positional_with_rest<'a>(
    v: &'a [String],
    name: &str,
) -> Result<(Option<String>, &'a [String]), AppError> {
    if v.first().is_some_and(|value| !value.starts_with('-')) {
        Ok((Some(nonempty(&v[0], name)?), &v[1..]))
    } else {
        Ok((None, v))
    }
}
fn users_page(v: &[String]) -> Result<(bool, Page), AppError> {
    let mut all = false;
    let mut page_args = Vec::new();
    let mut i = 0;
    while i < v.len() {
        if v[i] == "--all" {
            if all {
                return Err(AppError::input("duplicate option --all"));
            }
            all = true;
            i += 1;
        } else {
            page_args.push(v[i].clone());
            i += 1;
            if let Some(value) = v.get(i) {
                page_args.push(value.clone());
                i += 1;
            } else {
                return Err(AppError::input(format!(
                    "{} requires a value",
                    page_args.last().expect("flag")
                )));
            }
        }
    }
    Ok((all, page(&page_args)?))
}
fn document_list(v: &[String]) -> Result<DocumentCommand, AppError> {
    let target = document_target(v)?;
    let mut args = Vec::new();
    let mut i = 0;
    while i < v.len() {
        match v[i].as_str() {
            "--project" | "--issue" => i += 2,
            "--limit" | "--after" => {
                args.push(v[i].clone());
                i += 1;
                args.push(
                    v.get(i)
                        .ok_or_else(|| AppError::input("pagination option requires a value"))?
                        .clone(),
                );
                i += 1
            }
            x => return Err(AppError::input(format!("unknown option {x}"))),
        }
    }
    Ok(DocumentCommand::List {
        target,
        page: page(&args)?,
    })
}
fn document_view(v: &[String]) -> Result<DocumentCommand, AppError> {
    let (document, rest) = required_positional_with_rest(v, "document")?;
    let raw = flag(rest, "--raw")?;
    let web = flag(rest, "--web")? || flag(rest, "-w")?;
    if raw && web {
        return Err(AppError::input("--raw conflicts with --web"));
    }
    if rest
        .iter()
        .any(|x| x != "--raw" && x != "--web" && x != "-w")
    {
        return Err(extra());
    }
    Ok(DocumentCommand::View { document, raw, web })
}
fn document_target(v: &[String]) -> Result<DocumentTarget, AppError> {
    if let Some(flag) = v.iter().find(|x| {
        matches!(
            x.as_str(),
            "--team" | "--cycle" | "--initiative" | "--release"
        )
    }) {
        return Err(AppError::input(format!(
            "document target {} is unsupported: Linear exposes it as an internal association",
            flag
        )));
    }
    let project = option(v, "--project")?;
    let issue_value = option(v, "--issue")?;
    if project.is_some() && issue_value.is_some() {
        return Err(AppError::input("--project conflicts with --issue"));
    }
    if let Some(p) = project {
        Ok(DocumentTarget::Project(p))
    } else if let Some(i) = issue_value {
        Ok(DocumentTarget::Issue(issue(&i)?))
    } else {
        Ok(DocumentTarget::None)
    }
}
fn document_mutation(v: &[String], create: bool) -> Result<DocumentMutation, AppError> {
    let mut m = DocumentMutation {
        title: option(v, "--title")?,
        content: None,
        target: document_target(v)?,
    };
    let file = option(v, "--content-file")?;
    let stdin = flag(v, "--stdin")?;
    if file.is_some() && stdin {
        return Err(AppError::input("--content-file conflicts with --stdin"));
    }
    m.content = file
        .map(ContentSource::File)
        .or_else(|| stdin.then_some(ContentSource::Stdin));
    if create && m.title.is_none() {
        return Err(AppError::input("--title is required"));
    }
    if create && matches!(m.target, DocumentTarget::None) {
        return Err(AppError::input(
            "document create requires exactly one --project or --issue",
        ));
    }
    if !create
        && m.title.is_none()
        && m.content.is_none()
        && matches!(m.target, DocumentTarget::None)
    {
        return Err(AppError::input("document update requires a mutation flag"));
    }
    let mut i = 0;
    while i < v.len() {
        match v[i].as_str() {
            "--title" | "--project" | "--issue" | "--content-file" => i += 2,
            "--stdin" | "--force" => i += 1,
            "--permanent" => {
                return Err(AppError::input(
                    "document permanent delete is unsupported; Linear's public API only trashes documents",
                ));
            }
            x => return Err(AppError::input(format!("unknown option {x}"))),
        }
    }
    Ok(m)
}
fn document_delete(v: &[String]) -> Result<DocumentCommand, AppError> {
    if v.iter().any(|x| x == "--permanent") {
        return Err(AppError::input(
            "document permanent delete is unsupported; Linear's public API only trashes documents",
        ));
    }
    let dry_run = flag(v, "--dry-run")?;
    let bulk = flag(v, "--bulk")?;
    let mut docs = Vec::new();
    let mut confirms = Vec::new();
    let mut i = 0;
    while i < v.len() {
        match v[i].as_str() {
            "--dry-run" => i += 1,
            "--bulk" => {
                i += 1;
                while let Some(x) = v.get(i) {
                    if x.starts_with("--") {
                        break;
                    }
                    docs.push(nonempty(x, "document")?);
                    i += 1;
                }
            }
            "--confirm" => {
                i += 1;
                while let Some(x) = v.get(i) {
                    if x.starts_with("--") {
                        break;
                    }
                    confirms.push(nonempty(x, "confirmation")?);
                    i += 1;
                }
            }
            x if !x.starts_with('-') && !bulk && docs.is_empty() => {
                docs.push(nonempty(x, "document")?);
                i += 1
            }
            x => return Err(AppError::input(format!("unknown option {x}"))),
        }
    }
    if docs.is_empty() {
        return Err(AppError::input(
            "document delete requires DOCUMENT or --bulk DOCUMENT...",
        ));
    }
    if confirms.len() != docs.len() {
        return Err(AppError::input(
            "--confirm must be supplied exactly once for each document",
        ));
    }
    Ok(DocumentCommand::Delete {
        documents: docs,
        confirms,
        dry_run,
    })
}
fn milestone_page(v: &[String]) -> Result<Page, AppError> {
    let mut args = Vec::new();
    let mut i = 0;
    while i < v.len() {
        match v[i].as_str() {
            "--project" => {
                i += 2;
            }
            "--limit" | "--after" => {
                args.push(v[i].clone());
                i += 1;
                args.push(
                    v.get(i)
                        .ok_or_else(|| AppError::input("pagination option requires a value"))?
                        .clone(),
                );
                i += 1;
            }
            _ => return Err(AppError::input(format!("unknown option {}", v[i]))),
        }
    }
    page(&args)
}
fn milestone_create(v: &[String]) -> Result<MilestoneMutation, AppError> {
    let m = milestone_options(v, true)?;
    if m.project.is_none() {
        return Err(AppError::input("--project is required"));
    }
    Ok(m)
}
fn milestone_update(v: &[String]) -> Result<MilestoneMutation, AppError> {
    let m = milestone_options(v, false)?;
    if m == MilestoneMutation::default() {
        return Err(AppError::input("milestone update requires a mutation flag"));
    }
    Ok(m)
}
fn milestone_options(v: &[String], allow_project: bool) -> Result<MilestoneMutation, AppError> {
    let mut m = MilestoneMutation::default();
    let mut i = 0;
    while i < v.len() {
        let flag = &v[i];
        i += 1;
        let value = nonempty(
            v.get(i)
                .ok_or_else(|| AppError::input(format!("{flag} requires a value")))?,
            flag,
        )?;
        match flag.as_str() {
            "--project" if allow_project => set(&mut m.project, value, flag)?,
            "--name" => set(&mut m.name, value, flag)?,
            "--description" => set(&mut m.description, value, flag)?,
            "--target-date" => {
                if !crate::model::valid_date(&value) {
                    return Err(AppError::input(
                        "--target-date must be a valid YYYY-MM-DD date",
                    ));
                }
                set(&mut m.target_date, value, flag)?
            }
            "--project" => {
                return Err(AppError::input(
                    "--project is only valid for milestone create",
                ));
            }
            _ => return Err(AppError::input(format!("unknown option {flag}"))),
        };
        i += 1;
    }
    Ok(m)
}
fn project_create(v: &[String]) -> Result<ProjectCreate, AppError> {
    let mut result = ProjectCreate {
        name: required(v, "--name")?,
        ..Default::default()
    };
    let mut i = 0;
    while i < v.len() {
        let flag = &v[i];
        i += 1;
        let value = v
            .get(i)
            .ok_or_else(|| AppError::input(format!("{flag} requires a value")))?;
        match flag.as_str() {
            "--name" => {
                if result.name != *value {
                    return Err(AppError::input("duplicate option --name"));
                }
            }
            "--team" => result.teams.push(nonempty(value, "team")?),
            "--description" => set(
                &mut result.description,
                nonempty(value, "description")?,
                flag,
            )?,
            "--content-file" => set(
                &mut result.content_file,
                nonempty(value, "content file")?,
                flag,
            )?,
            "--lead" => set(&mut result.lead, nonempty(value, "lead")?, flag)?,
            "--member" => result.members.push(nonempty(value, "member")?),
            "--target-date" => {
                let date = nonempty(value, "target date")?;
                if !crate::model::valid_date(&date) {
                    return Err(AppError::input(
                        "--target-date must be a valid YYYY-MM-DD date",
                    ));
                }
                set(&mut result.target_date, date, flag)?;
            }
            "--priority" | "--label" => {
                return Err(AppError::input(format!(
                    "{flag} is unsupported: Linear's public ProjectCreateInput does not confirm this field"
                )));
            }
            _ => return Err(AppError::input(format!("unknown option {flag}"))),
        }
        i += 1;
    }
    if result.teams.is_empty() {
        return Err(AppError::input(
            "project create requires at least one --team TEAM",
        ));
    }
    Ok(result)
}
fn team_create(v: &[String]) -> Result<TeamCreate, AppError> {
    let name = required(v, "--name")?;
    for flag_name in ["--name", "--key", "--description", "--color", "--icon"] {
        let _ = option(v, flag_name)?;
    }
    // Validate that every token belongs to a supported scalar option.
    let mut i = 0;
    while i < v.len() {
        if !matches!(
            v[i].as_str(),
            "--name" | "--key" | "--description" | "--color" | "--icon"
        ) {
            return Err(AppError::input(format!("unknown option {}", v[i])));
        }
        i += 2;
    }
    Ok(TeamCreate {
        name,
        key: option(v, "--key")?,
        description: option(v, "--description")?,
        color: option(v, "--color")?,
        icon: option(v, "--icon")?,
    })
}
fn list(v: &[String], mine: bool) -> Result<(List, bool), AppError> {
    let mut l = List {
        page: Page {
            limit: Limit::Bounded(50),
            after: None,
        },
        ..Default::default()
    };
    let mut all = false;
    let mut i = 0;
    while i < v.len() {
        let flag = &v[i];
        if flag == "--all-teams" && !mine {
            if all {
                return Err(AppError::input("duplicate option --all-teams"));
            }
            all = true;
            i += 1;
            continue;
        }
        if matches!(flag.as_str(), "--web" | "-w" | "--app" | "-a") {
            if l.open != Open::None {
                return Err(AppError::input("duplicate open option"));
            }
            l.open = if matches!(flag.as_str(), "--web" | "-w") {
                Open::Web
            } else {
                Open::App
            };
            i += 1;
            continue;
        }
        i += 1;
        let value = nonempty(
            v.get(i)
                .ok_or_else(|| AppError::input(format!("{flag} requires a value")))?,
            flag,
        )?;
        match flag.as_str() {
            "--team" => set(&mut l.team, value, flag)?,
            "--state" | "-s" => set(&mut l.state, value, flag)?,
            "--sort" => {
                if !matches!(value.as_str(), "manual" | "priority") {
                    return Err(AppError::input("--sort must be manual or priority"));
                }
                set(&mut l.sort, value, flag)?;
            }
            "--project" => set(&mut l.project, value, flag)?,
            "--milestone" => set(&mut l.milestone, value, flag)?,
            "--limit" => {
                if l.page.limit != Limit::Bounded(50) {
                    return Err(AppError::input("duplicate option --limit"));
                }
                let n = value
                    .parse::<u16>()
                    .map_err(|_| AppError::input("--limit must be a non-negative integer"))?;
                l.page.limit = if n == 0 {
                    Limit::Unlimited
                } else {
                    Limit::Bounded(n)
                };
            }
            "--after" => set(&mut l.page.after, value, flag)?,
            _ => return Err(AppError::input(format!("unknown option {flag}"))),
        }
        i += 1;
    }
    Ok((l, all))
}
fn query_text(v: &[String]) -> Result<(String, &[String]), AppError> {
    if v.first().is_some_and(|x| x == "--search") {
        return Ok((
            nonempty(
                v.get(1)
                    .ok_or_else(|| AppError::input("--search requires a value"))?,
                "query",
            )?,
            &v[2..],
        ));
    }
    let s = v
        .first()
        .ok_or_else(|| AppError::input("issue query requires TEXT or --search"))?;
    if s.starts_with('-') {
        return Err(AppError::input("issue query requires TEXT or --search"));
    }
    Ok((nonempty(s, "query")?, &v[1..]))
}
fn validate_options(v: &[String], valued: &[&str], flags: &[&str]) -> Result<(), AppError> {
    let mut i = 0;
    while i < v.len() {
        let token = v[i].as_str();
        if valued.contains(&token) {
            i += 2;
            if i > v.len() {
                return Err(AppError::input(format!("{token} requires a value")));
            }
        } else if flags.contains(&token) {
            i += 1;
        } else {
            return Err(AppError::input(format!("unknown option {token}")));
        }
    }
    Ok(())
}
fn validate_destructive(v: &[String], valued: &[&str], flags: &[&str]) -> Result<(), AppError> {
    validate_options(v, valued, flags)
}
fn option(v: &[String], name: &str) -> Result<Option<String>, AppError> {
    option2(v, name, name)
}
fn option2(v: &[String], a: &str, b: &str) -> Result<Option<String>, AppError> {
    let mut found = None;
    let mut i = 0;
    while i < v.len() {
        if v[i] == a || v[i] == b {
            if found.is_some() {
                return Err(AppError::input(format!("duplicate option {a}")));
            }
            i += 1;
            found = Some(nonempty(
                v.get(i)
                    .ok_or_else(|| AppError::input(format!("{a} requires a value")))?,
                a,
            )?)
        }
        i += 1
    }
    Ok(found)
}
fn required(v: &[String], name: &str) -> Result<String, AppError> {
    option(v, name)?.ok_or_else(|| AppError::input(format!("{name} is required")))
}
fn flag(v: &[String], name: &str) -> Result<bool, AppError> {
    let n = v.iter().filter(|x| x.as_str() == name).count();
    if n > 1 {
        Err(AppError::input(format!("duplicate option {name}")))
    } else {
        Ok(n == 1)
    }
}
fn mutation(v: &[String]) -> Result<Mutation, AppError> {
    let mut m = Mutation::default();
    let mut i = 0;
    while i < v.len() {
        let f = &v[i];
        match f.as_str() {
            "--assignee" => {
                i += 1;
                if v.get(i).is_some_and(|x| x == "self") {
                    m.assignee_self = true
                } else {
                    return Err(AppError::input("--assignee must be self"));
                }
            }
            "--unassign" => m.unassign = true,
            "--remove-project" => m.remove_project = true,
            "--remove-milestone" => m.remove_milestone = true,
            _ => {
                i += 1;
                let x = nonempty(
                    v.get(i)
                        .ok_or_else(|| AppError::input(format!("{f} requires a value")))?,
                    f,
                )?;
                match f.as_str() {
                    "--title" | "-t" => set(&mut m.title, x, f)?,
                    "--description" | "-d" => set(&mut m.description, x, f)?,
                    "--team" => set(&mut m.team, x, f)?,
                    "--state" | "-s" => set(&mut m.state, x, f)?,
                    "--project" => set(&mut m.project, x, f)?,
                    "--milestone" => set(&mut m.milestone, x, f)?,
                    "--priority" => {
                        let priority = x
                            .parse()
                            .map_err(|_| AppError::input("--priority must be 0 through 4"))?;
                        if priority > 4 {
                            return Err(AppError::input("--priority must be 0 through 4"));
                        }
                        if m.priority.replace(priority).is_some() {
                            return Err(AppError::input("duplicate option --priority"));
                        }
                    }
                    _ => return Err(AppError::input(format!("unknown option {f}"))),
                }
            }
        }
        i += 1;
    }
    if m.assignee_self && m.unassign {
        return Err(AppError::input("--unassign conflicts with --assignee"));
    }
    if m.project.is_some() && m.remove_project {
        return Err(AppError::input("--remove-project conflicts with --project"));
    }
    if m.milestone.is_some() && m.remove_milestone {
        return Err(AppError::input(
            "--remove-milestone conflicts with --milestone",
        ));
    }
    Ok(m)
}
fn set(slot: &mut Option<String>, value: String, flag: &str) -> Result<(), AppError> {
    if slot.replace(value).is_some() {
        Err(AppError::input(format!("duplicate option {flag}")))
    } else {
        Ok(())
    }
}
fn comment(v: &[String]) -> Result<CommentCommand, AppError> {
    match v.first().map(String::as_str) {
        Some("list") => {
            let (i, r) = issue_prefix(&v[1..])?;
            Ok(CommentCommand::List {
                issue: i,
                page: page(r)?,
            })
        }
        Some("add") => {
            let (i, r) = issue_prefix(&v[1..])?;
            Ok(CommentCommand::Add {
                issue: i,
                body: required(r, "--body")?,
                parent: option2(r, "--parent", "-p")?,
            })
        }
        Some("update") => {
            let id = v
                .get(1)
                .ok_or_else(|| AppError::input("comment update requires COMMENT_ID"))?;
            Ok(CommentCommand::Update {
                id: nonempty(id, "comment id")?,
                body: required(&v[2..], "--body")?,
            })
        }
        Some("delete") => {
            let id = nonempty(
                v.get(1)
                    .ok_or_else(|| AppError::input("comment delete requires COMMENT_ID"))?,
                "comment id",
            )?;
            let r = &v[2..];
            validate_destructive(r, &["--confirm"], &["--dry-run"])?;
            Ok(CommentCommand::Delete {
                confirm: required(r, "--confirm")?,
                dry_run: flag(r, "--dry-run")?,
                id,
            })
        }
        _ => Err(AppError::input(
            "unknown or incomplete issue comment command",
        )),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn destructive_commands_reject_unknown_tokens_and_list_options_do_not_overwrite() {
        assert!(
            parse([
                "issue",
                "delete",
                "ENG-1",
                "--confirm",
                "ENG-1",
                "--unexpected"
            ])
            .is_err()
        );
        assert!(
            parse([
                "issue",
                "comment",
                "delete",
                "c1",
                "--confirm",
                "c1",
                "oops"
            ])
            .is_err()
        );
        assert!(parse(["milestone", "delete", "m1", "--confirm", "m1", "oops"]).is_err());
        assert!(
            parse([
                "issue", "list", "--after", "cursor", "--limit", "2", "--limit", "3"
            ])
            .is_err()
        );
        assert!(parse(["issue", "list", "--team", "ENG", "--team", "OPS"]).is_err());
        assert!(matches!(
            parse(["issue", "pr", "ENG-1", "-w"]).unwrap(),
            ParseResult::Command {
                command: Command::Pr { web: true, .. },
                ..
            }
        ));
    }
    #[test]
    fn team_and_user_parse() {
        assert!(
            matches!(parse(["team", "id", "ENG"]).unwrap(), ParseResult::Command { command: Command::TeamId(Some(key)), .. } if key == "ENG")
        );
        assert!(matches!(
            parse(["team", "members", "ENG", "--all", "--limit", "0"]).unwrap(),
            ParseResult::Command {
                command: Command::TeamMembers {
                    include_disabled: true,
                    page: Page {
                        limit: Limit::Unlimited,
                        ..
                    },
                    ..
                },
                ..
            }
        ));
        assert!(matches!(
            parse(["user", "list", "--all", "--after", "cursor"]).unwrap(),
            ParseResult::Command {
                command: Command::UserList {
                    include_disabled: true,
                    ..
                },
                ..
            }
        ));
        assert!(matches!(
            parse([
                "team",
                "create",
                "--name",
                "Engineering",
                "--key",
                "ENG",
                "--color",
                "#123456"
            ])
            .unwrap(),
            ParseResult::Command {
                command: Command::TeamCreate(_),
                ..
            }
        ));
        assert!(parse(["team", "create", "--key", "ENG"]).is_err());
        assert!(parse(["team", "members", "--all", "--all"]).is_err());
    }
    #[test]
    fn mutation_and_comment_parse() {
        assert!(matches!(
            parse([
                "issue",
                "create",
                "-t",
                "x",
                "-d",
                "d",
                "--project",
                "p",
                "--milestone",
                "m",
                "--priority",
                "4"
            ])
            .unwrap(),
            ParseResult::Command {
                command: Command::Create(_),
                ..
            }
        ));
        assert!(
            parse([
                "issue",
                "delete",
                "ENG-1",
                "--confirm",
                "ENG-1",
                "--dry-run"
            ])
            .is_ok()
        );
        assert!(parse(["issue", "comment", "add", "ENG-1", "--body", "x", "-p", "c"]).is_ok());
        assert!(parse(["issue", "comment", "delete", "x", "--confirm", "x"]).is_ok());
    }
    #[test]
    fn milestone_parser_alias_dates_and_confirmation() {
        assert!(matches!(
            parse(["m", "list", "--project", "Roadmap", "--limit", "0"]),
            Ok(ParseResult::Command {
                command: Command::Milestone(MilestoneCommand::List {
                    page: Page {
                        limit: Limit::Unlimited,
                        ..
                    },
                    ..
                }),
                ..
            })
        ));
        assert!(
            parse([
                "milestone",
                "create",
                "--project",
                "p",
                "--name",
                "M",
                "--target-date",
                "2027-02-29"
            ])
            .is_err()
        );
        assert!(parse(["m", "delete", "id", "--confirm", "id", "--dry-run"]).is_ok());
    }
    #[test]
    fn document_parser_enforces_public_targets_content_and_confirmations() {
        assert!(matches!(
            parse([
                "docs", "create", "--title", "Notes", "--issue", "ENG-1", "--stdin"
            ]),
            Ok(ParseResult::Command {
                command: Command::Document(DocumentCommand::Create(_)),
                ..
            })
        ));
        assert!(matches!(
            parse(["document", "list", "--project", "p", "--limit", "0"]),
            Ok(ParseResult::Command {
                command: Command::Document(DocumentCommand::List {
                    page: Page {
                        limit: Limit::Unlimited,
                        ..
                    },
                    ..
                }),
                ..
            })
        ));
        assert!(parse(["docs", "create", "--title", "x", "--team", "ENG"]).is_err());
        assert!(
            parse([
                "docs",
                "create",
                "--title",
                "x",
                "--issue",
                "ENG-1",
                "--stdin",
                "--content-file",
                "x"
            ])
            .is_err()
        );
        assert!(parse(["docs", "delete", "--bulk", "a", "b", "--confirm", "a", "b"]).is_ok());
        assert!(parse(["docs", "delete", "a", "--confirm", "a", "--permanent"]).is_err());
    }
    #[test]
    fn completion_parser_and_snapshots_are_stable() {
        for (shell, expected) in [
            (
                CompletionShell::Bash,
                include_str!("../tests/completions/workrus.bash"),
            ),
            (
                CompletionShell::Zsh,
                include_str!("../tests/completions/_workrus"),
            ),
            (
                CompletionShell::Fish,
                include_str!("../tests/completions/workrus.fish"),
            ),
            (
                CompletionShell::Powershell,
                include_str!("../tests/completions/workrus.ps1"),
            ),
        ] {
            assert_eq!(completion_script(shell), expected);
        }
        assert!(matches!(
            parse(["completion", "bash"]),
            Ok(ParseResult::Command {
                command: Command::Completion(CompletionShell::Bash),
                json: false
            })
        ));
        assert!(matches!(
            parse(["completions", "zsh"]),
            Ok(ParseResult::Command {
                command: Command::Completion(CompletionShell::Zsh),
                json: false
            })
        ));
        assert!(parse(["completion", "bash", "extra"]).is_err());
        assert!(parse(["--json", "completion", "bash"]).is_err());
    }
    #[test]
    fn project_parser_validates_repeated_teams_dates_and_unsupported_fields() {
        assert!(matches!(
            parse([
                "project",
                "create",
                "--name",
                "Roadmap",
                "--team",
                "ENG",
                "--team",
                "DES",
                "--target-date",
                "2028-02-29",
                "--member",
                "Ada"
            ]),
            Ok(ParseResult::Command {
                command: Command::Project(ProjectCommand::Create(_)),
                ..
            })
        ));
        assert!(
            parse([
                "project",
                "create",
                "--name",
                "Roadmap",
                "--team",
                "ENG",
                "--target-date",
                "2027-02-29"
            ])
            .is_err()
        );
        assert!(
            parse([
                "project",
                "create",
                "--name",
                "Roadmap",
                "--team",
                "ENG",
                "--priority",
                "1"
            ])
            .is_err()
        );
        assert!(matches!(
            parse(["project", "list", "--team", "ENG", "--limit", "0", "--web"]),
            Ok(ParseResult::Command {
                command: Command::Project(ProjectCommand::List {
                    page: Page {
                        limit: Limit::Unlimited,
                        ..
                    },
                    open: Open::Web,
                    ..
                }),
                ..
            })
        ));
    }
}
