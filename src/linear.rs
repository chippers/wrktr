use serde::Deserialize;

use crate::error::Error;

#[derive(Deserialize)]
struct Response {
    data: Option<Data>,
    errors: Option<Vec<GqlError>>,
}

#[derive(Deserialize)]
struct GqlError {
    message: String,
}

#[derive(Deserialize)]
struct Data {
    issue: Issue,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Issue {
    branch_name: Option<String>,
}

pub fn fetch_branch_name(issue_id: &str, api_key: &str) -> Result<String, Error> {
    let query = format!(r#"{{ issue(id: "{issue_id}") {{ branchName }} }}"#);
    let body = serde_json::json!({ "query": query });

    let resp: Response = reqwest::blocking::Client::new()
        .post("https://api.linear.app/graphql")
        .header("Authorization", api_key)
        .json(&body)
        .send()?
        .json()?;

    if let Some(errors) = resp.errors {
        let msgs: Vec<_> = errors.iter().map(|e| e.message.as_str()).collect();
        return Err(Error::Linear(msgs.join("; ")));
    }

    resp.data
        .and_then(|d| d.issue.branch_name)
        .ok_or_else(|| Error::Linear(format!("no branch name for issue {issue_id}")))
}

/// Returns true if `s` looks like a Linear issue ID (e.g. `FS-1801`, `ABC-42`).
pub fn looks_like_issue_id(s: &str) -> bool {
    let Some((prefix, num)) = s.split_once('-') else { return false };
    !prefix.is_empty()
        && prefix.chars().all(|c| c.is_ascii_uppercase())
        && !num.is_empty()
        && num.chars().all(|c| c.is_ascii_digit())
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

    #[test]
    fn looks_like_issue_id_valid() {
        assert!(looks_like_issue_id("FS-1801"));
        assert!(looks_like_issue_id("ABC-1"));
        assert!(looks_like_issue_id("TEAM-42"));
    }

    #[test]
    fn looks_like_issue_id_invalid() {
        assert!(!looks_like_issue_id("fs-1801")); // lowercase
        assert!(!looks_like_issue_id("FS-")); // no number
        assert!(!looks_like_issue_id("-123")); // no prefix
        assert!(!looks_like_issue_id("my-feature")); // lowercase
        assert!(!looks_like_issue_id("FS-123abc")); // non-digit suffix
    }
}
