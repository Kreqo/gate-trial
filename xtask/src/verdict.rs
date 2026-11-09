// SPDX-FileCopyrightText: Kreqo contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `cargo xtask verdict`: enforces the AI review's verdict (CI-03, CI-18).
//! The reviewer returns `review.json` under the schema of P02 Appendix A.4,
//! which the review rubric gives it. Only an approval passes, and only with
//! no blocking finding and nothing left unchecked; an approval that its own
//! record contradicts, with an acceptance criterion that no test covers or
//! whose tests are marked inadequate, or an assumption to escalate, fails
//! too. Everything else fails closed with a one-line reason: changes
//! requested, a question for the owner, a schema error, or a missing or
//! empty file (P02 §5, invariant 7).

use std::fs;
use std::io;
use std::path::Path;

use serde::Deserialize;

use crate::check::printable;

/// The AI review's record, `review.json`; unknown fields are refused.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Review {
    verdict: Verdict,
    /// At most three plain sentences for the digest, then any class-2
    /// choices it went ahead with (P02 §11.5).
    summary: String,
    acceptance_criteria: Vec<Criterion>,
    findings: Vec<Finding>,
    /// What the review couldn't verify, with why.
    not_checked: Vec<String>,
    /// Assumptions that should have been class 3, with why.
    assumptions_to_escalate: Vec<String>,
}

/// The verdict (REV-01, rubric "The verdict").
#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Verdict {
    Approve,
    ChangesRequested,
    NeedsHuman,
}

impl Verdict {
    fn name(self) -> &'static str {
        match self {
            Self::Approve => "approve",
            Self::ChangesRequested => "changes_requested",
            Self::NeedsHuman => "needs_human",
        }
    }
}

/// An acceptance criterion, with the tests that cover it and whether they
/// would catch it breaking.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Criterion {
    id: String,
    tests: Vec<String>,
    adequate: bool,
}

/// A finding, blocking or advisory (REV-02).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Finding {
    severity: Severity,
    category: Category,
    file: String,
    line: u64,
    issue: String,
    evidence: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Severity {
    Blocking,
    Advisory,
}

/// The rubric item a finding falls under.
#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Category {
    Intent,
    Correctness,
    Tests,
    CheckWeakening,
    Api,
    Dependency,
    Docs,
    Assumption,
}

impl Category {
    fn name(self) -> &'static str {
        match self {
            Self::Intent => "intent",
            Self::Correctness => "correctness",
            Self::Tests => "tests",
            Self::CheckWeakening => "check-weakening",
            Self::Api => "api",
            Self::Dependency => "dependency",
            Self::Docs => "docs",
            Self::Assumption => "assumption",
        }
    }
}

/// Checks the verdict in `file`: prints a short summary of it, and returns
/// whether it lets the pull request merge, with the reason on one line when
/// it doesn't.
pub fn run(file: &Path) -> anyhow::Result<bool> {
    let shown = file.display();
    let review = match read(file) {
        Ok(review) => review,
        Err(reason) => {
            eprintln!("The AI review fails: {shown}: {}", printable(&reason));
            return Ok(false);
        }
    };
    for line in summary(&review) {
        println!("{line}");
    }
    match judge(&review) {
        Ok(()) => {
            println!("The AI review approves (CI-03).");
            Ok(true)
        }
        Err(reason) => {
            eprintln!("The AI review fails: {}", printable(&reason));
            Ok(false)
        }
    }
}

/// Reads the verdict and checks its schema, or says why it can't be used.
fn read(file: &Path) -> Result<Review, String> {
    let text = match fs::read_to_string(file) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Err("missing, so there is no verdict, which fails closed".to_owned());
        }
        Err(error) => return Err(format!("couldn't read it: {error}")),
    };
    if text.trim().is_empty() {
        return Err("empty, so there is no verdict, which fails closed".to_owned());
    }
    let review: Review = serde_json::from_str(&text)
        .map_err(|error| format!("a schema error (P02 A.4): {error}"))?;
    if let Some(problem) = problem(&review) {
        return Err(format!("a schema error (P02 A.4): {problem}"));
    }
    Ok(review)
}

/// What the schema asks beyond the types: a summary for the digest, and a
/// file, a line from 1, the issue and the evidence for every blocking
/// finding (P02 §10.4).
fn problem(review: &Review) -> Option<String> {
    if review.summary.trim().is_empty() {
        return Some("the summary is empty".to_owned());
    }
    let incomplete = review.findings.iter().position(|finding| {
        finding.severity == Severity::Blocking
            && (finding.file.trim().is_empty()
                || finding.line == 0
                || finding.issue.trim().is_empty()
                || finding.evidence.trim().is_empty())
    })?;
    Some(format!(
        "finding {} is blocking but lacks its file, its line, its issue or its evidence",
        incomplete + 1
    ))
}

/// Whether the verdict lets the pull request merge, or why not.
fn judge(review: &Review) -> Result<(), String> {
    let blocking = blocking(review).count();
    match review.verdict {
        Verdict::ChangesRequested => Err(format!(
            "changes_requested, with {blocking} blocking finding(s), which hold the merge until a fix passes a new review (CI-18)"
        )),
        Verdict::NeedsHuman => Err(
            "needs_human: a question for the owner holds the merge until a decision settles it (REV-04, CI-18)"
                .to_owned(),
        ),
        Verdict::Approve => {
            let inadequate: Vec<&str> = review
                .acceptance_criteria
                .iter()
                .filter(|criterion| !criterion.adequate)
                .map(|criterion| criterion.id.as_str())
                .collect();
            // Every criterion has a test that would fail if it broke
            // (QA-09, rubric item 2).
            let untested: Vec<&str> = review
                .acceptance_criteria
                .iter()
                .filter(|criterion| criterion.tests.is_empty())
                .map(|criterion| criterion.id.as_str())
                .collect();
            if blocking > 0 {
                Err(format!(
                    "approve, but with {blocking} blocking finding(s), which an approval can't have"
                ))
            } else if !review.not_checked.is_empty() {
                Err(format!(
                    "approve, but with {} thing(s) not checked, which an approval can't have",
                    review.not_checked.len()
                ))
            } else if !inadequate.is_empty() {
                Err(format!(
                    "approve, but the tests of {} are marked inadequate, which an approval can't have",
                    inadequate.join(", ")
                ))
            } else if !untested.is_empty() {
                Err(format!(
                    "approve, but no test covers {}, which an approval can't have",
                    untested.join(", ")
                ))
            } else if !review.assumptions_to_escalate.is_empty() {
                Err(format!(
                    "approve, but with {} assumption(s) to escalate, which an approval can't have",
                    review.assumptions_to_escalate.len()
                ))
            } else {
                Ok(())
            }
        }
    }
}

/// The blocking findings.
fn blocking(review: &Review) -> impl Iterator<Item = &Finding> {
    review
        .findings
        .iter()
        .filter(|finding| matches!(finding.severity, Severity::Blocking))
}

/// A short summary of the verdict, line by line. Each line starts with a
/// label or a marker, and what the reviewer wrote is shown on one line, so
/// that none of it can pass for a workflow command (CI-09).
fn summary(review: &Review) -> Vec<String> {
    let mut lines = vec![format!("Verdict: {}.", review.verdict.name())];
    lines.extend(
        review
            .summary
            .lines()
            .map(|line| format!("  > {}", printable(line))),
    );
    let criteria: Vec<String> = review
        .acceptance_criteria
        .iter()
        .map(|criterion| {
            let tests = criterion.tests.len();
            let state = if criterion.adequate {
                "adequate"
            } else {
                "inadequate"
            };
            format!("{} ({tests} test(s), {state})", printable(&criterion.id))
        })
        .collect();
    lines.push(if criteria.is_empty() {
        "Acceptance criteria: none.".to_owned()
    } else {
        format!("Acceptance criteria: {}.", criteria.join(", "))
    });
    let blocking: Vec<&Finding> = blocking(review).collect();
    lines.push(format!(
        "Findings: {} blocking, {} advisory.",
        blocking.len(),
        review.findings.len() - blocking.len()
    ));
    lines.extend(blocking.iter().map(|finding| {
        format!(
            "  - {}:{} ({}): {}",
            printable(&finding.file),
            finding.line,
            finding.category.name(),
            printable(&finding.issue)
        )
    }));
    for (label, items) in [
        ("Not checked", &review.not_checked),
        ("To escalate", &review.assumptions_to_escalate),
    ] {
        if items.is_empty() {
            lines.push(format!("{label}: nothing."));
        } else {
            lines.push(format!("{label}:"));
            lines.extend(items.iter().map(|item| format!("  - {}", printable(item))));
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scratch::Scratch;

    /// An approval that passes, as the rubric's example writes one.
    const APPROVE: &str = r#"{
  "verdict": "approve",
  "summary": "Adds the parser.\nDecided by default: the name `parse`.",
  "acceptance_criteria": [
    { "id": "AC-1", "tests": ["tests/parse.rs::ac1_rejects_truncated_input"], "adequate": true }
  ],
  "findings": [
    {
      "severity": "advisory",
      "category": "docs",
      "file": "PR body",
      "line": 0,
      "issue": "",
      "evidence": ""
    }
  ],
  "not_checked": [],
  "assumptions_to_escalate": []
}"#;

    /// A blocking finding with everything it needs.
    const BLOCKING: &str = r#"{ "severity": "blocking", "category": "check-weakening", "file": "src/lib.rs", "line": 42, "issue": "An #[allow].", "evidence": "line 42" }"#;

    /// The review, with `from` replaced by `to` in APPROVE.
    fn approve_with(from: &str, to: &str) -> String {
        assert!(APPROVE.contains(from), "{from}");
        APPROVE.replace(from, to)
    }

    /// The findings of APPROVE replaced by these.
    fn with_findings(findings: &[&str]) -> String {
        let start = APPROVE.find("\"findings\"").unwrap_or_default();
        let end = APPROVE.find("\"not_checked\"").unwrap_or_default();
        format!(
            "{}\"findings\": [{}],\n  {}",
            APPROVE.get(..start).unwrap_or_default(),
            findings.join(", "),
            APPROVE.get(end..).unwrap_or_default()
        )
    }

    /// What `read` and then `judge` make of a review's text.
    fn outcome(text: &str) -> Result<(), String> {
        let scratch = Scratch::new().map_err(|error| error.to_string())?;
        let file = scratch.path().join("review.json");
        fs::write(&file, text).map_err(|error| error.to_string())?;
        judge(&read(&file)?)
    }

    #[test]
    fn passes_an_approval_with_nothing_blocking_or_unchecked() {
        assert_eq!(outcome(APPROVE), Ok(()));
        assert_eq!(outcome(&with_findings(&[])), Ok(()));
        assert_eq!(
            outcome(&approve_with(
                "\"acceptance_criteria\": [\n    { \"id\": \"AC-1\", \"tests\": [\"tests/parse.rs::ac1_rejects_truncated_input\"], \"adequate\": true }\n  ]",
                "\"acceptance_criteria\": []"
            )),
            Ok(())
        );
    }

    #[test]
    fn fails_every_other_verdict() {
        assert_eq!(
            outcome(&approve_with("\"approve\"", "\"needs_human\"")),
            Err("needs_human: a question for the owner holds the merge until a decision settles it (REV-04, CI-18)".to_owned())
        );
        assert_eq!(
            outcome(&with_findings(&[BLOCKING, BLOCKING]).replace("\"approve\"", "\"changes_requested\"")),
            Err("changes_requested, with 2 blocking finding(s), which hold the merge until a fix passes a new review (CI-18)".to_owned())
        );
        assert_eq!(
            outcome(&approve_with("\"approve\"", "\"changes_requested\"")),
            Err("changes_requested, with 0 blocking finding(s), which hold the merge until a fix passes a new review (CI-18)".to_owned())
        );
    }

    /// An approval that its own record contradicts fails.
    #[test]
    fn fails_an_approval_its_record_contradicts() {
        assert_eq!(
            outcome(&with_findings(&[BLOCKING])),
            Err("approve, but with 1 blocking finding(s), which an approval can't have".to_owned())
        );
        assert_eq!(
            outcome(&approve_with(
                "\"not_checked\": []",
                "\"not_checked\": [\"the demo: it didn't run\", \"x\"]"
            )),
            Err(
                "approve, but with 2 thing(s) not checked, which an approval can't have".to_owned()
            )
        );
        assert_eq!(
            outcome(&approve_with("\"adequate\": true }", "\"adequate\": false }, { \"id\": \"AC-2\", \"tests\": [], \"adequate\": true }, { \"id\": \"AC-3\", \"tests\": [], \"adequate\": false }")),
            Err("approve, but the tests of AC-1, AC-3 are marked inadequate, which an approval can't have".to_owned())
        );
        // A criterion that no test covers, even one marked adequate.
        assert_eq!(
            outcome(&approve_with(
                "\"adequate\": true }",
                "\"adequate\": true }, { \"id\": \"AC-2\", \"tests\": [], \"adequate\": true }, { \"id\": \"AC-3\", \"tests\": [\"t\"], \"adequate\": true }, { \"id\": \"AC-4\", \"tests\": [], \"adequate\": true }"
            )),
            Err("approve, but no test covers AC-2, AC-4, which an approval can't have".to_owned())
        );
        assert_eq!(
            outcome(&approve_with(
                "\"assumptions_to_escalate\": []",
                "\"assumptions_to_escalate\": [\"A2: should be class 3\"]"
            )),
            Err(
                "approve, but with 1 assumption(s) to escalate, which an approval can't have"
                    .to_owned()
            )
        );
    }

    /// The rubric's example of review.json follows this schema, so the two
    /// can't drift apart; and what the rubric allows passes it: a finding
    /// about the pull request's text, filed under `PR body`, and a summary
    /// whose last line lists the class-2 choices.
    #[test]
    fn reads_the_rubrics_example() {
        let rubric = include_str!("../../conventions/rubric/review.md");
        let example: String = rubric
            .lines()
            .skip_while(|line| *line != "```json")
            .skip(1)
            .take_while(|line| *line != "```")
            .map(|line| format!("{line}\n"))
            .collect();
        let requested = Err("changes_requested, with 1 blocking finding(s), which hold the merge until a fix passes a new review (CI-18)".to_owned());
        assert_eq!(outcome(&example), requested, "{example}");
        let on_the_body = example.replace("\"src/lib.rs\"", "\"PR body\"");
        assert_ne!(on_the_body, example);
        assert_eq!(outcome(&on_the_body), requested);
        let approved = on_the_body
            .replace("\"changes_requested\"", "\"approve\"")
            .replace("\"blocking\"", "\"advisory\"")
            .replace(
                "for the digest.",
                "for the digest. A fourth.\\nDecided by default: the name `parse`.",
            );
        assert_eq!(outcome(&approved), Ok(()), "{approved}");
    }

    /// A missing, empty or unreadable file fails closed.
    #[test]
    fn fails_closed_without_a_verdict() -> anyhow::Result<()> {
        let scratch = Scratch::new()?;
        let file = scratch.path().join("review.json");
        let error = |file: &Path| read(file).err();
        assert_eq!(
            error(&file).as_deref(),
            Some("missing, so there is no verdict, which fails closed")
        );
        for empty in ["", " \n\t"] {
            fs::write(&file, empty)?;
            assert_eq!(
                error(&file).as_deref(),
                Some("empty, so there is no verdict, which fails closed")
            );
        }
        fs::remove_file(&file)?;
        fs::create_dir(&file)?;
        let found = error(&file).unwrap_or_default();
        assert!(found.starts_with("couldn't read it: "), "{found}");
        Ok(())
    }

    /// Each schema error names what's wrong.
    fn schema_error(text: &str) -> String {
        match outcome(text) {
            Err(error) => error
                .strip_prefix("a schema error (P02 A.4): ")
                .unwrap_or(&error)
                .to_owned(),
            Ok(()) => "passed".to_owned(),
        }
    }

    #[test]
    fn refuses_what_the_schema_doesnt_allow() {
        for (text, expected) in [
            (
                approve_with("\"verdict\"", "\"extra\": 1, \"verdict\""),
                "unknown field `extra`",
            ),
            (
                approve_with("\"approve\"", "\"approved\""),
                "unknown variant `approved`",
            ),
            (
                approve_with("\"advisory\"", "\"minor\""),
                "unknown variant `minor`",
            ),
            (
                approve_with("\"docs\"", "\"style\""),
                "unknown variant `style`",
            ),
            (
                approve_with("\"adequate\": true", "\"adequate\": true, \"why\": \"\""),
                "unknown field `why`",
            ),
            (
                approve_with("\"evidence\": \"\"", "\"evidence\": \"\", \"fix\": \"\""),
                "unknown field `fix`",
            ),
            (
                approve_with("\"not_checked\": [],", ""),
                "missing field `not_checked`",
            ),
            (
                approve_with("\"line\": 0", "\"line\": -1"),
                "invalid value: integer `-1`",
            ),
            (
                approve_with("\"line\": 0", "\"line\": \"7\""),
                "invalid type: string \"7\"",
            ),
            (
                approve_with(
                    "\"verdict\": \"approve\"",
                    "\"verdict\": \"changes_requested\", \"verdict\": \"approve\"",
                ),
                "duplicate field `verdict`",
            ),
            (format!("{APPROVE}\n{{}}"), "trailing characters"),
            ("\"approve\"".to_owned(), "invalid type: string \"approve\""),
        ] {
            let found = schema_error(&text);
            assert!(found.starts_with(expected), "{expected}: {found}");
        }
    }

    /// Every category of the rubric is read.
    #[test]
    fn reads_every_category() {
        for category in [
            "intent",
            "correctness",
            "tests",
            "check-weakening",
            "api",
            "dependency",
            "docs",
            "assumption",
        ] {
            let text = approve_with("\"docs\"", &format!("\"{category}\""));
            assert_eq!(outcome(&text), Ok(()), "{category}");
        }
    }

    /// A blocking finding needs its file, a line from 1, its issue and its
    /// evidence; an advisory one may go without.
    #[test]
    fn refuses_a_blocking_finding_without_its_evidence() {
        let lacking =
            "finding 2 is blocking but lacks its file, its line, its issue or its evidence";
        for (from, to) in [
            ("\"src/lib.rs\"", "\" \""),
            ("42", "0"),
            ("\"An #[allow].\"", "\"\""),
            ("\"line 42\"", "\"\\n\""),
        ] {
            let incomplete = BLOCKING.replace(from, to);
            let text = with_findings(&[BLOCKING, &incomplete]);
            assert_eq!(schema_error(&text), lacking, "{incomplete}");
        }
        let one = BLOCKING.replace("42", "1");
        assert_eq!(
            schema_error(&with_findings(&[&one])),
            "approve, but with 1 blocking finding(s), which an approval can't have"
        );
        assert_eq!(
            schema_error(&approve_with(
                "Adds the parser.\\nDecided by default: the name `parse`.",
                " "
            )),
            "the summary is empty"
        );
    }

    #[test]
    fn summarizes_the_verdict() -> anyhow::Result<()> {
        let text = with_findings(&[BLOCKING, &BLOCKING.replace("blocking", "advisory")])
            .replace("\"not_checked\": []", "\"not_checked\": [\"the demo\"]")
            .replace(
                "\"adequate\": true }",
                "\"adequate\": true }, { \"id\": \"AC-2\", \"tests\": [], \"adequate\": false }",
            );
        let review: Review = serde_json::from_str(&text)?;
        assert_eq!(
            summary(&review),
            [
                "Verdict: approve.",
                "  > Adds the parser.",
                "  > Decided by default: the name `parse`.",
                "Acceptance criteria: AC-1 (1 test(s), adequate), AC-2 (0 test(s), inadequate).",
                "Findings: 1 blocking, 1 advisory.",
                "  - src/lib.rs:42 (check-weakening): An #[allow].",
                "Not checked:",
                "  - the demo",
                "To escalate: nothing.",
            ]
        );
        let review: Review = serde_json::from_str(
            &with_findings(&[])
                .replace("\"approve\"", "\"needs_human\"")
                .replace("\"assumptions_to_escalate\": []", "\"assumptions_to_escalate\": [\"A2\"]")
                .replace("{ \"id\": \"AC-1\", \"tests\": [\"tests/parse.rs::ac1_rejects_truncated_input\"], \"adequate\": true }", ""),
        )?;
        assert_eq!(
            summary(&review),
            [
                "Verdict: needs_human.",
                "  > Adds the parser.",
                "  > Decided by default: the name `parse`.",
                "Acceptance criteria: none.",
                "Findings: 0 blocking, 0 advisory.",
                "Not checked: nothing.",
                "To escalate:",
                "  - A2",
            ]
        );
        Ok(())
    }

    /// What the reviewer wrote reaches the log one line per line, behind a
    /// marker, so that none of it can pass for a workflow command.
    #[test]
    fn shows_what_the_reviewer_wrote_behind_a_marker() -> anyhow::Result<()> {
        let text = with_findings(&[&BLOCKING
            .replace("src/lib.rs", "a\\r::error::b")
            .replace("An #[allow].", "x\\u001b")])
        .replace("Adds the parser.", "::error::x\\ty")
        .replace(
            "\"not_checked\": []",
            "\"not_checked\": [\"\\n::warning::z\"]",
        );
        let review: Review = serde_json::from_str(&text)?;
        let lines = summary(&review);
        assert!(
            lines.contains(&"  > ::error::x\\ty".to_owned()),
            "{lines:?}"
        );
        assert!(
            lines.contains(&"  - a\\r::error::b:42 (check-weakening): x\\u{1b}".to_owned()),
            "{lines:?}"
        );
        assert!(
            lines.contains(&"  - \\n::warning::z".to_owned()),
            "{lines:?}"
        );
        assert!(
            lines
                .iter()
                .all(|line| !line.trim_start().starts_with("::")),
            "{lines:?}"
        );
        Ok(())
    }
}
