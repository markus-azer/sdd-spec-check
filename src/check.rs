use std::path::PathBuf;

use crate::collect::Collected;
use crate::config::{Config, Severity};

#[derive(Debug)]
pub enum Issue {
    UnknownId {
        id: String,
        file: PathBuf,
        line: usize,
    },
    MissingTest {
        id: String,
        file: PathBuf,
        line: usize,
    },
    TextMismatch {
        id: String,
        spec: String,
        test: String,
        file: PathBuf,
        line: usize,
    },
    EmptySpecGlobs,
    EmptyTestGlobs,
}

#[derive(Debug)]
pub struct Finding {
    pub severity: Severity,
    pub issue: Issue,
}

pub struct Outcome {
    pub findings: Vec<Finding>,
    pub spec_count: usize,
    pub test_count: usize,
}

impl Outcome {
    pub fn ok(&self) -> bool {
        !self.findings.iter().any(|f| f.severity == Severity::Error)
    }
}

pub fn run(collected: &Collected, config: &Config) -> Outcome {
    let mut findings = Vec::new();

    // No files matched the globs at all.
    if collected.spec_files == 0 {
        findings.push(Finding {
            severity: config.on_empty_glob,
            issue: Issue::EmptySpecGlobs,
        });
    }
    if collected.test_files == 0 {
        findings.push(Finding {
            severity: config.on_empty_glob,
            issue: Issue::EmptyTestGlobs,
        });
    }

    // For each test ref: is the id known, and does the text match?
    for (id, refs) in &collected.tests {
        let spec = collected.specs.get(id);
        for test_ref in refs {
            match spec {
                None => findings.push(Finding {
                    severity: config.on_unknown_id,
                    issue: Issue::UnknownId {
                        id: id.clone(),
                        file: test_ref.file.clone(),
                        line: test_ref.line,
                    },
                }),
                Some(rule) if rule.text != test_ref.text => findings.push(Finding {
                    severity: config.on_text_mismatch,
                    issue: Issue::TextMismatch {
                        id: id.clone(),
                        spec: rule.text.clone(),
                        test: test_ref.text.clone(),
                        file: test_ref.file.clone(),
                        line: test_ref.line,
                    },
                }),
                Some(_) => {}
            }
        }
    }

    // For each current spec rule: is there a test for it?
    for (id, rule) in &collected.specs {
        if !collected.tests.contains_key(id) {
            findings.push(Finding {
                severity: config.on_missing_test,
                issue: Issue::MissingTest {
                    id: id.clone(),
                    file: rule.file.clone(),
                    line: rule.line,
                },
            });
        }
    }

    findings.sort_by_key(sort_key);

    Outcome {
        findings,
        spec_count: collected.specs.len(),
        test_count: collected.tests.len(),
    }
}

// Sort order for printing. Empty-glob warnings come first.
// Other findings sorted by file, line, id.
fn sort_key(finding: &Finding) -> (u8, PathBuf, usize, String) {
    match &finding.issue {
        Issue::EmptySpecGlobs => (0, PathBuf::new(), 0, "specs".into()),
        Issue::EmptyTestGlobs => (0, PathBuf::new(), 0, "tests".into()),
        Issue::UnknownId { id, file, line }
        | Issue::MissingTest { id, file, line }
        | Issue::TextMismatch { id, file, line, .. } => (1, file.clone(), *line, id.clone()),
    }
}
