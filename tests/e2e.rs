use std::path::PathBuf;

use assert_cmd::Command;
use predicates::str::contains;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn cmd(fix: &str) -> Command {
    let mut c = Command::cargo_bin("sdd-spec-check").unwrap();
    c.current_dir(fixture(fix));
    c
}

#[test]
fn pass_is_exit_zero() {
    cmd("pass")
        .assert()
        .success()
        .stdout(contains("rules aligned"));
}

#[test]
fn missing_test_fails() {
    cmd("fail-missing-test")
        .assert()
        .failure()
        .stdout(contains("RULE-LOG-007"))
        .stdout(contains("no test for this rule"));
}

#[test]
fn text_mismatch_fails() {
    cmd("fail-text-mismatch")
        .assert()
        .failure()
        .stdout(contains("RULE-LOG-001"))
        .stdout(contains("text mismatch"));
}

#[test]
fn unknown_id_fails() {
    cmd("fail-unknown-id")
        .assert()
        .failure()
        .stdout(contains("RULE-LOG-042"))
        .stdout(contains("no current spec defines"));
}

#[test]
fn multi_lang_pass() {
    cmd("multi-lang-pass")
        .assert()
        .success()
        .stdout(contains("3 rules aligned"));
}

#[test]
fn custom_pattern_pass() {
    cmd("custom-pattern")
        .assert()
        .success()
        .stdout(contains("2 rules aligned"));
}

#[test]
fn frontmatter_status_only_reads_yaml_block() {
    // Body mentions `status: draft` in prose. Frontmatter says current.
    // Spec should still be in scope.
    cmd("frontmatter-body-mention")
        .assert()
        .success()
        .stdout(contains("1 rules aligned"));
}

#[test]
fn first_matching_spec_pattern_wins() {
    // Two patterns match the same spec line. If the second overwrote
    // the first, the captured spec text would not match the test text.
    cmd("spec-pattern-first-wins")
        .assert()
        .success()
        .stdout(contains("1 rules aligned"));
}

#[test]
fn output_is_deterministic() {
    let first = cmd("many-failures").output().unwrap().stdout;
    for _ in 0..4 {
        let next = cmd("many-failures").output().unwrap().stdout;
        assert_eq!(
            first, next,
            "output differs between runs (HashMap iteration leaked into stdout)"
        );
    }
}
