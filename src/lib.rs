#![warn(missing_docs)]
/*! An archive of NBM 1D viewer text files organized by initialization time and site.

*/
/* ------------------------------------------------------------------------------------------------
 *                                         Public API
 * --------------------------------------------------------------------------------------------- */
pub use crate::error::Error;
pub use crate::nbm_data::NBMData;
pub use crate::nbm_store::NBMStore;
pub use crate::site_validation::{SiteInfo, SiteValidation};
/* ------------------------------------------------------------------------------------------------
 *                                        Private Modules
 * --------------------------------------------------------------------------------------------- */
mod download;
mod error;
mod nbm_data;
mod nbm_store;
mod site_validation;
