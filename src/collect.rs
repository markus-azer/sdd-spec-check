use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;
use regex::Regex;

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct SpecRule {
    pub text: String,
    pub file: PathBuf,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct TestRef {
    pub text: String,
    pub file: PathBuf,
    pub line: usize,
}

pub struct Collected {
    pub specs: HashMap<String, SpecRule>,
    pub spec_files: usize,
    pub tests: HashMap<String, Vec<TestRef>>,
    pub test_files: usize,
}

pub fn collect(config: &Config) -> Result<Collected> {
    let spec_files = walk(&config.root, &config.spec_globs)?;
    let test_files = walk(&config.root, &config.test_globs)?;

    let specs = read_specs(&spec_files, &config.root, &config.spec_patterns)?;
    let tests = read_tests(&test_files, &config.root, &config.test_patterns)?;

    Ok(Collected {
        specs,
        spec_files: spec_files.len(),
        tests,
        test_files: test_files.len(),
    })
}

fn read_specs(
    files: &[PathBuf],
    root: &Path,
    patterns: &[Regex],
) -> Result<HashMap<String, SpecRule>> {
    let mut specs = HashMap::new();
    for path in files {
        let content = read_file(path)?;
        if !is_current(&content) {
            continue;
        }
        for (i, line) in content.lines().enumerate() {
            for pattern in patterns {
                let Some(captures) = pattern.captures(line) else {
                    continue;
                };
                let id = captures["id"].to_string();
                let text = normalize(&captures["text"]);
                specs.insert(
                    id,
                    SpecRule {
                        text,
                        file: relative_to(root, path),
                        line: i + 1,
                    },
                );
                break;
            }
        }
    }
    Ok(specs)
}

fn read_tests(
    files: &[PathBuf],
    root: &Path,
    patterns: &[Regex],
) -> Result<HashMap<String, Vec<TestRef>>> {
    let mut tests: HashMap<String, Vec<TestRef>> = HashMap::new();
    for path in files {
        let content = read_file(path)?;
        for (i, line) in content.lines().enumerate() {
            for pattern in patterns {
                for captures in pattern.captures_iter(line) {
                    let id = captures["id"].to_string();
                    let text = normalize(&captures["text"]);
                    tests.entry(id).or_default().push(TestRef {
                        text,
                        file: relative_to(root, path),
                        line: i + 1,
                    });
                }
            }
        }
    }
    dedupe_refs(&mut tests);
    Ok(tests)
}

// User patterns can overlap and match the same test ref twice. Drop the dupes.
fn dedupe_refs(tests: &mut HashMap<String, Vec<TestRef>>) {
    for refs in tests.values_mut() {
        refs.sort_by(|a, b| (&a.file, a.line, &a.text).cmp(&(&b.file, b.line, &b.text)));
        refs.dedup_by(|a, b| a.file == b.file && a.line == b.line && a.text == b.text);
    }
}

fn walk(root: &Path, patterns: &[String]) -> Result<Vec<PathBuf>> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern).with_context(|| format!("invalid glob `{pattern}`"))?);
    }
    let globs: GlobSet = builder.build()?;

    let mut matches = Vec::new();
    for entry in WalkBuilder::new(root).build() {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if globs.is_match(path.strip_prefix(root).unwrap_or(path)) {
            matches.push(path.to_path_buf());
        }
    }
    Ok(matches)
}

fn read_file(path: &Path) -> Result<String> {
    std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))
}

fn relative_to(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root)
        .map(PathBuf::from)
        .unwrap_or_else(|_| path.to_path_buf())
}

// A spec is current unless the frontmatter says otherwise.
// Only reads the YAML block between the first two `---` lines.
fn is_current(content: &str) -> bool {
    let Some(frontmatter) = leading_frontmatter(content) else {
        return true;
    };
    for line in frontmatter.lines() {
        if let Some(value) = line.strip_prefix("status:") {
            return value.trim() == "current";
        }
    }
    true
}

fn leading_frontmatter(content: &str) -> Option<&str> {
    let after_open = content.strip_prefix("---\n")?;
    let end = after_open.find("\n---")?;
    Some(&after_open[..end])
}

// Strip whitespace and a trailing period.
// So `writes JSON.` in a spec matches `writes JSON` in a test.
fn normalize(text: &str) -> String {
    let trimmed = text.trim();
    trimmed.strip_suffix('.').unwrap_or(trimmed).to_string()
}
