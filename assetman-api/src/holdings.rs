#[derive(Debug, Deserialize, Serialize)]
pub struct HoldingsRequest {
    arguments: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HoldingsAnswer {
    req: HoldingsRequest,
    holdings: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PluginInfo {
    description: String,
}