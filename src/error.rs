use chrono;

use crate::site_validation::SiteInfo;

#[derive(Debug)]
pub enum ValidationError {
    InitializationTimeNotAvailable(chrono::NaiveDateTime),
    NoMatch { requested_site: String },
    AmbiguousSite { matches: Vec<SiteInfo> },
    Other(Box<dyn std::error::Error + 'static>),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::NoMatch { requested_site } => {
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
            Self::Other(err) => {
                write!(f, "Other validation error: {}", err)
            }
        }
    }
}

impl std::error::Error for ValidationError {}
