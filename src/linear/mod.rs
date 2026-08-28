pub mod transport;
use crate::{
    error::AppError,
    model::{Comment, Document, Issue, Milestone, PageInfo, Project, State, Team, User},
};
use serde::Deserialize;
use serde_json::json;
use transport::{LinearClient, Request};
const TEAMS: &str = "query Teams($first: Int!, $after: String) { teams(first: $first, after: $after) { nodes { id key name description color icon } edges { cursor } pageInfo { hasNextPage endCursor } } }";
const TEAM_MEMBERS: &str = "query TeamMembers($id: String!, $includeDisabled: Boolean!, $first: Int!, $after: String) { team(id: $id) { members(includeDisabled: $includeDisabled, first: $first, after: $after) { nodes { id name displayName email active avatarUrl } edges { cursor } pageInfo { hasNextPage endCursor } } } }";
const USERS: &str = "query Users($includeDisabled: Boolean!, $first: Int!, $after: String) { viewer { organization { users(includeDisabled: $includeDisabled, first: $first, after: $after) { nodes { id name displayName email active avatarUrl } edges { cursor } pageInfo { hasNextPage endCursor } } } } }";
const TEAM_CREATE: &str = "mutation CreateTeam($input: TeamCreateInput!) { teamCreate(input: $input) { success team { id key name description color icon } } }";
const ISSUE: &str = "query FindIssue($id: String!) { issue(id: $id) { id identifier title description url branchName priority updatedAt createdAt team { id key name } state { id name type } assignee { id name displayName } } }";
const MINE: &str = "query MineIssues($filter: IssueFilter!, $sort: [IssueSortInput!], $first: Int!, $after: String) { issues(filter: $filter, sort: $sort, first: $first, after: $after) { nodes { id identifier title description url branchName priority updatedAt createdAt team { id key name } state { id name type } assignee { id name displayName } } edges { cursor } pageInfo { hasNextPage endCursor } } }";
const SEARCH: &str = "query SearchIssues($filter: IssueFilter, $sort: [IssueSortInput!], $first: Int!, $after: String) { issues(filter: $filter, sort: $sort, first: $first, after: $after) { nodes { id identifier title description url branchName priority updatedAt createdAt team { id key name } state { id name type } assignee { id name displayName } } edges { cursor } pageInfo { hasNextPage endCursor } } }";
const TEAM_STATES: &str =
    "query TeamStates($id: String!) { team(id: $id) { id key states { nodes { id name type } } } }";
const VIEWER: &str = "query Viewer { viewer { id name displayName email active avatarUrl } }";
const CREATE: &str = "mutation CreateIssue($input: IssueCreateInput!) { issueCreate(input: $input) { success issue { id identifier title description url branchName priority updatedAt createdAt team { id key name } state { id name type } assignee { id name displayName } } } }";
const UPDATE: &str = "mutation UpdateIssue($id: String!, $input: IssueUpdateInput!) { issueUpdate(id: $id, input: $input) { success issue { id identifier title description url branchName priority updatedAt createdAt team { id key name } state { id name type } assignee { id name displayName } } } }";
const ARCHIVE: &str = "mutation ArchiveIssue($id: String!) { issueArchive(id: $id) { success } }";
const COMMENTS: &str = "query IssueComments($id: String!, $first: Int!, $after: String) { issue(id: $id) { comments(first: $first, after: $after) { nodes { id body issueId parentId user { id name displayName } createdAt updatedAt } edges { cursor } pageInfo { hasNextPage endCursor } } } }";
const COMMENT_CREATE: &str = "mutation CreateComment($input: CommentCreateInput!) { commentCreate(input: $input) { success comment { id body issueId parentId user { id name displayName } createdAt updatedAt } } }";
const COMMENT_UPDATE: &str = "mutation UpdateComment($id: String!, $input: CommentUpdateInput!) { commentUpdate(id: $id, input: $input) { success comment { id body issueId parentId user { id name displayName } createdAt updatedAt } } }";
const COMMENT_DELETE: &str =
    "mutation DeleteComment($id: String!) { commentDelete(id: $id) { success entityId } }";
const PROJECTS: &str = "query Projects($filter: ProjectFilter, $first: Int!, $after: String) { projects(filter: $filter, first: $first, after: $after) { nodes { id name slugId description content url targetDate teams { nodes { id key name description color icon } } lead { id name displayName email active avatarUrl } members { nodes { id name displayName email active avatarUrl } } } edges { cursor } pageInfo { hasNextPage endCursor } } }";
const MILESTONES: &str = "query ProjectMilestones($filter: ProjectMilestoneFilter, $first: Int!, $after: String) { projectMilestones(filter: $filter, first: $first, after: $after) { nodes { id name description project { id } targetDate } edges { cursor } pageInfo { hasNextPage endCursor } } }";
const MILESTONE: &str = "query ProjectMilestone($id: String!) { projectMilestone(id: $id) { id name description project { id } targetDate } }";
const MILESTONE_CREATE: &str = "mutation CreateProjectMilestone($input: ProjectMilestoneCreateInput!) { projectMilestoneCreate(input: $input) { success projectMilestone { id name description project { id } targetDate } } }";
const MILESTONE_UPDATE: &str = "mutation UpdateProjectMilestone($id: String!, $input: ProjectMilestoneUpdateInput!) { projectMilestoneUpdate(id: $id, input: $input) { success projectMilestone { id name description project { id } targetDate } } }";
const MILESTONE_DELETE: &str = "mutation DeleteProjectMilestone($id: String!) { projectMilestoneDelete(id: $id) { success entityId } }";
const DOCUMENTS: &str = "query Documents($filter: DocumentFilter, $first: Int!, $after: String) { documents(filter: $filter, first: $first, after: $after) { nodes { id title content url slugId issue { id } project { id } comments(first: 100) { nodes { id documentContentId resolvedAt } pageInfo { hasNextPage endCursor } } trashed archivedAt } edges { cursor } pageInfo { hasNextPage endCursor } } }";
const DOCUMENT: &str = "query Document($id: String!) { document(id: $id) { id title content url slugId issue { id } project { id } comments(first: 100) { nodes { id documentContentId resolvedAt } pageInfo { hasNextPage endCursor } } trashed archivedAt } }";
const DOCUMENT_CREATE: &str = "mutation CreateDocument($input: DocumentCreateInput!) { documentCreate(input: $input) { success document { id title content url slugId issue { id } project { id } comments(first: 100) { nodes { id documentContentId resolvedAt } pageInfo { hasNextPage endCursor } } trashed archivedAt } } }";
const DOCUMENT_UPDATE: &str = "mutation UpdateDocument($id: String!, $input: DocumentUpdateInput!) { documentUpdate(id: $id, input: $input) { success document { id title content url slugId issue { id } project { id } comments(first: 100) { nodes { id documentContentId resolvedAt } pageInfo { hasNextPage endCursor } } trashed archivedAt } } }";
const DOCUMENT_DELETE: &str =
    "mutation DeleteDocument($id: String!) { documentDelete(id: $id) { success } }";
const PROJECT_CREATE: &str = "mutation CreateProject($input: ProjectCreateInput!) { projectCreate(input: $input) { success project { id name slugId description content url targetDate teams { nodes { id key name description color icon } } lead { id name displayName email active avatarUrl } members { nodes { id name displayName email active avatarUrl } } } } }";
#[derive(Debug)]
pub struct Collection<T> {
    pub items: Vec<T>,
    /// Per-item cursors, when the GraphQL connection supplied edges.
    pub item_cursors: Vec<String>,
    pub page_info: PageInfo,
}
impl LinearClient {
    pub fn teams(&self, limit: u8, after: Option<&str>) -> Result<Collection<Team>, AppError> {
        #[derive(Deserialize)]
        struct D {
            teams: C<Team>,
        }
        #[derive(Deserialize)]
        struct C<T> {
            nodes: Vec<T>,
            #[serde(default)]
            edges: Vec<Edge>,
            #[serde(rename = "pageInfo")]
            page_info: PageInfo,
        }
        #[derive(Deserialize)]
        struct Edge {
            cursor: String,
        }
        let d: D = self.execute(&Request {
            query: TEAMS,
            operation_name: "Teams",
            variables: json!({"first":limit,"after":after}),
        })?;
        Ok(Collection {
            items: d.teams.nodes,
            item_cursors: d.teams.edges.into_iter().map(|edge| edge.cursor).collect(),
            page_info: d.teams.page_info,
        })
    }
    pub fn team_members(
        &self,
        id: &str,
        include_disabled: bool,
        limit: u8,
        after: Option<&str>,
    ) -> Result<Collection<User>, AppError> {
        #[derive(Deserialize)]
        struct D {
            team: Option<Team>,
        }
        #[derive(Deserialize)]
        struct Team {
            members: C,
        }
        #[derive(Deserialize)]
        struct C {
            nodes: Vec<User>,
            #[serde(default)]
            edges: Vec<Edge>,
            #[serde(rename = "pageInfo")]
            page_info: PageInfo,
        }
        #[derive(Deserialize)]
        struct Edge {
            cursor: String,
        }
        let d: D = self.execute(&Request { query: TEAM_MEMBERS, operation_name: "TeamMembers", variables: json!({"id":id,"includeDisabled":include_disabled,"first":limit,"after":after}) })?;
        let members = d
            .team
            .ok_or_else(|| AppError::input(format!("team {id} is not accessible")))?
            .members;
        Ok(Collection {
            items: members.nodes,
            item_cursors: members.edges.into_iter().map(|edge| edge.cursor).collect(),
            page_info: members.page_info,
        })
    }
    pub fn users(
        &self,
        include_disabled: bool,
        limit: u8,
        after: Option<&str>,
    ) -> Result<Collection<User>, AppError> {
        #[derive(Deserialize)]
        struct D {
            viewer: Viewer,
        }
        #[derive(Deserialize)]
        struct Viewer {
            organization: Organization,
        }
        #[derive(Deserialize)]
        struct Organization {
            users: C,
        }
        #[derive(Deserialize)]
        struct C {
            nodes: Vec<User>,
            #[serde(default)]
            edges: Vec<Edge>,
            #[serde(rename = "pageInfo")]
            page_info: PageInfo,
        }
        #[derive(Deserialize)]
        struct Edge {
            cursor: String,
        }
        let d: D = self.execute(&Request {
            query: USERS,
            operation_name: "Users",
            variables: json!({"includeDisabled":include_disabled,"first":limit,"after":after}),
        })?;
        let users = d.viewer.organization.users;
        Ok(Collection {
            items: users.nodes,
            item_cursors: users.edges.into_iter().map(|edge| edge.cursor).collect(),
            page_info: users.page_info,
        })
    }
    pub fn team_create(&self, input: serde_json::Value) -> Result<Team, AppError> {
        #[derive(Deserialize)]
        struct D {
            #[serde(rename = "teamCreate")]
            result: R,
        }
        #[derive(Deserialize)]
        struct R {
            success: bool,
            team: Option<Team>,
        }
        let d: D = self.execute(&Request {
            query: TEAM_CREATE,
            operation_name: "CreateTeam",
            variables: json!({"input":input}),
        })?;
        if !d.result.success {
            return Err(AppError::operational(
                "Linear did not confirm team creation",
            ));
        }
        d.result
            .team
            .ok_or_else(|| AppError::operational("Linear returned no team after creation"))
    }
    pub fn documents(
        &self,
        filter: serde_json::Value,
        limit: u8,
        after: Option<&str>,
    ) -> Result<Collection<Document>, AppError> {
        #[derive(Deserialize)]
        struct D {
            documents: C,
        }
        #[derive(Deserialize)]
        struct C {
            nodes: Vec<Document>,
            #[serde(default)]
            edges: Vec<Edge>,
            #[serde(rename = "pageInfo")]
            page_info: PageInfo,
        }
        #[derive(Deserialize)]
        struct Edge {
            cursor: String,
        }
        let d: D = self.execute(&Request {
            query: DOCUMENTS,
            operation_name: "Documents",
            variables: json!({"filter":filter,"first":limit,"after":after}),
        })?;
        Ok(Collection {
            items: d.documents.nodes,
            item_cursors: d.documents.edges.into_iter().map(|e| e.cursor).collect(),
            page_info: d.documents.page_info,
        })
    }
    pub fn document(&self, id: &str) -> Result<Document, AppError> {
        #[derive(Deserialize)]
        struct D {
            document: Option<Document>,
        }
        self.execute::<D>(&Request {
            query: DOCUMENT,
            operation_name: "Document",
            variables: json!({"id":id}),
        })?
        .document
        .ok_or_else(|| AppError::input(format!("document {id} was not found")))
    }
    pub fn document_create(&self, input: serde_json::Value) -> Result<Document, AppError> {
        self.document_mutation(DOCUMENT_CREATE, "CreateDocument", None, input)
    }
    pub fn document_update(
        &self,
        id: &str,
        input: serde_json::Value,
    ) -> Result<Document, AppError> {
        self.document_mutation(DOCUMENT_UPDATE, "UpdateDocument", Some(id), input)
    }
    pub fn document_delete(&self, id: &str) -> Result<String, AppError> {
        #[derive(Deserialize)]
        struct D {
            #[serde(rename = "documentDelete")]
            result: R,
        }
        #[derive(Deserialize)]
        struct R {
            success: bool,
        }
        let d: D = self.execute(&Request {
            query: DOCUMENT_DELETE,
            operation_name: "DeleteDocument",
            variables: json!({"id":id}),
        })?;
        if !d.result.success {
            return Err(AppError::operational(
                "Linear did not confirm document trash",
            ));
        }
        Ok(id.to_owned())
    }
    fn document_mutation(
        &self,
        query: &str,
        operation_name: &str,
        id: Option<&str>,
        input: serde_json::Value,
    ) -> Result<Document, AppError> {
        #[derive(Deserialize)]
        struct D {
            #[serde(alias = "documentCreate", alias = "documentUpdate")]
            result: R,
        }
        #[derive(Deserialize)]
        struct R {
            success: bool,
            document: Option<Document>,
        }
        let variables = id
            .map(|id| json!({"id":id,"input":input}))
            .unwrap_or_else(|| json!({"input":input}));
        let d: D = self.execute(&Request {
            query,
            operation_name,
            variables,
        })?;
        if !d.result.success {
            return Err(AppError::operational(
                "Linear did not confirm document mutation",
            ));
        }
        d.result
            .document
            .ok_or_else(|| AppError::operational("Linear returned no document after mutation"))
    }
    pub fn projects(
        &self,
        team_id: Option<&str>,
        limit: u8,
        after: Option<&str>,
    ) -> Result<Collection<Project>, AppError> {
        #[derive(Deserialize)]
        struct D {
            projects: C,
        }
        #[derive(Deserialize)]
        struct C {
            nodes: Vec<Project>,
            #[serde(default)]
            edges: Vec<Edge>,
            #[serde(rename = "pageInfo")]
            page_info: PageInfo,
        }
        #[derive(Deserialize)]
        struct Edge {
            cursor: String,
        }
        let filter = team_id
            .map(|id| json!({"accessibleTeams":{"some":{"id":{"eq":id}}}}))
            .unwrap_or(serde_json::Value::Null);
        let d: D = self.execute(&Request {
            query: PROJECTS,
            operation_name: "Projects",
            variables: json!({"filter":filter,"first":limit,"after":after}),
        })?;
        Ok(Collection {
            items: d.projects.nodes,
            item_cursors: d.projects.edges.into_iter().map(|e| e.cursor).collect(),
            page_info: d.projects.page_info,
        })
    }
    pub fn project_create(&self, input: serde_json::Value) -> Result<Project, AppError> {
        #[derive(Deserialize)]
        struct D {
            #[serde(rename = "projectCreate")]
            result: R,
        }
        #[derive(Deserialize)]
        struct R {
            success: bool,
            project: Option<Project>,
        }
        let d: D = self.execute(&Request {
            query: PROJECT_CREATE,
            operation_name: "CreateProject",
            variables: json!({"input":input}),
        })?;
        if !d.result.success {
            return Err(AppError::operational(
                "Linear did not confirm project creation",
            ));
        }
        d.result
            .project
            .ok_or_else(|| AppError::operational("Linear returned no project after creation"))
    }
    pub fn milestones(
        &self,
        project_id: Option<&str>,
        limit: u8,
        after: Option<&str>,
    ) -> Result<Collection<Milestone>, AppError> {
        #[derive(Deserialize)]
        struct D {
            #[serde(rename = "projectMilestones")]
            milestones: C,
        }
        #[derive(Deserialize)]
        struct C {
            nodes: Vec<Milestone>,
            #[serde(default)]
            edges: Vec<Edge>,
            #[serde(rename = "pageInfo")]
            page_info: PageInfo,
        }
        #[derive(Deserialize)]
        struct Edge {
            cursor: String,
        }
        let filter = project_id
            .map(|id| json!({"project":{"id":{"eq":id}}}))
            .unwrap_or(serde_json::Value::Null);
        let d: D = self.execute(&Request {
            query: MILESTONES,
            operation_name: "ProjectMilestones",
            variables: json!({"filter":filter,"first":limit,"after":after}),
        })?;
        Ok(Collection {
            items: d.milestones.nodes,
            item_cursors: d.milestones.edges.into_iter().map(|e| e.cursor).collect(),
            page_info: d.milestones.page_info,
        })
    }
    pub fn milestone(&self, id: &str) -> Result<Milestone, AppError> {
        #[derive(Deserialize)]
        struct D {
            #[serde(rename = "projectMilestone")]
            milestone: Option<Milestone>,
        }
        self.execute::<D>(&Request {
            query: MILESTONE,
            operation_name: "ProjectMilestone",
            variables: json!({"id":id}),
        })?
        .milestone
        .ok_or_else(|| AppError::input(format!("milestone {id} was not found")))
    }
    pub fn milestone_create(&self, input: serde_json::Value) -> Result<Milestone, AppError> {
        self.milestone_mutation(MILESTONE_CREATE, "CreateProjectMilestone", None, input)
    }
    pub fn milestone_update(
        &self,
        id: &str,
        input: serde_json::Value,
    ) -> Result<Milestone, AppError> {
        self.milestone_mutation(MILESTONE_UPDATE, "UpdateProjectMilestone", Some(id), input)
    }
    pub fn milestone_delete(&self, id: &str) -> Result<String, AppError> {
        #[derive(Deserialize)]
        struct D {
            #[serde(rename = "projectMilestoneDelete")]
            result: R,
        }
        #[derive(Deserialize)]
        struct R {
            success: bool,
            #[serde(rename = "entityId")]
            entity_id: Option<String>,
        }
        let d: D = self.execute(&Request {
            query: MILESTONE_DELETE,
            operation_name: "DeleteProjectMilestone",
            variables: json!({"id":id}),
        })?;
        if !d.result.success {
            return Err(AppError::operational(
                "Linear did not confirm milestone deletion",
            ));
        }
        Ok(d.result.entity_id.unwrap_or_else(|| id.to_owned()))
    }
    fn milestone_mutation(
        &self,
        query: &str,
        operation_name: &str,
        id: Option<&str>,
        input: serde_json::Value,
    ) -> Result<Milestone, AppError> {
        #[derive(Deserialize)]
        struct D {
            #[serde(alias = "projectMilestoneCreate", alias = "projectMilestoneUpdate")]
            result: R,
        }
        #[derive(Deserialize)]
        struct R {
            success: bool,
            #[serde(rename = "projectMilestone")]
            milestone: Option<Milestone>,
        }
        let variables = id
            .map(|id| json!({"id":id,"input":input}))
            .unwrap_or_else(|| json!({"input":input}));
        let d: D = self.execute(&Request {
            query,
            operation_name,
            variables,
        })?;
        if !d.result.success {
            return Err(AppError::operational(
                "Linear did not confirm milestone mutation",
            ));
        }
        d.result
            .milestone
            .ok_or_else(|| AppError::operational("Linear returned no milestone after mutation"))
    }
    pub fn issue(&self, id: &str) -> Result<Issue, AppError> {
        #[derive(Deserialize)]
        struct D {
            issue: Option<Issue>,
        }
        let d: D = self.execute(&Request {
            query: ISSUE,
            operation_name: "FindIssue",
            variables: json!({"id":id}),
        })?;
        d.issue
            .ok_or_else(|| AppError::input(format!("issue {id} was not found")))
    }
    pub fn mine(
        &self,
        team: &str,
        limit: u8,
        after: Option<&str>,
    ) -> Result<Collection<Issue>, AppError> {
        self.issues(MINE, "MineIssues", json!({"filter":{"team":{"key":{"eq":team}},"assignee":{"isMe":{"eq":true}}},"first":limit,"after":after}))
    }
    pub fn search(
        &self,
        text: &str,
        team: Option<&str>,
        limit: u8,
        after: Option<&str>,
    ) -> Result<Collection<Issue>, AppError> {
        self.issues_filtered(Some(text), team, None, None, None, None, limit, after)
    }
    #[allow(clippy::too_many_arguments)]
    pub fn issues_filtered(
        &self,
        text: Option<&str>,
        team: Option<&str>,
        state: Option<&str>,
        sort: Option<&str>,
        project: Option<&str>,
        milestone: Option<&str>,
        limit: u8,
        after: Option<&str>,
    ) -> Result<Collection<Issue>, AppError> {
        let mut clauses = Vec::new();
        if let Some(team) = team {
            clauses.push(json!({"team":{"key":{"eq":team}}}));
        }
        if let Some(state) = state {
            clauses.push(json!({"state":{"name":{"eq":state}}}));
        }
        if let Some(project) = project {
            clauses.push(json!({"project":{"id":{"eq":project}}}));
        }
        if let Some(milestone) = milestone {
            clauses.push(json!({"projectMilestone":{"id":{"eq":milestone}}}));
        }
        if let Some(text) = text {
            clauses.push(json!({"or":[{"title":{"containsIgnoreCase":text}},{"description":{"containsIgnoreCase":text}},{"searchableContent":{"contains":text}}]}));
        }
        let filter = if clauses.is_empty() {
            serde_json::Value::Null
        } else {
            json!({"and":clauses})
        };
        let query = if text.is_some() { SEARCH } else { MINE };
        let operation = if text.is_some() {
            "SearchIssues"
        } else {
            "MineIssues"
        };
        let sort = match sort {
            None => serde_json::Value::Null,
            Some("manual") => json!([
                {"workflowState":{"order":"Descending"}},
                {"manual":{"nulls":"last","order":"Ascending"}}
            ]),
            Some("priority") => json!([
                {"workflowState":{"order":"Descending"}},
                {"priority":{"nulls":"last","order":"Descending"}},
                {"manual":{"nulls":"last","order":"Ascending"}}
            ]),
            Some(value) => {
                return Err(AppError::input(format!(
                    "unsupported issue sort {value}; use manual or priority"
                )));
            }
        };
        let variables = json!({"filter":filter,"sort":sort,"first":limit,"after":after});
        self.issues(query, operation, variables)
    }
    pub fn viewer(&self) -> Result<User, AppError> {
        #[derive(Deserialize)]
        struct D {
            viewer: User,
        }
        let data: D = self.execute(&Request {
            query: VIEWER,
            operation_name: "Viewer",
            variables: json!({}),
        })?;
        Ok(data.viewer)
    }
    pub fn team_states(&self, team_id: &str) -> Result<Vec<State>, AppError> {
        #[derive(Deserialize)]
        struct D {
            team: Option<T>,
        }
        #[derive(Deserialize)]
        struct T {
            states: Nodes,
        }
        #[derive(Deserialize)]
        struct Nodes {
            nodes: Vec<State>,
        }
        let d: D = self.execute(&Request {
            query: TEAM_STATES,
            operation_name: "TeamStates",
            variables: json!({"id":team_id}),
        })?;
        d.team
            .map(|t| t.states.nodes)
            .ok_or_else(|| AppError::operational("Linear team was not found"))
    }
    pub fn create(&self, input: serde_json::Value) -> Result<Issue, AppError> {
        self.mutate(CREATE, "CreateIssue", None, input)
    }
    pub fn update(&self, id: &str, input: serde_json::Value) -> Result<Issue, AppError> {
        self.mutate(UPDATE, "UpdateIssue", Some(id), input)
    }
    pub fn archive(&self, id: &str) -> Result<String, AppError> {
        #[derive(Deserialize)]
        struct D {
            #[serde(rename = "issueArchive")]
            result: Delete,
        }
        #[derive(Deserialize)]
        struct Delete {
            success: bool,
        }
        let d: D = self.execute(&Request {
            query: ARCHIVE,
            operation_name: "ArchiveIssue",
            variables: json!({"id":id}),
        })?;
        if !d.result.success {
            return Err(AppError::operational(
                "Linear did not confirm issue archive",
            ));
        }
        Ok(id.to_owned())
    }
    pub fn comments(
        &self,
        id: &str,
        limit: u8,
        after: Option<&str>,
    ) -> Result<Collection<Comment>, AppError> {
        #[derive(Deserialize)]
        struct D {
            issue: Option<IssueComments>,
        }
        #[derive(Deserialize)]
        struct IssueComments {
            comments: C,
        }
        #[derive(Deserialize)]
        struct C {
            nodes: Vec<Comment>,
            #[serde(default)]
            edges: Vec<Edge>,
            #[serde(rename = "pageInfo")]
            page_info: PageInfo,
        }
        #[derive(Deserialize)]
        struct Edge {
            cursor: String,
        }
        let d: D = self.execute(&Request {
            query: COMMENTS,
            operation_name: "IssueComments",
            variables: json!({"id":id,"first":limit,"after":after}),
        })?;
        let c = d
            .issue
            .ok_or_else(|| AppError::input(format!("issue {id} was not found")))?
            .comments;
        Ok(Collection {
            items: c.nodes,
            item_cursors: c.edges.into_iter().map(|edge| edge.cursor).collect(),
            page_info: c.page_info,
        })
    }
    pub fn comment_create(&self, input: serde_json::Value) -> Result<Comment, AppError> {
        self.comment_mutation(COMMENT_CREATE, "CreateComment", None, input)
    }
    pub fn comment_update(&self, id: &str, input: serde_json::Value) -> Result<Comment, AppError> {
        self.comment_mutation(COMMENT_UPDATE, "UpdateComment", Some(id), input)
    }
    pub fn comment_delete(&self, id: &str) -> Result<String, AppError> {
        #[derive(Deserialize)]
        struct D {
            #[serde(rename = "commentDelete")]
            result: Delete,
        }
        #[derive(Deserialize)]
        struct Delete {
            success: bool,
            #[serde(rename = "entityId")]
            entity_id: Option<String>,
        }
        let d: D = self.execute(&Request {
            query: COMMENT_DELETE,
            operation_name: "DeleteComment",
            variables: json!({"id":id}),
        })?;
        if !d.result.success {
            return Err(AppError::operational(
                "Linear did not confirm comment deletion",
            ));
        }
        Ok(d.result.entity_id.unwrap_or_else(|| id.to_owned()))
    }
    fn comment_mutation(
        &self,
        q: &str,
        n: &str,
        id: Option<&str>,
        input: serde_json::Value,
    ) -> Result<Comment, AppError> {
        #[derive(Deserialize)]
        struct D {
            #[serde(alias = "commentCreate", alias = "commentUpdate")]
            result: R,
        }
        #[derive(Deserialize)]
        struct R {
            success: bool,
            comment: Option<Comment>,
        }
        let variables = match id {
            Some(id) => json!({"id":id,"input":input}),
            None => json!({"input":input}),
        };
        let d: D = self.execute(&Request {
            query: q,
            operation_name: n,
            variables,
        })?;
        if !d.result.success {
            return Err(AppError::operational(
                "Linear did not confirm comment mutation",
            ));
        }
        d.result
            .comment
            .ok_or_else(|| AppError::operational("Linear returned no comment after mutation"))
    }
    fn mutate(
        &self,
        q: &str,
        n: &str,
        id: Option<&str>,
        input: serde_json::Value,
    ) -> Result<Issue, AppError> {
        #[derive(Deserialize)]
        struct D {
            #[serde(alias = "issueCreate", alias = "issueUpdate")]
            result: R,
        }
        #[derive(Deserialize)]
        struct R {
            success: bool,
            issue: Option<Issue>,
        }
        let variables = match id {
            Some(id) => json!({"id":id,"input":input}),
            None => json!({"input":input}),
        };
        let d: D = self.execute(&Request {
            query: q,
            operation_name: n,
            variables,
        })?;
        if !d.result.success {
            return Err(AppError::operational(
                "Linear did not confirm the issue mutation",
            ));
        }
        d.result
            .issue
            .ok_or_else(|| AppError::operational("Linear returned no issue after mutation"))
    }
    fn issues(
        &self,
        q: &str,
        n: &str,
        v: serde_json::Value,
    ) -> Result<Collection<Issue>, AppError> {
        #[derive(Deserialize)]
        struct D {
            #[serde(alias = "issues", alias = "searchIssues")]
            issues: C,
        }
        #[derive(Deserialize)]
        struct C {
            nodes: Vec<Issue>,
            #[serde(default)]
            edges: Vec<Edge>,
            #[serde(rename = "pageInfo")]
            page_info: PageInfo,
        }
        #[derive(Deserialize)]
        struct Edge {
            cursor: String,
        }
        let d: D = self.execute(&Request {
            query: q,
            operation_name: n,
            variables: v,
        })?;
        Ok(Collection {
            items: d.issues.nodes,
            item_cursors: d.issues.edges.into_iter().map(|edge| edge.cursor).collect(),
            page_info: d.issues.page_info,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linear::transport::decode_envelope;

    #[test]
    fn document_http_fixtures_use_public_markdown_fields_and_success() {
        use crate::{config::ApiKey, linear::transport::tests::serve_once};
        let document = r##"{"id":"d1","title":"Notes","content":"# Notes","url":"https://linear.app/document/d1","slugId":"notes","issue":null,"project":{"id":"p1"},"comments":{"nodes":[]},"trashed":false,"archivedAt":null}"##;
        for (operation, response) in [
            (
                "Documents",
                format!(
                    r#"{{"data":{{"documents":{{"nodes":[{document}],"edges":[{{"cursor":"d-cursor"}}],"pageInfo":{{"hasNextPage":false,"endCursor":"d-cursor"}}}}}}}}"#
                ),
            ),
            (
                "Document",
                format!(r#"{{"data":{{"document":{document}}}}}"#),
            ),
            (
                "CreateDocument",
                format!(
                    r#"{{"data":{{"documentCreate":{{"success":true,"document":{document}}}}}}}"#
                ),
            ),
            (
                "UpdateDocument",
                format!(
                    r#"{{"data":{{"documentUpdate":{{"success":true,"document":{document}}}}}}}"#
                ),
            ),
            (
                "DeleteDocument",
                r#"{"data":{"documentDelete":{"success":true}}}"#.into(),
            ),
        ] {
            let (endpoint, captured) = serve_once("200 OK", &response);
            let client = LinearClient::with_test_endpoint(ApiKey::for_test("secret"), &endpoint);
            match operation {
                "Documents" => assert_eq!(
                    client
                        .documents(json!({"project":{"id":{"eq":"p1"}}}), 100, None)
                        .unwrap()
                        .item_cursors,
                    ["d-cursor"]
                ),
                "Document" => assert_eq!(client.document("d1").unwrap().id, "d1"),
                "CreateDocument" => assert_eq!(
                    client
                        .document_create(
                            json!({"title":"Notes","projectId":"p1","content":"# Notes"})
                        )
                        .unwrap()
                        .id,
                    "d1"
                ),
                "UpdateDocument" => assert_eq!(
                    client
                        .document_update("d1", json!({"title":"Notes"}))
                        .unwrap()
                        .id,
                    "d1"
                ),
                "DeleteDocument" => assert_eq!(client.document_delete("d1").unwrap(), "d1"),
                _ => unreachable!(),
            }
            let request = captured.join().unwrap();
            assert!(request.contains(&format!(r#""operationName":"{operation}""#)));
            assert!(
                !request
                    .split("\r\n\r\n")
                    .nth(1)
                    .unwrap_or("")
                    .contains("secret")
            );
        }
        for operation in [
            DOCUMENTS,
            DOCUMENT,
            DOCUMENT_CREATE,
            DOCUMENT_UPDATE,
            DOCUMENT_DELETE,
        ] {
            assert!(
                !operation.contains("contentData")
                    && !operation.contains("bodyData")
                    && !operation.contains("teamId")
                    && !operation.contains("cycleId")
            );
        }
    }
    #[test]
    fn issue_collections_request_complete_stable_json_fields() {
        for query in [MINE, SEARCH] {
            assert!(query.contains("edges { cursor }"));
            assert!(query.contains("$sort: [IssueSortInput!]"));
            assert!(query.contains("sort: $sort"));
            assert!(!query.contains("IssueOrderBy"));
            assert!(query.contains("description"));
            assert!(query.contains("branchName"));
        }
        assert!(SEARCH.contains("issues(filter:"));
        assert!(!SEARCH.contains("searchIssues"));
        assert!(ARCHIVE.contains("issueArchive"));
        assert!(COMMENTS.contains("comments(first:") && COMMENTS.contains("edges { cursor }"));
        assert!(COMMENT_CREATE.contains("body") && !COMMENT_CREATE.contains("bodyData"));
        assert!(COMMENT_UPDATE.contains("body") && !COMMENT_UPDATE.contains("bodyData"));
        assert!(MILESTONES.contains("projectMilestones(filter:"));
        for query in [MILESTONES, MILESTONE, MILESTONE_CREATE, MILESTONE_UPDATE] {
            assert!(query.contains("project { id }"));
            assert!(!query.contains("projectId"));
        }
        assert!(
            MILESTONE_CREATE.contains("projectMilestoneCreate")
                && MILESTONE_UPDATE.contains("projectMilestoneUpdate")
                && MILESTONE_DELETE.contains("projectMilestoneDelete")
        );
        assert!(PROJECTS.contains("projects(filter:") && PROJECTS.contains("edges { cursor }"));
        assert!(
            PROJECT_CREATE.contains("ProjectCreateInput")
                && PROJECT_CREATE.contains("success project")
        );
    }

    #[test]
    fn issue_filter_http_fixture_uses_live_schema_sort_contract() {
        use crate::{config::ApiKey, linear::transport::tests::serve_once};
        let response = r#"{"data":{"issues":{"nodes":[],"edges":[],"pageInfo":{"hasNextPage":false,"endCursor":null}}}}"#;

        let (endpoint, captured) = serve_once("200 OK", response);
        let client = LinearClient::with_test_endpoint(ApiKey::for_test("secret"), &endpoint);

        client
            .issues_filtered(
                None,
                Some("ENG"),
                None,
                Some("priority"),
                None,
                None,
                1,
                None,
            )
            .unwrap();

        let request = captured.join().unwrap();
        assert!(request.contains("IssueSortInput"));
        assert!(request.contains(r#""sort":[{"workflowState":{"order":"Descending"}},{"priority":{"nulls":"last","order":"Descending"}},{"manual":{"nulls":"last","order":"Ascending"}}]"#));
        assert!(!request.contains("IssueOrderBy"));

        let (endpoint, captured) = serve_once("200 OK", response);
        let client = LinearClient::with_test_endpoint(ApiKey::for_test("secret"), &endpoint);
        client
            .issues_filtered(Some("needle"), None, None, None, None, None, 1, None)
            .unwrap();

        let request = captured.join().unwrap();
        assert!(request.contains("containsIgnoreCase"));
        assert!(request.contains("searchableContent"));
        assert!(!request.contains(r#""identifier":{"containsIgnoreCase""#));
    }

    #[test]
    fn mutation_operations_decode_success_and_reject_unsuccessful_results() {
        // Mutation result decoding is deliberately centralized: these fixtures protect the
        // public operation names and `success` handling without a live Linear workspace.
        for operation in [CREATE, UPDATE, COMMENT_CREATE, COMMENT_UPDATE] {
            assert!(operation.contains("success"));
        }
        assert!(ARCHIVE.contains("success") && !ARCHIVE.contains("entityId"));
        assert!(DOCUMENT_DELETE.contains("success") && !DOCUMENT_DELETE.contains("entityId"));
        for operation in [COMMENT_DELETE, MILESTONE_DELETE] {
            assert!(operation.contains("success entityId"));
        }
        // GraphQL-level failures must win even when a response carries data.
        assert!(
            decode_envelope::<serde_json::Value>(br#"{"data":{},"errors":[{"message":"denied"}]}"#)
                .is_err()
        );
    }

    #[test]
    fn mutation_http_fixtures_send_expected_operations_and_handle_failures() {
        use crate::{config::ApiKey, linear::transport::tests::serve_once};
        let issue = r#"{"id":"i1","identifier":"ENG-1","title":"T","description":null,"url":"https://linear.app/x","branchName":null,"priority":null,"updatedAt":null,"createdAt":null,"team":{"id":"t1","key":"ENG","name":"Engineering"},"state":null,"assignee":null}"#;
        let comment = r#"{"id":"c1","body":"B","issueId":"i1","parentId":null,"user":null,"createdAt":null,"updatedAt":null}"#;
        let cases = [
            (
                "CreateIssue",
                format!(r#"{{"data":{{"issueCreate":{{"success":true,"issue":{issue}}}}}}}"#),
            ),
            (
                "UpdateIssue",
                format!(r#"{{"data":{{"issueUpdate":{{"success":true,"issue":{issue}}}}}}}"#),
            ),
            (
                "ArchiveIssue",
                r#"{"data":{"issueArchive":{"success":true}}}"#.to_owned(),
            ),
            (
                "CreateComment",
                format!(r#"{{"data":{{"commentCreate":{{"success":true,"comment":{comment}}}}}}}"#),
            ),
            (
                "UpdateComment",
                format!(r#"{{"data":{{"commentUpdate":{{"success":true,"comment":{comment}}}}}}}"#),
            ),
            (
                "DeleteComment",
                r#"{"data":{"commentDelete":{"success":true,"entityId":"c1"}}}"#.to_owned(),
            ),
        ];
        for (operation, response) in cases {
            let (endpoint, captured) = serve_once("200 OK", &response);
            let client = LinearClient::with_test_endpoint(ApiKey::for_test("secret"), &endpoint);
            match operation {
                "CreateIssue" => assert_eq!(
                    client
                        .create(json!({"teamId":"t1","title":"T"}))
                        .unwrap()
                        .id,
                    "i1"
                ),
                "UpdateIssue" => {
                    assert_eq!(client.update("i1", json!({"title":"T"})).unwrap().id, "i1")
                }
                "ArchiveIssue" => assert_eq!(client.archive("i1").unwrap(), "i1"),
                "CreateComment" => assert_eq!(
                    client
                        .comment_create(json!({"issueId":"i1","body":"B"}))
                        .unwrap()
                        .id,
                    "c1"
                ),
                "UpdateComment" => assert_eq!(
                    client.comment_update("c1", json!({"body":"B"})).unwrap().id,
                    "c1"
                ),
                "DeleteComment" => assert_eq!(client.comment_delete("c1").unwrap(), "c1"),
                _ => unreachable!(),
            }
            assert!(
                captured
                    .join()
                    .unwrap()
                    .contains(&format!(r#""operationName":"{operation}""#))
            );
        }
        let (endpoint, captured) =
            serve_once("200 OK", r#"{"data":{"issueArchive":{"success":false}}}"#);
        let client = LinearClient::with_test_endpoint(ApiKey::for_test("secret"), &endpoint);
        assert!(client.archive("i1").is_err());
        assert!(captured.join().unwrap().contains("ArchiveIssue"));
    }

    #[test]
    fn team_and_user_http_fixtures_preserve_cursors_and_creation_input() {
        use crate::{config::ApiKey, linear::transport::tests::serve_once};
        let team = r#"{"id":"t1","key":"ENG","name":"Engineering","description":null,"color":null,"icon":null}"#;
        let user = r#"{"id":"u1","name":"Ada","displayName":"Ada Lovelace","email":"ada@example.test","active":true,"avatarUrl":null}"#;
        let cases = [
            (
                "Teams",
                format!(
                    r#"{{"data":{{"teams":{{"nodes":[{team}],"edges":[{{"cursor":"team-cursor"}}],"pageInfo":{{"hasNextPage":true,"endCursor":"team-cursor"}}}}}}}}"#
                ),
            ),
            (
                "TeamMembers",
                format!(
                    r#"{{"data":{{"team":{{"members":{{"nodes":[{user}],"edges":[{{"cursor":"member-cursor"}}],"pageInfo":{{"hasNextPage":false,"endCursor":"member-cursor"}}}}}}}}}}"#
                ),
            ),
            (
                "Users",
                format!(
                    r#"{{"data":{{"viewer":{{"organization":{{"users":{{"nodes":[{user}],"edges":[{{"cursor":"user-cursor"}}],"pageInfo":{{"hasNextPage":false,"endCursor":"user-cursor"}}}}}}}}}}}}"#
                ),
            ),
            (
                "CreateTeam",
                format!(r#"{{"data":{{"teamCreate":{{"success":true,"team":{team}}}}}}}"#),
            ),
        ];
        for (operation, response) in cases {
            let (endpoint, captured) = serve_once("200 OK", &response);
            let client = LinearClient::with_test_endpoint(ApiKey::for_test("secret"), &endpoint);
            match operation {
                "Teams" => assert_eq!(
                    client.teams(100, None).unwrap().item_cursors,
                    ["team-cursor"]
                ),
                "TeamMembers" => assert_eq!(
                    client.team_members("t1", true, 100, None).unwrap().items[0].id,
                    "u1"
                ),
                "Users" => assert_eq!(
                    client.users(false, 100, None).unwrap().items[0]
                        .email
                        .as_deref(),
                    Some("ada@example.test")
                ),
                "CreateTeam" => assert_eq!(
                    client
                        .team_create(json!({"name":"Engineering","key":"ENG"}))
                        .unwrap()
                        .key,
                    "ENG"
                ),
                _ => unreachable!(),
            }
            let request = captured.join().unwrap();
            assert!(request.contains(&format!(r#""operationName":"{operation}""#)));
            assert!(
                !request
                    .split("\r\n\r\n")
                    .nth(1)
                    .unwrap_or("")
                    .contains("secret")
            );
            if operation == "TeamMembers" {
                assert!(request.contains(r#""includeDisabled":true"#));
            }
        }
        assert!(TEAM_CREATE.contains("TeamCreateInput"));
        assert!(TEAM_MEMBERS.contains("edges { cursor }"));
        assert!(USERS.contains("includeDisabled"));
    }
    #[test]
    fn milestone_http_fixtures_use_lowercase_public_roots_and_success() {
        use crate::{config::ApiKey, linear::transport::tests::serve_once};
        let milestone = r#"{"id":"m1","name":"Beta","description":null,"project":{"id":"p1"},"targetDate":"2028-01-01"}"#;
        let cases = [
            (
                "ProjectMilestones",
                format!(
                    r#"{{"data":{{"projectMilestones":{{"nodes":[{milestone}],"edges":[{{"cursor":"m-cursor"}}],"pageInfo":{{"hasNextPage":false,"endCursor":"m-cursor"}}}}}}}}"#
                ),
            ),
            (
                "ProjectMilestone",
                format!(r#"{{"data":{{"projectMilestone":{milestone}}}}}"#),
            ),
            (
                "CreateProjectMilestone",
                format!(
                    r#"{{"data":{{"projectMilestoneCreate":{{"success":true,"projectMilestone":{milestone}}}}}}}"#
                ),
            ),
            (
                "UpdateProjectMilestone",
                format!(
                    r#"{{"data":{{"projectMilestoneUpdate":{{"success":true,"projectMilestone":{milestone}}}}}}}"#
                ),
            ),
            (
                "DeleteProjectMilestone",
                r#"{"data":{"projectMilestoneDelete":{"success":true,"entityId":"m1"}}}"#
                    .to_owned(),
            ),
        ];
        for (operation, response) in cases {
            let (endpoint, captured) = serve_once("200 OK", &response);
            let client = LinearClient::with_test_endpoint(ApiKey::for_test("secret"), &endpoint);
            match operation {
                "ProjectMilestones" => assert_eq!(
                    client
                        .milestones(Some("p1"), 100, None)
                        .unwrap()
                        .item_cursors,
                    ["m-cursor"]
                ),
                "ProjectMilestone" => assert_eq!(client.milestone("m1").unwrap().id, "m1"),
                "CreateProjectMilestone" => assert_eq!(
                    client
                        .milestone_create(json!({"projectId":"p1","name":"Beta"}))
                        .unwrap()
                        .id,
                    "m1"
                ),
                "UpdateProjectMilestone" => assert_eq!(
                    client
                        .milestone_update("m1", json!({"name":"Beta"}))
                        .unwrap()
                        .id,
                    "m1"
                ),
                "DeleteProjectMilestone" => {
                    assert_eq!(client.milestone_delete("m1").unwrap(), "m1")
                }
                _ => unreachable!(),
            }
            let request = captured.join().unwrap();
            assert!(request.contains(&format!(r#""operationName":"{operation}""#)));
            assert!(
                !request
                    .split("\r\n\r\n")
                    .nth(1)
                    .unwrap_or("")
                    .contains("secret")
            );
        }
    }

    #[test]
    fn project_http_fixtures_preserve_filters_cursors_and_create_input() {
        use crate::{config::ApiKey, linear::transport::tests::serve_once};
        let project = r#"{"id":"p1","name":"Roadmap","slugId":"roadmap","description":null,"content":null,"url":"https://linear.app/project/p1","targetDate":"2028-01-01","teams":{"nodes":[]},"lead":null,"members":{"nodes":[]}}"#;
        let response = format!(
            r#"{{"data":{{"projects":{{"nodes":[{project}],"edges":[{{"cursor":"p-cursor"}}],"pageInfo":{{"hasNextPage":false,"endCursor":"p-cursor"}}}}}}}}"#
        );
        let (endpoint, captured) = serve_once("200 OK", &response);
        let client = LinearClient::with_test_endpoint(ApiKey::for_test("secret"), &endpoint);
        let found = client.projects(Some("t1"), 100, None).unwrap();
        assert_eq!(found.items[0].slug_id.as_deref(), Some("roadmap"));
        assert_eq!(found.item_cursors, ["p-cursor"]);
        let request = captured.join().unwrap();
        assert!(request.contains(r#""operationName":"Projects""#));
        assert!(request.contains(r#""accessibleTeams":{"some":{"id":{"eq":"t1"}}}"#));

        let response =
            format!(r#"{{"data":{{"projectCreate":{{"success":true,"project":{project}}}}}}}"#);
        let (endpoint, captured) = serve_once("200 OK", &response);
        let client = LinearClient::with_test_endpoint(ApiKey::for_test("secret"), &endpoint);
        assert_eq!(
            client
                .project_create(
                    json!({"name":"Roadmap","teamIds":["t1"],"targetDate":"2028-01-01"})
                )
                .unwrap()
                .id,
            "p1"
        );
        let request = captured.join().unwrap();
        assert!(request.contains(r#""operationName":"CreateProject""#));
        assert!(request.contains(r#""teamIds":["t1"]"#));
    }
}
