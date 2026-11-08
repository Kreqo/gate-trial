// SPDX-FileCopyrightText: Kreqo contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Kreqo's project tasks, run as `cargo xtask` (ARCH-06).

mod attributes;
mod check;
mod codeowners;
mod crates;
mod deny;
mod git;
mod guard;
mod hooks;
mod rules;
mod scratch;
mod secrets;
mod tools;
mod verdict;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context, Result, ensure};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(about = "Kreqo's project tasks (ARCH-06)")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Runs what CI runs: every step, or only the ones named.
    Check {
        /// The steps to run, in order; all of them when none is named.
        #[arg(value_enum)]
        steps: Vec<check::Step>,
    },
    /// Writes .github/CODEOWNERS from conventions/protected-paths.txt (CI-15).
    Codeowners,
    /// Classifies a pull request's change as protected or not, and fails a
    /// protected one whose body links no decision record (CI-15). With
    /// GITHUB_OUTPUT set, appends `protected` and `decision` to that file.
    Guard {
        /// The pull request's base commit; the change is read from its merge
        /// base with HEAD.
        #[arg(long)]
        base: String,
        /// The file that holds the pull request's body.
        #[arg(long)]
        body_file: Option<PathBuf>,
    },
    /// Installs the git hooks (SEC-03).
    Hooks,
    /// Scans the staged changes for secrets; the pre-commit hook runs it (SEC-03).
    PreCommit,
    /// Scans a commit message for secrets; the commit-msg hook runs it (SEC-03).
    CommitMsg {
        /// The file that holds the message.
        file: PathBuf,
    },
    /// The rules engine: checks the rules data and generates its views (ARCH-06).
    Rules {
        #[command(subcommand)]
        command: rules::Command,
    },
    /// Scans every branch's and tag's history for secrets; the daily scan runs it (SEC-03).
    ScanAllRefs,
    /// Scans a pull request's title and description, from PULL_REQUEST_TITLE and
    /// PULL_REQUEST_BODY, for secrets; the description workflow runs it (SEC-03).
    ScanPullRequest,
    /// Prints the pinned tool versions as GitHub Actions outputs (CI-07).
    Tools,
    /// Checks the AI review's review.json, and fails unless it approves with
    /// nothing blocking or unchecked (CI-03, CI-18).
    Verdict {
        /// The file that holds the verdict.
        file: PathBuf,
    },
}

fn main() -> ExitCode {
    match run() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::FAILURE
        }
    }
}

/// Runs the command. `Ok(false)` means a check failed and has said why.
fn run() -> Result<bool> {
    let cli = Cli::parse();
    let root = workspace_root()?;
    match cli.command {
        Command::Check { steps } => check::run(&root, &steps),
        Command::Codeowners => codeowners::write(&root).map(|()| true),
        Command::Guard { base, body_file } => guard::run(&root, &base, body_file.as_deref()),
        Command::Hooks => hooks::install(&root).map(|()| true),
        Command::PreCommit => hooks::pre_commit(&root).map(|()| true),
        Command::CommitMsg { file } => hooks::commit_msg(&root, &file).map(|()| true),
        Command::Rules { command } => rules::run(&root, command).map(|()| true),
        Command::ScanAllRefs => secrets::scan_all_refs(&root).map(|()| true),
        Command::ScanPullRequest => secrets::scan_pull_request(&root).map(|()| true),
        Command::Tools => {
            tools::print_github_outputs();
            Ok(true)
        }
        Command::Verdict { file } => verdict::run(&file),
    }
}

/// The workspace root, which holds this crate's folder. `cargo run` sets
/// CARGO_MANIFEST_DIR when it runs xtask; the value compiled in is only a
/// fallback, because a build shared between checkouts would otherwise check
/// the checkout it was compiled in.
fn workspace_root() -> Result<PathBuf> {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .map_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")), PathBuf::from);
    let root = manifest_dir
        .parent()
        .map(Path::to_path_buf)
        .context("xtask's folder has no parent")?;
    ensure!(
        root.join("Cargo.toml").is_file(),
        "{} isn't a workspace root; run cargo xtask from the repository",
        root.display()
    );
    Ok(root)
}
