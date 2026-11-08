<!--
SPDX-FileCopyrightText: Kreqo contributors
SPDX-License-Identifier: CC-BY-4.0
-->

# Kreqo: notes for agents

This file only summarizes; no rule or process exists only here (DOC-13). The sources:

- `docs/decisions/`: accepted decision records, which win over everything else;
- `conventions/views/NOW.md` and `conventions/views/CHECKED.md`: the rules that apply now, generated from `conventions/rules/` (never edit the views by hand);
- `docs/handbook/`: the workflow;
- `conventions/rubric/review.md`: what the AI review checks.

## How a change lands

You work in the founder's local session, and the intent lives in the pull request body (DR-0007). The session runs as a separate OS user or in a container, and its only forge credential is a short-lived token of the agent's GitHub App, minted outside it. The founder's forge login, browser and the App's private key are out of your reach; never try to reach them (DR-0008 row 1).

- **Before coding,** restate the intent, write acceptance criteria `AC-1…n`, classify each open question under the dilemma protocol, and say whether a protected path is touched. Then wait until the founder has read and approved that intent; record the approval in the pull request (GOV-14).
- **Class 1:** decide and log it as an assumption. **Class 2:** go ahead with your recommended default and log it with its class. **Class 3:** stop that part and draft a decision pull request from `docs/decisions/TEMPLATE.md`, at most three options with a recommendation.
- **One topic per pull request** (CI-13). Every `AC-n` has a test that names it and would fail if it broke (QA-09, GOV-17).
- **Open the pull request** through the agent's GitHub App (glossary, "Agent identity"): push the branch with the App's token, open the pull request with every section of the PR template filled in (CI-12), the AI disclosure included (GOV-03), and label it `agent-pr`. A protected change needs a `Decision: DR-NNNN` line. Then enable auto-merge through the App, `gh pr merge --auto --squash`, so that the merge queue merges it once the gate passes.
- **If the AI review requests changes,** read its findings in the reviewer job's `review` artifact or its log, fix and push. After three failed attempts, label the pull request `blocked` and stop (CI-18). On `needs-decision`, stop until the decision pull request merges.

## Working rules

- Run `cargo xtask hooks` once per clone, and `cargo xtask check` before every push.
- Never commit with `--no-verify`. Never weaken a check, threshold or lint to get a green build (DR-0005).
- Never approve or merge a pull request by hand, and never remove `hold` (DR-0001 row 4). Enabling auto-merge when you open a pull request isn't merging by hand.
- Commits are authored by Dara Kong `<dara@kreqo.org>` and carry the `Co-Authored-By:` trailer naming the model, as the tooling emits it (DR-0001 row 3, GOV-03, GOV-16). Pull request titles follow Conventional Commits (REL-03).
- The paths in `conventions/protected-paths.txt` and the guarded fields are gated: change them only in a pull request that links its decision record (CI-15). Never write under `media/`.
- Every new file carries an SPDX header (LEGAL-02), with `SPDX-FileCopyrightText: Kreqo contributors` and the license DR-0004 sets for its kind. No secret ever enters the repository (SEC-03).
- Write American English. In rule text, write "the forge", not the product's name.
- Before changing a check in `xtask/`, read DR-0005.
