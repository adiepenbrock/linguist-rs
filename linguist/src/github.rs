use crate::error::LinguistError;
use crate::resolver::{HeuristicRule, Language, Scope};
use crate::serde::deserialize_languages;
use crate::utils::is_unsupported_regex_syntax;
use std::collections::HashMap;
use std::ffi::OsString;

use std::fmt::Display;
use std::path::Path;

#[derive(Debug, serde::Deserialize)]
pub struct GhLanguageDef {
    pub color: Option<String>,
    #[serde(skip)]
    pub name: String,
    #[serde(rename = "type")]
    pub scope: String,
    pub aliases: Option<Vec<String>>,
    pub extensions: Option<Vec<String>>,
    pub filenames: Option<Vec<String>>,
    pub interpreters: Option<Vec<String>>,
    pub group: Option<String>,
}

impl TryInto<Language> for GhLanguageDef {
    type Error = LinguistError;

    fn try_into(self) -> Result<Language, Self::Error> {
        Ok(Language {
            aliases: self.aliases.unwrap_or_default(),
            color: self.color.clone(),
            name: self.name.clone(),
            scope: Scope::from(self.scope),
            parent: self.group.clone(),
            filenames: self
                .filenames
                .unwrap_or_default()
                .iter()
                .map(OsString::from)
                .collect(),
            extensions: self
                .extensions
                .unwrap_or_default()
                .iter()
                .map(|ext| OsString::from(ext.replacen('.', "", 1)))
                .collect(),
            interpreters: self.interpreters.unwrap_or_default(),
        })
    }
}

pub fn load_github_linguist_languages(
    path: impl AsRef<Path>,
) -> Result<Vec<Language>, LinguistError> {
    if !path.as_ref().exists() {
        return Err(LinguistError::FileNotFound);
    }

    let languages = deserialize_languages::<GhLanguageDef>(path)?;
    Ok(languages)
}

#[derive(Debug, serde::Deserialize)]
struct Disambiguation {
    extensions: Vec<String>,
    rules: Vec<Rule>,
}

#[derive(Debug, serde::Deserialize)]
struct Rule {
    language: RuleLanguage,
    #[serde(rename = "and")]
    and_rules: Option<Vec<NamedPattern>>,
    pattern: Option<PatternValue>,
}

#[derive(Debug, serde::Deserialize)]
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

#[derive(Debug, serde::Deserialize)]
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

#[derive(Debug, serde::Deserialize)]
struct NamedPattern {
    pattern: Option<String>,
    named_pattern: Option<PatternValue>,
}

#[derive(Debug, serde::Deserialize)]
struct YamlContent {
    disambiguations: Vec<Disambiguation>,
    named_patterns: HashMap<String, RuleLanguage>,
}

#[cfg(feature = "matcher")]
pub fn load_github_linguist_heuristics(
    path: impl AsRef<Path>,
) -> Result<Vec<HeuristicRule>, LinguistError> {
    let content = std::fs::read_to_string(path)?;
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
    let content = std::fs::read_to_string(path)?;
    let raw = serde_yaml::from_str::<Vec<String>>(&content).unwrap();

    let mut data: Vec<String> = Vec::new();
    for rule in raw {
        if !is_unsupported_regex_syntax(rule.as_str()) {
            data.push(rule.to_string());
        }
    }

    Ok(data)
}

pub fn load_github_documentation(path: impl AsRef<Path>) -> Result<Vec<String>, LinguistError> {
    let content = std::fs::read_to_string(path)?;
    let raw = serde_yaml::from_str::<Vec<String>>(&content).unwrap();

    let mut data: Vec<String> = Vec::new();
    for rule in raw {
        if !is_unsupported_regex_syntax(rule.as_str()) {
            data.push(rule.to_string());
        }
    }

    Ok(data)
}
