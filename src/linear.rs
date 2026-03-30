use serde::Deserialize;

use crate::error::Error;

#[derive(Deserialize)]
struct Response {
    data: Data,
}

#[derive(Deserialize)]
struct Data {
    issue: Issue,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Issue {
    git_branch_name: Option<String>,
}

pub fn fetch_branch_name(issue_id: &str) -> Result<String, Error> {
    let api_key = std::env::var("LINEAR_API_KEY")
        .map_err(|_| Error::Linear("LINEAR_API_KEY not set".into()))?;

    let query = format!(r#"{{ issue(id: "{issue_id}") {{ gitBranchName }} }}"#);
    let body = serde_json::json!({ "query": query });

    let resp: Response = reqwest::blocking::Client::new()
        .post("https://api.linear.app/graphql")
        .header("Authorization", &api_key)
        .json(&body)
        .send()?
        .json()?;

    resp.data
        .issue
        .git_branch_name
        .ok_or_else(|| Error::Linear(format!("no branch name for issue {issue_id}")))
}

/// Extract issue ID from a Linear URL like https://linear.app/WORKSPACE/issue/FS-1801/slug
pub fn parse_issue_url(url: &str) -> Option<String> {
    let after = url.split("/issue/").nth(1)?;
    Some(after.split('/').next()?.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_url_with_slug() {
        let url = "https://linear.app/org/issue/FS-1801/some-issue-title";
        assert_eq!(parse_issue_url(url), Some("FS-1801".into()));
    }

    #[test]
    fn parse_url_without_slug() {
        let url = "https://linear.app/org/issue/FS-42";
        assert_eq!(parse_issue_url(url), Some("FS-42".into()));
    }

    #[test]
    fn parse_url_no_issue_segment() {
        assert_eq!(parse_issue_url("https://linear.app/org"), None);
    }

    #[test]
    fn parse_url_empty() {
        assert_eq!(parse_issue_url(""), None);
    }
}
