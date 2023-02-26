#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Parse(std::num::ParseIntError),
    Config(config::ConfigError),
    ScriptDependencyNotRun(String, String),
    ScriptNotFound(String),
    Unspecified(String),
    StdinClosed,
    StdoutClosed,
    StderrClosed,
    LoggingSetupFailed,
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

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Io(err) => write!(f, "IO error: {}", err),
            Error::Parse(err) => write!(f, "Parse error: {}", err),
            Error::Config(err) => write!(f, "Config error: {}", err),
            Error::Unspecified(err) => write!(f, "Unspecified error: {}", err),
            Error::ScriptDependencyNotRun(script, dep) => {
                write!(f, "Dependency of {} not run: {}", script, dep)
            }
            Error::ScriptNotFound(script) => write!(f, "script not found: {}", script),
            Error::LoggingSetupFailed => write!(f, "Failed setting up logger"),
            Error::StdinClosed => write!(f, "Stdin closed"),
            Error::StdoutClosed => write!(f, "Stdout closed"),
            Error::StderrClosed => write!(f, "Stderr closed"),
        }
    }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for Error {
    fn from(err: tokio::sync::mpsc::error::SendError<T>) -> Error {
        Error::Unspecified(err.to_string())
    }
}
