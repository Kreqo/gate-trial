// SPDX-FileCopyrightText: Kreqo contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The scan of commit messages, notes and tag messages (SEC-03): each is
//! read whole, as `git log` and `git notes show` show it, so that a finding
//! names the line of the message that holds the secret.

use anyhow::{Context, Result};

use crate::support::{Repo, fake_token};

/// Fails unless the history scan finds the token on the third line of the
/// message in the file named after `object`, with the token masked.
fn finds_on_the_third_line(repo: &Repo, object: &str) -> Result<()> {
    let run = repo.xtask(&["check", "secrets"]);
    run.fails_with(object)?;
    run.fails_with("│ 3 │ token = REDACTED")
}

/// A commit's or a tag's message starts after its headers and the blank
/// line below them, and a note is all message, so a finding's line is the
/// message's own, not the object's.
#[test]
#[ignore = "needs Betterleaks; cargo xtask check tool-tests runs it"]
fn a_finding_names_its_line_in_the_message() -> Result<()> {
    let token = fake_token();
    let message = format!("Add the settings\n\ntoken = {token}\n");

    let repo = Repo::new()?;
    let commit = repo.commit_all(&message)?;
    finds_on_the_third_line(&repo, &format!("commit {commit}"))?;

    let repo = Repo::new()?;
    let file = repo.write_outside("NOTE", &message)?;
    let file = file.to_str().context("the scratch path isn't UTF-8")?;
    repo.git(&["notes", "add", "--file", file, "HEAD"])?;
    let head = repo.git(&["rev-parse", "HEAD"])?;
    finds_on_the_third_line(&repo, &format!("note {}", head.trim()))?;

    let repo = Repo::new()?;
    let file = repo.write_outside("TAG_EDITMSG", &message)?;
    let file = file.to_str().context("the scratch path isn't UTF-8")?;
    repo.git(&["tag", "--annotate", "v1", "--file", file])?;
    let tag = repo.git(&["rev-parse", "v1"])?;
    finds_on_the_third_line(&repo, &format!("tag {}", tag.trim()))
}

/// A note made from a blob keeps the blob's bytes, final newline or not,
/// so a token that ends it is read to its last character.
#[test]
#[ignore = "needs Betterleaks; cargo xtask check tool-tests runs it"]
fn the_scan_reads_a_message_to_its_last_byte() -> Result<()> {
    let repo = Repo::new()?;
    let file = repo.write_outside("NOTE", fake_token())?;
    let file = file.to_str().context("the scratch path isn't UTF-8")?;
    let blob = repo.git(&["hash-object", "-w", file])?;
    repo.git(&["notes", "add", "-C", blob.trim(), "HEAD"])?;
    let head = repo.git(&["rev-parse", "HEAD"])?;
    let run = repo.xtask(&["check", "secrets"]);
    run.fails_with(&format!("note {}", head.trim()))?;
    run.fails_with("github-pat")
}
