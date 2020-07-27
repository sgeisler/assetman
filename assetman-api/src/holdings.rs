use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct HoldingsRequest {
    pub arguments: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HoldingsAnswer {
    pub req: HoldingsRequest,
    pub holdings: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PluginInfo {
    pub name: String,
    pub description: String,
}