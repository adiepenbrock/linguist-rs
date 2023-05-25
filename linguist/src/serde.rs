use std::{collections::HashMap, path::Path};

use serde::Deserialize;

use crate::{error::LinguistError, resolver::Language};

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
