use std::{collections::HashMap, ffi::OsString, path::Path};

use serde::Deserialize;

use crate::{
    error::LinguistError,
    resolver::{Language, Scope},
};

#[derive(Debug, Clone)]
pub struct StaticLanguage<'src> {
    pub name: &'src str,
    pub scope: &'src str,
    pub aliases: Option<&'src [&'src str]>,
    pub extensions: Option<&'src [&'src str]>,
    pub filenames: Option<&'src [&'src str]>,
    pub interpreters: Option<&'src [&'src str]>,
    pub color: Option<&'src str>,
    pub parent: Option<&'src str>,
}

impl<'src> From<&'src StaticLanguage<'src>> for Language {
    fn from(value: &'src StaticLanguage<'src>) -> Self {
        let parent = value.parent.map(String::from);
        let name = String::from(value.name);
        let aliases = value
            .aliases
            .map(|aliases| aliases.iter().map(|alias| String::from(*alias)).collect());
        let scope = Scope::from(value.scope);
        let extensions = value
            .extensions
            .map(|extensions| extensions.iter().map(|ext| OsString::from(*ext)).collect());
        let filenames = value.filenames.map(|filenames| {
            filenames
                .iter()
                .map(|filename| OsString::from(*filename))
                .collect()
        });
        let interpreters = value.interpreters.map(|interpreters| {
            interpreters
                .iter()
                .map(|interp| String::from(*interp))
                .collect()
        });
        let color = value.color.map(String::from);

        Language {
            parent,
            name,
            aliases: aliases.unwrap_or_default(),
            scope,
            extensions: extensions.unwrap_or_default(),
            filenames: filenames.unwrap_or_default(),
            interpreters: interpreters.unwrap_or_default(),
            color,
        }
    }
}

/// Deserialize a YAML file into a vector of languages. This supports the deserialization of
/// custom language definition types by taking a generic type parameter. The generic type must
/// implement the `TryInto<Language>` and the `serde::Deserialize` trait. Furthermore, the path
/// to the YAML file must be provided.
pub fn deserialize_languages<T>(path: impl AsRef<Path>) -> Result<Vec<Language>, LinguistError>
where
    for<'de> T: Deserialize<'de>,
    T: TryInto<Language>,
{
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let data: HashMap<String, T> = match serde_yaml::from_str(&content) {
        Ok(result) => result,
        Err(_) => {
            return Err(LinguistError::DeserializationError);
        }
    };

    let mut languages: Vec<Language> = Vec::new();
    for (name, item) in data.into_iter() {
        match item.try_into() {
            Ok(mut lang) => {
                lang.name = name;
                languages.push(lang)
            }
            Err(_) => {
                return Err(LinguistError::DeserializationError);
            }
        };
    }

    Ok(languages)
}

/// Deserialize a YAML file into a vector of strings.
pub fn deserialize_strings(path: impl AsRef<Path>) -> Result<Vec<String>, LinguistError> {
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let data: Vec<String> = match serde_yaml::from_str(&content) {
        Ok(result) => result,
        Err(_) => {
            return Err(LinguistError::DeserializationError);
        }
    };

    Ok(data)
}
