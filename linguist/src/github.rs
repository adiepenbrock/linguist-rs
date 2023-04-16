use crate::resolver::{HeuristicRule, Language, LinguistError};
use serde::Deserialize;
use std::collections::HashMap;
use std::ffi::OsString;

use std::fmt::Display;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct GhLanguageDef {
    color: Option<String>,
    #[serde(rename = "type")]
    scope: String,
    aliases: Option<Vec<String>>,
    extensions: Option<Vec<String>>,
    filenames: Option<Vec<String>>,
    group: Option<String>,
}

pub fn load_github_linguist_languages(
    path: impl AsRef<Path>,
) -> Result<Vec<Language>, LinguistError> {
    if !path.as_ref().exists() {
        return Err(LinguistError::FileNotFound);
    }

    let ldc = std::fs::read_to_string(path).expect("failed to read path");
    let defs: Result<HashMap<String, GhLanguageDef>, _> = serde_yaml::from_str(ldc.as_str());

    let mut languages: Vec<Language> = Vec::new();
    if let Ok(defs) = defs {
        for (name, lang) in defs.iter() {
            languages.push(Language {
                name: name.clone(),
                scope: lang.scope.clone().into(),
                extensions: lang
                    .extensions
                    .clone()
                    .unwrap_or_default()
                    .iter()
                    // because `Path.extension()` requires that an extension does not begin with `.`,
                    // we remove the first `.` from the extension
                    .map(|ext| OsString::from(ext.replacen('.', "", 1)))
                    .collect(),
                filenames: lang
                    .filenames
                    .clone()
                    .unwrap_or_default()
                    .iter()
                    .map(OsString::from)
                    .collect(),
                aliases: lang.aliases.clone().unwrap_or(vec![]),
                color: lang.color.clone(),
                parent: lang.group.clone(),
            });
        }
    }

    Ok(languages)
}

#[derive(Debug, Deserialize)]
struct Disambiguation {
    extensions: Vec<String>,
    rules: Vec<Rule>,
}

#[derive(Debug, Deserialize)]
struct Rule {
    language: RuleLanguage,
    #[serde(rename = "and")]
    and_rules: Option<Vec<NamedPattern>>,
    pattern: Option<PatternValue>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RuleLanguage {
    Single(String),
    Multiple(Vec<String>),
}

impl Display for RuleLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuleLanguage::Single(val) => write!(f, "{}", val),
            RuleLanguage::Multiple(val) => write!(f, "{}", val.join("|")),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum PatternValue {
    Single(String),
    Multiple(Vec<String>),
}

impl Display for PatternValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PatternValue::Single(val) => write!(f, "{}", val),
            PatternValue::Multiple(val) => write!(f, "{}", val.join("|")),
        }
    }
}

#[derive(Debug, Deserialize)]
struct NamedPattern {
    pattern: Option<String>,
    named_pattern: Option<PatternValue>,
}

#[derive(Debug, Deserialize)]
struct YamlContent {
    disambiguations: Vec<Disambiguation>,
    named_patterns: HashMap<String, RuleLanguage>,
}

#[cfg(feature = "matcher")]
pub fn load_github_linguist_heuristics(
    path: impl AsRef<Path>,
) -> Result<Vec<HeuristicRule>, LinguistError> {
    let content = std::fs::read_to_string(path).expect("unable to open heuristics file");
    let data = serde_yaml::from_str::<YamlContent>(&content);

    let mut rules: Vec<HeuristicRule> = Vec::new();
    if let Ok(data) = data {
        for disambiguation in data.disambiguations {
            for rule in disambiguation.rules {
                let lang = match rule.language {
                    RuleLanguage::Single(val) => val,
                    // TODO(multiple names): we should consider the case when more than
                    // one name is available to reference a certain rule as well...
                    _ => "".to_string(),
                };

                let mut heuristic_rule = HeuristicRule {
                    language: lang,
                    extensions: disambiguation
                        .extensions
                        .iter()
                        // because `Path.extension()` requires that an extension does not begin with `.`,
                        // we remove the first `.` from the extension
                        .map(|ext| OsString::from(ext.replacen('.', "", 1)))
                        .collect(),
                    patterns: vec![],
                };

                if let Some(pattern) = rule.pattern {
                    heuristic_rule.patterns.push(pattern.to_string());
                }

                if let Some(refs) = rule.and_rules {
                    for np_ref in refs {
                        if let Some(pattern) = np_ref.pattern {
                            heuristic_rule.patterns.push(pattern.to_string());
                        }

                        if let Some(pattern) = np_ref.named_pattern {
                            match pattern {
                                PatternValue::Single(val) => {
                                    if let Some(p_ref) = data.named_patterns.get(&val) {
                                        heuristic_rule.patterns.push(p_ref.to_string());
                                    }
                                }
                                PatternValue::Multiple(val) => {
                                    for val in val {
                                        if let Some(p_ref) = data.named_patterns.get(&val) {
                                            heuristic_rule.patterns.push(p_ref.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                rules.push(heuristic_rule);
            }
        }
    }

    Ok(rules)
}

pub fn load_github_vendors(path: impl AsRef<Path>) -> Result<Vec<String>, LinguistError> {
    let content = std::fs::read_to_string(path).expect("unable to open vendors file");
    let data = serde_yaml::from_str::<Vec<String>>(&content).unwrap();

    Ok(data)
}
