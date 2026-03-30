use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "wrktr")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Branch name, org/repo, or Linear issue URL
    #[arg(value_name = "TARGET")]
    pub target: Option<String>,

    /// Linear issue ID (e.g. FS-1801)
    #[arg(short, long)]
    pub issue: Option<String>,

    /// Linear API key (literal, op://vault/item/field, or bw://ItemName). Falls back to
    /// LINEAR_API_KEY env var.
    #[arg(long, env = "LINEAR_API_KEY")]
    pub linear_api_key: Option<String>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Clone a repo to ~/code/{org}/{repo}
    Clone { repo: String },
    /// Prune stale worktree references
    Prune,
    /// Remove a worktree
    Rm {
        /// Worktree name to remove
        worktree: Option<String>,
        /// Remove all worktrees
        #[arg(long)]
        all: bool,
    },
    /// Print the Linear-suggested git branch name for an issue
    Linear {
        /// Issue ID (e.g. FS-1801)
        issue: String,
    },
}
