// SPDX-FileCopyrightText: Kreqo contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `cargo xtask tools`, which the workflows read the pinned tool versions
//! from (CI-07).

use std::process::Command;

use anyhow::{Context, Result, ensure};

/// The versions DR-0004 pins, as `name=value` lines for `$GITHUB_OUTPUT`:
/// install-action takes `name@version`, and the tools installed by hand
/// take the version alone, with Betterleaks' archive checked against its
/// SHA-256.
#[test]
fn prints_the_pinned_versions_for_the_workflows() -> Result<()> {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .arg("tools")
        // xtask's root is the folder above its manifest's.
        .env("CARGO_MANIFEST_DIR", env!("CARGO_MANIFEST_DIR"))
        .output()
        .context("couldn't run xtask tools")?;
    ensure!(
        output.status.success(),
        "xtask tools failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let printed = String::from_utf8(output.stdout).context("xtask printed non-UTF-8 text")?;
    let (versions, digest) = printed
        .rsplit_once("betterleaks-sha256=")
        .context("xtask tools printed no betterleaks-sha256")?;
    assert_eq!(
        versions,
        "cargo-deny=cargo-deny@0.20.2\nzizmor=zizmor@1.31.1\nreuse=6.2.0\nbetterleaks=1.9.0\n"
    );
    let digest = digest.strip_suffix('\n').unwrap_or(digest);
    ensure!(
        digest.len() == 64 && digest.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')),
        "betterleaks-sha256 isn't a SHA-256: {digest:?}"
    );
    Ok(())
}
