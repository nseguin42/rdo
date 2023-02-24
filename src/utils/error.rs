#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Parse(std::num::ParseIntError),
    Config(config::ConfigError),
    Conversion(String),
    ScriptDependencyNotRun(String, String),
    ScriptNotFound(String),
    Unspecified(String),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Error {
        Error::Parse(err)
    }
}

impl From<config::ConfigError> for Error {
    fn from(err: config::ConfigError) -> Error {
        Error::Config(err)
    }
}

impl From<String> for Error {
    fn from(err: String) -> Error {
        Error::Unspecified(err)
    }
}

impl From<&str> for Error {
    fn from(err: &str) -> Error {
        Error::Unspecified(err.to_string())
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(err: std::string::FromUtf8Error) -> Error {
        Error::Conversion(err.to_string())
    }
}

impl From<std::string::FromUtf16Error> for Error {
    fn from(err: std::string::FromUtf16Error) -> Error {
        Error::Conversion(err.to_string())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Io(err) => write!(f, "IO error: {}", err),
            Error::Parse(err) => write!(f, "Parse error: {}", err),
            Error::Config(err) => write!(f, "Config error: {}", err),
            Error::Conversion(err) => write!(f, "Conversion error: {}", err),
            Error::Unspecified(err) => write!(f, "Unspecified error: {}", err),
            Error::ScriptDependencyNotRun(script, dep) => {
                write!(f, "Dependency of {} not run: {}", script, dep)
            }
            Error::ScriptNotFound(script) => write!(f, "script not found: {}", script),
        }
    }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for Error {
    fn from(err: tokio::sync::mpsc::error::SendError<T>) -> Error {
        Error::Unspecified(err.to_string())
    }
}
