use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct PriceRequest {
    pub asset: String,
    pub unit_of_account: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PriceAnswer {
    pub req: PriceRequest,
    pub price: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PricePluginInfo {
    pub pairs: Vec<(String, String)>,
}