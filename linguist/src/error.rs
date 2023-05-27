#[derive(Debug)]
pub enum LinguistError {
    /// Indicates that the language definition file could not be deserialized.
    #[cfg(feature = "serde")]
    DeserializationError,
    /// Indicates that a specific language cannot be found.
    LanguageNotFound,
    /// Indicates that a given file could not be found.
    #[cfg(feature = "serde")]
    FileNotFound,
    /// Represents an error that occurred while compiling a regular expression.
    PatternCompileError(regex::Error),
    /// Represents an error occured concerning io stuff.
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
