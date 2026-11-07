<!--
SPDX-FileCopyrightText: Kreqo contributors
SPDX-License-Identifier: CC-BY-4.0
-->

# DR-0001: Workflow foundation

Status: accepted   Date: 2026-11-06   Scope: org

## What you are approving

Kreqo adopts the AI-first workflow in `docs/handbook/` (still to be written): agents implement your approved intents, and a pull request merges once the checks and a separate reviewer pass. You approve only rule changes and crates.io publishes.
This record fixes the workflow's starting choices. Each one can be changed later through its own decision record.

## Context

The design was settled after an earlier internal prototype and two structured deliberations, all now closed. The prototype showed that hand-reviewing every change made the founder the bottleneck.

Evidence: judgment, not yet tested. Three short trials (gating, one intent end to end, and the reviewer's catch rate), the red-team tests and the first calibration report will test it before v0.1.0.

## Decisions

| # | Decision | Chosen | Reversibility |
| --- | --- | --- | --- |
| 1 | Repositories | One monorepo, `Kreqo/kreqo`, until a crate needs its own repository. No canary repository, rollout waves, admin App or rollout App until then. With a single repository, staged rollout and propagation checks are trivially met; they apply once a second repository exists. | Easy |
| 2 | Where agents run | Implementer, fixer and reviewer agents run in GitHub Actions. Interactive Claude Code sessions are also allowed, under the same gates. | Easy |
| 3 | Authorship | Dara Kong is the git author of every commit, and AI use is disclosed with a `Co-Authored-By` trailer. The agent's GitHub App opens every PR, including PRs from interactive sessions, so the owner can approve gated ones. The gating trial checks who GitHub records as the squash commit's author. If it is the App, a follow-up record picks the fix. | Easy |
| 4 | Merging | Squash merge only, with linear history. A PR auto-merges once every required check and the reviewer pass. A `hold` label on a PR pauses its auto-merge until the label is removed, so the owner can look at any PR first; nothing ever waits on that. | Easy |
| 5 | Human gates | The owner approves only changes to checks, conventions or licence policy, and every crates.io publish, each through a decision record or release summary. | Easy |
| 6 | Deliberation juries | A jury may add a recommendation to a class-3 decision PR that no test can settle. It never stands in for the owner's approval of a gated change. | Easy |
| 7 | How the owner is asked | An exception-first daily digest, near-silent on quiet days. Class-2 defaults go to the Monday rollup. At most 3 class-3 decisions are open at once, each with a recommendation. | Easy |
| 8 | Workflow tooling | The checks the spec assigns to `orgctl` (`check`, `licenses`, `guard`, `verdict`) are `cargo xtask` commands. A separate binary is extracted only if a standalone repository needs one. | Easy |
| 9 | Platform defaults | Conventions live in the monorepo. GitHub Free, with rulesets applied per repository as code. Public repositories. Permissive dependency licences for `MIT OR Apache-2.0` crates, with any other licence needing a decision record. Coverage of at least 80% with a ratchet, zero missed mutants in the diff, fuzz targets for formats and parsers, and clippy `pedantic` as errors, for published crates. MSRV is latest stable minus two. | Easy |
| 10 | Identities | Organization `Kreqo`, monorepo `kreqo`, owner `@darakreqo`, commit identity `Dara Kong <dara@kreqo.org>`. The App names and the digest's time zone are set when they are created. | Easy |
| 11 | v0.1.0 | Five real intents for the learning app's headless core merge through the loop with no code review. Red-team tests R0–R6, R11 and R12 pass. Two weeks of digests take under 20 minutes a day. One calibration report is delivered. Time box: three weeks from this record's acceptance (2026-11-27 if accepted today). Anything unfinished by then is deferred. | Easy |
| 12 | Until the repository is on GitHub | The owner approves records in chat. Each build step lands as one local commit. | Easy |

## Recommendation and why

Approve all twelve as a set. They keep the human out of the merge path and the gates narrow, as the deliberations required, and they cut the design to one repository so that real product work starts within weeks rather than after a second foundation.

## Rollout / undo plan

Each decision is a setting, a file or a workflow. Reversing one is a new decision record that supersedes the matching row here. Auto-merge can be switched off for the repository at any time, which falls back to manual squash merges.
