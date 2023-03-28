#[cfg(feature = "yaml-load")]
use std::path::Path;

#[derive(Debug, Clone)]
pub struct LanguageInfo {
    pub name: String,
    pub scope: String,
    pub extensions: Vec<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone)]
pub enum LanguageSetError {
    #[cfg(feature = "yaml-load")]
    FileNotFound,
}

#[derive(Debug)]
pub struct LanguageSet {
    languages: Vec<LanguageInfo>,
}

impl Default for LanguageSet {
    fn default() -> Self {
        Self {
            languages: Vec::new(),
        }
    }
}

impl LanguageSet {
    pub fn new() -> Self {
        Self::default()
    }

    /// Convenience constructor for loading a language set from a definition file.
    #[cfg(feature = "yaml-load")]
    pub fn load_from_path(path: impl AsRef<Path>) -> Result<LanguageSet, LanguageSetError> {
        if !path.as_ref().exists() {
            return Err(LanguageSetError::FileNotFound);
        }

        use serde::Deserialize;
        use std::collections::HashMap;

        #[derive(Deserialize)]
        struct GhLanguageDef {
            color: Option<String>,
            #[serde(rename = "type")]
            scope: String,
            extensions: Option<Vec<String>>,
        }

        let ldc = std::fs::read_to_string(path).expect("failed to read path");
        let defs: Result<HashMap<String, GhLanguageDef>, _> = serde_yaml::from_str(ldc.as_str());

        let mut ls = Self::default();

        if defs.is_ok() {
            defs.unwrap().iter().for_each(|(name, lang)| {
                ls.languages.push(LanguageInfo {
                    name: name.to_string(),
                    scope: lang.scope.clone(),
                    extensions: lang.extensions.clone().unwrap_or(vec![]),
                    color: lang.color.clone(),
                });
            });
        }

        Ok(ls)
    }

    pub fn register_language(&mut self, lang: LanguageInfo) -> Result<(), LanguageSetError> {
        // TODO (unique language name): only add the language iff there is no other language with
        // the same name already...

        self.languages.push(lang.clone());
        Ok(())
    }

    pub fn remove_language_by_name(&mut self, name: &str) -> Result<(), LanguageSetError> {
        if let Some(idx) = self
            .languages
            .iter()
            .position(|item| item.name == name.to_string())
        {
            self.languages.remove(idx);
        }
        Ok(())
    }

    pub fn resolve_language_by_extension(&self, ext: &str) -> Option<&LanguageInfo> {
        if let Some(idx) = self
            .languages
            .iter()
            .position(|lang| lang.extensions.contains(&ext.to_string()))
        {
            return Some(&self.languages[idx]);
        }

        None
    }
}
