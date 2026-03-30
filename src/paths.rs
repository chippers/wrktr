use std::path::{Path, PathBuf};

use directories::BaseDirs;

fn base() -> PathBuf {
    BaseDirs::new().expect("could not determine home directory").home_dir().join("code")
}

fn home() -> PathBuf {
    BaseDirs::new().expect("could not determine home directory").home_dir().to_path_buf()
}

fn claude_project_key(project_path: &Path) -> String {
    let s = project_path.to_string_lossy();
    let s = s.trim_end_matches('/');
    s.replace('/', "-")
}

/// Returns `~/.claude/projects/{key}/` for the given project path.
pub fn claude_project_dir(project_path: &Path) -> PathBuf {
    home().join(".claude").join("projects").join(claude_project_key(project_path))
}

/// Returns `~/.claude/projects/{key}/memory/` for the given project path.
pub fn claude_memory_dir(project_path: &Path) -> PathBuf {
    claude_project_dir(project_path).join("memory")
}

pub struct Repo {
    pub org: String,
    pub name: String,
}

impl Repo {
    pub fn new(org: &str, name: &str) -> Self {
        Self { org: org.to_string(), name: name.to_string() }
    }

    pub fn detect(cwd: &Path) -> Option<Self> {
        let rel = cwd.strip_prefix(base()).ok()?;
        let mut parts = rel.components();

        let first = parts.next()?.as_os_str().to_str()?.to_string();

        let (org, name) = if first == "worktree" {
            // ~/code/worktree/{org}/{name}/...
            let org = parts.next()?.as_os_str().to_str()?.to_string();
            let name = parts.next()?.as_os_str().to_str()?.to_string();
            (org, name)
        } else {
            // ~/code/{org}/{name}[/...]
            let name = parts.next()?.as_os_str().to_str()?.to_string();
            (first, name)
        };

        Some(Self { org, name })
    }

    pub fn path(&self) -> PathBuf {
        base().join(&self.org).join(&self.name)
    }

    pub fn worktree_path(&self, branch: &str) -> PathBuf {
        base().join("worktree").join(&self.org).join(&self.name).join(branch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_path_structure() {
        let r = Repo::new("chippers", "wrktr");
        assert_eq!(r.path(), base().join("chippers").join("wrktr"));
    }

    #[test]
    fn worktree_path_structure() {
        let r = Repo::new("chippers", "wrktr");
        assert_eq!(
            r.worktree_path("my-feature"),
            base().join("worktree").join("chippers").join("wrktr").join("my-feature")
        );
    }

    #[test]
    fn detect_from_repo_dir() {
        let cwd = base().join("chippers").join("wrktr");
        let r = Repo::detect(&cwd).unwrap();
        assert_eq!(r.org, "chippers");
        assert_eq!(r.name, "wrktr");
    }

    #[test]
    fn detect_from_worktree_dir() {
        let cwd = base().join("worktree").join("chippers").join("wrktr").join("my-feature");
        let r = Repo::detect(&cwd).unwrap();
        assert_eq!(r.org, "chippers");
        assert_eq!(r.name, "wrktr");
    }

    #[test]
    fn detect_from_unrelated_path() {
        assert!(Repo::detect(Path::new("/tmp/random")).is_none());
    }

    #[test]
    fn detect_from_code_root() {
        assert!(Repo::detect(&base()).is_none());
    }

    #[test]
    fn claude_project_dir_derives_key() {
        let p = base().join("chippers").join("wrktr");
        let dir = claude_project_dir(&p);
        let key = claude_project_key(&p);
        assert_eq!(dir, home().join(".claude").join("projects").join(key));
    }

    #[test]
    fn claude_memory_dir_appends_memory() {
        let p = base().join("chippers").join("wrktr");
        let dir = claude_memory_dir(&p);
        let key = claude_project_key(&p);
        assert_eq!(dir, home().join(".claude").join("projects").join(key).join("memory"));
    }
}
