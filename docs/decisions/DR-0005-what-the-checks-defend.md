<!--
SPDX-FileCopyrightText: Kreqo contributors
SPDX-License-Identifier: CC-BY-4.0
-->

# DR-0005: What the checks defend against, and the secret scan

Status: accepted   Date: 2026-11-06   Scope: org

## What you are approving

The checks' threat model, the secret scan's design and the known gaps below, which the ported `xtask` and workflows implement. Agents working on `xtask` read this before changing a check.

## Context

Origin: an earlier internal prototype, where four independent reviews shaped these checks; it is now closed. Evidence: in use there for two weeks, with tests, and confirmed by a spike in which the reviewer caught every planted weakening of these checks.

## Decision

**What the checks defend against.**
- The checks catch honest mistakes and shortcuts in code, configuration and pull request text, an AI agent's attempts to get CI green included: a lowered lint level, a cargo setting, a pasted token.
- They don't try to stop a change that deliberately rewrites `xtask`, the workflows or the tools' configuration. CI runs the pull request's own copies of all three, so no check can stop that. The reviewer catches it, and those paths are gated (DR-0001 row 5).
- A review finding that only such a change could exploit is recorded rather than fixed.
- Every fix to a check comes with a test that fails without it (QA-09). Tests that need Betterleaks or zizmor run in `cargo xtask check tool-tests`, which runs in CI's Linux checks job. Tests that need only git run on all three platforms.

**Attributes and lint levels (QA-01, SEC-01).**
- Every Rust file that git tracks or would track is read as tokens, so comments and strings never match and wrapped attributes stay whole.
- An `allow` attribute, inner or outer, fails. Clippy's `allow_attributes` skips inner attributes and any `cfg_attr` whose condition doesn't hold, which is why this check exists.
- Any `warn` or `expect` of `warnings`, of a denied lint, or of a group that holds one fails, outside SEC-01's crate-root `#![expect(unsafe_code, reason = "DR-NNNN …")]`, which must name an existing decision record. The reviewer checks that the record covers that code.
- The attributes that `doc(test(attr(…)))` gives doctests are read too, and a shebang is read as rustc reads it.
- Code that a derive crate generates can't use `expect`. The first derive crate that needs an `allow` brings a proposal for QA-01.
- The repository's `.cargo/config.toml` may hold only the `xtask` alias, exactly as `run --package xtask --locked --`.

**The secret scan has no escape hatches (SEC-03, CI-01).**
- Betterleaks runs with its built-in rules only, whatever config the scanned tree holds, and ignores inline `betterleaks:allow` and `gitleaks:allow` comments. A scanner config or ignore file anywhere in the tree fails the scan. Nobody commits with `--no-verify`.
- It covers every path, message and ref:
  - the path prefilter is off, and rename detection is off;
  - binary files, files marked `-diff`, merge commits and archives are scanned;
  - commit messages, notes and tag messages are scanned, each named after its object so a finding keeps its fingerprint.
- The hooks: the pre-commit hook reads staged binary files whole, with their paths. The commit-msg hook scans each message above git's cut line, and the tree too, since a merge skips the pre-commit hook.
- A pull request's title and description become its squash commit's message, so the Description workflow scans them on every change. Its `description` job is required.
- A daily scan covers every branch, tag and note.
- The scans need git 2.31 or later.

**Left to the reviewer.**
- A crate root that `#[path]` or a link mounts into another crate still counts as crate-level. `#[path]` modules without `.rs` and attribute names that `macro_rules!` builds go unchecked.
- A workflow step can set `CARGO_ALIAS_*` or other settings for the steps after it.
- A textconv driver can change what the pre-commit hook reads. CI's history scan reads past it.
- A `# zizmor: ignore[…]` comment, or a `disable` or `ignore` entry in `.github/zizmor.yml`, lowers an audit.
- A suppression's reason passes with an invisible letter or a doubled `until`.
- A `git commit -m` message carrying git's exact cut line stops the commit-msg hook there. CI's scan of the pushed commit reads it.

**Known gaps, to fix as their triggers arrive.**
- GitHub switches off a public repository's scheduled workflows after 60 days without activity, which would end the daily scan unnoticed.
- `reuse lint`'s licenses aren't checked against the allowed list (LEGAL-08), and reuse's own dependencies aren't pinned by hash.
- QA-01's generated copies of the lint table go unchecked until the first nested workspace.
- CI-02's semver check waits for the first library release.
- Project templates need a way past the member check.
- Betterleaks unpacks archives by name only, so it doesn't read inside `.docx` or `.jar` files.
- A ref deleted before the daily scan escapes it.
- A tagged tree's files aren't scanned with their paths, and nested tags' messages aren't scanned.
- A lint group allowed above a denied lint it holds lowers that lint.
- Cargo settings that turn failing tests or lints green are not refused: `overflow-checks` or `debug-assertions` off, or a member's `rust-version` below the workspace's.
- The Description run's log repeats the title and description.
- A byte-order mark before a shebang is refused.
- Cloud sessions need the pinned tools and the git hooks set up.

## Rollout / undo plan

Already in force through `xtask/src/attributes.rs`, `secrets.rs`, `hooks.rs`, `check.rs` and the workflows. Closing a known gap needs no record. Weakening anything above is a gated change.
