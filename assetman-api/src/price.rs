use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct PriceRequest {
    asset: String,
    unit_of_account: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PriceAnswer {
    req: PriceRequest,
    price: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PricePluginInfo {
    pairs: Vec<(String, String)>,
}