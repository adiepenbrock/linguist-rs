use std::{
    io::Write,
    path::{Path, PathBuf},
};

use linguist::{
    github::{
        load_github_documentation, load_github_linguist_heuristics, load_github_linguist_languages,
        load_github_vendors,
    },
    resolver::{HeuristicRule, Language},
};
use tempfile::tempdir;

pub static GITHUB_LINGUIST_LANGUAGES_URL: &str =
    "https://raw.githubusercontent.com/github-linguist/linguist/master/lib/linguist/languages.yml";
pub static GITHUB_LINGUIST_HEURISTICS_URL: &str =
    "https://raw.githubusercontent.com/github-linguist/linguist/master/lib/linguist/heuristics.yml";
pub static GITHUB_LINGUIST_VENDORS_URL: &str =
    "https://raw.githubusercontent.com/github-linguist/linguist/master/lib/linguist/vendor.yml";
pub static GITHUB_LINGUIST_DOCUMENTATION_URL: &str =
    "https://raw.githubusercontent.com/github-linguist/linguist/master/lib/linguist/documentation.yml";

/// The `Config` is used to configure the build process. It can be used to specify the `output path` and
/// the `definitions` to be generated.
#[derive(Clone, PartialEq, Eq)]
pub struct Config {
    /// The `out_path` is used to specify the path where the generated files will be written to.
    out_path: PathBuf,
    /// The `definitions` are used to specify which definitions should be generated.
    definitions: Vec<Definition>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            out_path: PathBuf::from(std::env::var_os("OUT_DIR").unwrap()),
            definitions: vec![],
        }
    }
}

/// A `Definition` is used to specify the `name`, [`Location`], and the [`Kind`] of an artifact
/// to generate. The `Location` can either be a `URL` or a `Path`. The `Kind` specifies the type of
/// artifact to generate, e.g., Languages, Heuristics, Vendors, or Documentation.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Definition {
    pub name: String,
    pub location: Location,
    pub kind: Kind,
}

/// Location is used to specify the path to the respective [`Definition`].
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Location {
    /// The `Path` variant is used to specify the path to the respective [`Definition`]. It must be
    /// available locally on the filesystem.
    Path(PathBuf),
    /// The `URL` variant is used to specify the URL to the respective [`Definition`]. It will be
    /// downloaded from the given URL.
    URL(String),
}

/// Kind is used to specify the type of artifact to generate, e.g., Languages, Heuristics, Vendors,
/// or Documentation.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Kind {
    Languages,
    Heuristics,
    Vendors,
    Documentation,
}

impl Config {
    /// Creates a new `Config` with default options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a [`Definition`] to the `Config`.
    pub fn add_definition(&mut self, definition: Definition) -> &mut Self {
        self.definitions.push(definition);
        self
    }

    /// Used internally to download a definition from the given `url` and write it to the given
    /// `out_dir`.
    fn download_from_url(&self, out_dir: &Path, url: &str) -> Result<PathBuf, ()> {
        match reqwest::blocking::get(url) {
            Ok(result) => {
                let path = out_dir.join("file.yml");
                let mut file = std::fs::File::create(path.clone()).expect("cannot create tempfile");
                std::io::copy(
                    &mut result
                        .text()
                        .expect("cannot unwrap response file")
                        .as_bytes(),
                    &mut file,
                )
                .expect("cannot copy reponse into file");
                Ok(path)
            }
            Err(_) => Err(()),
        }
    }

    /// Generate a [`Language`] definition and writes it to the `out_path`.
    fn generate_language(&self, name: &str, location: Location) {
        let tmpdir = tempdir().expect("failed to create a tempdir");
        let def_file = match location {
            Location::URL(url) => self.download_from_url(tmpdir.path(), &url).unwrap(),
            Location::Path(path) => path,
        };

        let data = load_github_linguist_languages(def_file).unwrap();
        let mut entries: Vec<String> = Vec::new();
        for item in data.iter() {
            entries.push(write_language_definition(item));
        }

        let target_path = self.out_path.clone();
        let mut target_file = std::fs::File::create(target_path.join(name)).unwrap();
        _ = target_file.write_all("use linguist::serde::StaticLanguage;\n\npub static LANGUAGES: &[&StaticLanguage] = &[\n".to_string().as_bytes());
        for str in entries {
            _ = target_file.write_all(format!("    {},\n", str).as_bytes());
        }
        _ = target_file.write_all("];\n".to_string().as_bytes());
        _ = target_file.flush();
    }

    /// Generate a [`HeuristicRule`] definition and writes it to the `out_path`.
    fn generate_heuristics(&self, name: &str, location: Location) {
        let tmpdir = tempdir().expect("failed to create a tempdir");
        let def_file = match location {
            Location::URL(url) => self.download_from_url(tmpdir.path(), &url).unwrap(),
            Location::Path(path) => path,
        };

        let data = load_github_linguist_heuristics(def_file).unwrap();
        let mut entries: Vec<String> = Vec::new();
        for item in data.iter() {
            entries.push(write_heuristic_definition(item));
        }

        let target_path = self.out_path.clone();
        let mut target_file = std::fs::File::create(target_path.join(name)).unwrap();
        _ = target_file.write_all("use std::ffi::OsString;\nuse linguist::resolver::HeuristicRule;\n\npub fn heuristics() -> Vec<HeuristicRule> {\n let langs: Vec<HeuristicRule> = vec![".to_string().as_bytes());
        for str in entries {
            _ = target_file.write_all(format!("    {},\n", str).as_bytes());
        }
        _ = target_file.write_all("];\nlangs\n}\n".to_string().as_bytes());
        _ = target_file.flush();
    }

    /// Generate a `Vendor` definition and writes it to the `out_path`.
    fn generate_vendors(&self, name: &str, location: Location) {
        let tmpdir = tempdir().expect("failed to create a tempdir");
        let def_file = match location {
            Location::URL(url) => self.download_from_url(tmpdir.path(), &url).unwrap(),
            Location::Path(path) => path,
        };

        let data = load_github_vendors(def_file).unwrap();

        let target_path = self.out_path.clone();
        let mut target_file = std::fs::File::create(target_path.join(name)).unwrap();
        _ = target_file
            .write_all(format!("pub static VENDORS: &[&str; {}] = &[", data.len()).as_bytes());
        for str in data {
            _ = target_file.write_all(format!("    r\"{}\",\n", str).as_bytes());
        }

        _ = target_file.write_all("];\n".to_string().as_bytes());
        _ = target_file.flush();
    }

    /// Generate a `Documentation` definition and writes it to the `out_path`.
    fn generate_documentation(&self, name: &str, location: Location) {
        let tmpdir = tempdir().expect("failed to create a tempdir");
        let def_file = match location {
            Location::URL(url) => self.download_from_url(tmpdir.path(), &url).unwrap(),
            Location::Path(path) => path,
        };

        let data = load_github_documentation(def_file).unwrap();

        let target_path = self.out_path.clone();
        let mut target_file = std::fs::File::create(target_path.join(name)).unwrap();
        _ = target_file.write_all(
            format!("pub static DOCUMENTATION: &[&str; {}] = &[", data.len()).as_bytes(),
        );
        for str in data {
            _ = target_file.write_all(format!("    r\"{}\",\n", str).as_bytes());
        }

        _ = target_file.write_all("];\n".to_string().as_bytes());
        _ = target_file.flush();
    }

    /// Generates all configured definitions and writes them to the `out_path`.
    pub fn generate(&self) {
        for def in self.definitions.iter() {
            match def.kind {
                Kind::Languages => self.generate_language(&def.name, def.location.clone()),
                Kind::Heuristics => self.generate_heuristics(&def.name, def.location.clone()),
                Kind::Vendors => self.generate_vendors(&def.name, def.location.clone()),
                Kind::Documentation => self.generate_documentation(&def.name, def.location.clone()),
            };
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

fn write_language_definition(lang: &Language) -> String {
    let mut str = String::new();
    str.push_str("&StaticLanguage {");

    if let Some(parent) = &lang.parent {
        str.push_str(format!("parent: Some(\"{}\"), ", parent).as_str());
    } else {
        str.push_str("parent: None, ");
    }

    str.push_str(format!("name: \"{}\", ", &lang.name).as_str());

    if !lang.aliases.is_empty() {
        str.push_str(
            format!(
                "aliases: Some(&[{}]), ",
                &lang
                    .aliases
                    .iter()
                    .map(|s| format!("\"{}\"", s))
                    .collect::<Vec<String>>()
                    .join(", ")
            )
            .as_str(),
        );
    } else {
        str.push_str("aliases: None, ");
    }

    // str.push_str(format!("scope: Scope::{}, ", &lang.scope.to_string()).as_str());
    str.push_str(format!("scope: \"{}\", ", &lang.scope.to_string()).as_str());

    if !lang.extensions.is_empty() {
        str.push_str(
            format!(
                "extensions: Some(&[{}]), ",
                &lang
                    .extensions
                    .iter()
                    .map(|s| format!("\"{}\"", s.to_str().expect("cannot unwrap extension")))
                    .collect::<Vec<String>>()
                    .join(", ")
            )
            .as_str(),
        );
    } else {
        str.push_str("extensions: None, ");
    }

    if !lang.filenames.is_empty() {
        str.push_str(
            format!(
                "filenames: Some(&[{}]), ",
                &lang
                    .filenames
                    .iter()
                    .map(|s| format!("\"{}\"", s.to_str().expect("cannot unwrap filename")))
                    .collect::<Vec<String>>()
                    .join(", ")
            )
            .as_str(),
        );
    } else {
        str.push_str("filenames: None, ");
    }

    if !lang.interpreters.is_empty() {
        str.push_str(
            format!(
                "interpreters: Some(&[{}]), ",
                &lang
                    .interpreters
                    .iter()
                    .map(|s| format!("\"{}\"", s))
                    .collect::<Vec<String>>()
                    .join(", ")
            )
            .as_str(),
        );
    } else {
        str.push_str("interpreters: None, ");
    }

    if let Some(color) = &lang.color {
        str.push_str(format!("color: Some(\"{}\") ", color).as_str());
    } else {
        str.push_str("color: None ");
    }

    str.push('}');
    str
}

/// Convert a [`HeuristicRule`] into a string representation (as rust code).
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
                        s.replace('\\', "\\\\").replace('\"', "\\\"")
                    ))
                    .collect::<Vec<String>>()
                    .join(", ")
            )
            .as_str(),
        );
    } else {
        str.push_str("patterns: vec![] ");
    }

    str.push('}');
    str
}
