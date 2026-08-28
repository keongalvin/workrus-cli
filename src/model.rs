use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct IssueIdentifier(String);
impl IssueIdentifier {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl fmt::Display for IssueIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
impl FromStr for IssueIdentifier {
    type Err = AppError;
    fn from_str(v: &str) -> Result<Self, AppError> {
        let Some((a, b)) = v.split_once('-') else {
            return Err(AppError::input("issue identifier must look like ENG-123"));
        };
        if a.is_empty()
            || !a.as_bytes()[0].is_ascii_alphabetic()
            || !a.bytes().all(|x| x.is_ascii_alphanumeric())
            || b.is_empty()
            || b.as_bytes()[0] == b'0'
            || !b.bytes().all(|x| x.is_ascii_digit())
            || v.matches('-').count() != 1
        {
            return Err(AppError::input("issue identifier must look like ENG-123"));
        };
        Ok(Self(format!("{}-{b}", a.to_ascii_uppercase())))
    }
}
pub fn issue_identifier_in_branch(branch: &str) -> Option<IssueIdentifier> {
    for part in branch.split(['/', '_']) {
        let components: Vec<_> = part.split('-').collect();
        for pair in components.windows(2) {
            if let Ok(identifier) = format!("{}-{}", pair[0], pair[1]).parse() {
                return Some(identifier);
            }
        }
    }
    None
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Team {
    pub id: String,
    pub key: String,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct State {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User {
    pub id: String,
    pub name: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub active: Option<bool>,
    #[serde(rename = "avatarUrl")]
    pub avatar_url: Option<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Issue {
    pub id: String,
    pub identifier: String,
    pub title: String,
    pub description: Option<String>,
    pub url: String,
    pub priority: Option<f64>,
    pub team: Team,
    pub state: Option<State>,
    pub assignee: Option<User>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
    #[serde(rename = "branchName")]
    pub branch_name: Option<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Comment {
    pub id: String,
    pub body: String,
    #[serde(rename = "issueId")]
    pub issue_id: Option<String>,
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
    pub user: Option<User>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DocumentComment {
    pub id: String,
    #[serde(rename = "documentContentId")]
    pub document_content_id: Option<String>,
    #[serde(rename = "resolvedAt")]
    pub resolved_at: Option<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DocumentComments {
    pub nodes: Vec<DocumentComment>,
    #[serde(rename = "pageInfo")]
    pub page_info: Option<PageInfo>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Document {
    pub id: String,
    pub title: String,
    pub content: Option<String>,
    pub url: Option<String>,
    #[serde(rename = "slugId")]
    pub slug_id: Option<String>,
    pub issue: Option<DocumentTargetRef>,
    pub project: Option<DocumentTargetRef>,
    pub comments: Option<DocumentComments>,
    pub trashed: Option<bool>,
    #[serde(rename = "archivedAt")]
    pub archived_at: Option<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DocumentTargetRef {
    pub id: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(from = "MilestoneWire")]
pub struct Milestone {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "projectId")]
    pub project_id: String,
    #[serde(rename = "targetDate")]
    pub target_date: Option<String>,
}

#[derive(Deserialize)]
struct MilestoneWire {
    id: String,
    name: String,
    description: Option<String>,
    project: MilestoneProjectWire,
    #[serde(rename = "targetDate")]
    target_date: Option<String>,
}

#[derive(Deserialize)]
struct MilestoneProjectWire {
    id: String,
}

impl From<MilestoneWire> for Milestone {
    fn from(wire: MilestoneWire) -> Self {
        Self {
            id: wire.id,
            name: wire.name,
            description: wire.description,
            project_id: wire.project.id,
            target_date: wire.target_date,
        }
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PageInfo {
    #[serde(rename = "hasNextPage")]
    pub has_next_page: bool,
    #[serde(rename = "endCursor")]
    pub end_cursor: Option<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    #[serde(rename = "slugId")]
    pub slug_id: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub url: Option<String>,
    #[serde(rename = "targetDate")]
    pub target_date: Option<String>,
    pub teams: Option<ProjectTeams>,
    pub lead: Option<User>,
    pub members: Option<ProjectMembers>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProjectTeams {
    pub nodes: Vec<Team>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProjectMembers {
    pub nodes: Vec<User>,
}

pub fn valid_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || !bytes
            .iter()
            .enumerate()
            .all(|(i, b)| matches!(i, 4 | 7) || b.is_ascii_digit())
    {
        return false;
    }
    let year = value[0..4].parse::<u32>().ok();
    let month = value[5..7].parse::<u32>().ok();
    let day = value[8..10].parse::<u32>().ok();
    match (year, month, day) {
        (Some(y), Some(m @ 1..=12), Some(d)) => {
            let max = match m {
                2 if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) => 29,
                2 => 28,
                4 | 6 | 9 | 11 => 30,
                _ => 31,
            };
            d >= 1 && d <= max
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issue_identifier_is_canonicalized() {
        let identifier: IssueIdentifier = "eng-123".parse().unwrap();

        assert_eq!(identifier.as_str(), "ENG-123");
    }

    #[test]
    fn branch_inference_uses_first_identifier_before_title_slug() {
        let identifier = issue_identifier_in_branch("alice/eng-123-oauth-2").unwrap();

        assert_eq!(identifier.as_str(), "ENG-123");
    }

    #[test]
    fn issue_priority_accepts_linear_float_values() {
        let issue: Issue = serde_json::from_value(serde_json::json!({
            "id": "issue-id",
            "identifier": "ENG-1",
            "title": "Example",
            "description": null,
            "url": "https://linear.app/issue/ENG-1",
            "priority": 1.0,
            "team": { "id": "team-id", "key": "ENG", "name": "Engineering" },
            "state": null,
            "assignee": null,
            "createdAt": null,
            "updatedAt": null,
            "branchName": null
        }))
        .unwrap();

        assert_eq!(issue.priority, Some(1.0));
    }
    #[test]
    fn dates_require_real_calendar_days() {
        assert!(valid_date("2028-02-29"));
        assert!(!valid_date("2027-02-29"));
        assert!(!valid_date("2028-13-01"));
    }
}
