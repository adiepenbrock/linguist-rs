use std::collections::HashMap;
use std::ffi::OsString;
use std::fmt::Display;
use std::path::Path;
use std::usize;

#[cfg(feature = "matcher")]
use fancy_regex::Regex;

use crate::utils::is_binary;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Language {
    pub parent: Option<String>,
    pub name: String,
    pub aliases: Vec<String>,
    pub scope: String,
    pub extensions: Vec<OsString>,
    pub filenames: Vec<OsString>,
    pub color: Option<String>,
}

impl Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "matcher", derive(serde::Serialize, serde::Deserialize))]
pub struct HeuristicRule {
    pub language: String,
    pub extensions: Vec<OsString>,
    pub patterns: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinguistError {
    LanguageNotFound,
    #[cfg(feature = "serde")]
    FileNotFound,
}

pub trait Container {
    fn get_language_by_name(&self, name: &str) -> Option<&Language>;
    fn get_languages_by_extension(&self, file: impl AsRef<Path>) -> Option<Vec<&Language>>;
    fn get_languages_by_filename(&self, file: impl AsRef<Path>) -> Option<Vec<&Language>>;
    #[cfg(feature = "matcher")]
    fn get_heuristics_by_extension(&self, file: impl AsRef<Path>) -> Option<&Vec<HeuristicRule>>;
}

#[derive(Debug, Default)]
pub struct InMemoryLanguageContainer {
    languages: Vec<Language>,
    heuristics: HashMap<OsString, Vec<HeuristicRule>>,
}

impl InMemoryLanguageContainer {
    pub fn register_language(&mut self, lang: Language) {
        self.languages.push(lang);
    }

    #[cfg(feature = "matcher")]
    pub fn register_heuristic_rule(&mut self, ext: OsString, rule: HeuristicRule) {
        if let Some(heuristic) = self.heuristics.get_mut(&ext) {
            if !heuristic.contains(&rule) {
                heuristic.push(rule);
            } else {
            }
        } else {
            self.heuristics.insert(ext.to_os_string(), vec![rule]);
        }
    }
}

impl Container for InMemoryLanguageContainer {
    fn get_language_by_name(&self, name: &str) -> Option<&Language> {
        self.languages.iter().find(|lang| lang.name == *name)
    }

    fn get_languages_by_extension(&self, file: impl AsRef<Path>) -> Option<Vec<&Language>> {
        let ext = match file.as_ref().extension() {
            Some(ext) => ext,
            _ => match file.as_ref().file_name() {
                Some(name) => name,
                _ => return None,
            },
        };

        let candidates: Vec<&Language> = self
            .languages
            .iter()
            .filter(|lang| lang.extensions.contains(&OsString::from(ext)))
            .collect();

        if !candidates.is_empty() {
            Some(candidates)
        } else {
            None
        }
    }

    fn get_languages_by_filename(&self, file: impl AsRef<Path>) -> Option<Vec<&Language>> {
        let candidates: Vec<&Language> = self
            .languages
            .iter()
            .filter(|lang| {
                lang.filenames
                    .contains(&file.as_ref().as_os_str().to_os_string())
            })
            .collect();

        if !candidates.is_empty() {
            Some(candidates)
        } else {
            None
        }
    }

    #[cfg(feature = "matcher")]
    fn get_heuristics_by_extension(&self, file: impl AsRef<Path>) -> Option<&Vec<HeuristicRule>> {
        let ext = match file.as_ref().extension() {
            Some(val) => val,
            _ => return None,
        };

        let heuristics = self.heuristics.get(&ext.to_os_string());
        heuristics
    }
}

pub fn resolve_languages_by_filename(
    file: impl AsRef<Path>,
    container: &impl Container,
) -> Result<Vec<&Language>, LinguistError> {
    match container.get_languages_by_filename(file) {
        Some(langs) => Ok(langs),
        _ => Err(LinguistError::LanguageNotFound),
    }
}

pub fn resolve_languages_by_extension(
    file: impl AsRef<Path>,
    container: &impl Container,
) -> Result<Vec<&Language>, LinguistError> {
    match container.get_languages_by_extension(file) {
        Some(langs) => Ok(langs),
        _ => Err(LinguistError::LanguageNotFound),
    }
}

#[cfg(feature = "matcher")]
pub fn resolve_language_by_content(
    file: impl AsRef<Path>,
    container: &impl Container,
) -> Result<Option<&Language>, LinguistError> {
    let content = match std::fs::read_to_string(file.as_ref()) {
        Ok(content) => content,
        _ => return Err(LinguistError::FileNotFound),
    };

    if let Some(rules) = container.get_heuristics_by_extension(file.as_ref()) {
        for rule in rules {
            let matcher = if let Ok(regex) = Regex::new(&rule.patterns.join("|")) {
                regex
            } else {
                continue;
            };

            if matcher.is_match(&content).is_ok() {
                return Ok(container.get_language_by_name(&rule.language));
            }
        }
    }

    Err(LinguistError::LanguageNotFound)
}

pub fn resolve_language(
    file: impl AsRef<Path>,
    container: &impl Container,
) -> Result<Option<&Language>, LinguistError> {
    if is_binary(&file) {
        return Ok(None);
    }
    let mut probabilities: HashMap<String, usize> = HashMap::new();

    if let Ok(candidates) = resolve_languages_by_filename(&file, container) {
        for candidate in candidates {
            *probabilities.entry(candidate.name.clone()).or_insert(1) += 1;
        }
    }

    if let Ok(candidates) = resolve_languages_by_extension(&file, container) {
        for candidate in candidates {
            *probabilities.entry(candidate.name.clone()).or_insert(1) += 1;
        }
    }

    if let Ok(candidate) = resolve_language_by_content(&file, container) {
        if let Some(candidate) = candidate {
            *probabilities.entry(candidate.name.clone()).or_insert(1) += 1;
        }
    }

    let mut ordered: Vec<(&String, &usize)> = probabilities.iter().collect();
    ordered.sort_by_key(|&(_, v)| v);
    ordered.reverse();

    if !ordered.is_empty() {
        return Ok(Some(
            container
                .get_language_by_name(ordered.get(0).unwrap().0)
                .unwrap(),
        ));
    }
    Err(LinguistError::LanguageNotFound)
}
