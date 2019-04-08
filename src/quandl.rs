use quandl_v3::prelude::{ApiCall, ApiParameters, DataParameters, DataQuery};

/// Quandl API client
pub struct QuandlClient {
    api_key: String,
}

impl QuandlClient {
    /// Construct a new authenticated quandl API client
    pub fn new(api_key: String) -> QuandlClient {
        QuandlClient {
            api_key,
        }
    }

    /// Query the most recent row from a dataset and return a certain column
    pub fn query_last(&self, database: &str, dataset: &str, col_idx: usize) -> Result<f64, Error> {
        let request = {
            let mut req = DataQuery::new(database, dataset);
            req.limit(1);
            req.api_key(&self.api_key);
            req.column_index(col_idx);
            req
        };

        let (_, value): (String, f64) = request
            .send()?
            .into_iter()
            .next()
            .ok_or(Error::NoRowsReturned)?;

        Ok(value)
    }
}

#[derive(Debug, PartialEq)]
pub enum Error {
    ApiError(quandl_v3::Error),
    NoRowsReturned,
}

impl From<quandl_v3::Error> for Error {
    fn from(e: quandl_v3::Error) -> Self {
        Error::ApiError(e)
    }
}