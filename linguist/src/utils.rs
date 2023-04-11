use fancy_regex::Regex;
use std::{io::Read, path::Path};

pub static CONFIGURATION_EXTENSIONS: [&str; 6] = ["xml", "json", "toml", "yaml", "ini", "sql"];

/// Check if a file is a configuration file by checking if it has a configuration extension.
pub fn is_configuration(file: impl AsRef<Path>) -> bool {
    if let Some(ext) = file.as_ref().to_path_buf().extension() {
        return CONFIGURATION_EXTENSIONS.contains(&ext.to_str().unwrap());
    }
    false
}

pub fn is_documentation(_file: impl AsRef<Path>) -> bool {
    false
}

/// Check if a file is a dotfile by checking if it starts with a dot.
pub fn is_dotfile(file: impl AsRef<Path>) -> bool {
    if let Some(name) = file
        .as_ref()
        .to_path_buf()
        .file_name()
        .and_then(|name| name.to_str())
    {
        return name.starts_with('.') && name != ".";
    }
    false
}

/// Check if a file is a vendor file by checking if it matches any of the vendor rules.
pub fn is_vendor(file: impl AsRef<Path>, rules: Vec<String>) -> bool {
    for rule in rules {
        let matcher = Regex::new(&rule).unwrap();

        if matcher
            .is_match(file.as_ref().to_path_buf().to_str().unwrap())
            .unwrap()
        {
            return true;
        }
    }
    false
}

/// Check if a file is binary or not by checking if it contains a null byte.
/// this is based on [https://git.kernel.org/pub/scm/git/git.git/tree/xdiff-interface.c?id=HEAD#n198]
pub fn is_binary(file: impl AsRef<Path>) -> Result<bool, ()> {
    let path = file.as_ref().canonicalize();
    if path.is_err() {
        return Err(());
    }

    let data = std::fs::File::open(path.unwrap());
    let mut buffer = [0; 8000];

    if data.is_err() {
        return Err(());
    }

    if let Ok(n) = data.unwrap().read(&mut buffer) {
        for i in 0..n {
            if buffer[i] == 0 {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

pub fn is_generated(_file: impl AsRef<Path>) -> bool {
    false
}
