use crate::error::AppError;
use atomic_write_file::AtomicWriteFile;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env, fmt, fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

const MAX_CONFIG_BYTES: u64 = 64 * 1024;

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ConfigFile {
    team_id: Option<String>,
    workspace: Option<String>,
    issue_sort: Option<IssueSort>,
    issue_create_ask_project: Option<bool>,
    issue_create_assign_self: Option<AssignSelf>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum IssueSort {
    Manual,
    Priority,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AssignSelf {
    Always,
    Auto,
    Never,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeConfig {
    pub team_id: Option<String>,
    pub workspace: Option<String>,
    pub issue_sort: Option<IssueSort>,
    pub issue_create_ask_project: Option<bool>,
    pub issue_create_assign_self: Option<AssignSelf>,
}

/// An API key intentionally cannot reveal its value through formatting traits.
pub struct ApiKey(String);

impl ApiKey {
    pub fn authorization_value(&self) -> &str {
        &self.0
    }
    #[cfg(test)]
    pub(crate) fn for_test(value: &str) -> Self {
        Self(value.to_owned())
    }
}
impl fmt::Debug for ApiKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ApiKey([REDACTED])")
    }
}
impl fmt::Display for ApiKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("[REDACTED]")
    }
}

pub fn api_key_from_env() -> Result<ApiKey, AppError> {
    let value =
        env::var("LINEAR_API_KEY").map_err(|_| AppError::input("LINEAR_API_KEY must be set"))?;
    if value.trim().is_empty() || value.bytes().any(|byte| byte.is_ascii_control()) {
        return Err(AppError::input(
            "LINEAR_API_KEY is invalid; set a non-empty header-safe value",
        ));
    }
    Ok(ApiKey(value))
}

pub fn global_config_path() -> Result<PathBuf, AppError> {
    global_config_path_from(&environment())
}

fn global_config_path_from(values: &HashMap<String, String>) -> Result<PathBuf, AppError> {
    #[cfg(windows)]
    let base = values
        .get("APPDATA")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| AppError::input("APPDATA must be set to locate global configuration"))?;
    #[cfg(not(windows))]
    let base = values
        .get("XDG_CONFIG_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            values
                .get("HOME")
                .filter(|value| !value.is_empty())
                .map(|value| PathBuf::from(value).join(".config"))
        })
        .ok_or_else(|| {
            AppError::input("XDG_CONFIG_HOME or HOME must be set to locate global configuration")
        })?;
    Ok(base.join("linear").join("linear.toml"))
}

fn environment() -> HashMap<String, String> {
    env::vars().collect()
}

pub fn runtime_config() -> Result<RuntimeConfig, AppError> {
    let values = environment();
    runtime_config_from(
        &values,
        read_file(&global_config_path_from(&values)?)?.unwrap_or_default(),
    )
}

fn runtime_config_from(
    values: &HashMap<String, String>,
    file: ConfigFile,
) -> Result<RuntimeConfig, AppError> {
    let team_id = override_value(values, "LINEAR_TEAM_ID")?.or(file.team_id);
    let workspace = override_value(values, "LINEAR_WORKSPACE")?.or(file.workspace);
    let issue_sort = enum_override(values, "LINEAR_ISSUE_SORT")?.or(file.issue_sort);
    let issue_create_ask_project =
        bool_override(values, "LINEAR_ISSUE_CREATE_ASK_PROJECT")?.or(file.issue_create_ask_project);
    let issue_create_assign_self = assign_override(values)?.or(file.issue_create_assign_self);
    if let Some(team) = &team_id {
        validate_team(team)?;
    }
    if let Some(workspace) = &workspace {
        validate_workspace(workspace)?;
    }
    Ok(RuntimeConfig {
        team_id,
        workspace,
        issue_sort,
        issue_create_ask_project,
        issue_create_assign_self,
    })
}

fn override_value(values: &HashMap<String, String>, key: &str) -> Result<Option<String>, AppError> {
    match values.get(key) {
        None => Ok(None),
        Some(value) if value.is_empty() => Err(AppError::input(format!("{key} must not be empty"))),
        Some(value) => Ok(Some(value.clone())),
    }
}
fn enum_override(
    values: &HashMap<String, String>,
    key: &str,
) -> Result<Option<IssueSort>, AppError> {
    override_value(values, key)?
        .map(|value| match value.as_str() {
            "manual" => Ok(IssueSort::Manual),
            "priority" => Ok(IssueSort::Priority),
            _ => Err(AppError::input(
                "LINEAR_ISSUE_SORT must be manual or priority",
            )),
        })
        .transpose()
}
fn bool_override(values: &HashMap<String, String>, key: &str) -> Result<Option<bool>, AppError> {
    override_value(values, key)?
        .map(|value| match value.as_str() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(AppError::input(format!("{key} must be true or false"))),
        })
        .transpose()
}
fn assign_override(values: &HashMap<String, String>) -> Result<Option<AssignSelf>, AppError> {
    override_value(values, "LINEAR_ISSUE_CREATE_ASSIGN_SELF")?
        .map(|value| match value.as_str() {
            "always" => Ok(AssignSelf::Always),
            "auto" => Ok(AssignSelf::Auto),
            "never" => Ok(AssignSelf::Never),
            _ => Err(AppError::input(
                "LINEAR_ISSUE_CREATE_ASSIGN_SELF must be always, auto, or never",
            )),
        })
        .transpose()
}

pub fn resolve_team(
    cli_team: Option<&str>,
    configured_team: Option<String>,
) -> Result<String, AppError> {
    if let Some(team) = cli_team {
        validate_team(team)?;
        return Ok(team.to_owned());
    }
    configured_team.ok_or_else(|| AppError::input("no default team configured; run `workrus config TEAM`, set LINEAR_TEAM_ID, or pass --team"))
}

pub fn default_team() -> Result<Option<String>, AppError> {
    Ok(runtime_config()?.team_id)
}

pub fn write_default_team(team: &str) -> Result<(), AppError> {
    validate_team(team)?;
    let path = global_config_path()?;
    let mut file = read_file(&path)?.unwrap_or_default();
    file.team_id = Some(team.to_owned());
    write_file(&path, &file)
}

#[cfg(unix)]
fn open_read_nofollow(path: &Path) -> std::io::Result<fs::File> {
    use std::os::unix::fs::OpenOptionsExt;
    fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
}

#[cfg(windows)]
fn open_read_nofollow(path: &Path) -> std::io::Result<fs::File> {
    use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;

    let file = fs::OpenOptions::new()
        .read(true)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)?;
    if file.metadata()?.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "refusing to follow a reparse point",
        ));
    }
    Ok(file)
}

#[cfg(not(any(unix, windows)))]
fn open_read_nofollow(path: &Path) -> std::io::Result<fs::File> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "refusing to follow a symlink",
        ));
    }
    fs::File::open(path)
}

fn read_file(path: &Path) -> Result<Option<ConfigFile>, AppError> {
    let directory = path.parent().expect("global config has parent");
    match fs::symlink_metadata(directory) {
        Ok(metadata) if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() => {}
        Ok(_) => {
            return Err(AppError::input(format!(
                "invalid configuration directory: {}",
                directory.display()
            )));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(AppError::operational(format!(
                "cannot inspect {}: {error}",
                directory.display()
            )));
        }
    }
    let file = match open_read_nofollow(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) if error.kind() == std::io::ErrorKind::InvalidInput => {
            return Err(AppError::input(format!(
                "invalid configuration file: {}",
                path.display()
            )));
        }
        Err(error) => {
            return Err(AppError::operational(format!(
                "cannot read {}: {error}",
                path.display()
            )));
        }
    };
    let metadata = file.metadata().map_err(|error| {
        AppError::operational(format!("cannot inspect {}: {error}", path.display()))
    })?;
    if !metadata.is_file() || metadata.len() > MAX_CONFIG_BYTES {
        return Err(AppError::input(format!(
            "invalid configuration file: {}",
            path.display()
        )));
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take(MAX_CONFIG_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| {
            AppError::operational(format!("cannot read {}: {error}", path.display()))
        })?;
    if bytes.len() as u64 > MAX_CONFIG_BYTES {
        return Err(AppError::input(format!(
            "invalid configuration file: {}",
            path.display()
        )));
    }
    let input = String::from_utf8(bytes)
        .map_err(|_| AppError::input(format!("invalid configuration file: {}", path.display())))?;
    toml::from_str(&input)
        .map(Some)
        .map_err(|_| AppError::input(format!("invalid configuration file: {}", path.display())))
}

fn write_file(path: &Path, config: &ConfigFile) -> Result<(), AppError> {
    let directory = path.parent().expect("global config has parent");
    match fs::symlink_metadata(directory) {
        Ok(metadata) if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() => {}
        Ok(_) => {
            return Err(AppError::input(format!(
                "invalid configuration directory: {}",
                directory.display()
            )));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => fs::create_dir_all(directory)
            .map_err(|error| {
                AppError::operational(format!("cannot create {}: {error}", directory.display()))
            })?,
        Err(error) => {
            return Err(AppError::operational(format!(
                "cannot inspect {}: {error}",
                directory.display()
            )));
        }
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(directory, fs::Permissions::from_mode(0o700)).map_err(|error| {
            AppError::operational(format!("cannot secure {}: {error}", directory.display()))
        })?;
    }
    if let Ok(metadata) = fs::symlink_metadata(path)
        && (!metadata.file_type().is_file() || metadata.file_type().is_symlink())
    {
        return Err(AppError::input(format!(
            "invalid configuration file: {}",
            path.display()
        )));
    }
    let content = toml::to_string(config)
        .map_err(|_| AppError::operational("could not encode configuration"))?;
    let mut destination = AtomicWriteFile::open(path).map_err(|error| {
        AppError::operational(format!("cannot prepare {}: {error}", path.display()))
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        destination
            .set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(|error| {
                AppError::operational(format!("cannot secure {}: {error}", path.display()))
            })?;
    }
    destination.write_all(content.as_bytes()).map_err(|error| {
        AppError::operational(format!("cannot write {}: {error}", path.display()))
    })?;
    destination.commit().map_err(|error| {
        AppError::operational(format!("cannot install {}: {error}", path.display()))
    })
}

fn validate_team(team: &str) -> Result<(), AppError> {
    if team.is_empty()
        || team.trim() != team
        || !team
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return Err(AppError::input(
            "team must contain only ASCII letters, digits, `_`, or `-` without surrounding whitespace",
        ));
    }
    Ok(())
}
fn validate_workspace(workspace: &str) -> Result<(), AppError> {
    if workspace.is_empty()
        || workspace.len() > 63
        || !workspace
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        || workspace.starts_with('-')
        || workspace.ends_with('-')
    {
        return Err(AppError::input("workspace must be a lowercase URL slug"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn global_path_uses_xdg_then_home_without_repository_fallback() {
        let values = HashMap::from([
            (String::from("XDG_CONFIG_HOME"), String::from("/xdg")),
            (String::from("HOME"), String::from("/home/a")),
        ]);
        assert_eq!(
            global_config_path_from(&values).unwrap(),
            PathBuf::from("/xdg/linear/linear.toml")
        );
        let values = HashMap::from([(String::from("HOME"), String::from("/home/a"))]);
        assert_eq!(
            global_config_path_from(&values).unwrap(),
            PathBuf::from("/home/a/.config/linear/linear.toml")
        );
        assert!(global_config_path_from(&HashMap::new()).is_err());
    }
    #[test]
    fn environment_overrides_toml_and_validates_supported_values() {
        let file = ConfigFile {
            team_id: Some("ENG".into()),
            workspace: Some("old".into()),
            issue_sort: Some(IssueSort::Manual),
            issue_create_ask_project: Some(false),
            issue_create_assign_self: Some(AssignSelf::Never),
        };
        let values = HashMap::from([
            (String::from("LINEAR_TEAM_ID"), String::from("OPS")),
            (String::from("LINEAR_WORKSPACE"), String::from("new-space")),
            (String::from("LINEAR_ISSUE_SORT"), String::from("priority")),
            (
                String::from("LINEAR_ISSUE_CREATE_ASK_PROJECT"),
                String::from("true"),
            ),
            (
                String::from("LINEAR_ISSUE_CREATE_ASSIGN_SELF"),
                String::from("always"),
            ),
        ]);
        let config = runtime_config_from(&values, file).unwrap();
        assert_eq!(config.team_id.as_deref(), Some("OPS"));
        assert_eq!(config.issue_sort, Some(IssueSort::Priority));
        assert_eq!(config.issue_create_ask_project, Some(true));
        assert_eq!(config.issue_create_assign_self, Some(AssignSelf::Always));
    }
    #[test]
    fn toml_rejects_credentials_and_unknown_keys() {
        assert!(toml::from_str::<ConfigFile>("api_key = 'secret'").is_err());
        assert!(toml::from_str::<ConfigFile>("vcs = 'jj'").is_err());
    }

    #[test]
    fn global_config_can_be_atomically_replaced() {
        let root = std::env::temp_dir().join(format!(
            "workrus-config-replace-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let path = root.join("linear/linear.toml");
        let first = ConfigFile {
            team_id: Some("ENG".into()),
            ..ConfigFile::default()
        };
        let second = ConfigFile {
            team_id: Some("OPS".into()),
            ..ConfigFile::default()
        };

        write_file(&path, &first).unwrap();
        write_file(&path, &second).unwrap();
        assert_eq!(read_file(&path).unwrap(), Some(second));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn global_config_never_follows_a_symlink() {
        use std::os::unix::fs::symlink;

        let root = std::env::temp_dir().join(format!(
            "workrus-config-symlink-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let directory = root.join("linear");
        let path = directory.join("linear.toml");
        let target = root.join("target.toml");
        fs::create_dir_all(&directory).unwrap();
        fs::write(&target, "team_id = 'SAFE'\n").unwrap();
        symlink(&target, &path).unwrap();

        assert!(read_file(&path).is_err());
        assert!(write_file(&path, &ConfigFile::default()).is_err());
        assert_eq!(fs::read_to_string(&target).unwrap(), "team_id = 'SAFE'\n");
        fs::remove_dir_all(root).unwrap();
    }
}
