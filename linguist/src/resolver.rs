use std::collections::HashMap;
use std::ffi::OsString;
use std::fmt::Display;
use std::io::{BufRead, BufReader};
use std::path::Path;

use std::usize;

#[cfg(feature = "matcher")]
use fancy_regex::Regex;

use crate::utils::{determine_multiline_exec, has_shebang, is_binary};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Language {
    pub parent: Option<String>,
    pub name: String,
    pub aliases: Vec<String>,
    pub scope: Scope,
    pub extensions: Vec<OsString>,
    pub filenames: Vec<OsString>,
    pub interpreters: Vec<String>,
    pub color: Option<String>,
}

impl Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Scope {
    Programming,
    Markup,
    Data,
    Prose,
    Unknown,
}

impl From<String> for Scope {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "programming" => Scope::Programming,
            "markup" => Scope::Markup,
            "data" => Scope::Data,
            "prose" => Scope::Prose,
            _ => Scope::Unknown,
        }
    }
}

impl From<&str> for Scope {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "programming" => Scope::Programming,
            "markup" => Scope::Markup,
            "data" => Scope::Data,
            "prose" => Scope::Prose,
            _ => Scope::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "matcher", derive(serde::Serialize, serde::Deserialize))]
pub struct HeuristicRule {
    pub language: String,
    pub extensions: Vec<OsString>,
    pub patterns: Vec<String>,
}

#[derive(Debug)]
pub enum LinguistError {
    LanguageNotFound,
    #[cfg(feature = "serde")]
    FileNotFound,
    PatternCompileError(fancy_regex::Error),
    IOError(std::io::Error),
}

impl From<std::io::Error> for LinguistError {
    fn from(value: std::io::Error) -> Self {
        LinguistError::IOError(value)
    }
}

impl From<fancy_regex::Error> for LinguistError {
    fn from(value: fancy_regex::Error) -> Self {
        LinguistError::PatternCompileError(value)
    }
}

pub trait Container {
    fn get_language_by_name(&self, name: &str) -> Option<&Language>;
    fn get_languages_by_extension(&self, file: impl AsRef<Path>) -> Option<Vec<&Language>>;
    fn get_languages_by_filename(&self, file: impl AsRef<Path>) -> Option<Vec<&Language>>;
    fn get_languages_by_interpreter(&self, interpreter: &str) -> Option<Vec<&Language>>;
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
            let matcher = Regex::new(&rule.patterns.join("|"))?;

            if let Ok(result) = matcher.is_match(&content) {
                if result {
                    return Ok(container.get_language_by_name(&rule.language));
                }
            }
        }
    }

    Err(LinguistError::LanguageNotFound)
}

pub fn resolve_languages_by_shebang(
    file: impl AsRef<Path>,
    container: &impl Container,
) -> Result<Option<Vec<&Language>>, LinguistError> {
    // load first line of file
    let file = match std::fs::File::open(&file) {
        Ok(file) => file,
        Err(err) => return Err(LinguistError::IOError(err)),
    };
    let mut buf = BufReader::new(file);
    let mut line = String::new();
    let _ = buf.read_line(&mut line);

    // check whether the first line of the file is a shebang
    if !has_shebang(line.as_bytes()) {
        return Ok(None);
    }

    let line = line[2..].trim();
    let mut fields = line.split_whitespace().collect::<Vec<&str>>();
    if fields.is_empty() {
        return Ok(None);
    }

    let mut interpreter = Path::new(fields[0])
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();

    if interpreter == "env" {
        if fields.len() == 1 {
            return Ok(None);
        }

        let env_opt_args = Regex::new(r"^-[a-zA-Z]+$").unwrap();
        let env_var_args = Regex::new(r"^\$[a-zA-Z_]+$").unwrap();

        let _i = 1;
        while fields.len() > 2 {
            if env_opt_args.is_match(fields[1])? || env_var_args.is_match(fields[1])? {
                fields.remove(1);
                continue;
            }
            break;
        }
        interpreter = Path::new(fields[1])
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
    }

    let mut interpreter = interpreter;
    if interpreter == "sh" {
        interpreter = determine_multiline_exec(buf.buffer()).unwrap();
    }

    let python_version = Regex::new(r"^python[0-9]*\.[0-9]*").unwrap();
    if python_version.is_match(&interpreter)? {
        interpreter = interpreter.split('.').next().unwrap().to_owned();
    }
    // If osascript is called with argument -l it could be different language so do not rely on it
    // To match linguist behavior, see ref https://github.com/github/linguist/blob/d95bae794576ab0ef2fcb41a39eb61ea5302c5b5/lib/linguist/shebang.rb#L63
    if interpreter == "osascript" && line.contains("-l") {
        interpreter = "".to_string();
    }

    let results = container.get_languages_by_interpreter(&interpreter);
    if results.is_some() {
        Ok(results)
    } else {
        Ok(None)
    }
}

pub fn resolve_language(
    file: impl AsRef<Path>,
    container: &impl Container,
) -> Result<Option<&Language>, LinguistError> {
    if is_binary(&file)? {
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
