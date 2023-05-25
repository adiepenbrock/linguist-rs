use std::{
    io::{BufRead, Cursor, Read},
    path::Path,
};

use crate::error::LinguistError;
use regex::{Regex, RegexSet};

pub static CONFIGURATION_EXTENSIONS: [&str; 6] = ["xml", "json", "toml", "yaml", "ini", "sql"];

/// Check if a file is a configuration file by checking if it has a configuration extension.
pub fn is_configuration(file: impl AsRef<Path>) -> bool {
    if let Some(ext) = file.as_ref().to_path_buf().extension() {
        return CONFIGURATION_EXTENSIONS.contains(&ext.to_str().unwrap());
    }
    false
}

pub static DOCS: [&str; 18] = [
    "[Dd]ocs?/",
    "(^|/)[Dd]ocumentation/",
    "(^|/)[Gg]roovydoc/",
    "(^|/)[Jj]avadoc/",
    "^[Mm]an/",
    "^[Ee]xamples/",
    "^[Dd]emos?/",
    "(^|/)inst/doc/",
    "(^|/)CITATION(\\.cff|(S)?(\\.(bib|md))?)$",
    "(^|/)CHANGE(S|LOG)?(\\.|$)",
    "(^|/)CONTRIBUTING(\\.|$)",
    "(^|/)COPYING(\\.|$)",
    "(^|/)INSTALL(\\.|$)",
    "(^|/)LICEN[CS]E(\\.|$)",
    "(^|/)[Ll]icen[cs]e(\\.|$)",
    "(^|/)README(\\.|$)",
    "(^|/)[Rr]eadme(\\.|$)",
    "^[Ss]amples?/",
];

pub fn is_documentation(file: impl AsRef<Path>, matcher: &RegexSet) -> bool {
    matcher.is_match(file.as_ref().display().to_string().as_str())
}

/// Check if a file is a dotfile by checking if it starts with a dot.
pub fn is_dotfile(file: impl AsRef<Path>) -> bool {
    file.as_ref().display().to_string().starts_with('.')
}

/// Check if a file is a vendor file by checking if it matches any of the vendor rules.
pub fn is_vendor(file: impl AsRef<Path>, matcher: &RegexSet) -> bool {
    matcher.is_match(file.as_ref().to_str().unwrap_or(""))
}

const FIRST_FEW_BYTES: usize = 8000;

/// Check if a file is binary or not by checking if it contains a null byte.
/// this is based on [https://git.kernel.org/pub/scm/git/git.git/tree/xdiff-interface.c?id=HEAD#n198]
pub fn is_binary(path: impl AsRef<Path>) -> Result<bool, LinguistError> {
    let mut file = std::fs::File::open(path.as_ref())?;
    let mut buf = [0; FIRST_FEW_BYTES];
    let n = file.read(&mut buf)?;

    for byte in &buf[..n] {
        if *byte == 0 {
            return Ok(true);
        }
    }
    Ok(false)
}

pub static GENERATED_NAMES_EXTENSIONS: [&str; 3] = ["nib", "xcworkspacedata", "xcuserstate"];

pub fn is_generated(file: impl AsRef<Path>) -> bool {
    if let Some(ext) = file.as_ref().to_path_buf().extension() {
        return GENERATED_NAMES_EXTENSIONS.contains(&ext.to_str().unwrap());
    }
    false
}

pub(crate) fn has_shebang(data: &[u8]) -> bool {
    data.starts_with(b"#!")
}

pub(crate) fn determine_multiline_exec(data: &[u8]) -> Option<String> {
    let mut interpreter = "sh";
    let mut cursor = Cursor::new(data);
    let mut buf = String::new();

    let shebang_exec = Regex::new(r#"exec (\w+).+\$0.+\$@"#).unwrap();

    for _i in 0..5 {
        buf.clear();
        if let Ok(result) = cursor.read_line(&mut buf) {
            if result == 0 {
                break;
            }
        }

        if shebang_exec.is_match(&buf) {
            interpreter = shebang_exec.captures(&buf).unwrap().get(1).unwrap().into();
            break;
        }
    }

    Some(interpreter.to_string())
}

/// Checks whether the supplied input contains constructs that are not supported by the
/// [regex crate](https://crates.io/crates/regex), e.g.:
/// - lookbehind & lookahead
/// - non-backtracking subexpressions
/// - named & numbered capturing group / after text matching
/// - backreference
/// - possessive quantifier
///
/// For a detailed reference on supported syntax see [RE2 Syntax](https://github.com/google/re2/wiki/Syntax)
pub(crate) fn is_unsupported_regex_syntax(input: &str) -> bool {
    input.contains("(?<")
        || input.contains("(?=")
        || input.contains("(?!")
        || input.contains("(?>")
        || input.contains("\\1")
        || input.contains("*+")
}
