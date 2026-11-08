// SPDX-FileCopyrightText: Kreqo contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The AI review's verdict (CI-03, CI-18): `cargo xtask verdict` passes
//! only an approval, and fails closed on anything else.

use anyhow::{Context, Result};

use crate::support::Repo;

/// An approval with nothing blocking or unchecked.
const APPROVE: &str = r#"{"verdict": "approve", "summary": "Adds the parser.", "acceptance_criteria": [], "findings": [], "not_checked": [], "assumptions_to_escalate": []}"#;

/// Runs the command on a verdict file outside the repository, as the
/// workflow's step does.
fn verdict(repo: &Repo, text: Option<&str>) -> Result<crate::support::Run> {
    let file = match text {
        Some(text) => repo.write_outside("review.json", text)?,
        None => repo.outside("review.json"),
    };
    let file = file.to_str().context("the scratch path isn't UTF-8")?;
    Ok(repo.xtask(&["verdict", file]))
}

#[test]
fn passes_only_an_approval() -> Result<()> {
    let repo = Repo::new()?;
    verdict(&repo, Some(APPROVE))?.passes()?;
    verdict(
        &repo,
        Some(&APPROVE.replace("approve", "changes_requested")),
    )?
    .fails_with("The AI review fails: changes_requested")?;
    verdict(&repo, Some(&APPROVE.replace("approve", "needs_human")))?
        .fails_with("The AI review fails: needs_human")
}

#[test]
fn fails_closed_without_a_readable_verdict() -> Result<()> {
    let repo = Repo::new()?;
    verdict(&repo, None)?.fails_with("review.json: missing")?;
    verdict(&repo, Some("\n"))?.fails_with("review.json: empty")?;
    verdict(&repo, Some(&APPROVE.replace("\"findings\"", "\"notes\"")))?
        .fails_with("review.json: a schema error (P02 A.4): unknown field `notes`")
}
