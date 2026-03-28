use std::fmt;

pub enum Error {
    Git(String),
    Io(std::io::Error),
    Http(reqwest::Error),
    Linear(String),
    InvalidArgument(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Git(msg) => write!(f, "git: {msg}"),
            Self::Io(err) => write!(f, "io: {err}"),
            Self::Http(err) => write!(f, "http: {err}"),
            Self::Linear(msg) => write!(f, "linear: {msg}"),
            Self::InvalidArgument(msg) => write!(f, "{msg}"),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Self::Http(e)
    }
}
