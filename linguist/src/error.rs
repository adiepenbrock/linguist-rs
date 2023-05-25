#[derive(Debug)]
pub enum LinguistError {
    #[cfg(feature = "serde")]
    DeserializationError,
    LanguageNotFound,
    #[cfg(feature = "serde")]
    FileNotFound,
    PatternCompileError(regex::Error),
    IOError(std::io::Error),
}

impl From<std::io::Error> for LinguistError {
    fn from(value: std::io::Error) -> Self {
        LinguistError::IOError(value)
    }
}

impl From<regex::Error> for LinguistError {
    fn from(value: regex::Error) -> Self {
        LinguistError::PatternCompileError(value)
    }
}
