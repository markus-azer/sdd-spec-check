use owo_colors::OwoColorize;

use crate::check::{Issue, Outcome};
use crate::config::Severity;

pub fn print(outcome: &Outcome) {
    let errors = count(outcome, Severity::Error);
    let warns = count(outcome, Severity::Warn);

    for finding in &outcome.findings {
        let mark = match finding.severity {
            Severity::Error => "✗".red().to_string(),
            Severity::Warn => "!".yellow().to_string(),
            Severity::Silent => continue,
        };
        print_issue(&mark, &finding.issue);
    }

    if errors == 0 {
        println!(
            "{} {} rules aligned across {} tests",
            "✓".green(),
            outcome.spec_count,
            outcome.test_count
        );
        if warns > 0 {
            println!("  {warns} warning(s)");
        }
    } else {
        println!(
            "\n{errors} failure(s) across {} rules ({warns} warning(s))",
            outcome.spec_count
        );
    }
}

fn count(outcome: &Outcome, severity: Severity) -> usize {
    outcome
        .findings
        .iter()
        .filter(|f| f.severity == severity)
        .count()
}

fn print_issue(mark: &str, issue: &Issue) {
    match issue {
        Issue::UnknownId { id, file, line } => {
            println!("{mark} {id}  no current spec defines this id");
            println!("    at:   {}:{line}", file.display());
        }
        Issue::MissingTest { id, file, line } => {
            println!("{mark} {id}  no test for this rule");
            println!("    spec: {}:{line}", file.display());
        }
        Issue::TextMismatch {
            id,
            spec,
            test,
            file,
            line,
        } => {
            println!("{mark} {id}  text mismatch");
            println!("    spec: {spec:?}");
            println!("    test: {test:?}");
            println!("    at:   {}:{line}", file.display());
        }
        Issue::EmptySpecGlobs => println!("{mark} spec globs matched no files"),
        Issue::EmptyTestGlobs => println!("{mark} test globs matched no files"),
    }
}
