use std::{path::Path, process::Command};

use serde::Deserialize;

use crate::error::Error;

/// Resolve a raw key value, shelling out to a secret manager CLI if prefixed.
///
/// - `op://vault/item/field` → `op read "op://..." --no-newline`
/// - `bw://ItemName`         → `bw get password "ItemName"`
/// - anything else           → returned as-is
pub fn resolve(raw: &str) -> Result<String, Error> {
    if raw.starts_with("op://") {
        let op = which::which("op").map_err(|_| {
            Error::InvalidArgument("op:// secret specified but `op` CLI not found in PATH".into())
        })?;
        let out = Command::new(op).args(["read", raw, "--no-newline"]).output()?;
        if !out.status.success() {
            let msg = String::from_utf8_lossy(&out.stderr).trim().to_string();
            return Err(Error::InvalidArgument(format!("op read failed: {msg}")));
        }
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else if let Some(item) = raw.strip_prefix("bw://") {
        let bw = which::which("bw").map_err(|_| {
            Error::InvalidArgument("bw:// secret specified but `bw` CLI not found in PATH".into())
        })?;
        let out = Command::new(bw).args(["get", "password", item]).output()?;
        if !out.status.success() {
            let msg = String::from_utf8_lossy(&out.stderr).trim().to_string();
            return Err(Error::InvalidArgument(format!("bw get failed: {msg}")));
        }
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        Ok(raw.to_string())
    }
}

/// Try to auto-discover an API key by searching secret managers for items
/// matching the given URL. Returns the secret if exactly one match is found,
/// or an error listing the available items if there are multiple.
pub fn discover(url: &str) -> Result<String, Error> {
    if let Ok(op) = which::which("op") {
        return discover_op(&op, url);
    }
    if let Ok(bw) = which::which("bw") {
        return discover_bw(&bw, url);
    }
    Err(Error::InvalidArgument(
        "Linear API key required: set LINEAR_API_KEY, pass --linear-api-key, \
         or install a secret manager CLI (op, bw)"
            .into(),
    ))
}

// -- 1Password ---------------------------------------------------------------

#[derive(Deserialize)]
struct OpItem {
    title: String,
    vault: OpVault,
}

#[derive(Deserialize)]
struct OpVault {
    name: String,
}

fn discover_op(op: &Path, url: &str) -> Result<String, Error> {
    let out = Command::new(op).args(["item", "list", "--url", url, "--format", "json"]).output()?;
    if !out.status.success() {
        let msg = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(Error::InvalidArgument(format!("op item list failed: {msg}")));
    }

    let items: Vec<OpItem> = serde_json::from_slice(&out.stdout)
        .map_err(|e| Error::InvalidArgument(format!("failed to parse op output: {e}")))?;

    match items.as_slice() {
        [] => Err(Error::InvalidArgument(format!("no 1Password items found with URL \"{url}\""))),
        [item] => {
            let uri = format!("op://{}/{}/credential", item.vault.name, item.title);
            resolve(&uri)
        }
        items => {
            let list: String = items
                .iter()
                .map(|i| format!("  op://{}/{}/credential", i.vault.name, i.title))
                .collect::<Vec<_>>()
                .join("\n");
            Err(Error::InvalidArgument(format!(
                "multiple 1Password items found for \"{url}\", pass one explicitly:\n{list}"
            )))
        }
    }
}

// -- Bitwarden ---------------------------------------------------------------

#[derive(Deserialize)]
struct BwItem {
    name: String,
    login: Option<BwLogin>,
}

#[derive(Deserialize)]
struct BwLogin {
    password: Option<String>,
}

fn discover_bw(bw: &Path, url: &str) -> Result<String, Error> {
    let out = Command::new(bw).args(["list", "items", "--url", url]).output()?;
    if !out.status.success() {
        let msg = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(Error::InvalidArgument(format!("bw list failed: {msg}")));
    }

    let items: Vec<BwItem> = serde_json::from_slice(&out.stdout)
        .map_err(|e| Error::InvalidArgument(format!("failed to parse bw output: {e}")))?;

    match items.as_slice() {
        [] => Err(Error::InvalidArgument(format!("no Bitwarden items found with URL \"{url}\""))),
        [item] => item.login.as_ref().and_then(|l| l.password.clone()).ok_or_else(|| {
            Error::InvalidArgument(format!(
                "Bitwarden item \"{}\" has no password field",
                item.name
            ))
        }),
        items => {
            let list: String =
                items.iter().map(|i| format!("  bw://{}", i.name)).collect::<Vec<_>>().join("\n");
            Err(Error::InvalidArgument(format!(
                "multiple Bitwarden items found for \"{url}\", pass one explicitly:\n{list}"
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_passthrough() {
        assert_eq!(resolve("lin_api_abc123").unwrap(), "lin_api_abc123");
    }

    #[test]
    fn literal_passthrough_empty() {
        assert_eq!(resolve("").unwrap(), "");
    }

    #[test]
    #[ignore = "requires op CLI installed and authenticated"]
    fn op_prefix_dispatches_to_op_cli() {
        let result = resolve("op://Private/Linear/api-key");
        assert!(result.is_ok());
    }

    #[test]
    #[ignore = "requires bw CLI installed and unlocked"]
    fn bw_prefix_dispatches_to_bw_cli() {
        let result = resolve("bw://LinearAPIKey");
        assert!(result.is_ok());
    }

    #[test]
    fn op_prefix_without_op_installed_errors() {
        if which::which("op").is_ok() {
            return;
        }
        let err = resolve("op://vault/item/field").unwrap_err();
        assert!(err.to_string().contains("`op` CLI not found"));
    }

    #[test]
    fn bw_prefix_without_bw_installed_errors() {
        if which::which("bw").is_ok() {
            return;
        }
        let err = resolve("bw://SomeItem").unwrap_err();
        assert!(err.to_string().contains("`bw` CLI not found"));
    }

    #[test]
    fn discover_no_cli_available_errors() {
        if which::which("op").is_ok() || which::which("bw").is_ok() {
            return;
        }
        let err = discover("api.linear.app").unwrap_err();
        assert!(err.to_string().contains("install a secret manager CLI"));
    }
}
