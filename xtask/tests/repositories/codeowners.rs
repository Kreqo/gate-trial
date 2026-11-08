// SPDX-FileCopyrightText: Kreqo contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! CODEOWNERS (CI-15): `cargo xtask codeowners` writes it from the
//! protected-paths list, and `cargo xtask check codeowners` fails until it
//! is written again after the list changes.

use std::fs;

use anyhow::{Result, ensure};

use crate::support::Repo;

#[test]
fn keeps_codeowners_in_step_with_the_list() -> Result<()> {
    let repo = Repo::new()?;
    repo.write(
        "conventions/protected-paths.txt",
        "owner @owner\n/conventions  # convention files\n",
    )?;
    repo.xtask(&["check", "codeowners"])
        .fails_with("CODEOWNERS is missing; run `cargo xtask codeowners`")?;
    repo.xtask(&["codeowners"]).passes()?;
    let written = fs::read_to_string(repo.root().join(".github/CODEOWNERS"))?;
    ensure!(written.ends_with("\n\n/conventions @owner\n"), "{written}");
    repo.xtask(&["check", "codeowners"]).passes()?;
    repo.write(
        "conventions/protected-paths.txt",
        "owner @owner\n/conventions  # convention files\n/media  # human-only\n",
    )?;
    repo.xtask(&["check", "codeowners"])
        .fails_with("run `cargo xtask codeowners`")?;
    repo.xtask(&["codeowners"]).passes()?;
    repo.xtask(&["check", "codeowners"]).passes()
}
