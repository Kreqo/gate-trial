// SPDX-FileCopyrightText: Kreqo contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `cargo xtask guard`: classifies a pull request's change as protected or
//! not, and fails a protected one whose body links no decision record
//! (CI-15). A change is protected when it touches a path on
//! `conventions/protected-paths.txt` or changes a guarded field of a
//! manifest: a gated change, which only the owner approves. The forge's
//! code-owner review is the authority; this check supplements it, and the
//! workflow labels the pull request from its output. Whether the owner
//! approved the latest commit is the forge's and the workflow's to check.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::check::printable;
use crate::codeowners::{self, Protected};

/// The folder of the decision records, from the root.
const DECISIONS: &str = "docs/decisions/";

/// A guarded field: its keys from the top of the file, and how the output
/// names it.
type Field = (&'static [&'static str], &'static str);

/// The guarded fields of every Cargo.toml: a crate's license, by its
/// expression or its file (LEGAL-01), whether it's published, and its lints
/// (QA-01).
const MANIFEST_FIELDS: [Field; 4] = [
    (&["package", "license"], "[package] license"),
    (&["package", "license-file"], "[package] license-file"),
    (&["package", "publish"], "[package] publish"),
    (&["lints"], "[lints]"),
];

/// The further guarded fields of the root Cargo.toml: the lints, the
/// license and the publishing that crates inherit.
const ROOT_MANIFEST_FIELDS: [Field; 4] = [
    (&["workspace", "lints"], "[workspace.lints]"),
    (
        &["workspace", "package", "license"],
        "[workspace.package] license",
    ),
    (
        &["workspace", "package", "license-file"],
        "[workspace.package] license-file",
    ),
    (
        &["workspace", "package", "publish"],
        "[workspace.package] publish",
    ),
];

/// The guarded fields of every project.toml (DR-0003): what rules apply,
/// by kind, and the license group (LEGAL-01).
const PROJECT_FIELDS: [Field; 2] = [(&["kind"], "kind"), (&["license"], "license")];

/// How a path changed between the merge base and HEAD.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Status {
    Added,
    Deleted,
    Modified,
}

/// A changed path, from the root.
#[derive(Debug, PartialEq, Eq)]
struct Change {
    status: Status,
    path: String,
}

/// Classifies the change between the merge base of `base` and HEAD, with
/// the pull request's body from `body_file`, if given; with
/// GITHUB_OUTPUT set, also appends `protected` and `decision` to that file
/// for the workflow. Returns whether it passed: a protected change passes
/// only with a line `Decision: DR-NNNN` in the body that names a record at
/// HEAD.
pub fn run(root: &Path, base: &str, body_file: Option<&Path>) -> Result<bool> {
    let body = match body_file {
        Some(path) => {
            fs::read_to_string(path).with_context(|| format!("couldn't read {}", path.display()))?
        }
        None => String::new(),
    };
    let base = commit(root, base)?;
    let merge_base = merge_base(root, &base)?;
    let changes = parse_changes(&git(
        root,
        &[
            "diff-tree",
            "-r",
            "-z",
            "--no-renames",
            "--name-status",
            &merge_base,
            "HEAD",
        ],
    )?)?;
    let lists = protected_lists(root, &merge_base, &changes)?;
    let mut reasons = Vec::new();
    for change in &changes {
        let path = printable(&change.path);
        if let Some(pattern) = lists.iter().find_map(|list| list.covering(&change.path)) {
            reasons.push(format!(
                "{path}: under {} ({})",
                pattern.path,
                printable(&pattern.reason)
            ));
        }
        for field in changed_fields(root, &merge_base, change)? {
            reasons.push(format!("{path}: {field} changed"));
        }
    }
    let records = blobs(&git(root, &["ls-tree", "-z", "HEAD", "--", DECISIONS])?);
    let decision = linked_record(&body, &records);
    let protected = !reasons.is_empty();
    if let Some(file) = std::env::var_os("GITHUB_OUTPUT") {
        let id = decision.as_ref().map_or("", |(id, _)| id.as_str());
        append(Path::new(&file), &outputs(protected, id))?;
    }
    println!(
        "{} path(s) changed between the merge base {merge_base} and HEAD.",
        changes.len()
    );
    if !protected {
        println!("Not protected: no protected path or guarded field changed (CI-15).");
        return Ok(true);
    }
    println!("Protected: a gated change, which only the owner approves (CI-15):");
    for reason in &reasons {
        println!("  - {reason}");
    }
    match decision {
        Ok((id, path)) => {
            println!("Decision: {id}, in {path}.");
            Ok(true)
        }
        Err(why) => {
            eprintln!(
                "A protected change needs a line `Decision: DR-NNNN` in the pull request's body, naming a record in {DECISIONS} at HEAD, but {why}. The paths and fields above need it (CI-15)."
            );
            Ok(false)
        }
    }
}

/// Runs git in the checkout and returns what it printed.
fn git(root: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .context("couldn't run git")?;
    if !output.status.success() {
        bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    String::from_utf8(output.stdout).context("git printed non-UTF-8 text")
}

/// The commit that a revision names in this checkout.
fn commit(root: &Path, revision: &str) -> Result<String> {
    let found = git(
        root,
        &[
            "rev-parse",
            "--verify",
            "--quiet",
            "--end-of-options",
            &format!("{revision}^{{commit}}"),
        ],
    )
    .with_context(|| format!("the base {revision} isn't a commit in this checkout; fetch it"))?;
    Ok(found.trim().to_owned())
}

/// The merge base of a commit and HEAD, from which the pull request's
/// change is read, so that what the base branch gained since doesn't count.
fn merge_base(root: &Path, base: &str) -> Result<String> {
    let found = git(root, &["merge-base", base, "HEAD"]).with_context(|| {
        format!("found no merge base of {base} and HEAD; fetch enough history to reach it")
    })?;
    Ok(found.trim().to_owned())
}

/// Reads `git diff-tree --name-status -z` without rename detection, so that
/// a renamed path shows as its old path deleted and its new one added, and
/// both count.
fn parse_changes(listed: &str) -> Result<Vec<Change>> {
    let mut fields = listed.split('\0').filter(|field| !field.is_empty());
    let mut changes = Vec::new();
    while let Some(status) = fields.next() {
        let Some(path) = fields.next() else {
            bail!("git diff-tree printed the status {status} without a path");
        };
        let status = match status {
            "A" => Status::Added,
            "D" => Status::Deleted,
            "M" | "T" => Status::Modified,
            other => bail!("git diff-tree printed the unexpected status {other} for {path}"),
        };
        changes.push(Change {
            status,
            path: path.to_owned(),
        });
    }
    Ok(changes)
}

/// The protected-path lists to classify the change by: HEAD's, and the
/// merge base's unless the change adds the list, so that removing a path
/// from the list doesn't unprotect it in the same change.
fn protected_lists(root: &Path, merge_base: &str, changes: &[Change]) -> Result<Vec<Protected>> {
    let status = changes
        .iter()
        .find(|change| change.path == codeowners::LIST)
        .map(|change| change.status);
    let mut revisions = Vec::new();
    if status != Some(Status::Deleted) {
        revisions.push("HEAD");
    }
    if status != Some(Status::Added) {
        revisions.push(merge_base);
    }
    revisions
        .into_iter()
        .map(|revision| {
            let text = blob(root, revision, codeowners::LIST)?;
            Protected::parse(&text)
                .with_context(|| format!("{} at {revision} is invalid", codeowners::LIST))
        })
        .collect()
}

/// A file's text at a revision.
fn blob(root: &Path, revision: &str, path: &str) -> Result<String> {
    git(root, &["cat-file", "blob", &format!("{revision}:{path}")])
}

/// The guarded fields of a file, by its path from the root.
fn guarded_fields(path: &str) -> Vec<Field> {
    let name = path.rsplit('/').next().unwrap_or(path);
    match name {
        "Cargo.toml" if path == "Cargo.toml" => MANIFEST_FIELDS
            .into_iter()
            .chain(ROOT_MANIFEST_FIELDS)
            .collect(),
        "Cargo.toml" => MANIFEST_FIELDS.to_vec(),
        "project.toml" => PROJECT_FIELDS.to_vec(),
        _ => Vec::new(),
    }
}

/// The names of the guarded fields that a change to a manifest changed.
/// A manifest that can't be parsed fails the check rather than passing
/// unread.
fn changed_fields(root: &Path, merge_base: &str, change: &Change) -> Result<Vec<&'static str>> {
    let fields = guarded_fields(&change.path);
    if fields.is_empty() {
        return Ok(Vec::new());
    }
    let read = |revision: &str| -> Result<toml::Table> {
        blob(root, revision, &change.path)?
            .parse()
            .with_context(|| format!("couldn't parse {} at {revision}", change.path))
    };
    let old = match change.status {
        Status::Added => None,
        Status::Deleted | Status::Modified => Some(read(merge_base)?),
    };
    let new = match change.status {
        Status::Deleted => None,
        Status::Added | Status::Modified => Some(read("HEAD")?),
    };
    Ok(differing(&fields, old.as_ref(), new.as_ref()))
}

/// The names of the fields whose values differ between two versions of a
/// manifest, either of which may be absent. Only values count, so a change
/// in formatting or order changes no field.
fn differing(
    fields: &[Field],
    old: Option<&toml::Table>,
    new: Option<&toml::Table>,
) -> Vec<&'static str> {
    fields
        .iter()
        .filter(|(keys, _)| lookup(old, keys) != lookup(new, keys))
        .map(|(_, name)| *name)
        .collect()
}

/// The value at these keys, if the table holds one.
fn lookup<'a>(table: Option<&'a toml::Table>, keys: &[&str]) -> Option<&'a toml::Value> {
    let (first, rest) = keys.split_first()?;
    rest.iter()
        .try_fold(table?.get(*first)?, |value, key| value.get(*key))
}

/// The paths of the files in a `git ls-tree -z` listing.
fn blobs(listing: &str) -> Vec<String> {
    listing
        .split('\0')
        .filter_map(|entry| {
            // Each entry is `<mode> <type> <id>`, a tab and its path.
            let (object, path) = entry.split_once('\t')?;
            (object.split(' ').nth(1) == Some("blob")).then(|| path.to_owned())
        })
        .collect()
}

/// The record that the body links: the first line `Decision: DR-NNNN`
/// that names one of the records, as its ID and path; or why none does.
fn linked_record<'a>(body: &str, records: &'a [String]) -> Result<(String, &'a str), String> {
    let named: Vec<&str> = body
        .lines()
        .filter_map(|line| line.trim().strip_prefix("Decision:"))
        .map(str::trim)
        .collect();
    for id in &named {
        if is_record_id(id)
            && let Some(path) = record_path(records, id)
        {
            return Ok(((*id).to_owned(), path));
        }
    }
    if named.is_empty() {
        return Err("the body has no such line".to_owned());
    }
    let named: Vec<String> = named
        .iter()
        .map(|id| format!("`{}`", printable(id)))
        .collect();
    Err(format!(
        "its `Decision:` lines name no record there: {}",
        named.join(", ")
    ))
}

/// Whether a value is a record's ID, `DR-` and four digits.
fn is_record_id(value: &str) -> bool {
    value
        .strip_prefix("DR-")
        .is_some_and(|number| number.len() == 4 && number.bytes().all(|byte| byte.is_ascii_digit()))
}

/// The path of the record with this ID, `docs/decisions/DR-NNNN-*.md`.
fn record_path<'a>(records: &'a [String], id: &str) -> Option<&'a str> {
    let start = format!("{DECISIONS}{id}-");
    records
        .iter()
        .map(String::as_str)
        .find(|path| path.starts_with(&start) && path.ends_with(".md"))
}

/// The workflow's outputs: whether the change is protected, and the record
/// it links, if any.
fn outputs(protected: bool, decision: &str) -> String {
    format!("protected={protected}\ndecision={decision}\n")
}

/// Appends text to a file, creating it if needed.
fn append(path: &Path, text: &str) -> Result<()> {
    OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)
        .and_then(|mut file| file.write_all(text.as_bytes()))
        .with_context(|| format!("couldn't write to {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_each_changed_path_with_its_status() -> Result<()> {
        let changes = parse_changes("A\0new\0D\0gone\0M\0a b\0T\0link\0")?;
        let expected = [
            (Status::Added, "new"),
            (Status::Deleted, "gone"),
            (Status::Modified, "a b"),
            (Status::Modified, "link"),
        ]
        .map(|(status, path)| Change {
            status,
            path: path.to_owned(),
        });
        assert_eq!(changes, expected);
        assert_eq!(parse_changes("")?, []);
        Ok(())
    }

    /// A status that rename or copy detection would print, or a status
    /// without its path, fails rather than being misread.
    #[test]
    fn refuses_what_it_cant_read_as_a_change() {
        for (listed, expected) in [
            (
                "R100\0old\0new\0",
                "git diff-tree printed the unexpected status R100 for old",
            ),
            ("M\0", "git diff-tree printed the status M without a path"),
        ] {
            let error = parse_changes(listed).map_err(|error| format!("{error:#}"));
            assert_eq!(error, Err(expected.to_owned()));
        }
    }

    /// The names of a path's guarded fields.
    fn names(path: &str) -> Vec<&'static str> {
        guarded_fields(path).iter().map(|(_, name)| *name).collect()
    }

    /// Every Cargo.toml guards its crate's license, publishing and lints;
    /// the root's also guards the lints and license its crates inherit; a
    /// project.toml guards its kind and license; nothing else is guarded.
    #[test]
    fn guards_the_fields_of_each_manifest() {
        assert_eq!(
            names("Cargo.toml"),
            [
                "[package] license",
                "[package] license-file",
                "[package] publish",
                "[lints]",
                "[workspace.lints]",
                "[workspace.package] license",
                "[workspace.package] license-file",
                "[workspace.package] publish",
            ]
        );
        for path in ["xtask/Cargo.toml", "crates/a/b/Cargo.toml"] {
            assert_eq!(
                names(path),
                [
                    "[package] license",
                    "[package] license-file",
                    "[package] publish",
                    "[lints]"
                ]
            );
        }
        for path in ["project.toml", "tools/a/project.toml"] {
            assert_eq!(names(path), ["kind", "license"]);
        }
        for path in ["Cargo.lock", "a/Cargo.toml.orig", "a/xCargo.toml"] {
            assert!(names(path).is_empty(), "{path}");
        }
    }

    /// The guarded fields that differ between two manifests, either of
    /// which may be absent.
    fn changed(path: &str, old: Option<&str>, new: Option<&str>) -> Result<Vec<&'static str>> {
        let parse = |text: Option<&str>| text.map(str::parse::<toml::Table>).transpose();
        Ok(differing(
            &guarded_fields(path),
            parse(old)?.as_ref(),
            parse(new)?.as_ref(),
        ))
    }

    const MANIFEST: &str = "[package]\nname = \"a\"\nlicense = \"MIT OR Apache-2.0\"\npublish = false\n\n[lints]\nworkspace = true\n";

    #[test]
    fn finds_the_guarded_fields_a_change_changes() -> Result<()> {
        let path = "crates/a/Cargo.toml";
        let same = Some(MANIFEST);
        for (new, expected) in [
            (
                MANIFEST.replace("MIT OR Apache-2.0", "MIT"),
                "[package] license",
            ),
            (
                MANIFEST.replace("publish = false\n", ""),
                "[package] publish",
            ),
            (
                MANIFEST.replace("workspace = true", "workspace = false"),
                "[lints]",
            ),
            (
                MANIFEST.replace(
                    "license = \"MIT OR Apache-2.0\"",
                    "license.workspace = true",
                ),
                "[package] license",
            ),
            (
                MANIFEST.replace(
                    "publish = false",
                    "publish = false\nlicense-file = \"LICENSE\"",
                ),
                "[package] license-file",
            ),
        ] {
            assert_eq!(changed(path, same, Some(&new))?, [expected], "{new}");
        }
        // Other fields, formatting and order change nothing guarded.
        let reworded = "# A comment.\n[lints]\nworkspace = true\n\n[package]\npublish = false\nlicense = 'MIT OR Apache-2.0'\nname = \"b\"\nversion = \"1.0.0\"\n";
        assert_eq!(changed(path, same, Some(reworded))?, Vec::<&str>::new());
        // A manifest added or removed changes each guarded field it holds.
        let every = ["[package] license", "[package] publish", "[lints]"];
        assert_eq!(changed(path, None, same)?, every);
        assert_eq!(changed(path, same, None)?, every);
        assert_eq!(
            changed(path, None, Some("[package]\nname = \"a\"\n"))?,
            Vec::<&str>::new()
        );
        Ok(())
    }

    #[test]
    fn finds_the_root_manifests_guarded_fields() -> Result<()> {
        let root = "[workspace]\nmembers = [\"a\"]\n\n[workspace.package]\nlicense = \"MIT\"\npublish = false\nedition = \"2024\"\n\n[workspace.lints.rust]\nunsafe_code = \"deny\"\n";
        for (new, expected) in [
            (root.replace("\"deny\"", "\"warn\""), "[workspace.lints]"),
            (
                root.replace("\"MIT\"", "\"GPL-3.0-or-later\""),
                "[workspace.package] license",
            ),
            (
                root.replace(
                    "publish = false\n",
                    "license-file = \"LICENSE\"\npublish = false\n",
                ),
                "[workspace.package] license-file",
            ),
            (
                root.replace("publish = false", "publish = true"),
                "[workspace.package] publish",
            ),
        ] {
            assert_eq!(changed("Cargo.toml", Some(root), Some(&new))?, [expected]);
        }
        let other = root
            .replace("2024", "2021")
            .replace("[\"a\"]", "[\"a\", \"b\"]");
        assert_eq!(
            changed("Cargo.toml", Some(root), Some(&other))?,
            Vec::<&str>::new()
        );
        Ok(())
    }

    #[test]
    fn finds_a_projects_guarded_fields() -> Result<()> {
        let project = "kind = \"tool\"\nstage = \"incubating\"\nlicense = \"MIT OR Apache-2.0\"\n";
        let path = "tools/a/project.toml";
        for (new, expected) in [
            (project.replace("tool", "library"), "kind"),
            (
                project.replace("MIT OR Apache-2.0", "GPL-3.0-or-later"),
                "license",
            ),
        ] {
            assert_eq!(changed(path, Some(project), Some(&new))?, [expected]);
        }
        let stage = project.replace("incubating", "stable");
        assert_eq!(
            changed(path, Some(project), Some(&stage))?,
            Vec::<&str>::new()
        );
        Ok(())
    }

    #[test]
    fn looks_up_nested_keys() -> Result<()> {
        let table: toml::Table = "a = 1\n[b]\nc = 2\nd = \"text\"\n".parse()?;
        let found = |keys: &[&str]| lookup(Some(&table), keys).map(ToString::to_string);
        assert_eq!(found(&["a"]).as_deref(), Some("1"));
        assert_eq!(found(&["b", "c"]).as_deref(), Some("2"));
        assert_eq!(found(&["b", "e"]), None);
        assert_eq!(found(&["e", "c"]), None);
        // A key below a value that isn't a table.
        assert_eq!(found(&["a", "c"]), None);
        assert_eq!(found(&["b", "d", "c"]), None);
        assert_eq!(found(&[]), None);
        assert_eq!(lookup(None, &["a"]), None);
        Ok(())
    }

    #[test]
    fn lists_only_the_files_of_a_tree() {
        let listing = "100644 blob 1a\tdocs/decisions/DR-0001-a.md\x00040000 tree 2b\tdocs/decisions/DR-0002-b.md\x00100755 blob 3c\tdocs/decisions/x y.md\x00";
        assert_eq!(
            blobs(listing),
            ["docs/decisions/DR-0001-a.md", "docs/decisions/x y.md"]
        );
        assert!(blobs("").is_empty());
    }

    /// The records of a checkout, as `blobs` lists them.
    fn records() -> Vec<String> {
        [
            "docs/decisions/DR-0001-start.md",
            "docs/decisions/DR-0002-next.txt",
            "docs/decisions/DR-003-short.md",
            "docs/decisions/DR-00004-long.md",
            "docs/decisions/DR-0005.md",
            "docs/decisions/TEMPLATE.md",
        ]
        .map(String::from)
        .to_vec()
    }

    #[test]
    fn finds_the_record_the_body_links() {
        let records = records();
        let linked = |body: &str| linked_record(body, &records);
        let found = Ok(("DR-0001".to_owned(), "docs/decisions/DR-0001-start.md"));
        assert_eq!(linked("Intent: x\nDecision: DR-0001\n"), found);
        // Indented, with CRLF line endings, and after a line that names
        // none.
        assert_eq!(
            linked("Decision: DR-NNNN\r\n  Decision:   DR-0001  \r\n"),
            found
        );
        assert_eq!(
            linked("Summary.\nDecision: DR-0001 and DR-0002\n"),
            Err("its `Decision:` lines name no record there: `DR-0001 and DR-0002`".to_owned())
        );
    }

    /// A record's file is `docs/decisions/DR-NNNN-*.md`, and only a line
    /// that names one by its four-digit ID links it.
    #[test]
    fn links_only_a_record_named_by_its_id() {
        let records = records();
        let refused = |id: &str| linked_record(&format!("Decision: {id}\n"), &records);
        for id in [
            "DR-0002",
            "DR-003",
            "DR-00004",
            "DR-0005",
            "DR-0009",
            "TEMPLATE",
            "DR-0001-start",
        ] {
            assert_eq!(
                refused(id),
                Err(format!(
                    "its `Decision:` lines name no record there: `{id}`"
                )),
                "{id}"
            );
        }
        assert_eq!(
            linked_record("Decided: DR-0001\nNo decision: DR-0001.\n", &records),
            Err("the body has no such line".to_owned())
        );
        assert_eq!(
            linked_record("", &[]),
            Err("the body has no such line".to_owned())
        );
    }

    #[test]
    fn recognizes_record_ids() {
        assert!(is_record_id("DR-0001"));
        assert!(is_record_id("DR-9999"));
        for value in [
            "DR-001",
            "DR-00001",
            "DR-00a1",
            "DR-NNNN",
            "dr-0001",
            "0001",
            "DR-٠٠٠١",
        ] {
            assert!(!is_record_id(value), "{value}");
        }
    }

    /// A line of the pull request's body reaches the log on one line, so
    /// that it can't pass for a workflow command.
    #[test]
    fn shows_what_the_body_names_on_one_line() {
        assert_eq!(
            linked_record("Decision: a\r::error::x\n", &[]),
            Err("its `Decision:` lines name no record there: `a\\r::error::x`".to_owned())
        );
        assert_eq!(
            linked_record("Decision: a\u{1b}b\n", &[]),
            Err("its `Decision:` lines name no record there: `a\\u{1b}b`".to_owned())
        );
    }

    #[test]
    fn writes_the_workflows_outputs() {
        assert_eq!(
            outputs(true, "DR-0007"),
            "protected=true\ndecision=DR-0007\n"
        );
        assert_eq!(outputs(false, ""), "protected=false\ndecision=\n");
    }
}
