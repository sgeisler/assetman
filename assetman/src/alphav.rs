use std::collections::HashMap;
use std::cell::RefCell;

use reqwest;
use serde_json;

const BASE_URL: &'static str = "https://www.alphavantage.co/query";
const MAX_TRIES: u8 = 6;

#[derive(Debug)]
pub enum AlphaVantageError {
    HttpError(reqwest::Error),
    ApiError(String),
    MalformedApiResponse(&'static str),
    MalformedQuery,
}

pub struct AlphaVantageClient {
    api_key: String,
    forex_cache: RefCell<HashMap<String, f64>>,
}

impl AlphaVantageClient {
    pub fn new(api_key: String) -> AlphaVantageClient {
        AlphaVantageClient {
            api_key,
            forex_cache: RefCell::new(HashMap::new()),
        }
    }

    fn retry_request(&self, query: &[(&str, &str)]) -> Result<serde_json::Value, AlphaVantageError> {
        use std::{thread, time};

        for try_no in 0..MAX_TRIES {
            let response = reqwest::Client::new().get(BASE_URL)
                .query(query)
                .send()?
                .json::<serde_json::Value>()?;

            if let Some(note) = response.get("Note").and_then(serde_json::Value::as_str) {
                if note.contains("API call frequency") {
                    println!("Waiting for rate limit ({}) ...", try_no);
                    thread::sleep(time::Duration::from_secs(21));
                    continue;
                } else {
                    return Err(AlphaVantageError::ApiError(note.into()));
                }
            } else {
                return Ok(response);
            }
        }

        Err(AlphaVantageError::ApiError(format!("API rate limit hit (probably daily)")))
    }

    pub fn forex_rate(&self, from: &str, to: &str) -> Result<f64, AlphaVantageError> {
        let pair_code = format!("{}/{}", from, to);
        if let Some(rate) = {
            let cache = self.forex_cache.borrow();
            let value = cache.get(&pair_code).cloned();
            drop(cache);
            value
        } {
            Ok(rate)
        } else {
            let price= self.retry_request(&[
                    ("function", "CURRENCY_EXCHANGE_RATE"),
                    ("from_currency", from),
                    ("to_currency", to),
                    ("apikey", &self.api_key)
                ])?
                .get("Realtime Currency Exchange Rate")
                .ok_or(AlphaVantageError::MalformedApiResponse("Realtim Rate wasn't included in response"))?
                .get("5. Exchange Rate")
                .ok_or(AlphaVantageError::MalformedApiResponse("Forex price wan't returned in the response"))?
                .as_str()
                .ok_or(AlphaVantageError::MalformedApiResponse("Returned price isn't a float string"))?
                .parse::<f64>()
                .map_err(|_| AlphaVantageError::MalformedApiResponse("Returned price isn't a float"))?;

            self.forex_cache.borrow_mut().insert(pair_code, price);

            Ok(price)
        }
    }

    pub fn stock_price(&self, ticker: &str) -> Result<f64, AlphaVantageError> {
        let response = self.retry_request(&[
                ("function", "GLOBAL_QUOTE"),
                ("symbol", ticker),
                ("apikey", &self.api_key)
            ])?;

        if let Some(error) = response.get("Error Message") {
            return Err(AlphaVantageError::ApiError(
                error.as_str().unwrap_or("Returned error message wasn't a string.").into()
            ));
        }

        response.get("Global Quote")
            .ok_or(AlphaVantageError::MalformedApiResponse("Realtim Rate wasn't included in response"))?
            .get("05. price")
            .ok_or(AlphaVantageError::MalformedApiResponse("Forex price wan't returned in the response"))?
            .as_str()
            .ok_or(AlphaVantageError::MalformedApiResponse("Returned price isn't a float string"))?
            .parse::<f64>()
            .map_err(|_| AlphaVantageError::MalformedApiResponse("Returned price isn't a float"))
    }

    fn query_inner(&self, query: &str) -> Result<f64, AlphaVantageError> {
        let mut query_parts = query.split('/');
        match query_parts.next() {
            Some("stock") => {
                let ticker = query_parts.next().ok_or(AlphaVantageError::MalformedQuery)?;
                let price_usd = self.stock_price(ticker)?;

                if let Some(from_currency) = query_parts.next() {
                    let to_currency = query_parts.next().ok_or(AlphaVantageError::MalformedQuery)?;
                    Ok(price_usd * self.forex_rate(from_currency, to_currency)?)
                } else {
                    Ok(price_usd)
                }
            },
            Some("forex") => {
                let from = query_parts.next().ok_or(AlphaVantageError::MalformedQuery)?;
                let to = query_parts.next().ok_or(AlphaVantageError::MalformedQuery)?;
                self.forex_rate(from, to)
            },
            Some("static") => {
                query_parts.next()
                    .ok_or(AlphaVantageError::MalformedQuery)?
                    .parse::<f64>()
                    .map_err(|_| AlphaVantageError::MalformedQuery)
            }
            None | Some(_) => Err(AlphaVantageError::MalformedQuery)
        }
    }

    pub fn query(&self, query: &str) -> Result<f64, AlphaVantageError> {
        let res = self.query_inner(query);
        if res.is_err() {
            eprintln!("Couldn't fetch '{}'", query);
        }
        res
    }
}

impl From<reqwest::Error> for AlphaVantageError {
    fn from(e: reqwest::Error) -> Self {
        AlphaVantageError::HttpError(e)
    }
}

#[cfg(test)]
mod tests {
    use super::{AlphaVantageClient, AlphaVantageError};

    #[test]
    fn test_forex_rate() {
        dotenv::dotenv().ok();
        let av_client = AlphaVantageClient::new(
            std::env::var("ALPHA_VANTAGE_KEY").expect("ALPHA_VANTAGE_KEY must be set")
        );
        let btc_cny = av_client.forex_rate("BTC", "CNY").unwrap();
        assert!(btc_cny > 9000.0); // make sure vegeta memes still work correctly

        let btc_cny_2 = av_client.query("forex/BTC/CNY").unwrap();
        assert_eq!(btc_cny, btc_cny_2);
    }

    #[test]
    fn test_stock_price() {
        dotenv::dotenv().ok();
        let av_client = AlphaVantageClient::new(
            std::env::var("ALPHA_VANTAGE_KEY").expect("ALPHA_VANTAGE_KEY must be set")
        );
        let price = av_client.stock_price("MSFT").unwrap();
        assert!(price > 42.0);

        let price_2 = av_client.query("stock/MSFT").unwrap();
        assert_eq!(price, price_2);

        let price_eur = av_client.query("stock/MSFT/EUR").unwrap();
        assert_ne!(price, price_eur);
    }

    #[test]
    fn test_static() {
        dotenv::dotenv().ok();
        let av_client = AlphaVantageClient::new(
            std::env::var("ALPHA_VANTAGE_KEY").expect("ALPHA_VANTAGE_KEY must be set")
        );
        assert_eq!(av_client.query("static/42.1234").unwrap(), 42.1234);
    }

    #[test]
    fn test_malformed_query() {
        dotenv::dotenv().ok();
        let av_client = AlphaVantageClient::new(
            std::env::var("ALPHA_VANTAGE_KEY").expect("ALPHA_VANTAGE_KEY must be set")
        );

        match av_client.query("test/BTC/USD") {
            Err(AlphaVantageError::MalformedQuery) => {},
            _ => panic!("'test/BTC/USD' didn't return a query error"),
        };
        match av_client.query("stock") {
            Err(AlphaVantageError::MalformedQuery) => {},
            _ => panic!("'stock' didn't return a query error"),
        };
        match av_client.query("forex/BTC") {
            Err(AlphaVantageError::MalformedQuery) => {},
            _ => panic!("'forex/BTC' didn't return a query error"),
        };
    }
}