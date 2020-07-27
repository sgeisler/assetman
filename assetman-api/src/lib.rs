pub mod holdings;
pub mod price;

#[derive(Debug, Deserialize, Serialize)]
pub struct Error {
    code: u64,
    description: String,
}