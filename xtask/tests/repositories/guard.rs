// SPDX-FileCopyrightText: Kreqo contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The guard (CI-15): a pull request that changes a protected path or a
//! guarded field passes only with a `Decision: DR-NNNN` line naming a
//! record at HEAD, and the workflow reads the classification from
//! GITHUB_OUTPUT.

use std::fs;

use anyhow::{Context, Result, ensure};

use crate::support::{Repo, Run};

/// A protected-paths list.
const LIST: &str = "# The list.\nowner @owner\n\n/conventions  # convention files\n/.github  # CI definitions\n/docs/decisions  # the owner's decisions\n/media  # human-only\n.claude  # agent settings\n";

/// A crate's manifest, with its guarded fields.
const MANIFEST: &str = "[package]\nname = \"a\"\nlicense = \"MIT OR Apache-2.0\"\npublish = false\n\n[lints]\nworkspace = true\n";

/// A repository whose `main` holds the list, a record, a crate and some
/// code, checked out on a new branch `pr`; and `main`'s commit.
fn pull_request() -> Result<(Repo, String)> {
    let repo = Repo::new()?;
    repo.write("conventions/protected-paths.txt", LIST)?;
    repo.write("conventions/rules.toml", "a = 1\n")?;
    repo.write("docs/decisions/DR-0001-start.md", "# DR-0001: Start\n")?;
    repo.write("crates/a/Cargo.toml", MANIFEST)?;
    repo.write(
        "crates/a/project.toml",
        "kind = \"library\"\nlicense = \"MIT OR Apache-2.0\"\n",
    )?;
    repo.write("crates/a/src/lib.rs", "")?;
    let base = repo.commit_all("Base")?;
    repo.git(&["switch", "--quiet", "--create", "pr"])?;
    Ok((repo, base))
}

/// Runs the guard against `base`, with this body, and returns the run and
/// what it appended to GITHUB_OUTPUT.
fn guard(repo: &Repo, base: &str, body: &str) -> Result<(Run, String)> {
    let body = repo.write_outside("body", body)?;
    let body = body.to_str().context("the scratch path isn't UTF-8")?;
    let output = repo.outside("output");
    if output.exists() {
        fs::remove_file(&output)?;
    }
    let file = output.to_str().context("the scratch path isn't UTF-8")?;
    let run = repo.xtask_with(
        &["guard", "--base", base, "--body-file", body],
        &[("GITHUB_OUTPUT", file)],
    );
    let written = fs::read_to_string(&output).unwrap_or_default();
    Ok((run, written))
}

#[test]
fn an_unprotected_change_needs_no_decision() -> Result<()> {
    let (repo, base) = pull_request()?;
    repo.write("crates/a/src/lib.rs", "pub fn a() {}\n")?;
    repo.write("README.md", "# Kreqo\n")?;
    repo.commit_all("Change the code")?;
    let (run, written) = guard(&repo, &base, "")?;
    run.passes()?;
    ensure!(written == "protected=false\ndecision=\n", "{written}");
    // A record the body names is reported, protected or not.
    let (run, written) = guard(&repo, &base, "Decision: DR-0001\n")?;
    run.passes()?;
    ensure!(
        written == "protected=false\ndecision=DR-0001\n",
        "{written}"
    );
    Ok(())
}

#[test]
fn a_protected_change_needs_a_decision_record() -> Result<()> {
    let (repo, base) = pull_request()?;
    repo.write("conventions/rules.toml", "a = 2\n")?;
    repo.write(".github/workflows/ci.yml", "name: CI\n")?;
    repo.commit_all("Change a rule")?;
    let (run, written) = guard(&repo, &base, "Summary.\n")?;
    run.fails_with("but the body has no such line")?;
    run.fails_with("  - conventions/rules.toml: under /conventions (convention files)")?;
    run.fails_with("  - .github/workflows/ci.yml: under /.github (CI definitions)")?;
    ensure!(written == "protected=true\ndecision=\n", "{written}");
    let (run, written) = guard(&repo, &base, "Summary.\n\nDecision: DR-0001\n")?;
    run.passes()?;
    ensure!(written == "protected=true\ndecision=DR-0001\n", "{written}");
    Ok(())
}

/// The record must exist at HEAD: on the base branch, or added by the pull
/// request itself, as a decision pull request adds it.
#[test]
fn the_record_must_exist_at_head() -> Result<()> {
    let (repo, base) = pull_request()?;
    repo.write("conventions/rules.toml", "a = 2\n")?;
    repo.commit_all("Change a rule")?;
    let (run, written) = guard(&repo, &base, "Decision: DR-0002\n")?;
    run.fails_with("its `Decision:` lines name no record there: `DR-0002`")?;
    ensure!(written == "protected=true\ndecision=\n", "{written}");
    // A file in the working tree isn't at HEAD.
    repo.write("docs/decisions/DR-0002-rule.md", "# DR-0002: The rule\n")?;
    guard(&repo, &base, "Decision: DR-0002\n")?
        .0
        .fails_with("`DR-0002`")?;
    repo.commit_all("Add the record")?;
    let (run, written) = guard(&repo, &base, "Decision: DR-0002\n")?;
    run.passes()?;
    ensure!(written == "protected=true\ndecision=DR-0002\n", "{written}");
    // A folder named like a record isn't one.
    repo.write("docs/decisions/DR-0003-folder.md/x", "")?;
    repo.commit_all("Add a folder")?;
    guard(&repo, &base, "Decision: DR-0003\n")?
        .0
        .fails_with("`DR-0003`")
}

/// A rename counts both its paths, so moving a file out of a protected
/// folder is protected.
#[test]
fn a_rename_counts_both_its_paths() -> Result<()> {
    let (repo, base) = pull_request()?;
    repo.git(&["mv", "conventions/rules.toml", "rules.toml"])?;
    repo.commit_all("Move the rules out")?;
    guard(&repo, &base, "")?
        .0
        .fails_with("  - conventions/rules.toml: under /conventions")
}

#[test]
fn a_guarded_field_change_is_protected() -> Result<()> {
    let (repo, base) = pull_request()?;
    // Formatting and other fields change nothing guarded.
    repo.write(
        "crates/a/Cargo.toml",
        format!(
            "# The crate.\n{}",
            MANIFEST
                .replace("name = \"a\"\n", "version = \"0.1.0\"\nname = \"a\"\n")
                .replace("publish = false", "publish = false # Not yet.")
                .replace("[lints]", "[package.metadata]\nnote = 1\n\n[lints]")
        ),
    )?;
    repo.commit_all("Reword the manifest")?;
    let (run, written) = guard(&repo, &base, "")?;
    run.passes()?;
    ensure!(written == "protected=false\ndecision=\n", "{written}");
    repo.write(
        "crates/a/Cargo.toml",
        MANIFEST.replace("publish = false\n", ""),
    )?;
    repo.write(
        "crates/a/project.toml",
        "kind = \"app\"\nlicense = \"MIT OR Apache-2.0\"\n",
    )?;
    repo.commit_all("Publish the crate as an app")?;
    let (run, written) = guard(&repo, &base, "")?;
    run.fails_with("  - crates/a/Cargo.toml: [package] publish changed")?;
    run.fails_with("  - crates/a/project.toml: kind changed")?;
    ensure!(written == "protected=true\ndecision=\n", "{written}");
    Ok(())
}

/// The root manifest guards the lints and license its crates inherit, and
/// a new crate's guarded fields are changes too.
#[test]
fn the_root_manifest_and_new_crates_are_guarded() -> Result<()> {
    let (repo, base) = pull_request()?;
    repo.write(
        "Cargo.toml",
        "[workspace]\n\n[workspace.lints.rust]\nunsafe_code = \"allow\"\n",
    )?;
    repo.write(
        "tools/b/Cargo.toml",
        "[package]\nname = \"b\"\nlicense = \"MIT\"\n",
    )?;
    repo.commit_all("Add a tool and allow unsafe code")?;
    let run = guard(&repo, &base, "")?.0;
    run.fails_with("  - Cargo.toml: [workspace.lints] changed")?;
    run.fails_with("  - tools/b/Cargo.toml: [package] license changed")
}

/// A link that git stores under a protected folder's name is protected,
/// whether the pattern is rooted or a name for any folder: a link
/// `.claude` to an unprotected folder would otherwise make every session
/// load that folder's settings and hooks.
#[test]
fn a_link_with_a_protected_name_is_protected() -> Result<()> {
    let (repo, base) = pull_request()?;
    repo.write(
        "tools/agent-cfg/settings.json",
        "{ \"permissions\": { \"allow\": [\"Bash\"] } }\n",
    )?;
    repo.git(&["add", "--all"])?;
    // Links made in the index, so that the test needs no link support in
    // the file system, as on Windows.
    for (name, target) in [(".claude", "tools/agent-cfg"), ("media", "tools")] {
        let file = repo.write_outside("target", target)?;
        let file = file.to_str().context("the scratch path isn't UTF-8")?;
        let blob = repo.git(&["hash-object", "-w", "--no-filters", file])?;
        let entry = format!("120000,{},{name}", blob.trim());
        repo.git(&["update-index", "--add", "--cacheinfo", &entry])?;
    }
    repo.git(&[
        "commit",
        "--quiet",
        "--message",
        "Link the agent's settings",
    ])?;
    let (run, written) = guard(&repo, &base, "")?;
    run.fails_with("  - .claude: under .claude (agent settings)")?;
    run.fails_with("  - media: under /media (human-only)")?;
    ensure!(written == "protected=true\ndecision=\n", "{written}");
    Ok(())
}

/// A name without `/` on the list protects that name in every folder,
/// whatever its ASCII case, as a case-insensitive checkout reads it.
#[test]
fn a_protected_name_is_protected_in_any_folder() -> Result<()> {
    let (repo, base) = pull_request()?;
    repo.write("crates/a/.Claude/settings.json", "{}\n")?;
    repo.commit_all("Add a crate's agent settings")?;
    let (run, written) = guard(&repo, &base, "")?;
    run.fails_with("  - crates/a/.Claude/settings.json: under .claude (agent settings)")?;
    ensure!(written == "protected=true\ndecision=\n", "{written}");
    Ok(())
}

/// A manifest that can't be parsed fails the guard rather than passing
/// unread.
#[test]
fn an_unreadable_manifest_fails() -> Result<()> {
    let (repo, base) = pull_request()?;
    repo.write("crates/a/Cargo.toml", "[package\n")?;
    repo.commit_all("Break the manifest")?;
    guard(&repo, &base, "")?
        .0
        .fails_with("couldn't parse crates/a/Cargo.toml at HEAD")
}

/// The list at the merge base applies too, so a pull request can't
/// unprotect a path by removing it from the list in the same change, nor
/// by deleting the list.
#[test]
fn the_merge_bases_list_still_applies() -> Result<()> {
    let (repo, base) = pull_request()?;
    repo.write(
        "conventions/protected-paths.txt",
        LIST.replace("/.github  # CI definitions\n", ""),
    )?;
    repo.write(".github/workflows/ci.yml", "name: CI\n")?;
    repo.commit_all("Unprotect the workflows")?;
    guard(&repo, &base, "")?
        .0
        .fails_with("  - .github/workflows/ci.yml: under /.github")?;
    repo.git(&["rm", "--quiet", "conventions/protected-paths.txt"])?;
    repo.commit_all("Remove the list")?;
    guard(&repo, &base, "")?
        .0
        .fails_with("  - .github/workflows/ci.yml: under /.github")
}

/// A list that the pull request adds applies to it, with no list at the
/// merge base to read.
#[test]
fn a_list_added_by_the_change_applies() -> Result<()> {
    let repo = Repo::new()?;
    let base = repo.git(&["rev-parse", "HEAD"])?;
    repo.write("conventions/protected-paths.txt", LIST)?;
    repo.write(".github/workflows/ci.yml", "name: CI\n")?;
    repo.commit_all("Protect the workflows")?;
    let run = guard(&repo, base.trim(), "")?.0;
    run.fails_with("  - conventions/protected-paths.txt: under /conventions")?;
    run.fails_with("  - .github/workflows/ci.yml: under /.github")
}

/// Without a list, the guard can't classify anything, so it fails.
#[test]
fn fails_without_a_list() -> Result<()> {
    let repo = Repo::new()?;
    let base = repo.commit_all("Nothing")?;
    repo.write("a.txt", "a\n")?;
    repo.commit_all("Add a file")?;
    guard(&repo, &base, "")?
        .0
        .fails_with("cat-file blob HEAD:conventions/protected-paths.txt failed")
}

/// What the base branch gained after the pull request branched off isn't
/// the pull request's change.
#[test]
fn reads_the_change_from_the_merge_base() -> Result<()> {
    let (repo, base) = pull_request()?;
    repo.write("crates/a/src/lib.rs", "pub fn a() {}\n")?;
    repo.commit_all("Change the code")?;
    repo.git(&["switch", "--quiet", "main"])?;
    repo.write("conventions/rules.toml", "a = 2\n")?;
    let main = repo.commit_all("Change a rule on main")?;
    repo.git(&["switch", "--quiet", "pr"])?;
    let (run, written) = guard(&repo, &main, "")?;
    run.passes()?;
    ensure!(written == "protected=false\ndecision=\n", "{written}");
    // The same holds for a revision name, which guard resolves.
    guard(&repo, "main", "")?.0.passes()?;
    guard(&repo, &base, "")?.0.passes()
}

#[test]
fn fails_on_a_base_it_cant_find() -> Result<()> {
    let (repo, _) = pull_request()?;
    guard(&repo, "no-such-branch", "")?
        .0
        .fails_with("the base no-such-branch isn't a commit in this checkout; fetch it")?;
    // A base with no history in common with HEAD.
    repo.git(&["switch", "--quiet", "--orphan", "other"])?;
    repo.write("a.txt", "a\n")?;
    let other = repo.commit_all("Unrelated")?;
    repo.git(&["switch", "--quiet", "pr"])?;
    guard(&repo, &other, "")?
        .0
        .fails_with("found no merge base")
}

/// The outputs are appended, as GitHub Actions expects, after what earlier
/// steps wrote.
#[test]
fn appends_to_the_workflows_outputs() -> Result<()> {
    let (repo, base) = pull_request()?;
    let output = repo.write_outside("earlier", "earlier=1\n")?;
    let file = output.to_str().context("the scratch path isn't UTF-8")?;
    repo.xtask_with(&["guard", "--base", &base], &[("GITHUB_OUTPUT", file)])
        .passes()?;
    let written = fs::read_to_string(&output)?;
    ensure!(
        written == "earlier=1\nprotected=false\ndecision=\n",
        "{written}"
    );
    Ok(())
}

/// Without a body file, a protected change has no decision line; a body
/// file that can't be read fails.
#[test]
fn reads_the_body_from_its_file() -> Result<()> {
    let (repo, base) = pull_request()?;
    repo.write("conventions/rules.toml", "a = 2\n")?;
    repo.commit_all("Change a rule")?;
    // GITHUB_OUTPUT is set even here, so that a run in CI never writes to
    // the outputs of the step that runs the tests.
    let output = repo.outside("output");
    let output = output.to_str().context("the scratch path isn't UTF-8")?;
    repo.xtask_with(&["guard", "--base", &base], &[("GITHUB_OUTPUT", output)])
        .fails_with("but the body has no such line")?;
    let missing = repo.outside("missing");
    let missing = missing.to_str().context("the scratch path isn't UTF-8")?;
    repo.xtask_with(
        &["guard", "--base", &base, "--body-file", missing],
        &[("GITHUB_OUTPUT", output)],
    )
    .fails_with(&format!("couldn't read {missing}"))
}
