use crate::error::ValidationError;

use chrono;

use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct SiteValidation {
    site: SiteInfo,
    initialization_time: chrono::NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct SiteInfo {
    name: String,
    id: String,
    latitude: f32,
    longitude: f32,
    state_prov: String,
}

impl Display for SiteInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{:6} {:6.2}, {:7.2}, {}, {}",
            self.id, self.latitude, self.longitude, self.name, self.state_prov
        )
    }
}
