pub mod holdings;
pub mod price;

#[derive(Debug, Deserialize, Serialize)]
pub struct Error {
    pub code: u64,
    pub description: String,
}