use std::convert::TryFrom;

pub struct NBMData {}

impl TryFrom<&str> for NBMData {
    type Error = crate::error::Error;

    fn try_from(_text: &str) -> Result<Self, Self::Error> {
        unimplemented!()
    }
}
