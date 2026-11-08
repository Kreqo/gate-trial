<!--
SPDX-FileCopyrightText: Kreqo contributors
SPDX-License-Identifier: CC-BY-4.0
-->

# The Kreqo workflow

Agents write the code. Checks and an independent AI review decide what merges. You, the founder, state what to build, settle the few decisions that are yours, and approve the gated changes. Nobody reads code to approve it.

This page describes the workflow and points to its sources: the decision records in `docs/decisions/` (DR-0001 to DR-0008), which win where anything disagrees; the rules that apply now, in `conventions/views/NOW.md` (the ones that need you) and `conventions/views/CHECKED.md` (the ones checked for you); and the reviewer's rubric, `conventions/rubric/review.md`. Rules are named by ID; look them up in the views.

Until the repository is on the forge, DR-0001 row 12 applies: you approve records in chat, and each build step lands as one local commit.

## From intent to merge

Agents work in your local Claude Code sessions, not in CI, and the intent lives in the pull request body (DR-0007). Each session runs as a separate OS user or in a container, and its only forge credential is a short-lived token of the agent's GitHub App, minted outside it: your own forge login, your browser and the App's private key are out of its reach (DR-0008 row 1).

1. **You state an intent** in a session: what to build and why, and what you want to see when it's done.
2. **The session plans before it codes.** It restates the intent, writes acceptance criteria (`AC-1`, `AC-2` …), classifies each open question under the dilemma protocol (below), and says whether any protected path is touched. **You read that intent and its acceptance criteria and approve them** before any code is written; the pull request records your approval (GOV-14). If a question is class 3, it stops that part and drafts a decision pull request.
3. **It implements** on a branch, with tests that name the criterion they check, and runs `cargo xtask check` before pushing. Commits carry your name and disclose AI use (see "Working in the repository").
4. **It opens the pull request.** It pushes the branch and opens the pull request through the agent's GitHub App (glossary, "Agent identity"), labels it `agent-pr`, and enables auto-merge through the App (`gh pr merge --auto --squash`). The body follows the PR template (CI-12): the intent and its acceptance criteria, which are the record of your input, then the tests, assumptions, demo evidence, new dependencies, risks and AI disclosure.
5. **CI runs the gate** (next section).
6. **If the review asks for changes,** the session reads the findings in the reviewer job's `review` artifact or its log, fixes them and pushes. After three failed attempts it labels the pull request `blocked` and stops, and the digest lists it (CI-18).
7. **It merges itself.** Because auto-merge is on, the merge queue squash-merges the pull request once the gate passes, and once you approved it if it's gated (CI-06, CI-13).

Work moves only while a session runs. Nothing waits on your approval of each change (DR-0007 row 5).

## The gate

Two checks are required on the default branch:

- **`gate`** passes only when `ci` (every check `cargo xtask check` runs, CI-02), `guard`, the AI review and, for a change to `xtask/`, the `mutants` job (QA-08) all pass; the `hold` label is off; and, for a gated change, your latest review approves the head commit (CI-15). A missing check counts as failed. Only a run for a pull request into `main`, or for a merge group, reports `gate`; a run on any other event, such as one started by hand, reports under another name, such as `gate (workflow_dispatch)`, which counts for nothing.
- **`description`** scans the pull request's title and body for secrets, because they become the squash commit's message (DR-0005).

The **AI review** runs on Claude Opus (DR-0002) in its own job, with a fresh context and read-only access (CI-03). It judges the change against the rubric and returns a verdict: `approve`, `changes_requested` or `needs_human`, kept with its findings as the job's `review` artifact. Anything but a readable `approve` fails the gate. The reviewer never runs the pull request's code.

**What you approve.** Only gated changes (DR-0001 row 5, as DR-0006 restates it):

- changes to the checks, the conventions or license policy. `conventions/protected-paths.txt` lists the paths, and it generates `.github/CODEOWNERS`, which makes you their owner (CI-15). Decision records, every `CLAUDE.md` file and `.claude` folder that agent sessions load, the tools' configuration files and `media/` are on the list too;
- every crates.io publish (REL-17);
- AI-drafted user documentation, which you revise before a release ships it (GOV-14).

A pull request that touches a protected path or a guarded field (a crate's `license`, `license-file`, `publish` or lint table, the ones the root `Cargo.toml` lets crates inherit, or a project's `kind` or `license`) gets the `needs-human` label. The guard fails it unless its body has a line `Decision: DR-NNNN` naming a record that exists. You read that record, which fits on one screen, and approve the pull request on the forge's website or in its mobile app, never from an agent session. You never read the diff.

**What your approval proves** (DR-0008 row 4). It covers the commit you approved: a later push dismisses it, and the gate passes only while your latest review approves the head commit. The `Intent approved by:` line in the body is the agent's record of the session, not a check; nothing verifies it. A description edited after you approved keeps your approval, so the intent or the `Decision:` link can change after you looked; the guard and the AI review run again on the edit.

**A decision pull request** holds a decision record and whatever change it decides. Approve it to accept the recommendation, ask in a comment for another option and approve once the record is updated, or close it to reject.

**Nothing else waits on you.** To look at any pull request before it merges, add the `hold` label; auto-merge waits until you remove it (DR-0001 row 4). Only you remove it: if anyone else does, the gate keeps failing until you add it and remove it again. `hold` doesn't stop a pull request whose merge group has already passed `gate`; remove that one from the merge queue instead. Agents never approve, never merge by hand and never remove `hold`; enabling auto-merge when a session opens a pull request isn't merging by hand.

| Label | Meaning |
| --- | --- |
| `agent-pr` | opened by the agent's GitHub App |
| `needs-human` | touches a protected path or changes a guarded field; needs your approval |
| `needs-decision` | the AI review escalated a class-3 question |
| `decision` | a decision pull request |
| `blocked` | three fix attempts failed |
| `hold` | you paused its auto-merge |
| `dependencies` | from the dependency-update bot |

**Changing the gate's own commands** (DR-0008 row 5). The workflow runs `cargo xtask guard` and `cargo xtask verdict` as built from the base commit, so a pull request that changes their arguments in one step fails its own gate. Such a change lands in two pull requests: the first makes the commands accept both the old and the new arguments, the second switches the workflow. For the same reason, the ruleset is applied only once `main` has these commands.

## The dilemma protocol

Agents settle open questions by class (glossary, "Dilemma protocol"):

| Class | Question | What happens |
| --- | --- | --- |
| 1 | Reversible and technical | The agent decides and logs it as an assumption in the pull request. |
| 2 | Reversible but product-shaping: a name, a default, behavior users see | The agent goes ahead with its recommended default. The AI review lets it through and lists it, and the digest's Monday rollup shows it. To change one, state a new intent. |
| 3 | Irreversible, gated, a license, a public-interface or published-format commitment, or a question a rule settles by decision record | The affected work stops, and a decision pull request offers at most three options with a recommendation. |

At most three class-3 decisions are open at once, each with a recommendation; more wait in line (DR-0001 row 7). Class-2 defaults stand without any action; class-3 decisions never time out.

When the AI review returns `needs_human`, for a class-3 question or a change outside the intent, the pull request gets `needs-decision` and stays unmerged. A decision pull request follows. Once it merges, the pull request is updated and reviewed again (REV-04, CI-18).

## The digest

The digest is how you hear about everything else. It isn't running yet; a later build step adds it. It is written each morning from CI data, exception-first, and near-silent on quiet days (DR-0001 row 7):

- **Needs you:** open class-3 decisions and publishes awaiting approval, three at most. A red default branch or a new security advisory always comes first and doesn't count toward the three.
- **Shipped:** what merged, with its test and demo evidence.
- **Blocked:** pull requests marked `blocked` (CI-18).
- **Human-only steps** that are due (glossary).

On a quiet day it is three lines at most. The **Monday rollup** adds the week's class-2 decisions, trends in coverage and mutation scores, and how often the AI review caught what the checks missed. Reading it should take under 20 minutes a day (DR-0001 row 11).

## Releases

No release has happened yet; these rules apply from the first one (`conventions/views/LATER.md`).

- **Every release** goes through a release pull request whose description is the release summary, which an agent writes and discloses as AI-written (REL-05, GOV-14). CI attaches each release check's evidence to it. It merges through the normal gate.
- **A crates.io publish** then waits in a protected environment for your approval of the release summary, and publishes through trusted publishing (REL-17). That approval is your only step.
- **Apps** release when an intent asks for one, through the normal gate. They aren't gated, except that AI-drafted user documentation waits for your revision (GOV-14).

The threats this workflow defends against, and what it leaves to you, are in [threat-model.md](threat-model.md).

## Human-only steps

Some steps only a human can take. The digest lists them, and each one blocks nothing unless its rule says so (glossary).

**Now:**

- Answer security reports (SEC-04).
- Keep phishing-resistant MFA on every account (SEC-12) and the forge hardened (CI-11).
- Keep the asset register and its escrow (LIFE-02).
- Write anything that speaks for you or the project, such as replies and announcements (GOV-14).
- Contribute to other projects yourself (GOV-04, ARCH-05).
- Make the art and media; agents never write under `media/` (GOV-22).
- Set up the forge once: the agent's GitHub App (`conventions/forge/agent-app-manifest.json`, with the read access DR-0009 allows), the Claude federation rule for CI, the rulesets (`conventions/forge/ruleset-default-branch.json` and `ruleset-tags.json`), the labels (`conventions/forge/labels.toml`), the merge settings (squash only, auto-merge, delete branches on merge) and four repository variables: `ANTHROPIC_FEDERATION_RULE_ID`, `ANTHROPIC_ORGANIZATION_ID`, `ANTHROPIC_SERVICE_ACCOUNT_ID` and `REVIEW_ALLOWED_BOTS` (the agent App's bot login). Until they're set, every AI review fails closed. Under DR-0008, before the first agent session works in the repository:
  - run agent sessions as a separate OS user or in a container, whose only forge credential is a short-lived installation token of the App, minted outside it; your forge login, your browser and the App's private key stay out of its reach, and you approve from the forge's website or mobile app (row 1);
  - give the App the contents and pull requests permissions, and issues if a rule needs them, but never `workflows`, `actions`, `checks`, `statuses` or `administration`; you push workflow changes yourself (row 2);
  - keep the ruleset's "Require approval of the most recent push" off, as the JSON sets it, so that you can approve a workflow change you pushed. The gate's head-commit check and the dismissal of stale approvals cover the same risk (row 3).

**From the first release:** release notes (REL-13); revising AI-drafted user documentation, which holds the release (GOV-14); RustSec advisories and CVE requests (SEC-14); pre-notifying packagers (SEC-22); announcing wider data collection (PRIV-05); Flathub submissions (PLAT-07); reviewing translations (I18N-12); the accessibility and old-hardware tests CI can't run (A11Y-05, PERF-06); and custody of signing keys (SEC-16).

The decisions that are yours by rule, such as the UI toolkit or a privacy review, are listed in `conventions/views/NOW.md`.

## Stopping everything

In an incident, stop automatic merges first, then the agents, then revoke and rotate (SEC-15).

1. **Stop merges.** Turn off auto-merge (Settings, General, "Allow auto-merge"), then disable Actions for the repository (Settings, Actions, General). Turning off auto-merge alone isn't enough: the merge queue can still merge a pull request. With Actions off, no required check can pass, so nothing merges.

   ```sh
   gh repo edit Kreqo/kreqo --enable-auto-merge=false
   gh api -X PUT repos/Kreqo/kreqo/actions/permissions -F enabled=false
   ```

2. **Stop the agents.** End every Claude Code session, and stop minting the App's token (`systemctl --user stop kreqo-app-token.timer`). Suspend the agent's GitHub App (organization Settings, GitHub Apps, Configure, Suspend), which stops it from pushing, opening pull requests or commenting. Archive the CI federation rule in the Claude Console, so no CI agent can authenticate; an archived rule can't be restored.
3. **Revoke and rotate** what may have leaked, such as the App's private key, and handle affected releases as SEC-15 says.
4. **Resume** once the cause is fixed, in reverse order: a new federation rule, whose ID replaces the `ANTHROPIC_FEDERATION_RULE_ID` variable, then the App and its token timer, Actions, then auto-merge. Run the Daily workflow by hand, because its secret scan was off.

## Working in the repository

These apply to people and agents alike. `CLAUDE.md` summarizes them for agents.

- Install the git hooks once per clone with `cargo xtask hooks`; they scan each commit for secrets (SEC-03).
- Run `cargo xtask check` before every push. It runs what CI runs, with the pinned tool versions (DR-0004).
- Never commit with `--no-verify` (DR-0005), and never weaken a check, threshold or lint to get a green build. A check that is wrong is changed through a decision record.
- Dara Kong `<dara@kreqo.org>` is the author of every commit (DR-0001 rows 3 and 10). AI use is disclosed with the `Co-Authored-By:` trailer naming the model, as the tooling emits it (GOV-03); AI never appears as author or committer (GOV-16).
- Pull request titles follow Conventional Commits (REL-03).
- Every file carries an SPDX header (LEGAL-02), with the license DR-0004 sets for its kind.
- Text is in American English, the project's one English variant (CI-19). Rule text says "the forge", never a product name (glossary).
