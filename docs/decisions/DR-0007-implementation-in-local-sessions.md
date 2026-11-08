<!--
SPDX-FileCopyrightText: Kreqo contributors
SPDX-License-Identifier: CC-BY-4.0
-->

# DR-0007: Implementation runs in local sessions

Status: accepted   Date: 2026-11-08   Scope: org

## What you are approving

Agents implement changes in contributors' own interactive Claude Code sessions, on their own machines and usage limits, not in CI. A change still reaches the default branch only as a pull request that passes the organization's checks, the AI review and gate G, and then auto-merges.

## Context

Origin: the founder's choice on 2026-11-08, after weighing CI agents against the subscription the founder already pays for.

Evidence: judgment, not yet tested. The verdict behind DR-0001 calls for agents that implement each approved intent in a pull request. P02 §4 treats running them in Actions as only one way to execute that, so a local session meets the verdict too.

## Decisions

| # | Decision | Chosen |
| --- | --- | --- |
| 1 | Where agents implement | In the contributor's local interactive session. The founder's runs on the founder's subscription. No implementer or fixer agent runs in CI. Supersedes DR-0001 row 2, whose reviewer part stays: the AI review runs in CI and authenticates by workload identity federation, with no stored Claude secret. |
| 2 | Intents | An intent is stated in the session and written into the pull request body, with its acceptance criteria. That body is the record of the human's creative input. There are no intent issues and no `go` or `proposed` labels. |
| 3 | Who opens the pull request | The agent's GitHub App opens every pull request, as DR-0001 row 3 says, from the contributor's machine, so the founder can approve gated ones. The gating trial in step 6 confirms this works. If it doesn't, a later record picks another route. |
| 4 | Fixes | When the AI review asks for changes, the author fixes the pull request in a session. A pull request that stays blocked is listed in the digest. |
| 5 | Pace | Work moves only while a contributor is in a session. That is accepted: nothing waits on the founder's approval of each change, which is the bottleneck the verdict warned against. |
| 6 | What stays | Auto-merge with the `hold` label (DR-0001 row 4), gate G (DR-0006), the digest (CI-generated and read-only), and the AI review. |

## Consequences

- **Smaller scope.** P02's Phase 2 shrinks to the digest. Step 7 of the plan becomes the first change made end to end from a local session.
- **Less to defend.** The prompt-injection exposure (R1) narrows to the reviewer, which only reads.
- **Rules to update.** These rules name intents as issues or a `go` label and change to match: the glossary's "Intent" and "Dilemma protocol", CI-12, DOC-12, GOV-02, GOV-07, GOV-14, REV-01 and REV-02.

## Options

1. **Local sessions, as above (recommended):** no CI implementer to build or secure, and agents use the founder's existing subscription. Reversibility: easy, since CI agents can be added later by a superseding record.
2. **CI agents, as P02 specifies:** work moves without the founder present, at the cost of building and securing the implementer and fixer and paying per token. Reversibility: easy.

## Recommendation and why

Option 1. It meets the verdict, drops the riskiest part of the build, and fits the time box.

## Rollout / undo plan

On acceptance, DR-0001's status gains "row 2 superseded by DR-0007". The rule updates land in the same commit, and the step 5 handbook and templates describe local sessions. Undo: a later record reinstates CI agents.
