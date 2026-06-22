use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use clap::ValueEnum;
use regex::Regex;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Silent,
    Warn,
    Error,
}

const DEFAULT_NAMES: &[&str] = &["sdd-spec-check.config.toml", ".sdd-spec-check.config.toml"];

const DEFAULT_SPEC_PATTERNS: &[&str] = &[r"^-\s+\[(?<id>RULE-[A-Z0-9-]+)\]\s+(?<text>.+)$"];

const DEFAULT_TEST_PATTERNS: &[&str] = &[
    r#""(?<id>RULE-[A-Z0-9-]+):\s+(?<text>[^"]*?)""#,
    r"'(?<id>RULE-[A-Z0-9-]+):\s+(?<text>[^']*?)'",
    r"`(?<id>RULE-[A-Z0-9-]+):\s+(?<text>[^`]*?)`",
];

#[derive(Debug, Deserialize, Default)]
struct Raw {
    #[serde(default)]
    specs: Vec<String>,
    #[serde(default)]
    tests: Vec<String>,
    #[serde(default)]
    spec_patterns: Vec<String>,
    #[serde(default)]
    test_patterns: Vec<String>,
    #[serde(default)]
    on_empty_glob: Option<Severity>,
    #[serde(default)]
    on_unknown_id: Option<Severity>,
    #[serde(default)]
    on_missing_test: Option<Severity>,
    #[serde(default)]
    on_text_mismatch: Option<Severity>,
}

#[derive(Debug)]
pub struct Config {
    pub root: PathBuf,
    pub spec_globs: Vec<String>,
    pub test_globs: Vec<String>,
    pub spec_patterns: Vec<Regex>,
    pub test_patterns: Vec<Regex>,
    pub on_empty_glob: Severity,
    pub on_unknown_id: Severity,
    pub on_missing_test: Severity,
    pub on_text_mismatch: Severity,
}

impl Config {
    pub fn load(
        path: Option<&Path>,
        cli_specs: &[String],
        cli_tests: &[String],
        cli_on_empty_glob: Option<Severity>,
    ) -> Result<Self> {
        let (root, raw) = match path {
            Some(p) => load_file(p)?,
            None => discover()?,
        };

        let spec_globs = use_cli_or_file(cli_specs, &raw.specs, "specs")?;
        let test_globs = use_cli_or_file(cli_tests, &raw.tests, "tests")?;
        let spec_patterns = compile_patterns(&raw.spec_patterns, DEFAULT_SPEC_PATTERNS)?;
        let test_patterns = compile_patterns(&raw.test_patterns, DEFAULT_TEST_PATTERNS)?;

        Ok(Self {
            root,
            spec_globs,
            test_globs,
            spec_patterns,
            test_patterns,
            on_empty_glob: cli_on_empty_glob
                .or(raw.on_empty_glob)
                .unwrap_or(Severity::Warn),
            on_unknown_id: raw.on_unknown_id.unwrap_or(Severity::Error),
            on_missing_test: raw.on_missing_test.unwrap_or(Severity::Error),
            on_text_mismatch: raw.on_text_mismatch.unwrap_or(Severity::Error),
        })
    }
}

fn use_cli_or_file(cli: &[String], file: &[String], field: &str) -> Result<Vec<String>> {
    if !cli.is_empty() {
        return Ok(cli.to_vec());
    }
    if !file.is_empty() {
        return Ok(file.to_vec());
    }
    Err(anyhow!(
        "no `{field}` globs (set in config or pass --{field})"
    ))
}

fn compile_patterns(user: &[String], defaults: &[&str]) -> Result<Vec<Regex>> {
    let mut out = Vec::new();
    if user.is_empty() {
        for source in defaults {
            out.push(compile_pattern(source)?);
        }
    } else {
        for source in user {
            out.push(compile_pattern(source)?);
        }
    }
    Ok(out)
}

fn compile_pattern(source: &str) -> Result<Regex> {
    let regex = Regex::new(source).with_context(|| format!("invalid pattern `{source}`"))?;
    let names: Vec<_> = regex.capture_names().flatten().collect();
    if !names.contains(&"id") || !names.contains(&"text") {
        return Err(anyhow!(
            "pattern `{source}` must capture both `id` and `text`"
        ));
    }
    Ok(regex)
}

fn load_file(path: &Path) -> Result<(PathBuf, Raw)> {
    let content =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let raw: Raw =
        toml::from_str(&content).with_context(|| format!("parsing {}", path.display()))?;
    let root = path
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    Ok((root, raw))
}

fn discover() -> Result<(PathBuf, Raw)> {
    let mut dir = std::env::current_dir()?;
    loop {
        for name in DEFAULT_NAMES {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return load_file(&candidate);
            }
        }
        if !dir.pop() {
            return Err(anyhow!(
                "no sdd-spec-check.config.toml found. use --config <path> or create one"
            ));
        }
    }
}
