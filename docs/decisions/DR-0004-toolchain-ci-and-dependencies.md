<!--
SPDX-FileCopyrightText: Kreqo contributors
SPDX-License-Identifier: CC-BY-4.0
-->

# DR-0004: Toolchain, CI and dependency policy

Status: accepted   Date: 2026-11-06   Scope: org

## What you are approving

The toolchain, the pinned CI tools, file licensing and the dependency policy below, which the ported `xtask`, `deny.toml` and workflows enforce. The license allow list is the permissive set DR-0001 chose.

## Context

Origin: an earlier internal prototype, now closed. Evidence: in use there for two weeks, with tests. The license allow list is the exception: it is new, chosen 2026-11-06 to match DR-0001. The old list also allowed MPL-2.0 and Unlicense, which no current dependency needs.

## Decision

**Toolchain.**
- `rust-toolchain.toml` pins Rust 1.99.1 with rustfmt and Clippy.
- The workspace uses edition 2024 and resolver 3.
- The MSRV window is latest stable minus two, so crates declare 1.97 (REL-04).
- `xtask` depends only on maintained, widely used and permissively licensed crates (ARCH-05): clap, anyhow, serde, toml, cargo_metadata and proc-macro2.

**File licensing.**
- Every file that can carry a header reads `SPDX-FileCopyrightText: Kreqo contributors` and names its license. No AI is named (LEGAL-09).
- Code, configuration and workflows are MIT OR Apache-2.0. The conventions, the documentation and CLAUDE.md are CC BY 4.0, as are art, icons and media (LEGAL-08).
- `REUSE.toml` covers generated files: Cargo.lock is CC0-1.0, and the rule views are CC BY 4.0.

**CI.**
- CI uses only actions/checkout, Swatinem/rust-cache and taiki-e/install-action, each pinned by full commit SHA, on ubuntu-24.04, windows-2025 and macos-15.
- The tools are pinned in `xtask/src/tools.rs`: cargo-deny 0.20.2, zizmor 1.31.1, reuse 6.2.0 and Betterleaks 1.9.0. Betterleaks is checked against the SHA-256 in its release's signed checksums file.
- `cargo xtask check` fails on a machine that has other versions.
- zizmor runs its online audits in CI with the job's read-only token, and offline without one.
- `.gitignore` ignores only the root's `/target/`, so a source folder named `target` can't vanish from a commit.

**Dependencies (SEC-02, LEGAL-03, PRIV-09, ARCH-13).**
- An advisory that a crate is vulnerable, unsound or yanked blocks merges and releases, whatever its severity. An unmaintained crate blocks when the workspace depends on it directly.
- Every suppression gives its reason and ends it with `until YYYY-MM-DD`, at most 90 days ahead, so each one comes back for review at least once a quarter.
- A license exception can't carry a reason in cargo-deny 0.20, so a comment above it gives both, and review checks it.
- PRIV-09 bans the crates.io clients for Sentry, PostHog, Segment, RudderStack, Bugsnag, Rollbar, Honeybadger, Mixpanel and Amplitude.
- Crates come from crates.io only, never from git or another registry.
- Dependencies of MIT OR Apache-2.0 crates may use MIT, MIT-0, Apache-2.0, Apache-2.0 WITH LLVM-exception, BSD-2-Clause, BSD-3-Clause, ISC, Zlib, 0BSD, CC0-1.0, BSL-1.0, Unicode-3.0 and Unicode-DFS-2016. Any other license, MPL-2.0 included, needs a decision record for that dependency.

## Rollout / undo plan

Already in force through `deny.toml`, `xtask/src/deny.rs`, `xtask/src/tools.rs` and the workflows. A change to a pinned version or to the policy is a gated change under DR-0001 row 5.
