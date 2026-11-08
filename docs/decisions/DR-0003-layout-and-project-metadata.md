<!--
SPDX-FileCopyrightText: Kreqo contributors
SPDX-License-Identifier: CC-BY-4.0
-->

# DR-0003: The monorepo's layout, and each project's metadata

Status: accepted   Date: 2026-11-06   Scope: org

## What you are approving

The monorepo keeps the folder layout and the `project.toml` metadata below, which `cargo xtask check crates` enforces.

## Context

Origin: an earlier internal prototype, now closed. Evidence: in use there for two weeks, with tests. `xtask` carries them over unchanged.

## Decision

- **Layout.** The repository is `kreqo`:
  - `crates/` holds shared libraries;
  - `tools/` holds command-line tools;
  - `apps/` holds the apps, each with its core, its CLI and later its shell;
  - `experiments/` holds experiments;
  - `xtask/`, `conventions/`, `templates/` and `docs/` hold the tooling, the rules, the project templates and the documentation.
- **Project metadata.** Each project's metadata lives in a `project.toml` in its folder. It starts with `kind`, `stage` and `license`, and the other fields (platform tiers, stricter lints, risk-tier paths, project facts, an app's ID) join it with the checks that use them.
- **One project per crate.** Every crate belongs to exactly one project, the nearest `project.toml` above it. A `project.toml` at the repository root is refused, because it would claim every crate. Every crate is a member of the root workspace and inherits its lint table.

## Rollout / undo plan

Already in force through `xtask/src/crates.rs`. A change is a superseding record, together with the matching change to the check and a test for it.
