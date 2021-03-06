use rusqlite::{OptionalExtension, ToSql};
use std::fmt::Display;

/// This is the result of validating a request for data from a site at a specific time.
///
/// This includes all the information available about the site and the most recent initialization
/// time that this site has data available. The most recent time depends on the time for which data
/// was requested, and the initialization time will always be the nearest time before the requested
/// time.
#[derive(Debug, Clone)]
pub struct SiteValidation {
    /// The information about the validated site as retrieved from the store.
    pub site: SiteInfo,
    /// The initialization time for which data is available. This may be different than the time
    /// for which data was requested.
    pub initialization_time: chrono::NaiveDateTime,
}

impl SiteValidation {
    /// Create a new SiteValidation object.
    pub fn new(site: SiteInfo, initialization_time: chrono::NaiveDateTime) -> Self {
        Self {
            site,
            initialization_time,
        }
    }

    /// Get the file name associated with the validation.
    pub(crate) fn file_name(&self) -> String {
        self.site.id.clone() + ".csv"
    }
}

/// Validate a site against a "locations.csv" file.
pub(crate) fn validate(site: &str, locations_str: &str) -> Result<SiteInfo, crate::error::Error> {
    let loc_db = rusqlite::Connection::open_in_memory()?;
    build_locations_database(&loc_db, locations_str)?;

    match find_exact_case_insensitive_match(&loc_db, site) {
        Ok(Some(site_info)) => return Ok(site_info),
        Err(err) => return Err(err),
        Ok(None) => {}
    }

    let matches = find_similar_sites(&loc_db, site)?;

    dbg!(matches.len());

    if matches.is_empty() {
        Err(crate::error::Error::NoMatch(site.to_owned()))
    } else if matches.len() > 1 {
        Err(crate::error::Error::AmbiguousSite { matches })
    } else {
        Ok(matches.into_iter().next().unwrap())
    }
}

fn build_locations_database(
    conn: &rusqlite::Connection,
    locations_str: &str,
) -> Result<(), crate::error::Error> {
    const INIT_LOCATIONS_DB: &'static str = r#"
      CREATE TABLE locations (                
        id    TEXT NOT NULL,                 
        name  TEXT NOT NULL,                 
        state TEXT NOT NULL,                 
        lat   REAL NOT NULL,                 
        lon   REAL NOT NULL,                 
        PRIMARY KEY (id) ON CONFLICT IGNORE) 
    "#;

    const INSERT_LOCATION: &'static str = r#"
        INSERT INTO locations (id, name, state, lat, lon) VALUES (?, ?, ?, ?, ?)
    "#;

    conn.execute(INIT_LOCATIONS_DB, rusqlite::NO_PARAMS)?;

    let mut stmt = conn.prepare(INSERT_LOCATION)?;

    let mut rdr = csv::Reader::from_reader(locations_str.as_bytes());

    for rec in rdr
        .records()
        .filter_map(|res| res.ok())
    {

        if let (Some(id), Some(name), Some(state_prov), Some(lat_str), Some(lon_str)) =
            (rec.get(0), rec.get(1), rec.get(2), rec.get(3), rec.get(4))
        {
            let latitude: f64 = if let Ok(val) = lat_str.parse() {
                val 
            } else {
                continue;
            };

            let longitude: f64 = if let Ok(val) = lon_str.parse() {
                val 
            } else {
                continue;
            };

            let id = id.trim();
            let name = name.trim();
            let state_prov = state_prov.trim();

            stmt.execute(&[&id as &dyn ToSql, &name, &state_prov, &latitude, &longitude])?;
        }
    }

    Ok(())
}

fn find_exact_case_insensitive_match(
    conn: &rusqlite::Connection,
    site: &str,
) -> Result<Option<SiteInfo>, crate::error::Error> {
    let query_exact_case_insensitive_match = format!(
        "SELECT id, name, state, lat, lon FROM locations WHERE id = '{}'",
        site
    );

    Ok(conn
        .query_row(
            &query_exact_case_insensitive_match,
            rusqlite::NO_PARAMS,
            |row| {
                let id = row.get(0)?;
                let name = row.get(1)?;
                let state_prov = row.get(2)?;
                let latitude: f64 = row.get(3)?;
                let longitude: f64 = row.get(4)?;

                let latitude = latitude as f32;
                let longitude = longitude as f32;

                Ok(SiteInfo {
                    id,
                    name,
                    state_prov,
                    latitude,
                    longitude,
                })
            },
        )
        .optional()?)
}

fn find_similar_sites(
    conn: &rusqlite::Connection,
    site: &str,
) -> Result<Vec<SiteInfo>, crate::error::Error> {
    let query_similar_sites = format!(
        "SELECT id, name, state, lat, lon FROM locations WHERE id LIKE '%{}%' OR name LIKE '%{}%' OR state LIKE '{}'",
        site, site, site
    );

    let mut stmt = conn.prepare(&query_similar_sites)?;

    let temp = stmt.query_map(rusqlite::NO_PARAMS, |row| {
        let id = row.get(0)?;
        let name = row.get(1)?;
        let state_prov = row.get(2)?;
        let latitude: f64 = row.get(3)?;
        let longitude: f64 = row.get(4)?;

        let latitude = latitude as f32;
        let longitude = longitude as f32;

        Ok(SiteInfo {
            id,
            name,
            state_prov,
            latitude,
            longitude,
        })
    })?;

    Ok(temp.filter_map(|res| res.ok()).collect())
}

/// All the available information about a site as retrived from the store.
#[derive(Debug, Clone)]
pub struct SiteInfo {
    /// The alpha numeric identifier of the location. This should be unique and this library
    /// assumes that it is.
    pub id: String,
    /// The name of the location. This may not be unique in the store.
    pub name: String,
    /// The state/providence for this location.
    pub state_prov: String,
    /// The latitude of the location.
    pub latitude: f32,
    /// The longitude of the location.
    pub longitude: f32,
}

impl SiteInfo {
    /// Create a new SiteInfo object.
    pub fn new(
        name: String,
        id: String,
        latitude: f32,
        longitude: f32,
        state_prov: String,
    ) -> Self {
        Self {
            name,
            id,
            latitude,
            longitude,
            state_prov,
        }
    }
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
