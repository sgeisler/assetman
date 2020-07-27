use serde::{Deserialize, Serialize};
use std::fmt::{Display, Debug};
use serde::export::Formatter;

pub mod holdings;
pub mod price;

#[derive(Debug, Deserialize, Serialize)]
pub struct Error {
    pub code: u64,
    pub description: String,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}