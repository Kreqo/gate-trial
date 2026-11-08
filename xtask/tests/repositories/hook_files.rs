// SPDX-FileCopyrightText: Kreqo contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The hook files the installer writes or finds (SEC-03), which need only
//! git. The installer's git reads the environment, which may point it at
//! another repository, as GIT_DIR does under `git rebase -x`; so it runs
//! here only as the binary, with the test's own git settings.

use std::fs;

use anyhow::{Result, ensure};

use crate::support::Repo;

/// Git runs a hook only when it's executable.
#[cfg(unix)]
#[test]
fn installs_hooks_that_git_can_run() -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let repo = Repo::new()?;
    repo.xtask(&["hooks"]).passes()?;
    let hooks = repo.root().join(".git").join("hooks");
    for name in ["pre-commit", "commit-msg"] {
        let mode = fs::metadata(hooks.join(name))?.permissions().mode();
        ensure!(mode & 0o111 == 0o111, "{name} isn't executable: {mode:o}");
    }
    Ok(())
}

/// A hook of the user's own that isn't text, such as a compiled program,
/// stays, and no hook is written beside it.
#[test]
fn leaves_a_hook_that_isnt_text_alone() -> Result<()> {
    let repo = Repo::new()?;
    let hooks = repo.root().join(".git").join("hooks");
    fs::create_dir_all(&hooks)?;
    let program = [0x7f, b'E', b'L', b'F', 0xff, 0xfe, 0x00];
    fs::write(hooks.join("pre-commit"), program)?;
    repo.xtask(&["hooks"]).fails_with("couldn't read")?;
    ensure!(
        fs::read(hooks.join("pre-commit"))? == program,
        "the pre-commit hook was overwritten"
    );
    ensure!(
        !hooks.join("commit-msg").exists(),
        "the commit-msg hook was written after the refusal"
    );
    Ok(())
}
