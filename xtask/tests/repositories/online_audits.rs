// SPDX-FileCopyrightText: Kreqo contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! zizmor's online audits (CI-07, CI-08): they need a token, so in CI the
//! workflows step fails without one, and elsewhere zizmor stays offline.

use anyhow::Result;

use crate::support::Repo;

/// A repository with one workflow that has no findings, and the real
/// zizmor config.
fn repo_with_a_workflow() -> Result<Repo> {
    let repo = Repo::new()?;
    repo.write(
        ".github/zizmor.yml",
        include_str!("../../../.github/zizmor.yml"),
    )?;
    repo.write(
        ".github/workflows/clean.yml",
        "name: Clean\non: push\npermissions: {}\njobs:\n  clean:\n    runs-on: ubuntu-24.04\n    steps:\n      - run: echo clean\n",
    )?;
    repo.commit_all("Add a workflow")?;
    Ok(repo)
}

/// GitHub Actions sets CI to `true`; there, a missing or empty GH_TOKEN
/// fails the step rather than leaving the online audits off. Elsewhere,
/// an empty GH_TOKEN counts as missing, since zizmor refuses an empty one.
#[test]
#[ignore = "needs zizmor; cargo xtask check tool-tests runs it"]
fn ci_needs_a_token_for_the_online_audits() -> Result<()> {
    let repo = repo_with_a_workflow()?;
    repo.xtask(&["check", "workflows"]).passes()?;
    repo.xtask_with(&["check", "workflows"], &[("GH_TOKEN", "")])
        .passes()?;
    let missing = "CI runs zizmor's online audits, but GH_TOKEN is missing or empty";
    repo.xtask_with(&["check", "workflows"], &[("CI", "true")])
        .fails_with(missing)?;
    repo.xtask_with(&["check", "workflows"], &[("CI", "true"), ("GH_TOKEN", "")])
        .fails_with(missing)
}
