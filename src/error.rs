use chrono;

use crate::SiteInfo;

/// A general error type.
#[derive(Debug)]
pub enum Error {
    /// A general error with a message describing it.
    General(String),
    /// An error with the local store
    LocalStore(filedb::Error),
    /// Any other error is passed up this way.
    Internal(Box<dyn std::error::Error>),

    /// The NBMData doesn't have a matching column
    NBMData(nbm_tools::Error),

    /// No data for that initialization time is available for any location.
    InitializationTimeNotAvailable(chrono::NaiveDateTime),
    /// There was no match for the requested site, the internal value is the requested site.
    NoMatch(String),
    /// There were multiple matches for the requested site.
    AmbiguousSite {
        /// A list of sites that are potential matches.
        matches: Vec<SiteInfo>,
    },
}

impl Error {
    /// Create a new error.
    pub fn general_error(msg: String) -> Self {
        Error::General(msg)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::General(msg) => write!(f, "{}", msg),
            Self::LocalStore(err) => write!(f, "filedb err: {}", err),
            Self::Internal(err) => write!(f, "{}", err),
            Self::NBMData(err) => write!(f, "NBMData err: {}", err),
            Self::NoMatch(requested_site) => {
                write!(f, "No match found for site {}", requested_site)
            }
            Self::AmbiguousSite { matches } => {
                writeln!(f, "Ambiguous site name, possible matches are")?;
                for site_info in matches {
                    writeln!(f, "     {}", site_info)?;
                }
                Ok(())
            }
            Self::InitializationTimeNotAvailable(init_time) => {
                write!(f, "No data available for initialization time {}", init_time)
            }
        }
    }
}

impl std::error::Error for Error {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match self {
            Self::Internal(err) => Some(err.as_ref()),
            _ => None,
        }
    }

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Internal(err) => Some(err.as_ref()),
            _ => None,
        }
    }
}

impl From<filedb::Error> for Error {
    fn from(err: filedb::Error) -> Self {
        Self::LocalStore(err)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Self::Internal(err.into())
    }
}

impl From<rusqlite::Error> for Error {
    fn from(err: rusqlite::Error) -> Self {
        Self::Internal(err.into())
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::Internal(err.into())
    }
}

impl From<nbm_tools::Error> for Error {
    fn from(err: nbm_tools::Error) -> Self {
        Self::NBMData(err)
    }
}

impl From<csv::Error> for Error {
    fn from(err: csv::Error) -> Self {
        Self::Internal(err.into())
    }
}

