<!--
SPDX-FileCopyrightText: Kreqo contributors
SPDX-License-Identifier: CC-BY-4.0
-->

# DR-0008: Agent sessions can't reach the founder's forge credentials

Status: accepted; row 2 superseded by DR-0009   Date: 2026-11-08   Scope: org

## What you are approving

Agent sessions run where they can't use the founder's forge login, and the agent's GitHub App gets no power over workflows or checks. Together these keep gate G from being defeated from inside a session.

## Context

Origin: the independent review of the gate machinery, on 2026-11-08.

DR-0007 moved agents onto the founder's machine. Its "less to defend" consequence understated the risk. A session that can reach the founder's `gh` login, git credentials, browser or the App's private key could:
- approve its own pull requests as the owner;
- remove `hold`;
- change the ruleset.

Separately, any identity that can push to `.github/workflows/` can run a workflow that posts a passing `gate`.

Evidence: judgment, from the review. Step 6's red-team tests confirm it.

## Decisions

| # | Decision | Chosen |
| --- | --- | --- |
| 1 | Where sessions run | As a separate OS user or in a container. Its only forge credential is a short-lived App installation token, minted outside it. The founder's own login, the App's private key and the founder's browser are unreachable from it. The founder approves from the forge's website or mobile app. |
| 2 | The App's permissions | Contents and pull requests, and issues if a rule needs them. Never `workflows`, `actions`, `checks`, `statuses` or `administration`. The founder pushes workflow changes. |
| 3 | The ruleset | "Require approval of the most recent push" is off, so the founder can approve a workflow change the founder pushed. Two things cover the same risk: the gate checks that the owner's latest approval is for the head commit, and stale approvals are dismissed (P02 §12.2's note). |
| 4 | What the owner's approval proves | It covers the commit approved. The "Intent approved by" line is the agent's record of the session, not a check. A description edited after approval keeps the approval. The handbook says both. |
| 5 | Changing the gate's own commands | A change to the arguments of `guard` or `verdict` lands in two steps. First, the commands accept both the old and the new arguments; then the workflow switches. The workflow runs those commands as built from the base, so a single-step change would fail its own gate. The ruleset is applied only once `main` has these commands. |

## Rollout / undo plan

Steps 1 to 3 are set up in step 6, before the first agent session works in the repository, and the red-team tests confirm them. Any of them can be loosened later by a superseding record.
