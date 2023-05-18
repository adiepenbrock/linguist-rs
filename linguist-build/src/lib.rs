use std::{io::Write, path::PathBuf};

use linguist::{
    github::{
        load_github_linguist_heuristics, load_github_linguist_languages, load_github_vendors,
    },
    resolver::{HeuristicRule, Language},
};

pub static GITHUB_LINGUIST_LANGUAGES_URL: &'static str =
    "https://raw.githubusercontent.com/github-linguist/linguist/master/lib/linguist/languages.yml";
pub static GITHUB_LINGUIST_HEURISTICS_URL: &'static str =
    "https://raw.githubusercontent.com/github-linguist/linguist/master/lib/linguist/heuristics.yml";
pub static GITHUB_LINGUIST_VENDOR_URL: &'static str =
    "https://raw.githubusercontent.com/github-linguist/linguist/master/lib/linguist/vendor.yml";

#[derive(Clone, PartialEq, Eq)]
pub struct Config {
    // out_path: PathBuf,
    definitions: Vec<DefinitionSet>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            // out_path: PathBuf::from(std::env::var_os("OUT_DIR")),
            definitions: vec![],
        }
    }
}

impl Config {
    /// Creates a new `Config` with default options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a [definition](DefinitionSet) to the `Config`.
    pub fn add_definition(&mut self, definition: DefinitionSet) -> &mut Self {
        self.definitions.push(definition);
        self
    }

    pub fn generate(&self) {
        let tempdir = tempfile::tempdir().expect("failed to create tempdir");

        for def in self.definitions.iter() {
            if let Some(path) = &def.language_url {
                match reqwest::blocking::get(path) {
                    Ok(result) => {
                        let path = tempdir.path().join(format!("{}_langs.yml", &def.out_name));
                        let mut out_file =
                            std::fs::File::create(path.clone()).expect("cannot create out_file");
                        std::io::copy(
                            &mut result
                                .text()
                                .expect("cannot unwrap definition response")
                                .as_bytes(),
                            &mut out_file,
                        )
                        .expect("cannot copy temp definition file");

                        let languages = load_github_linguist_languages(path.clone()).unwrap();

                        let mut lang_defs: Vec<String> = Vec::new();
                        languages.iter().for_each(|lang| {
                            lang_defs.push(write_language_definition(lang));
                        });

                        let target_path = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
                        let mut target_file = std::fs::File::create(
                            target_path.join(format!("{}_languages.rs", &def.out_name)),
                        )
                        .unwrap();

                        // TODO: maybe it would be better if we have the Languages as static values instead of a function...
                        _ = target_file.write_all(format!("use std::ffi::OsString;\nuse linguist::resolver::{{Language, Scope}};\n\npub fn github_languages() -> Vec<Language> {{\n let langs: Vec<Language> = vec![").as_bytes());
                        for str in lang_defs {
                            _ = target_file.write_all(format!("    {},\n", str).as_bytes());
                        }
                        _ = target_file.write_all(format!("];\nlangs\n}}\n").as_bytes());
                        _ = target_file.flush();
                    }
                    Err(err) => eprintln!("{:?}", err),
                }
            }
            if let Some(path) = &def.heuristics_url {
                match reqwest::blocking::get(path) {
                    Ok(result) => {
                        let path = tempdir
                            .path()
                            .join(format!("{}_heuristics.yml", &def.out_name));
                        let mut out_file =
                            std::fs::File::create(path.clone()).expect("cannot create out_file");
                        std::io::copy(
                            &mut result
                                .text()
                                .expect("cannot unwrap heuristics response")
                                .as_bytes(),
                            &mut out_file,
                        )
                        .expect("cannot copy temp heuristics file");

                        let raw = load_github_linguist_heuristics(path.clone()).unwrap();

                        let mut heuristics: Vec<String> = Vec::new();
                        raw.iter().for_each(|heuristic| {
                            heuristics.push(write_heuristic_definition(heuristic));
                        });

                        let target_path = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
                        let mut target_file = std::fs::File::create(
                            target_path.join(format!("{}_heuristics.rs", &def.out_name)),
                        )
                        .unwrap();

                        // TODO: maybe it would be better if we have the Languages as static values instead of a function...
                        _ = target_file.write_all(format!("use linguist::resolver::HeuristicRule;\n\npub fn github_heuristics() -> Vec<HeuristicRule> {{\n let langs: Vec<HeuristicRule> = vec![").as_bytes());
                        for str in heuristics {
                            _ = target_file.write_all(format!("    {},\n", str).as_bytes());
                        }
                        _ = target_file.write_all(format!("];\nlangs\n}}\n").as_bytes());
                        _ = target_file.flush();
                    }
                    Err(err) => eprintln!("{:?}", err),
                }
            }
            if let Some(path) = &def.vendor_url {
                match reqwest::blocking::get(path) {
                    Ok(result) => {
                        let path = tempdir
                            .path()
                            .join(format!("{}_vendors.yml", &def.out_name));
                        let mut out_file =
                            std::fs::File::create(path.clone()).expect("cannot create out_file");
                        std::io::copy(
                            &mut result
                                .text()
                                .expect("cannot unwrap vendors response")
                                .as_bytes(),
                            &mut out_file,
                        )
                        .expect("cannot copy temp vendors file");

                        let vendors = load_github_vendors(path.clone()).unwrap();

                        let target_path = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
                        let mut target_file = std::fs::File::create(
                            target_path.join(format!("{}_vendors.rs", &def.out_name)),
                        )
                        .unwrap();

                        // TODO: maybe it would be better if we have the Languages as static values instead of a function...
                        _ = target_file.write_all(format!("pub fn github_vendors() -> Vec<String> {{\n let vendors: Vec<String> = vec![").as_bytes());
                        for str in vendors {
                            _ = target_file.write_all(
                                format!("    \"{}\".to_string(),\n", str.replace("\\", "\\\\"))
                                    .as_bytes(),
                            );
                        }
                        _ = target_file.write_all(format!("];\nvendors\n}}\n").as_bytes());
                        _ = target_file.flush();
                    }
                    Err(err) => eprintln!("{:?}", err),
                }
            }
        }
    }
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("definitions", &self.definitions)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionSet {
    pub language_url: Option<String>,
    pub heuristics_url: Option<String>,
    pub vendor_url: Option<String>,
    pub out_name: String,
}

impl Default for DefinitionSet {
    fn default() -> Self {
        DefinitionSet {
            language_url: None,
            heuristics_url: None,
            vendor_url: None,
            out_name: "linguist".to_string(),
        }
    }
}

fn write_language_definition(lang: &Language) -> String {
    let mut str = String::new();
    str.push_str("Language {");

    if let Some(parent) = &lang.parent {
        str.push_str(format!("parent: Some(\"{}\".to_string()), ", parent).as_str());
    } else {
        str.push_str("parent: None, ");
    }

    str.push_str(format!("name: \"{}\".to_string(), ", &lang.name).as_str());

    if !lang.aliases.is_empty() {
        str.push_str(
            format!(
                "aliases: vec![{}], ",
                &lang
                    .aliases
                    .iter()
                    .map(|s| format!("\"{}\".to_string()", s))
                    .collect::<Vec<String>>()
                    .join(", ")
            )
            .as_str(),
        );
    } else {
        str.push_str("aliases: vec![], ");
    }

    str.push_str(format!("scope: Scope::{}, ", &lang.scope.to_string()).as_str());

    if !lang.extensions.is_empty() {
        str.push_str(
            format!(
                "extensions: vec![{}], ",
                &lang
                    .extensions
                    .iter()
                    .map(|s| format!(
                        "OsString::from(\"{}\")",
                        s.to_str().expect("cannot unwrap extension")
                    ))
                    .collect::<Vec<String>>()
                    .join(", ")
            )
            .as_str(),
        );
    } else {
        str.push_str("extensions: vec![], ");
    }

    if !lang.filenames.is_empty() {
        str.push_str(
            format!(
                "filenames: vec![{}], ",
                &lang
                    .filenames
                    .iter()
                    .map(|s| format!(
                        "OsString::from(\"{}\")",
                        s.to_str().expect("cannot unwrap filename")
                    ))
                    .collect::<Vec<String>>()
                    .join(", ")
            )
            .as_str(),
        );
    } else {
        str.push_str("filenames: vec![], ");
    }

    if !lang.interpreters.is_empty() {
        str.push_str(
            format!(
                "interpreters: vec![{}], ",
                &lang
                    .interpreters
                    .iter()
                    .map(|s| format!("\"{}\".to_string()", s))
                    .collect::<Vec<String>>()
                    .join(", ")
            )
            .as_str(),
        );
    } else {
        str.push_str("interpreters: vec![], ");
    }

    if let Some(color) = &lang.color {
        str.push_str(format!("color: Some(\"{}\".to_string()) ", color).as_str());
    } else {
        str.push_str("color: None ");
    }

    str.push_str("}");
    str
}

fn write_heuristic_definition(rule: &HeuristicRule) -> String {
    let mut str = String::new();
    str.push_str("HeuristicRule {");

    str.push_str(format!("language: \"{}\".to_string(), ", &rule.language).as_str());

    if !rule.extensions.is_empty() {
        str.push_str(
            format!(
                "extensions: vec![{}], ",
                &rule
                    .extensions
                    .iter()
                    .map(|s| format!(
                        "OsString::from(\"{}\")",
                        s.to_str().expect("cannot unwrap extension")
                    ))
                    .collect::<Vec<String>>()
                    .join(", ")
            )
            .as_str(),
        );
    } else {
        str.push_str("extensions: vec![], ");
    }

    if !rule.patterns.is_empty() {
        str.push_str(
            format!(
                "patterns: vec![{}], ",
                &rule
                    .patterns
                    .iter()
                    .map(|s| format!(
                        "\"{}\".to_string()",
                        s.replace("\\", "\\\\").replace("\"", "\\\"")
                    ))
                    .collect::<Vec<String>>()
                    .join(", ")
            )
            .as_str(),
        );
    } else {
        str.push_str("patterns: vec![] ");
    }

    str.push_str("}");
    str
}
