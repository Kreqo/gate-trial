<!--
SPDX-FileCopyrightText: Kreqo contributors
SPDX-License-Identifier: CC-BY-4.0
-->

# DR-0006: The conventions fit the workflow

Status: accepted   Date: 2026-11-07   Scope: org

## What you are approving

101 rules, the glossary, the moments, the README's "Levels" and the decision record template change so the conventions describe the workflow in DR-0001 to DR-0005 instead of hand review. Two exceptions are deliberate: the founder revises AI-drafted user documentation before it ships, and agents review French for now.

## Context

Origin: the conventions were adopted from an earlier internal draft, written for a founder who reviewed and merged every change. Every rule starts with no change history.

Evidence: judgment, applied with care:
- six parallel reviews of all 316 rules against the workflow, which found conflicts in 125;
- the founder's decisions on ten patterns and nine rules;
- a rewrite;
- a consistency pass;
- an independent review, which judged the result faithful and flagged two rules for the founder.

The working material sits outside the repository.

## Decisions

Each pattern is applied to the rules listed with it. A rule can follow more than one.

| # | Pattern | Rules |
| --- | --- | --- |
| 1 | `who = "you"` only for the founder's decisions and human-only steps. Everything agents do is `ai` or `tool`, so the AI review or a check covers it. | AIF-01, AIF-02, AIF-03, ARCH-05, BIZ-03, CI-03, DATA-04, DATA-05, DATA-11, DOC-12, GOV-02, LEGAL-08, PLAT-09, PLAT-18, REL-09, REL-10, SEC-11, UX-04 |
| 2 | No human review or approval of each change: the AI review and the narrow gate replace it. Second-maintainer versions become reminders to decide the role by ADR. | CI-03, CI-06, CI-15, CI-18, GOV-02, GOV-13, REL-01, REV-01, REV-02, REV-03, SEC-18, UX-02, UX-08, UX-24 |
| 3 | One way to decide: ADRs and decision pull requests. DOC-04 moves ADRs to `docs/decisions/DR-NNNN` on TEMPLATE.md, drops MADR, the Y-statement and the rejected and deprecated statuses, and allows partial supersession. The glossary's ADR entry follows. RFCs return once outside contributions open. | BIZ-01, CI-01, CI-12, DOC-04, DOC-12, GOV-01, GOV-07, GOV-09, LIFE-05, REL-16, REV-04 |
| 4 | ADRs only for class-3 questions. A SHOULD deviation is stated in its pull request with a reason the AI review checks, unless it is class 3. A MUST changes only through the conventions, except where a rule names its own exception. | ARCH-05, ARCH-15, ARCH-16, BRAND-01, CI-01, DOC-06, REL-05, UX-15, README "Levels" |
| 5 | The stricter checks for published crates (tier H) and mutation testing of `xtask`, as DR-0001 row 9 and DR-0002 set them. | CI-02, CI-17, GOV-17, QA-01, QA-07, QA-08, QA-13, REL-02 |
| 6 | App releases pass the normal gates, and checklist items become CI evidence on the release pull request. REL-17 holds the founder's approval of each crates.io publish. | A11Y-04, REL-05, REL-06, REL-13, REL-17 |
| 7 | Evidence, not claims: checks that need a running app come from demo evidence, and noisy checks run nightly. | A11Y-01, A11Y-02, A11Y-04, A11Y-06, A11Y-08, A11Y-10, BRAND-01, BRAND-05, CI-12, GOV-17, I18N-05, PERF-04, UX-24, WEB-07, WEB-09 |
| 8 | The agent identity: the founder's agents act through it, the founder answers for them, and the rules for outside contributors stay. | CI-09, CI-10, CI-12, CI-13, GOV-03, GOV-05, GOV-07, GOV-08, GOV-14, GOV-16, LIFE-02, REV-03 |
| 9 | CI security with agents: pull-request workflows hold only the model key and the agent identity's key, an incident stops automatic merges first, and action and toolchain updates come monthly. | CI-09, CI-10, LIFE-02, SEC-15, SEC-17, SEC-20 |
| 10 | Per-project facts go in `project.toml`, and references to one repository per project, a risk file and pre-receive hooks are removed. | CI-05, GOV-01, GOV-02, LIFE-01, LIFE-04, PERF-01, PLAT-03, PLAT-04, SEC-03 |

**Decisions on specific rules:**
- **SEC-01:** the AI review judges each `unsafe` block's soundness. The SAFETY comment, Miri and the crate's ADR stay.
- **PRIV-08:** the founder approves each privacy review as a class-3 ADR, and approval by someone other than the author returns at `second-maintainer`.
- **SEC-13:** an embargoed fix is made in a local interactive session with the same checks and AI review. Its pull request opens at disclosure.
- **EXT-05:** each plugin listing passes a recorded review against the published policy.
- **A11Y-05, PERF-06, I18N-12:** CI does what it can, and the rest is a human-only step that doesn't block the release.
- **PLAT-07:** Flathub text stays hand-written, as a human-only step.
- **ARCH-05:** a contribution upstream is a human-only step, and a focused crate of our own may stand in until it lands.
- **LEGAL-03:** states DR-0004's license list.
- **Human-only steps:** SEC-04, SEC-14, SEC-22 and PRIV-05 name theirs.
- **I18N-08:** routes the translation platform's pull requests through the same gates. Who authors them is left to the gating trial (DR-0001 row 3).

**The founder's two exceptions:**
- **GOV-14:** AI-drafted user documentation ships only after the founder revises it, with disclosure. A release that ships it waits for that revision. This widens DR-0001 row 5, which now reads: "The owner approves changes to checks, conventions or license policy and every crates.io publish, each through a decision record or release summary, and revises AI-drafted user documentation before a release ships it (GOV-14)."
- **I18N-12:** while no French reviewer other than the founder exists, an agent other than the translating one reviews French, so French never holds a release. No human reviews French before it ships.

**Supporting changes:**
- The glossary adds Intent, Agent identity, The AI review, Decision pull request, Dilemma protocol, Gated change, The digest, Demo evidence, Human-only step and Release summary. It retires "Sign-off list" and rewrites five entries.
- Five moment titles now describe decisions rather than work.
- The decision record template's status line allows partial supersession.

## Recommendation and why

Approve. Without these changes the conventions contradict the workflow in 27 places and burden it in 65 more. The founder keeps the decisions that are theirs: the two exceptions above, privacy reviews, and every commitment that class 3 covers.

## Rollout / undo plan

- **On acceptance:** DR-0001's status becomes "accepted; row 5 superseded by DR-0006".
- **To undo a pattern or a rule:** a later record supersedes that row here, with the matching rule changes.
- **The reviewer's rubric:** it should follow P02 §11.5 for class-2 choices. It belongs with the workflow handbook, not with these rules.
