use chrono;

/// The interface to our storage for NBM 1D text files.
///
/// The NBMStore is backed by a private local store. When data is not available in the local store
/// it will fetch it from the internet and then keep a copy in the local store for faster retrieval
/// later.
pub struct NBMStore {
    local_store: crate::local_store::LocalStore,
}

impl NBMStore {
    /// Connect to a NBMStore.
    ///
    /// The path refers to a directory where the local store can save data. The path must be a
    /// directory and not a file name. The local store will handle naming and organization of files
    /// within the directory provided by the path. If the path is [Option::None], then a default
    /// path will be chosen in the user's home directory.
    pub fn connect(path: Option<&std::path::Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let path_buf: std::path::PathBuf = match path {
            Some(p) => std::path::PathBuf::from(p),
            None => Self::default_local_store_path()?,
        };

        unimplemented!()
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
    ) -> Result<crate::site_validation::SiteValidation, crate::error::ValidationError> {
        unimplemented!()
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
    pub fn validate_most_recent_available(&self, site: &str, request_time: chrono::NaiveDateTime,
    ) -> Result<crate::site_validation::SiteValidation, crate::error::ValidationError> {
        let mut attempts_left = 20_i32;
        let mut attempt_request_time = request_time;
        
        loop {
            let validation = self.validate_request(site, attempt_request_time);

            match &validation {
                Err(crate::error::ValidationError::InitializationTimeNotAvailable(init_time)) => {
                    attempt_request_time = *init_time - chrono::Duration::hours(1);
                },
                _ => return validation,
            }

            attempts_left -= 1;
            if attempts_left < 1 {
                return validation;
            }
        }
    }


    fn default_local_store_path() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
        unimplemented!()
    }
}
