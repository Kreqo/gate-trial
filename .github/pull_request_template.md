<!--
Fill in every section (CI-12), and replace each comment like this one with
your text: this description becomes the squash commit's message (CI-13), so
a comment left here stays in the history. The title follows Conventional
Commits (REL-03).
-->

## Intent

<!--
The intent this pull request implements, in full, exactly as the
contributor read and approved it before work started: what to build and
why, then its acceptance criteria, AC-1 to AC-n, each one observable. The
agent writes it, so it's AI-written, and the AI disclosure below says so
(glossary, "Intent"; DR-0007; GOV-14). A later pull request of a planned
series links the first one, which holds the shared plan (DOC-12).

Then fill in the approval line below, which every pull request needs: the
contributor's login and the date they approved the intent in the session.
The AI review blocks a pull request without it.

A protected change, one that touches a path in
conventions/protected-paths.txt or a guarded field, also needs a line of
its own that reads "Decision: DR-NNNN", with the number of the decision
record that covers it. The guard job checks that the record exists, and the
owner approves the pull request (CI-15).
-->

Intent approved by: @<login> in session, YYYY-MM-DD

## Summary

<!-- What changes, in plain language, in five lines at most. -->

## Acceptance criteria and tests

<!--
Each acceptance criterion of the intent, by its ID, with the tests that
would fail if it broke (QA-09), one line each:
- AC-1 → `<path>::<test>`
-->

## Assumptions

<!--
Each assumption with its class under the dilemma protocol and its reason,
one line each:
- A1 (class 1 or 2): <what was decided> — reason: <why> — reversible: yes
A class-3 question is never an assumption: it needs a decision pull
request. State a deviation from a SHOULD here too, with its reason. Write
"None." if there are none.
-->

## Demo evidence

<!--
What the change does when it runs: the demo commands' output, with
before/after screenshots or a recording for a change to an interface
(CI-12). CI's demo job fills this in once it exists; until then, paste the
commands you ran and what they printed.
-->

## New dependencies

<!--
Each new dependency: its name, why it's needed, its license, and its step
on the dependency ladder (ARCH-05). Write "None." if there are none.
-->

## Docs

<!-- The docs this updates, or why none needed updating. -->

## Risks and rollback

<!--
What could go wrong, the compatibility impact (a breaking change is flagged
and has a migration-guide fragment, REL-14), and how to roll it back.
-->

## AI disclosure

<!--
How AI was used and what it produced, with the model's name; the human
creative input (the intent the contributor approved, design, selection,
editing); and what verified the change: the checks, the AI review (CI-03)
and any human check (GOV-03). A description written through the agent
identity says it's AI-written, the intent's text included (GOV-14).
-->
