<!--
SPDX-FileCopyrightText: Kreqo contributors
SPDX-License-Identifier: CC-BY-4.0
-->

# Threat model: prompt injection and leaked credentials

The repository is public, so anyone can write text an agent later reads: an issue, a comment, a pull request, a dependency's code, a web page. For each actor, this page lists what it reads, what it holds, what a planted instruction could make it do, what stops that, and what remains. DR-0005 covers what the checks defend against; the decision records win over this page.

## Your agent sessions

- **Reads:** your instructions; the repository, with the `CLAUDE.md` files and `.claude` settings it loads; pull request bodies and review findings; issues and web pages when asked; dependencies, whose build scripts and macros run when it builds.
- **Holds:** a token of the agent's App, minted outside the session, for one repository and one hour; the sandbox user's own Claude login; open network access. Your forge login, browser, SSH keys and the App's private key are out of its reach (DR-0008 row 1).
- **An injection could:** put a harmful or check-weakening change in its pull request, do anything the App can (next section), write a false `Intent approved by:` line, or send its token and Claude login elsewhere.
- **Mitigation:** the separate OS user or container (DR-0008 row 1), with no claude.ai connector that acts on the forge as you, since Claude Desktop hands its connectors to the sessions it starts; the App's narrow permissions; the gate; protected paths that put every file a session loads as instructions under your approval (`CLAUDE.md`, `CLAUDE.local.md`, `.claude`, `.mcp.json`, in any folder and as links; `conventions/protected-paths.txt`, CI-15); and your approvals, given only on the forge's website or mobile app, from an account with phishing-resistant MFA (SEC-12).
- **Remains:** egress is open, so the token and the Claude login can leave. The intent line is the agent's record, not a check (DR-0008 row 4). A session that checks out someone else's branch loads its `CLAUDE.md` and `.claude` hooks.

## The agent's GitHub App and its token

- **Reads:** nothing on its own; sessions act through it.
- **Holds:** write access to contents and pull requests, and to issues only if a rule needs it; never `workflows`, `actions`, `checks`, `statuses` or `administration` (DR-0008 row 2). Its private key stays in your account (LIFE-02).
- **An injection could:** push any branch but `main`, delete branches, push tags, create releases, and open, edit, label or close pull requests. It can't push a workflow, run CI, post a check, change a setting or approve its own pull request, and its review of anyone else's counts for nothing.
- **Mitigation:** DR-0008 row 2; a ruleset with no bypass actor (`conventions/forge/ruleset-default-branch.json`); a gate that counts only your latest review of the head commit, and treats `hold` as still on when anyone else removed it (`.github/workflows/ci.yml`, job `gate`; CI-15, DR-0001 row 4). In an incident, suspend the App (handbook, "Stopping everything").
- **Remains:** a leaked private key mints tokens until you revoke it. An approval survives a description edit, though the guard and the AI review run again (DR-0008 row 4). No tag ruleset exists yet (CI-11).

## The AI review in CI

- **Reads:** the pull request's title, body, file list, diff and head commit (in `pr/`), all untrusted; the base's rubric, `CLAUDE.md`, `CHECKED.md` and decision records.
- **Holds:** a short-lived Claude credential from workload identity federation, with no stored secret (DR-0007 row 1, CI-09); the job's OIDC request token; a read-only job token.
- **An injection could:** win approval for a change it should block, get a credential quoted in its findings (the log and the `review` artifact are public), or burn its turns.
- **Mitigation** (`ci.yml`, job `reviewer`; rubric, "What you work with"):
  - Read, Grep and Glob only, and no reads outside the base checkout and the review folder, nor in `.git` or `/proc`;
  - no settings, hooks, MCP servers or `CLAUDE.md` files load, and nothing from the pull request is built or run; the diff ignores its `.gitattributes`;
  - the rubric makes an instruction addressed to the reviewer a blocking finding;
  - it writes no file, and `cargo xtask verdict`, built from the base, fails closed on anything but a consistent approval (`xtask/src/verdict.rs`, CI-03);
  - the federation rule accepts only this repository's `pull_request` runs; a fork's run gets no OIDC token.
- **Remains:** a subtle change outside the protected paths that wins approval: the checks still run, but DR-0002 measured the catch rate on a small sample. A same-repository pull request runs its own `ci.yml`, `id-token` included, before anyone approves, and the federation rule can't tell it from the base's copy; that's why only you push workflow changes (DR-0008 row 2).

## The guard and the gate

- **Reads:** the pull request's body and tree, as data; its labels, label events and reviews; the base's owner line.
- **Holds:** `contents: read`; the gate adds `pull-requests: write` for labels, and the Gate re-run workflow `actions: write`. No model, no secret.
- **An injection could:** work only as script injection or a misread: a crafted title, body, branch or label, a `Decision:` line naming the wrong record, or another event's run posting `gate`.
- **Mitigation:** untrusted text reaches scripts only through environment variables, and zizmor audits the workflows (CI-09). `guard` and `verdict` are built from the base, without a cache (`xtask/src/guard.rs`). The linked record must exist, and the AI review checks it covers the change. Only a pull request into `main` or a merge group reports `gate`, and a missing job fails it.
- **Remains:** a pull request's own workflows run with their permissions before approval, so a fork that edits them can post a passing `gate` on its head commit; code-owner review of `/.github` stops that merge (CI-15). The merge queue doesn't repeat the guard or the AI review, and `hold` can't stop a merge group that has passed.

## The digest

Not built yet; this is what it must meet.

- **Reads:** pull request titles and bodies, review summaries (which echo pull request text), labels, check results, advisories. **Holds:** a read-only token, plus what delivery needs (DR-0007 row 6).
- **An injection could:** if a model writes it, drop a "Needs you" item, show a red branch as green, or add a link.
- **Mitigation, to build:** "Needs you", "Blocked" and red-branch lines come from labels, checks and advisories, never from text; untrusted text appears only quoted and shortened; links are built from numbers.
- **Remains:** you read text an attacker can shape.

## Outside contributors, once contributions open

Contributions aren't open (`conventions/state.toml`), but anyone can already open an issue or comment.

- **Writes:** issues, comments and, later, fork pull requests, which every actor above may read. **Holds:** nothing of ours; a fork's run gets a read-only token and no secret or OIDC token.
- **An injection could:** reach a session that reads their text, the AI review, and the digest.
- **Mitigation:** sessions take instructions only from you (DR-0007 row 2) and read findings from the `review` artifact, not comments. Fork runs need your approval (CI-10), their AI review fails closed, and code-owner review stops a forged `gate`. GOV-05, GOV-08 and REV-03 wait for the trigger.
- **Remains:** no interaction limits are set. Fork pull requests can't pass the AI review, and the Gate re-run workflow gets no write token on their events, so they need a route when contributions open. Until then, no session checks out a fork's branch.

## Residual risks you accept

1. A planted instruction wins approval for a subtle unprotected change; the checks, `hold` and the calibration drill (DR-0002) limit it.
2. A compromised session spends the App's token for up to an hour, and the sandbox's Claude login for longer, but can't merge past the gate or approve.
3. The intent line is unverified, and an approval survives a description edit (DR-0008 row 4).
4. A same-repository pull request's own workflows run before approval.
5. The AI review's output is public, so anything it reads beyond its limits is published.

## Red-team tests

Step 6 adapts P02's acceptance suite to DR-0007 and DR-0008. Run it again after any change to the workflows, the ruleset, the rubric or the App's permissions.

| Mitigation | Tests |
| --- | --- |
| An unprotected change merges with no approval | R0 |
| Sessions can't reach your credentials (DR-0008 row 1) | R12 |
| The App's limits; its approvals and removal of `hold` don't count (DR-0008 row 2) | R3, R12 |
| Code-owner review and approval of the head commit (CI-15, DR-0008 row 3) | R3, R4 |
| The guard: protected paths, links, guarded fields | R3, R5, R15 |
| Only pull request and merge group runs report `gate` | R3 |
| The checks: Clippy, licenses, attributes, mutation tests | R1, R2, R5, R8 |
| The AI review applies the rubric and acceptance criteria | R5, R6 |
| The AI review fails closed | R11 |
| The AI review's isolation | R16 |
| Fork runs and a forged `gate` | R17, once contributions open |
| The digest | added with the digest |
