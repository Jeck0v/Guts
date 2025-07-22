use crate::core::hash;
use crate::core::object::Commit;
use anyhow::Result;
use clap::Args;
use std::env;
use std::path::PathBuf;

#[derive(Args)]
pub struct CommitObject {
    pub tree: String,
    #[arg(short = 'p', long)]
    pub parent: Option<String>,
    #[arg(short = 'm', long)]
    pub message: String,
    /// Author name and email in format "Name <email>"
    #[arg(long, default_value = "guts <guts@example.com>")]
    pub author: String,
    /// Committer name and email in format "Name <email>"
    #[arg(long, default_value = "guts <guts@example.com>")]
    pub committer: String,
    /// Unix timestamp for author date
    #[arg(long)]
    pub author_date: Option<i64>,
    /// Unix timestamp for committer date
    #[arg(long)]
    pub committer_date: Option<i64>,
    /// Current directory for the operation (injected by TUI)
    pub dir: Option<PathBuf>,
}

pub fn run(args: &CommitObject) -> Result<String> {
    let current_dir = args
        .dir
        .clone()
        .unwrap_or_else(|| env::current_dir().expect("could not get the current dir"));

    let git_dir = current_dir.join(".git");

    if !git_dir.exists() {
        anyhow::bail!("No .git directory at {}", git_dir.display());
    }

    let now = chrono::Utc::now().timestamp();
    let author_date = args.author_date.unwrap_or(now);
    let committer_date = args.committer_date.unwrap_or(author_date);

    let commit = Commit {
        tree: args.tree.clone(),
        parent: args.parent.clone(),
        message: args.message.clone(),
        author: args.author.clone(),
        committer: args.committer.clone(),
        author_date,
        committer_date,
    };

    let oid = hash::write_object(&commit)?;
    Ok(oid)
}
