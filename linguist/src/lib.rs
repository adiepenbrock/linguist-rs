use std::ffi::OsString;
use std::path::Path;
use std::{collections::HashMap, io::Read};

#[derive(Debug, Clone)]
pub struct LanguageInfo {
    pub parent: Option<String>,
    pub name: String,
    pub aliases: Vec<String>,
    pub scope: String,
    pub extensions: Vec<OsString>,
    pub filenames: Vec<OsString>,
    pub color: Option<String>,
}

#[derive(Debug, Clone)]
pub enum LanguageSetError {
    LanguageNotFound,
    #[cfg(feature = "yaml-load")]
    FileNotFound,
}

#[derive(Debug, Default)]
pub struct LanguageSet {
    languages: Vec<LanguageInfo>,
    extensions: HashMap<OsString, Vec<usize>>,
    filenames: HashMap<OsString, Vec<usize>>,
}

impl LanguageSet {
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(feature = "yaml-load")]
    pub fn load_from_path(path: impl AsRef<Path>) -> Result<LanguageSet, LanguageSetError> {
        use serde::Deserialize;

        if !path.as_ref().exists() {
            return Err(LanguageSetError::FileNotFound);
        }

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

        let ldc = std::fs::read_to_string(path).expect("failed to read path");
        let defs: Result<HashMap<String, GhLanguageDef>, _> = serde_yaml::from_str(ldc.as_str());

        let mut ls = Self::default();
        if let Ok(defs) = defs {
            defs.iter().for_each(|(name, lang)| {
                let _ = ls.register_language(LanguageInfo {
                    name: name.clone(),
                    scope: lang.scope.clone(),
                    extensions: lang
                        .extensions
                        .clone()
                        .unwrap_or_default()
                        .iter()
                        .map(OsString::from)
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
            });
        }

        Ok(ls)
    }

    pub fn register_language(&mut self, lang: LanguageInfo) -> Result<(), LanguageSetError> {
        let idx = self.languages.len();

        // before we finally add the language to our vec of languages, we add it to our internal
        // cache of extension -> lang-idx
        lang.extensions.iter().for_each(|ext| {
            if let Some(langs) = self.extensions.get_mut(ext) {
                langs.push(idx);
            } else {
                self.extensions.insert(ext.clone(), vec![idx]);
            }
        });
        lang.filenames.iter().for_each(|name| {
            if let Some(langs) = self.filenames.get_mut(name) {
                langs.push(idx);
            } else {
                self.filenames.insert(name.clone(), vec![idx]);
            }
        });
        self.languages.push(lang);

        Ok(())
    }

    pub fn remove_language_by_name(&mut self, name: &str) -> Result<(), LanguageSetError> {
        let idx = self
            .languages
            .iter()
            .position(|lang| lang.name == name || lang.aliases.contains(&name.to_string()))
            .ok_or(LanguageSetError::LanguageNotFound)?;

        // remove the idx of all extensions that have a reference to this particular language
        for (_, value) in self.extensions.iter_mut() {
            value.retain(|&x| x != idx);
        }

        self.languages.remove(idx);

        Ok(())
    }

    /// Resolves the programming language by the given extension.
    pub fn resolve_languages_by_extension(
        &self,
        filename: impl AsRef<Path>,
    ) -> Option<Vec<&LanguageInfo>> {
        // first, check if we can get an extension from the given `Path`; second, some extensions
        // may not be identified as an extension, e.g., .vhost, so we also try to use the filename
        // to resolve the possible languages
        let ext = match filename.as_ref().extension() {
            Some(ext) => ext,
            _ => match filename.as_ref().file_name() {
                Some(name) => name,
                _ => return None,
            },
        };

        let languages: Vec<&LanguageInfo> = self
            .extensions
            .get(&ext.to_os_string())
            .unwrap()
            .iter()
            .map(|idx| &self.languages[*idx])
            .collect();

        if !languages.is_empty() {
            Some(languages)
        } else {
            None
        }
    }

    pub fn resolve_languages_by_filename(
        &self,
        filename: impl AsRef<Path>,
    ) -> Option<Vec<&LanguageInfo>> {
        let languages: Vec<&LanguageInfo> = self
            .filenames
            .get(&filename.as_ref().as_os_str().to_os_string())
            .unwrap()
            .iter()
            .map(|idx| &self.languages[*idx])
            .collect();

        if !languages.is_empty() {
            Some(languages)
        } else {
            None
        }
    }
}
