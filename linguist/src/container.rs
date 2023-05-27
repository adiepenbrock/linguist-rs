use std::{collections::HashMap, ffi::OsString, path::Path};

use crate::resolver::{HeuristicRule, Language};

/// A `Container` can be used to implement a storage that holds [`Language`] and [`HeuristicRule`] definitions.
/// 
/// ## Features
/// When the `matcher` feature is enabled, the `Container` trait will also expose methods to retrieve [`HeuristicRule`] definitions.
pub trait Container {
    /// Returns a list of all [`Language`] definitions identified by its name.
    fn get_language_by_name(&self, name: &str) -> Option<&Language>;
    /// Returns a list of all [`Language`] definitions identified by the extension of the given file.
    fn get_languages_by_extension(&self, file: impl AsRef<Path>) -> Option<Vec<&Language>>;
    /// Returns a list of all [`Language`] definitions identified by the name of the given file.
    fn get_languages_by_filename(&self, file: impl AsRef<Path>) -> Option<Vec<&Language>>;
    /// Returns a list of all [`Language`] definitions identified by its interpreter.
    fn get_languages_by_interpreter(&self, interpreter: &str) -> Option<Vec<&Language>>;
    /// Returns a list of all [`HeuristicRule`] definitions identified by the extension of the given file.
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
        self.languages
            .iter()
            .find(|lang| lang.name.to_lowercase() == *name.to_lowercase())
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

    fn get_languages_by_interpreter(&self, interpreter: &str) -> Option<Vec<&Language>> {
        let interpreters: Vec<&Language> = self
            .languages
            .iter()
            .filter(|lang| lang.interpreters.contains(&interpreter.to_string()))
            .collect();

        if !interpreters.is_empty() {
            Some(interpreters)
        } else {
            None
        }
    }
}
