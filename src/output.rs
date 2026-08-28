use crate::model::{Comment, Document, Issue, Milestone, PageInfo, Project, Team, User};
use serde_json::json;
pub fn escaped(s: &str) -> String {
    s.chars()
        .flat_map(|c| match c {
            '\n' => "\\n".chars().collect(),
            '\r' => "\\r".chars().collect(),
            '\x1b' => "\\u{001b}".chars().collect(),
            c if c.is_control() || matches!(c, '\u{200e}'..='\u{206f}') => {
                format!("\\u{{{:04x}}}", c as u32).chars().collect()
            }
            c => vec![c],
        })
        .collect()
}
pub fn team_collection(teams: &[Team], page: &PageInfo, json_output: bool) -> String {
    if json_output {
        json!({"items":teams,"pageInfo":page}).to_string()
    } else {
        teams
            .iter()
            .map(|t| format!("{}\t{}", escaped(&t.key), escaped(&t.name)))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
    }
}
pub fn user_collection(users: &[User], page: &PageInfo, json_output: bool) -> String {
    if json_output {
        json!({"items":users,"pageInfo":page}).to_string()
    } else {
        users
            .iter()
            .map(|user| {
                format!(
                    "{}\t{}\t{}",
                    escaped(&user.id),
                    escaped(
                        user.display_name
                            .as_deref()
                            .or(user.name.as_deref())
                            .unwrap_or("")
                    ),
                    escaped(user.email.as_deref().unwrap_or(""))
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
    }
}
pub fn team(team: &Team, json_output: bool) -> String {
    if json_output {
        json!({"team":team}).to_string()
    } else {
        format!("{}\t{}\n", escaped(&team.key), escaped(&team.name))
    }
}
pub fn project_collection(projects: &[Project], page: &PageInfo, json_output: bool) -> String {
    if json_output {
        json!({"items":projects,"pageInfo":page}).to_string()
    } else {
        projects
            .iter()
            .map(|p| format!("{}\t{}", escaped(&p.id), escaped(&p.name)))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
    }
}
pub fn project(project: &Project, json_output: bool) -> String {
    if json_output {
        json!({"project":project}).to_string()
    } else {
        format!("{}\t{}\n", escaped(&project.id), escaped(&project.name))
    }
}
pub fn document_collection(items: &[Document], page: &PageInfo, json_output: bool) -> String {
    if json_output {
        json!({"items":items,"pageInfo":page}).to_string()
    } else {
        items
            .iter()
            .map(|d| format!("{}\t{}", escaped(&d.id), escaped(&d.title)))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
    }
}
pub fn document(item: &Document, raw: bool, json_output: bool) -> String {
    if json_output {
        json!({"document":item}).to_string()
    } else if raw {
        // Raw means Markdown rather than the summary view, never unescaped terminal controls.
        escaped(item.content.as_deref().unwrap_or(""))
    } else {
        format!("{}\t{}\n", escaped(&item.id), escaped(&item.title))
    }
}
pub fn bulk_partial(completed: &[Document], retry: &str, json_output: bool) -> String {
    if json_output {
        json!({"result":"partial_failure","completedPhase":"remote","completedItems":completed.iter().map(|d| json!({"id":d.id,"slugId":d.slug_id})).collect::<Vec<_>>(),"retry":retry}).to_string()
    } else {
        format!(
            "Documents trashed: {}\nRetry: {retry}\n",
            completed
                .iter()
                .map(|d| escaped(&d.id))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}
pub fn milestone_collection(items: &[Milestone], page: &PageInfo, json_output: bool) -> String {
    if json_output {
        json!({"items":items,"pageInfo":page}).to_string()
    } else {
        items
            .iter()
            .map(|m| format!("{}\t{}", escaped(&m.id), escaped(&m.name)))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
    }
}
pub fn milestone(item: &Milestone, json_output: bool) -> String {
    if json_output {
        json!({"milestone":item}).to_string()
    } else {
        format!("{}\t{}\n", escaped(&item.id), escaped(&item.name))
    }
}
pub fn issue_collection(issues: &[Issue], page: &PageInfo, json_output: bool) -> String {
    if json_output {
        json!({"items":issues,"pageInfo":page}).to_string()
    } else {
        issues
            .iter()
            .map(|i| format!("{}\t{}", escaped(&i.identifier), escaped(&i.title)))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
    }
}
pub fn issue(issue: &Issue, json_output: bool) -> String {
    if json_output {
        json!({"issue":issue}).to_string()
    } else {
        format!(
            "{} {}\n{}\n",
            escaped(&issue.identifier),
            escaped(&issue.title),
            escaped(&issue.url)
        )
    }
}
pub fn scalar(k: &str, v: &str, json_output: bool) -> String {
    if json_output {
        json!({k:v}).to_string()
    } else {
        format!("{}\n", escaped(v))
    }
}
pub fn partial_start(
    identifier: &str,
    issue_id: &str,
    branch: &str,
    branch_action: &str,
    state: &crate::model::State,
    json_output: bool,
) -> String {
    if json_output {
        json!({"result":"partial_failure","completedPhase":"git","issue":{"id":issue_id,"identifier":identifier},"branch":{"name":branch,"action":branch_action},"workflow":{"action":"not_confirmed","stateId":state.id,"stateName":state.name},"retry":format!("workrus issue start {identifier}")}).to_string()
    } else {
        format!(
            "Git branch {}: {}\nLinear workflow state not confirmed: {}\nRetry: workrus issue start {}\n",
            escaped(branch_action),
            escaped(branch),
            escaped(&state.name),
            escaped(identifier)
        )
    }
}
pub fn start(
    identifier: &str,
    issue_id: &str,
    branch: &str,
    branch_action: &str,
    workflow_action: &str,
    state: &crate::model::State,
    json_output: bool,
) -> String {
    if json_output {
        json!({"result":"started","issue":{"id":issue_id,"identifier":identifier},"branch":{"name":branch,"action":branch_action},"workflow":{"action":workflow_action,"stateId":state.id,"stateName":state.name}}).to_string()
    } else {
        format!(
            "{}\t{}\t{}\n",
            escaped(identifier),
            escaped(branch),
            workflow_action
        )
    }
}
pub fn pull_request(
    url: &str,
    issue_id: &str,
    identifier: &str,
    json_output: bool,
) -> Result<String, crate::error::AppError> {
    if !url.starts_with("https://") {
        return Err(crate::error::AppError::operational(
            "gh returned a non-HTTPS pull request URL",
        ));
    }
    Ok(if json_output {
        json!({"pullRequest":{"url":url,"issue":{"id":issue_id,"identifier":identifier}}})
            .to_string()
    } else {
        format!("{}\n", escaped(url))
    })
}
pub fn comment_collection(comments: &[Comment], page: &PageInfo, json_output: bool) -> String {
    if json_output {
        json!({"items":comments,"pageInfo":page}).to_string()
    } else {
        comments
            .iter()
            .map(|c| format!("{}\t{}", escaped(&c.id), escaped(&c.body)))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
    }
}
pub fn comment(comment: &Comment, json_output: bool) -> String {
    if json_output {
        json!({"comment":comment}).to_string()
    } else {
        format!("{}\t{}\n", escaped(&comment.id), escaped(&comment.body))
    }
}
pub fn archived(id: &str, json_output: bool) -> String {
    if json_output {
        json!({"result":"archived","resource":{"id":id}}).to_string()
    } else {
        format!("archived\t{}\n", escaped(id))
    }
}
pub fn archived_documents(documents: &[Document], json_output: bool) -> String {
    if json_output {
        json!({"result":"archived","resources":documents.iter().map(|document| json!({"id":document.id,"slugId":document.slug_id})).collect::<Vec<_>>()}).to_string()
    } else {
        format!(
            "archived\t{}\n",
            documents
                .iter()
                .map(|document| escaped(&document.id))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}
pub fn dry_run(action: &str, target: &str, json_output: bool) -> String {
    if json_output {
        json!({"result":"dry_run","action":action,"target":target}).to_string()
    } else {
        format!("dry run\t{}\t{}\n", escaped(action), escaped(target))
    }
}
pub fn config(team: &Team, json_output: bool) -> String {
    if json_output {
        json!({"team":team}).to_string()
    } else {
        format!("{}\t{}\n", escaped(&team.key), escaped(&team.name))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn escapes_controls() {
        assert_eq!(escaped("a\n\x1b"), "a\\n\\u{001b}")
    }
    #[test]
    fn destructive_json_contracts_are_stable() {
        let archive: serde_json::Value =
            serde_json::from_str(&archived("remote\n-id", true)).unwrap();
        assert_eq!(archive["result"], "archived");
        assert_eq!(archive["resource"]["id"], "remote\n-id");
        let dry_run: serde_json::Value =
            serde_json::from_str(&dry_run("archive", "ENG-1", true)).unwrap();
        assert_eq!(dry_run["result"], "dry_run");
        assert_eq!(dry_run["target"], "ENG-1");
    }

    #[test]
    fn user_collection_is_stable_json_and_escaped_human_text() {
        let user = User {
            id: "u1".into(),
            name: Some("Ada".into()),
            display_name: Some("Ada\nLovelace".into()),
            email: Some("ada@example.test".into()),
            active: Some(true),
            avatar_url: None,
        };
        let page = PageInfo {
            has_next_page: false,
            end_cursor: None,
        };
        let json: serde_json::Value =
            serde_json::from_str(&user_collection(std::slice::from_ref(&user), &page, true))
                .unwrap();
        assert_eq!(json["items"][0]["displayName"], "Ada\nLovelace");
        assert!(user_collection(&[user], &page, false).contains("Ada\\nLovelace"));
    }

    #[test]
    fn scalar_text_escapes_remote_controls() {
        assert_eq!(scalar("title", "a\n\x1b", false), "a\\n\\u{001b}\n");
        assert_eq!(scalar("title", "a\n", true), "{\"title\":\"a\\n\"}");
    }

    #[test]
    fn json_partial_start_preserves_stable_contract() {
        let state = crate::model::State {
            id: "state-id".to_owned(),
            name: "In Progress".to_owned(),
            kind: "started".to_owned(),
        };

        let rendered = partial_start("ENG-1", "issue-id", "alice/eng-1", "created", &state, true);
        let value: serde_json::Value = serde_json::from_str(&rendered).unwrap();

        assert_eq!(value["result"], "partial_failure");
        assert_eq!(value["completedPhase"], "git");
        assert_eq!(value["retry"], "workrus issue start ENG-1");
    }

    #[test]
    fn human_partial_start_exposes_completed_git_work_and_retry() {
        let state = crate::model::State {
            id: "state-id".to_owned(),
            name: "In Progress".to_owned(),
            kind: "started".to_owned(),
        };

        let rendered = partial_start("ENG-1", "issue-id", "alice/eng-1", "created", &state, false);

        assert!(rendered.contains("Git branch created: alice/eng-1"));
        assert!(rendered.contains("Linear workflow state not confirmed: In Progress"));
        assert!(rendered.contains("Retry: workrus issue start ENG-1"));
    }
    #[test]
    fn project_json_envelopes_are_stable() {
        let item = Project {
            id: "p1".into(),
            name: "Roadmap".into(),
            slug_id: Some("roadmap".into()),
            description: None,
            content: None,
            url: None,
            target_date: None,
            teams: None,
            lead: None,
            members: None,
        };
        let page = PageInfo {
            has_next_page: false,
            end_cursor: None,
        };
        let value: serde_json::Value = serde_json::from_str(&project_collection(
            std::slice::from_ref(&item),
            &page,
            true,
        ))
        .unwrap();
        assert_eq!(value["items"][0]["slugId"], "roadmap");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&project(&item, true)).unwrap()["project"]["id"],
            "p1"
        );
    }
}
