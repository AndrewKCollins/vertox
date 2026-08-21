use anyhow::{bail, Context, Result};
use include_dir::{include_dir, Dir};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

static BUILTIN_RULES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/rules/evm");

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Rule {
    pub id: String,
    pub title: String,
    pub severity: Severity,
    pub languages: Vec<String>,
    pub pattern: String,
    pub message: String,
    pub recommendation: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    pub fn rank(self) -> u8 {
        match self {
            Self::Info => 0,
            Self::Low => 1,
            Self::Medium => 2,
            Self::High => 3,
            Self::Critical => 4,
        }
    }

    pub fn parse(input: &str) -> Result<Self> {
        match input.to_ascii_lowercase().as_str() {
            "info" => Ok(Self::Info),
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            "critical" => Ok(Self::Critical),
            _ => bail!("invalid severity '{input}'; expected info, low, medium, high, or critical"),
        }
    }
}

impl Ord for Severity {
    fn cmp(&self, other: &Self) -> Ordering {
        self.rank().cmp(&other.rank())
    }
}

impl PartialOrd for Severity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::Info => "INFO",
            Self::Low => "LOW",
            Self::Medium => "MEDIUM",
            Self::High => "HIGH",
            Self::Critical => "CRITICAL",
        };
        write!(f, "{label}")
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Finding {
    pub rule_id: String,
    pub title: String,
    pub severity: Severity,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub snippet: String,
    pub message: String,
    pub recommendation: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ScanReport {
    pub files_scanned: usize,
    pub rules_loaded: usize,
    pub findings: Vec<Finding>,
}

pub fn scan_project(
    root: &Path,
    custom_rules: Option<&Path>,
    use_builtin: bool,
) -> Result<ScanReport> {
    if !root.exists() {
        bail!("scan target does not exist: {}", root.display());
    }

    let mut rules = Vec::new();
    if use_builtin {
        rules.extend(load_builtin_rules()?);
    }
    if let Some(dir) = custom_rules {
        rules.extend(load_rules_from_dir(dir)?);
    }
    if rules.is_empty() {
        bail!("no scan rules loaded");
    }

    let compiled = compile_rules(rules)?;
    let files = discover_source_files(root)?;
    let mut findings = Vec::new();

    for file in &files {
        let language = language_for_path(file).unwrap();
        let source = fs::read_to_string(file)
            .with_context(|| format!("failed to read {}", file.display()))?;

        for (rule, regex) in &compiled {
            if !rule
                .languages
                .iter()
                .any(|lang| lang.eq_ignore_ascii_case(language))
            {
                continue;
            }

            for hit in regex.find_iter(&source) {
                let (line, column) = line_column(&source, hit.start());
                findings.push(Finding {
                    rule_id: rule.id.clone(),
                    title: rule.title.clone(),
                    severity: rule.severity,
                    file: file.display().to_string(),
                    line,
                    column,
                    snippet: line_snippet(&source, line),
                    message: rule.message.clone(),
                    recommendation: rule.recommendation.clone(),
                });
            }
        }
    }

    findings.sort_by(|a, b| {
        b.severity
            .cmp(&a.severity)
            .then_with(|| a.file.cmp(&b.file))
            .then_with(|| a.line.cmp(&b.line))
            .then_with(|| a.rule_id.cmp(&b.rule_id))
    });

    Ok(ScanReport {
        files_scanned: files.len(),
        rules_loaded: compiled.len(),
        findings,
    })
}

pub fn should_fail(report: &ScanReport, threshold: Severity) -> bool {
    report
        .findings
        .iter()
        .any(|finding| finding.severity >= threshold)
}

fn load_builtin_rules() -> Result<Vec<Rule>> {
    let mut rules = Vec::new();
    for file in BUILTIN_RULES.files() {
        if file.path().extension().and_then(|ext| ext.to_str()) != Some("toml") {
            continue;
        }
        let source = file
            .contents_utf8()
            .ok_or_else(|| anyhow::anyhow!("bundled rule is not UTF-8: {}", file.path().display()))?;
        rules.push(parse_rule(source, &file.path().display().to_string())?);
    }
    rules.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(rules)
}

fn load_rules_from_dir(dir: &Path) -> Result<Vec<Rule>> {
    if !dir.is_dir() {
        bail!("custom rules path is not a directory: {}", dir.display());
    }
    let mut rules = Vec::new();
    for entry in WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
    {
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("toml") {
            continue;
        }
        let source = fs::read_to_string(entry.path())
            .with_context(|| format!("failed to read rule {}", entry.path().display()))?;
        rules.push(parse_rule(&source, &entry.path().display().to_string())?);
    }
    Ok(rules)
}

fn parse_rule(source: &str, origin: &str) -> Result<Rule> {
    let rule: Rule = toml::from_str(source).with_context(|| format!("invalid rule file {origin}"))?;
    if rule.id.trim().is_empty() || rule.title.trim().is_empty() {
        bail!("rule in {origin} must define non-empty id and title");
    }
    if rule.languages.is_empty() {
        bail!("rule {} in {origin} must define at least one language", rule.id);
    }
    Regex::new(&rule.pattern)
        .with_context(|| format!("rule {} contains invalid regex", rule.id))?;
    Ok(rule)
}

fn compile_rules(rules: Vec<Rule>) -> Result<Vec<(Rule, Regex)>> {
    rules
        .into_iter()
        .map(|rule| {
            let regex = Regex::new(&rule.pattern)
                .with_context(|| format!("invalid regex in rule {}", rule.id))?;
            Ok((rule, regex))
        })
        .collect()
}

fn discover_source_files(root: &Path) -> Result<Vec<PathBuf>> {
    if root.is_file() {
        if language_for_path(root).is_some() {
            return Ok(vec![root.to_path_buf()]);
        }
        bail!("unsupported source file: {}", root.display());
    }

    let mut files = Vec::new();
    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| !ignored(entry))
    {
        let entry = entry.with_context(|| format!("failed while walking {}", root.display()))?;
        if entry.file_type().is_file() && language_for_path(entry.path()).is_some() {
            files.push(entry.path().to_path_buf());
        }
    }
    files.sort();
    Ok(files)
}

fn ignored(entry: &DirEntry) -> bool {
    if entry.depth() == 0 {
        return false;
    }
    let name = entry.file_name().to_string_lossy();
    entry.file_type().is_dir()
        && matches!(
            name.as_ref(),
            ".git" | "node_modules" | "target" | "out" | "artifacts" | "cache" | "lib"
        )
}

fn language_for_path(path: &Path) -> Option<&'static str> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("sol") => Some("solidity"),
        Some("vy") => Some("vyper"),
        _ => None,
    }
}

fn line_column(source: &str, byte_offset: usize) -> (usize, usize) {
    let prefix = &source[..byte_offset.min(source.len())];
    let line = prefix.bytes().filter(|b| *b == b'\n').count() + 1;
    let column = prefix
        .rsplit_once('\n')
        .map(|(_, tail)| tail.chars().count() + 1)
        .unwrap_or_else(|| prefix.chars().count() + 1);
    (line, column)
}

fn line_snippet(source: &str, line: usize) -> String {
    source
        .lines()
        .nth(line.saturating_sub(1))
        .unwrap_or_default()
        .trim()
        .chars()
        .take(180)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn finds_tx_origin() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("Bad.sol");
        let mut out = fs::File::create(file).unwrap();
        writeln!(out, "contract Bad {{ function x() external {{ require(tx.origin == msg.sender); }} }}").unwrap();
        let report = scan_project(dir.path(), None, true).unwrap();
        assert!(report.findings.iter().any(|f| f.rule_id == "solidity-tx-origin-auth"));
    }
}
