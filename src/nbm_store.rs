use std::str::FromStr;
use chrono::{Datelike, Timelike};

/// The interface to our storage for NBM 1D text files.
///
/// The NBMStore is backed by a private local store. When data is not available in the local store
/// it will fetch it from the internet and then keep a copy in the local store for faster retrieval
/// later.
pub struct NBMStore {
    local_store: filedb::FileDB,
}

impl NBMStore {
    /// Connect to a NBMStore.
    ///
    /// The path refers to a directory where the local store can save data. The path must be a
    /// directory and not a file name. The local store will handle naming and organization of files
    /// within the directory provided by the path. If the path is [Option::None], then a default
    /// path will be chosen in the user's home directory.
    pub fn connect<'a, OP: Into<Option<&'a std::path::Path>>>(
        path: OP,
    ) -> Result<Self, crate::Error> {
        let path: Option<&std::path::Path> = path.into();

        let path_buf: std::path::PathBuf = match path {
            Some(p) => std::path::PathBuf::from(p),
            None => Self::default_local_store_path()?,
        };

        let local_store = filedb::FileDB::connect(&path_buf)?;

        Ok(Self { local_store })
    }

    /// Validate a request.
    ///
    /// This function will find the closest NBM intialization time prior to request time and find
    /// the closest match to site. If there is no data available for the closest intitialization
    /// time, no close matches for sites, too many close matches, or some other kind of error, then
    /// a [ValidationError] is returned.
    pub fn validate_request(
        &self,
        site: &str,
        request_time: chrono::NaiveDateTime,
    ) -> Result<crate::SiteValidation, crate::Error> {
        let init_time = calculate_next_most_recent_nmb_initialization_time(request_time);

        let locations_str_bytes = self.local_store.retrieve_file("locations.csv", init_time)?;

        let locations_str = if let Some(bytes) = locations_str_bytes {
            Some(String::from_utf8(bytes)?)
        } else {
            match crate::download::download_file("locations.csv", init_time) {
                Ok(str_data) => {
                    let _err =
                        self.local_store
                            .add_file("locations.csv", init_time, str_data.as_bytes());
                    Some(str_data)
                }
                Err(_) => None,
            }
        };

        let locations_str =
            locations_str.ok_or_else(|| crate::Error::InitializationTimeNotAvailable(init_time))?;

        crate::site_validation::validate(site, &locations_str)
            .map(|site_info| crate::SiteValidation::new(site_info, init_time))
    }

    /// Validate a request, but keep going back in time until an available initialization time
    /// is found.
    ///
    /// Unlike [Self::validate_request()], this will not return an error for an initialization
    /// time not being available unless necessary. It will keep trying earlier request times until
    /// it finds a valid initialization time with some data. Otherwise, it behaves the same as
    /// [Self::validate_request]
    ///
    /// There is a limit to how far back it will go, right now it will only look back for 20
    /// versions, and if it can't find one it will still fail with an error.
    pub fn validate_most_recent_available(
        &self,
        site: &str,
        request_time: chrono::NaiveDateTime,
    ) -> Result<crate::SiteValidation, crate::Error> {
        let mut attempts_left = 20_i32;
        let mut attempt_request_time = request_time;

        loop {
            let validation = self.validate_request(site, attempt_request_time);

            match &validation {
                Err(crate::Error::InitializationTimeNotAvailable(init_time)) => {
                    attempt_request_time = *init_time - chrono::Duration::hours(1);
                }
                _ => return validation,
            }

            attempts_left -= 1;
            if attempts_left < 1 {
                return validation;
            }
        }
    }

    /// Once a validation has been completed, it can be used to load a text file.
    pub fn retrieve(
        &self,
        validation: crate::SiteValidation,
    ) -> Result<nbm_tools::NBMData, crate::Error> {
        let file_name = validation.file_name();

        let data_str = self
            .local_store
            .retrieve_file(&file_name, validation.initialization_time)?;

        let data_str = match data_str {
            Some(text) => Ok(String::from_utf8(text)?),
            None => {
                match crate::download::download_file(&file_name, validation.initialization_time) {
                    Ok(text) => {
                        self.local_store.add_file(
                            &file_name,
                            validation.initialization_time,
                            &text.as_bytes(),
                        )?;

                        Ok(text)
                    }
                    err @ Err(_) => err,
                }
            }
        }?;

        Ok(nbm_tools::NBMData::from_str(data_str.as_ref())?)
    }

    fn default_local_store_path() -> Result<std::path::PathBuf, crate::Error> {
        dirs::data_dir()
            .map(|mut p| {
                p.push("nbm-report");
                p.push("nbm_cache.sqlite3");
                p
            })
            .ok_or_else(|| {
                crate::Error::general_error("Couldn't find default local store".to_owned()).into()
            })
    }
}

fn calculate_next_most_recent_nmb_initialization_time(
    requested_time: chrono::NaiveDateTime,
) -> chrono::NaiveDateTime {
    let year = requested_time.year();
    let month = requested_time.month();
    let day = requested_time.day();
    let hour = requested_time.hour();

    let delta: chrono::Duration = match hour {
        hr if hour >= 19 => chrono::Duration::hours(i64::from(hr) - 19),
        hr if hour >= 13 => chrono::Duration::hours(i64::from(hr) - 13),
        hr if hour >= 7 => chrono::Duration::hours(i64::from(hr) - 7),
        hr if hour >= 1 => chrono::Duration::hours(i64::from(hr) - 1),
        _hr => chrono::Duration::hours(24 - 19),
    };

    chrono::NaiveDate::from_ymd(year, month, day).and_hms(hour, 0, 0) - delta
}

#[cfg(test)]
mod test {
    use crate as nbmarch;

    struct TestArchive {
        _temp_db_file: tempfile::NamedTempFile,
        arch: nbmarch::NBMStore,
    }

    fn create_test_archive() -> Result<TestArchive, Box<dyn std::error::Error>> {
        let temp_db_file = tempfile::NamedTempFile::new()?;
        let db_fname = temp_db_file.path();
        let arch = nbmarch::NBMStore::connect(db_fname)?;

        Ok(TestArchive {
            _temp_db_file: temp_db_file,
            arch,
        })
    }

    #[test]
    fn test_connect() -> Result<(), Box<dyn std::error::Error>> {
        let temp_db_file = tempfile::NamedTempFile::new()?;
        let db_fname = temp_db_file.path();

        let _arch = nbmarch::NBMStore::connect(db_fname)?;

        Ok(())
    }

    #[test]
    fn test_simple_validation() -> Result<(), Box<dyn std::error::Error>> {
        let arch = &create_test_archive()?.arch;

        let request_time = chrono::NaiveDate::from_ymd(2021, 2, 28).and_hms(15, 15, 0);
        let valid_time = chrono::NaiveDate::from_ymd(2021, 2, 28).and_hms(13, 0, 0);

        let validation = arch.validate_request("KMSO", request_time)?;
        assert_eq!(&validation.site.id, "KMSO");
        assert_eq!(&validation.file_name(), "KMSO.csv");
        assert_eq!(&validation.site.name, "MISSOULA");
        assert_eq!(validation.initialization_time, valid_time);

        let validation = arch.validate_request("missoula", request_time)?;
        assert_eq!(&validation.site.id, "KMSO");
        assert_eq!(&validation.file_name(), "KMSO.csv");
        assert_eq!(&validation.site.name, "MISSOULA");
        assert_eq!(validation.initialization_time, valid_time);

        let validation = arch.validate_request("logan", valid_time);
        assert!(validation.is_err());
        match validation {
            Err(nbmarch::Error::AmbiguousSite { .. }) => {}
            _ => panic!("Invalid error, should be ambiguous site"),
        }

        Ok(())
    }

    #[test]
    fn test_retrieve() -> Result<(), Box<dyn std::error::Error>> {
        let arch = &create_test_archive()?.arch;

        let request_time = chrono::NaiveDate::from_ymd(2021, 2, 28).and_hms(15, 15, 0);
        let valid_time = chrono::NaiveDate::from_ymd(2021, 2, 28).and_hms(13, 0, 0);

        let validation = arch.validate_request("KMSO", request_time)?;
        assert_eq!(&validation.site.id, "KMSO");
        assert_eq!(&validation.site.name, "MISSOULA");
        assert_eq!(validation.initialization_time, valid_time);

        let _nbm = arch.retrieve(validation)?;

        Ok(())
    }
}
