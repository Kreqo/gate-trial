<!--
SPDX-FileCopyrightText: Kreqo contributors
SPDX-License-Identifier: CC-BY-4.0
-->

# DR-0002: The reviewer runs on Opus, and mutation testing covers xtask

Status: accepted   Date: 2026-11-06   Scope: org

## What you are approving

The reviewer agent that every pull request must pass runs on Claude Opus. Mutation testing, which the workflow reserves for published crates, also covers `xtask`, the tooling that enforces every rule.

## Context

Origin: a short trial on an earlier copy of `xtask`, 6,894 lines with 88 tests, where 10 bugs and 5 clean changes were disguised as agent pull requests. Evidence: tested in a spike.

- The checks caught 3 of the 10 bugs. The reviewer on Opus rejected all 10, including the 7 the checks missed, and rejected no correct code as wrong. The reviewer on Sonnet rejected 9: it approved the removal of a key check on license exceptions.
- All 5 mutation-based bugs were mutants that survived the tests. 60 of the 439 mutants tested survived, so the tests have gaps the checks can't see.
- The sample is small. 10 of 10 is consistent with a true catch rate as low as about 70%, and the planted bugs and the reviewers come from the same model family. The weekly calibration drill keeps measuring.

## Options

1. **Opus reviewer, and mutation testing on `xtask` (recommended).** Each review takes about 30 seconds and about 47,000 tokens. Reversibility: easy.
2. Opus and Sonnet, both required to pass. Twice the cost, and Sonnet caught nothing that Opus missed. Reversibility: easy.
3. Sonnet reviewer, with P02's mutation-testing scope. Cheaper, but it approved a check-weakening change. Reversibility: easy.

## Recommendation and why

Option 1. The reviewer is the only layer that stopped the checks' misses, and Opus was the one that held on every case. Closing the test gaps moves part of that load onto checks that don't depend on any model.

## Rollout / undo plan

- **The reviewer:** the reviewer job names its model in one place. Changing it is a superseding record.
- **The surviving mutants:** the port adds tests until none of today's survivors is left, running cargo-mutants at low priority.
- **New pull requests:** once the workflow runs on the forge, every pull request that touches `xtask/` runs `cargo mutants --in-diff`, and a surviving mutant in the diff fails it. A pure deletion has no new lines to mutate, so it stays the reviewer's job.
