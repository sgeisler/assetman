use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Deserialize, Serialize)]
pub struct Request {
    pub arguments: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Answer {
    pub answer: f64,
}

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

#[derive(Debug, Deserialize, Serialize)]
pub struct PluginInfo {
    pub name: String,
    pub plugin_type: PluginType,
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum PluginType {
    Holdings,
    Price,
    Any,
}
